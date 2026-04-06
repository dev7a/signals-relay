use anyhow::{Context, Result};
use aws_lambda_events::event::cloudwatch_logs::LogsEvent;
use aws_sdk_kinesis::Client as KinesisClient;
use aws_sdk_sqs::Client as SqsClient;
use lambda_runtime::{run, service_fn, Error as LambdaError, LambdaEvent};
use signals_relay::{
    annotate_current_span_error,
    parser::{build_partition_batches, AwsPartitionerIo, PartitionerRuntime},
};
use std::{env, sync::Arc};
use tracing::info;

async fn function_handler(
    event: LambdaEvent<LogsEvent>,
    runtime: Arc<PartitionerRuntime>,
) -> Result<(), LambdaError> {
    let log_group = event.payload.aws_logs.data.log_group.clone();
    let build_result = build_partition_batches(event.payload).map_err(|err| {
        LambdaError::from(annotate_current_span_error("build_partition_batches", &err))
    })?;

    let publish_result = runtime
        .publish_batches(&build_result.batches, event.context.deadline())
        .await
        .map_err(|err| {
            LambdaError::from(annotate_current_span_error(
                "publish_partition_batches",
                &err,
            ))
        })?;

    info!(
        log_group = %log_group,
        input_records = build_result.input_records,
        publishable_records = build_result.published_records,
        malformed_records = build_result.malformed_records,
        publish_batches = build_result.batches.len(),
        published_to_kinesis = publish_result.published_records,
        retried_records = publish_result.retried_records,
        recovered_on_retry = publish_result.recovered_on_retry,
        sent_to_failure_queue = publish_result.sent_to_failure_queue,
        failure_queue_send_failures = publish_result.failure_queue_send_failures,
        publish_attempts = publish_result.publish_attempts,
        "Processed aws/spans CloudWatch batch for partitioning"
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), LambdaError> {
    lambda_runtime::tracing::init_default_subscriber();

    let stream_name = env::var("KINESIS_STREAM_NAME")
        .context("KINESIS_STREAM_NAME environment variable is required")?;
    let failure_queue_url = env::var("PARTITIONER_FAILURE_QUEUE_URL")
        .context("PARTITIONER_FAILURE_QUEUE_URL environment variable is required")?;
    let aws_config = aws_config::load_from_env().await;
    let io = Arc::new(AwsPartitionerIo::new(
        KinesisClient::new(&aws_config),
        SqsClient::new(&aws_config),
    ));
    let runtime = Arc::new(PartitionerRuntime::new(stream_name, failure_queue_url, io)?);

    run(service_fn(move |event: LambdaEvent<LogsEvent>| {
        let runtime = Arc::clone(&runtime);
        async move { function_handler(event, runtime).await }
    }))
    .await
}
