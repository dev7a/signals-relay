pub mod export;
pub mod otlp;
pub mod parser;
pub mod state;

use anyhow::{Context, Result};
use opentelemetry::{trace::Status, KeyValue};
use serverless_otlp_forwarder_core::{
    compact_telemetry_payloads, span_compactor::SpanCompactionConfig, telemetry::TelemetryData,
    InstrumentedHttpClient,
};
use signals_relay_core::EncodedOtlpPayload;
use state::ParsedBatch;
use tracing::{info, Span};
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::export::{send_compacted_telemetry_batch, RelayExportTarget};

pub fn annotate_current_span_error(stage: &str, error: &impl std::fmt::Display) -> String {
    let message = error.to_string();
    let span = Span::current();
    span.set_attribute("error", true);
    span.set_attribute("error.message", message.clone());
    span.set_attribute("processor.error.stage", stage.to_string());
    span.add_event(
        "exception",
        vec![
            KeyValue::new("exception.message", message.clone()),
            KeyValue::new("processor.error.stage", stage.to_string()),
        ],
    );
    span.set_status(Status::error(message.clone()));
    message
}

pub async fn send_parsed_batch(
    parsed_batch: ParsedBatch,
    http_client: &InstrumentedHttpClient,
    compaction_config: &SpanCompactionConfig,
    export_target: &RelayExportTarget,
) -> Result<()> {
    let telemetry_items_count = parsed_batch.telemetry_items.len();
    let emitted_trace_ids_count = parsed_batch.emitted_trace_ids.len();

    if telemetry_items_count == 0 {
        return Ok(());
    }

    let telemetry_items = parsed_batch
        .telemetry_items
        .into_iter()
        .map(encoded_payload_to_telemetry_data)
        .collect::<Vec<_>>();

    let compacted = compact_telemetry_payloads(telemetry_items, compaction_config)
        .context("Failed to compact OTLP telemetry payloads")?;

    send_compacted_telemetry_batch(http_client, compacted, export_target)
        .await
        .context("Failed to send OTLP telemetry batch")?;

    info!(
        telemetry_items_count = telemetry_items_count,
        emitted_trace_ids_count, "Emitted telemetry batch"
    );

    Ok(())
}

fn encoded_payload_to_telemetry_data(item: EncodedOtlpPayload) -> TelemetryData {
    TelemetryData {
        source: item.source,
        endpoint: String::new(),
        payload: item.payload,
        content_type: item.content_type,
        content_encoding: item.content_encoding,
    }
}
