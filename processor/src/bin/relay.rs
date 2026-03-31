use anyhow::Result;
use aws_lambda_events::{
    event::kinesis::{KinesisTimeWindowEvent, KinesisTimeWindowEventResponse},
    time_window::TimeWindowEventResponseProperties,
};
use signals_relay::{
    annotate_current_span_error,
    export::{collector_local_otlp_endpoint, resolve_relay_export_target, RelayExportTarget},
    parser::process_kinesis_time_window_event,
    send_parsed_batch,
};
use lambda_otel_lite::{
    init_telemetry, LambdaSpanProcessor, OtelTracingLayer, SpanAttributes, SpanAttributesExtractor,
    TelemetryConfig,
};
use lambda_runtime::{tower::ServiceBuilder, Error as LambdaError, LambdaEvent, Runtime};
use opentelemetry::Value as OtelValue;
use opentelemetry_otlp::{Protocol, WithExportConfig, WithHttpConfig};
use reqwest::Client as ReqwestClient;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use serverless_otlp_forwarder_core::{
    span_compactor::SpanCompactionConfig, InstrumentedHttpClient,
};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Debug, Clone, Deserialize, Serialize)]
struct RelayEventWrapper(KinesisTimeWindowEvent);

impl SpanAttributesExtractor for RelayEventWrapper {
    fn extract_span_attributes(&self) -> SpanAttributes {
        let mut attributes: HashMap<String, OtelValue> = HashMap::new();
        let window = &self.0.time_window_properties;

        attributes.insert(
            "processor.type".to_string(),
            OtelValue::String("signals_relay".into()),
        );
        attributes.insert(
            "processor.mode".to_string(),
            OtelValue::String("relay".into()),
        );
        attributes.insert(
            "faas.trigger.type".to_string(),
            OtelValue::String("kinesis".into()),
        );
        attributes.insert(
            "aws.kinesis.records.count".to_string(),
            OtelValue::I64(self.0.kinesis_event.records.len() as i64),
        );
        attributes.insert(
            "aws.kinesis.window.final_invoke".to_string(),
            OtelValue::Bool(window.is_final_invoke_for_window),
        );
        if let Some(shard_id) = &window.shard_id {
            attributes.insert(
                "aws.kinesis.shard_id".to_string(),
                OtelValue::String(shard_id.clone().into()),
            );
        }

        SpanAttributes::builder()
            .span_name("signals_relay".to_string())
            .kind("consumer".to_string())
            .attributes(attributes)
            .build()
    }
}

async fn function_handler(
    event: LambdaEvent<RelayEventWrapper>,
    http_client: Arc<InstrumentedHttpClient>,
    export_target: Arc<RelayExportTarget>,
) -> Result<KinesisTimeWindowEventResponse, LambdaError> {
    let compaction_config = SpanCompactionConfig::default();
    let processing = process_kinesis_time_window_event(event.payload.0).map_err(|err| {
        LambdaError::from(annotate_current_span_error(
            "process_kinesis_time_window_event",
            &err,
        ))
    })?;

    send_parsed_batch(
        processing.parsed_batch,
        http_client.as_ref(),
        &compaction_config,
        export_target.as_ref(),
    )
    .await
    .map_err(|err| LambdaError::from(annotate_current_span_error("send_parsed_batch", &err)))?;

    let mut response = KinesisTimeWindowEventResponse::default();
    let mut response_properties = TimeWindowEventResponseProperties::default();
    response_properties.state = processing.response_state;
    response.time_window_event_response_properties = response_properties;
    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    let export_target = Arc::new(
        resolve_relay_export_target()
            .await
            .map_err(LambdaError::from)?,
    );

    let exporter_builder = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary);

    let otlp_http_exporter = match export_target.as_ref() {
        RelayExportTarget::Collector => exporter_builder
            .with_endpoint(collector_local_otlp_endpoint().to_string())
            .build()?,
        RelayExportTarget::DirectEnv => exporter_builder.build()?,
        RelayExportTarget::Direct(target) => exporter_builder
            .with_endpoint(target.endpoint().to_string())
            .with_headers(target.header_map_for_exporter())
            .build()?,
    };

    let (_, completion_handler) = init_telemetry(
        TelemetryConfig::builder()
            .with_span_processor(
                LambdaSpanProcessor::builder()
                    .exporter(otlp_http_exporter)
                    .build(),
            )
            .build(),
    )
    .await?;

    let base_reqwest_client = ReqwestClient::new();
    let client_with_middleware = ClientBuilder::new(base_reqwest_client)
        .with(TracingMiddleware::default())
        .build();
    let instrumented_client = InstrumentedHttpClient::new(client_with_middleware);
    let http_client_for_forwarding = Arc::new(instrumented_client);

    let service =
        ServiceBuilder::new()
            .layer(OtelTracingLayer::new(completion_handler))
            .service_fn(move |event: LambdaEvent<RelayEventWrapper>| {
                let client_for_handler = Arc::clone(&http_client_for_forwarding);
                let export_target_for_handler = Arc::clone(&export_target);
                async move {
                    function_handler(event, client_for_handler, export_target_for_handler).await
                }
            });

    Runtime::new(service).run().await
}
