use anyhow::{Context, Result};
use async_trait::async_trait;
use aws_lambda_events::event::{
    cloudwatch_logs::LogsEvent,
    kinesis::{KinesisEventRecord, KinesisTimeWindowEvent},
};
use aws_sdk_kinesis::{
    error::ProvideErrorMetadata, primitives::Blob, types::PutRecordsRequestEntry,
    Client as KinesisClient,
};
use aws_sdk_sqs::{types::SendMessageBatchRequestEntry, Client as SqsClient};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::{
    collections::{BTreeSet, HashMap},
    hash::{Hash, Hasher},
    sync::Arc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tokio::time::sleep;
use tracing::{info, warn};

use crate::state::{
    trace_id_of_record, ParsedBatch, PartitionedSpanRecord, RelayWindowState, MAX_PUT_RECORDS,
};

const FAILURE_KIND_KINESIS_PUBLISH: &str = "kinesis_publish_failure";
const MAX_FAILURE_QUEUE_BATCH_SIZE: usize = 10;
const MAX_PUBLISH_ATTEMPTS: usize = 3;
const RETRY_TIME_FLOOR_MS: u64 = 5_000;
#[cfg(not(test))]
const BASE_RETRY_BACKOFF_MS: u64 = 200;
#[cfg(test)]
const BASE_RETRY_BACKOFF_MS: u64 = 1;
#[cfg(not(test))]
const MAX_RETRY_BACKOFF_MS: u64 = 2_000;
#[cfg(test)]
const MAX_RETRY_BACKOFF_MS: u64 = 8;

#[async_trait]
pub trait PartitionerIo: Send + Sync {
    async fn put_records(
        &self,
        stream_name: &str,
        records: &[KinesisPublishRecord],
        attempt_count: usize,
    ) -> PublishAttemptSummary;

    async fn send_failure_messages(
        &self,
        queue_url: &str,
        messages: &[FailureQueueEnvelope],
    ) -> Result<()>;
}

#[derive(Clone)]
pub struct AwsPartitionerIo {
    kinesis_client: KinesisClient,
    sqs_client: SqsClient,
}

impl AwsPartitionerIo {
    pub fn new(kinesis_client: KinesisClient, sqs_client: SqsClient) -> Self {
        Self {
            kinesis_client,
            sqs_client,
        }
    }
}

#[async_trait]
impl PartitionerIo for AwsPartitionerIo {
    async fn put_records(
        &self,
        stream_name: &str,
        records: &[KinesisPublishRecord],
        attempt_count: usize,
    ) -> PublishAttemptSummary {
        if records.is_empty() {
            return PublishAttemptSummary::default();
        }

        let request_entries = match records
            .iter()
            .map(|record| {
                PutRecordsRequestEntry::builder()
                    .partition_key(record.partition_key.clone())
                    .data(Blob::new(record.payload.clone()))
                    .build()
            })
            .collect::<std::result::Result<Vec<_>, _>>()
        {
            Ok(request_entries) => request_entries,
            Err(err) => {
                return request_level_failures(
                    records,
                    attempt_count,
                    "BuildRequestEntryFailed",
                    &err.to_string(),
                    false,
                );
            }
        };

        match self
            .kinesis_client
            .put_records()
            .stream_name(stream_name)
            .set_records(Some(request_entries))
            .send()
            .await
        {
            Ok(response) => {
                if response.records().len() != records.len() {
                    return request_level_failures(
                        records,
                        attempt_count,
                        "UnexpectedResponseRecordCount",
                        &format!(
                            "Kinesis returned {} result entries for {} records",
                            response.records().len(),
                            records.len()
                        ),
                        true,
                    );
                }

                let failed_records = records
                    .iter()
                    .zip(response.records())
                    .filter_map(|(record, response_record)| {
                        response_record.error_code().map(|error_code| {
                            let error_message = response_record
                                .error_message()
                                .unwrap_or_default()
                                .to_string();
                            KinesisPublishFailure {
                                record: record.clone(),
                                error_code: error_code.to_string(),
                                error_message,
                                retryable: is_retryable_kinesis_error(error_code),
                                attempt_count,
                            }
                        })
                    })
                    .collect::<Vec<_>>();

                PublishAttemptSummary {
                    successful_records: records.len().saturating_sub(failed_records.len()),
                    failed_records,
                }
            }
            Err(err) => {
                let error_code = err.code().unwrap_or("UnknownKinesisError").to_string();
                let error_message = err
                    .message()
                    .map(str::to_string)
                    .unwrap_or_else(|| err.to_string());
                request_level_failures(
                    records,
                    attempt_count,
                    error_code.as_str(),
                    error_message.as_str(),
                    is_retryable_kinesis_error(error_code.as_str()),
                )
            }
        }
    }

    async fn send_failure_messages(
        &self,
        queue_url: &str,
        messages: &[FailureQueueEnvelope],
    ) -> Result<()> {
        if messages.is_empty() {
            return Ok(());
        }

        let entries = messages
            .iter()
            .enumerate()
            .map(|(index, message)| {
                SendMessageBatchRequestEntry::builder()
                    .id(format!("entry-{index}"))
                    .message_body(
                        serde_json::to_string(message)
                            .context("Failed to serialize failure queue message")?,
                    )
                    .build()
                    .context("Failed to build failure queue message entry")
            })
            .collect::<Result<Vec<_>>>()?;

        let response = self
            .sqs_client
            .send_message_batch()
            .queue_url(queue_url)
            .set_entries(Some(entries))
            .send()
            .await
            .context("Failed to send partitioner failure messages to SQS")?;

        if response.failed().is_empty() {
            return Ok(());
        }

        let details = response
            .failed()
            .iter()
            .map(|entry| {
                format!(
                    "id={} code={} message={}",
                    entry.id(),
                    entry.code(),
                    entry.message().unwrap_or_default()
                )
            })
            .collect::<Vec<_>>()
            .join("; ");
        Err(anyhow::anyhow!(
            "SQS failure queue send reported {} failed entries: {details}",
            response.failed().len()
        ))
    }
}

#[derive(Clone)]
pub struct PartitionerRuntime {
    stream_name: String,
    failure_queue_url: String,
    io: Arc<dyn PartitionerIo>,
}

impl PartitionerRuntime {
    pub fn new(
        stream_name: String,
        failure_queue_url: String,
        io: Arc<dyn PartitionerIo>,
    ) -> Result<Self> {
        if stream_name.trim().is_empty() {
            return Err(anyhow::anyhow!("KINESIS_STREAM_NAME is required"));
        }
        if failure_queue_url.trim().is_empty() {
            return Err(anyhow::anyhow!("PARTITIONER_FAILURE_QUEUE_URL is required"));
        }

        Ok(Self {
            stream_name,
            failure_queue_url,
            io,
        })
    }

    pub async fn publish_batches(
        &self,
        batches: &[Vec<KinesisPublishRecord>],
        deadline: SystemTime,
    ) -> Result<PartitionerPublishResult> {
        let mut result = PartitionerPublishResult::default();
        let mut retried_ids = BTreeSet::new();
        let mut recovered_ids = BTreeSet::new();

        for (batch_index, batch) in batches.iter().enumerate() {
            let mut pending = batch.clone();
            let mut attempt_count = 1usize;

            loop {
                let attempted_ids = pending
                    .iter()
                    .map(|record| record.source_event_id.clone())
                    .collect::<BTreeSet<_>>();

                result.publish_attempts += 1;
                let summary = self
                    .io
                    .put_records(self.stream_name.as_str(), &pending, attempt_count)
                    .await;
                let failed_ids = summary
                    .failed_records
                    .iter()
                    .map(|failure| failure.record.source_event_id.clone())
                    .collect::<BTreeSet<_>>();

                result.published_records += summary.successful_records;
                if attempt_count > 1 {
                    for source_event_id in attempted_ids {
                        if !failed_ids.contains(&source_event_id)
                            && retried_ids.contains(&source_event_id)
                        {
                            recovered_ids.insert(source_event_id);
                        }
                    }
                }

                if summary.failed_records.is_empty() {
                    break;
                }

                for failure in &summary.failed_records {
                    warn!(
                        batch_index,
                        source_event_id = %failure.record.source_event_id,
                        partition_key = %failure.record.partition_key,
                        attempt_count = failure.attempt_count,
                        retryable = failure.retryable,
                        error_code = %failure.error_code,
                        error_message = %failure.error_message,
                        "Kinesis PutRecords entry failed"
                    );
                }

                let (retryable_failures, terminal_failures): (Vec<_>, Vec<_>) = summary
                    .failed_records
                    .into_iter()
                    .partition(|failure| failure.retryable);

                if !terminal_failures.is_empty() {
                    self.queue_failures(&terminal_failures, &mut result).await?;
                }

                if retryable_failures.is_empty() {
                    break;
                }

                if attempt_count >= MAX_PUBLISH_ATTEMPTS || !has_retry_time_budget(deadline) {
                    warn!(
                        batch_index,
                        failed_records = retryable_failures.len(),
                        attempt_count,
                        "Retry budget exhausted or deadline too close, routing publish failures to SQS"
                    );
                    self.queue_failures(&retryable_failures, &mut result)
                        .await?;
                    break;
                }

                for failure in &retryable_failures {
                    retried_ids.insert(failure.record.source_event_id.clone());
                }

                let delay = retry_backoff_delay(&retryable_failures, attempt_count);
                info!(
                    batch_index,
                    failed_records = retryable_failures.len(),
                    attempt_count,
                    retry_delay_ms = delay.as_millis() as u64,
                    "Retrying failed Kinesis publish subset"
                );
                sleep(delay).await;
                pending = retryable_failures
                    .into_iter()
                    .map(|failure| failure.record)
                    .collect();
                attempt_count += 1;
            }
        }

        result.retried_records = retried_ids.len();
        result.recovered_on_retry = recovered_ids.len();

        Ok(result)
    }

    async fn queue_failures(
        &self,
        failures: &[KinesisPublishFailure],
        result: &mut PartitionerPublishResult,
    ) -> Result<()> {
        for chunk in failures.chunks(MAX_FAILURE_QUEUE_BATCH_SIZE) {
            let messages = chunk
                .iter()
                .map(|failure| {
                    FailureQueueEnvelope::from_failure(self.stream_name.as_str(), failure)
                })
                .collect::<Vec<_>>();

            if let Err(err) = self
                .io
                .send_failure_messages(self.failure_queue_url.as_str(), &messages)
                .await
            {
                result.failure_queue_send_failures += chunk.len();
                return Err(err).context("Failed to enqueue partitioner publish failures");
            }

            result.sent_to_failure_queue += chunk.len();
        }

        Ok(())
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct KinesisPublishRecord {
    pub partition_key: String,
    pub source_event_id: String,
    pub trace_id: String,
    pub source_log_group: String,
    pub record: JsonValue,
    pub payload: Vec<u8>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PartitionerBuildResult {
    pub input_records: usize,
    pub published_records: usize,
    pub malformed_records: usize,
    pub batches: Vec<Vec<KinesisPublishRecord>>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PartitionerPublishResult {
    pub published_records: usize,
    pub retried_records: usize,
    pub recovered_on_retry: usize,
    pub sent_to_failure_queue: usize,
    pub failure_queue_send_failures: usize,
    pub publish_attempts: usize,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct PublishAttemptSummary {
    pub successful_records: usize,
    pub failed_records: Vec<KinesisPublishFailure>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct KinesisPublishFailure {
    pub record: KinesisPublishRecord,
    pub error_code: String,
    pub error_message: String,
    pub retryable: bool,
    pub attempt_count: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct FailureQueueEnvelope {
    pub kind: String,
    pub stream_name: String,
    pub partition_key: String,
    pub source_event_id: String,
    pub trace_id: String,
    pub source_log_group: String,
    pub record: JsonValue,
    pub attempt_count: usize,
    pub last_error_code: String,
    pub last_error_message: String,
    pub failed_at: u64,
}

impl FailureQueueEnvelope {
    fn from_failure(stream_name: &str, failure: &KinesisPublishFailure) -> Self {
        Self {
            kind: FAILURE_KIND_KINESIS_PUBLISH.to_string(),
            stream_name: stream_name.to_string(),
            partition_key: failure.record.partition_key.clone(),
            source_event_id: failure.record.source_event_id.clone(),
            trace_id: failure.record.trace_id.clone(),
            source_log_group: failure.record.source_log_group.clone(),
            record: failure.record.record.clone(),
            attempt_count: failure.attempt_count,
            last_error_code: failure.error_code.clone(),
            last_error_message: failure.error_message.clone(),
            failed_at: now_unix_millis(),
        }
    }
}

#[derive(Debug, Default)]
pub struct RelayProcessingResult {
    pub parsed_batch: ParsedBatch,
    pub response_state: HashMap<String, String>,
    pub deduped_source_events: usize,
    pub aggregated_traces: usize,
    pub emitted_spans: usize,
    pub late_decorators_dropped: usize,
    pub serialized_state_bytes: usize,
    pub is_final_invoke_for_window: bool,
}

pub fn build_partition_batches(event_payload: LogsEvent) -> Result<PartitionerBuildResult> {
    let log_group = event_payload.aws_logs.data.log_group;
    let mut result = PartitionerBuildResult::default();
    let mut current_batch = Vec::new();

    for log_event in event_payload.aws_logs.data.log_events {
        result.input_records += 1;

        let record: JsonValue = match serde_json::from_str(&log_event.message) {
            Ok(record) => record,
            Err(err) => {
                result.malformed_records += 1;
                warn!(error = %err, "Failed to parse aws/spans JSON, skipping record");
                continue;
            }
        };

        let Some(trace_id) = trace_id_of_record(&record) else {
            result.malformed_records += 1;
            warn!("aws/spans record missing traceId, skipping");
            continue;
        };

        let payload = serde_json::to_vec(&PartitionedSpanRecord {
            source_log_group: log_group.clone(),
            source_event_id: log_event.id.clone(),
            trace_id: trace_id.clone(),
            record: record.clone(),
        })
        .context("Failed to serialize partitioned Kinesis payload")?;

        current_batch.push(KinesisPublishRecord {
            partition_key: trace_id.clone(),
            source_event_id: log_event.id,
            trace_id,
            source_log_group: log_group.clone(),
            record,
            payload,
        });
        result.published_records += 1;

        if current_batch.len() == MAX_PUT_RECORDS {
            result.batches.push(std::mem::take(&mut current_batch));
        }
    }

    if !current_batch.is_empty() {
        result.batches.push(current_batch);
    }

    Ok(result)
}

pub fn process_kinesis_time_window_event(
    event_payload: KinesisTimeWindowEvent,
) -> Result<RelayProcessingResult> {
    let mut relay_state =
        RelayWindowState::from_response_state(&event_payload.time_window_properties.state)?;
    let mut result = RelayProcessingResult {
        is_final_invoke_for_window: event_payload
            .time_window_properties
            .is_final_invoke_for_window,
        ..RelayProcessingResult::default()
    };

    for record in event_payload.kinesis_event.records {
        let Some(partitioned_record) = decode_partitioned_record(record)? else {
            continue;
        };

        if !relay_state.record_is_new(partitioned_record.source_event_id.clone()) {
            result.deduped_source_events += 1;
            continue;
        }

        relay_state.apply_record(partitioned_record)?;
    }

    result.aggregated_traces = relay_state.traces.len();
    result.serialized_state_bytes = relay_state.serialized_size_bytes()?;

    if event_payload
        .time_window_properties
        .is_final_invoke_for_window
    {
        let finalized = relay_state.finalize();
        result.emitted_spans = finalized.parsed_batch.telemetry_items.len();
        result.late_decorators_dropped = finalized.late_decorators_dropped;
        result.parsed_batch = finalized.parsed_batch;
        result.response_state = HashMap::new();
    } else {
        result.response_state = relay_state.to_response_state()?;
    }

    info!(
        deduped_source_events = result.deduped_source_events,
        aggregated_traces = result.aggregated_traces,
        emitted_spans = result.emitted_spans,
        late_decorators_dropped = result.late_decorators_dropped,
        serialized_state_bytes = result.serialized_state_bytes,
        is_final_invoke_for_window = result.is_final_invoke_for_window,
        "Processed Kinesis relay window batch"
    );

    Ok(result)
}

fn decode_partitioned_record(record: KinesisEventRecord) -> Result<Option<PartitionedSpanRecord>> {
    let mut partitioned_record: PartitionedSpanRecord =
        match serde_json::from_slice(record.kinesis.data.as_slice()) {
            Ok(partitioned_record) => partitioned_record,
            Err(err) => {
                warn!(error = %err, "Failed to parse partitioned Kinesis payload, skipping record");
                return Ok(None);
            }
        };

    if partitioned_record.source_event_id.is_empty() {
        partitioned_record.source_event_id = record
            .event_id
            .unwrap_or_else(|| record.kinesis.sequence_number.clone());
    }

    if partitioned_record.source_log_group.is_empty() {
        partitioned_record.source_log_group = "aws/spans".to_string();
    }

    if partitioned_record.trace_id.is_empty() {
        let Some(trace_id) = trace_id_of_record(&partitioned_record.record) else {
            warn!("Partitioned Kinesis payload missing traceId, skipping record");
            return Ok(None);
        };
        partitioned_record.trace_id = trace_id;
    }

    Ok(Some(partitioned_record))
}

fn request_level_failures(
    records: &[KinesisPublishRecord],
    attempt_count: usize,
    error_code: &str,
    error_message: &str,
    retryable: bool,
) -> PublishAttemptSummary {
    PublishAttemptSummary {
        successful_records: 0,
        failed_records: records
            .iter()
            .cloned()
            .map(|record| KinesisPublishFailure {
                record,
                error_code: error_code.to_string(),
                error_message: error_message.to_string(),
                retryable,
                attempt_count,
            })
            .collect(),
    }
}

fn is_retryable_kinesis_error(error_code: &str) -> bool {
    matches!(
        error_code,
        "ProvisionedThroughputExceededException"
            | "InternalFailure"
            | "ServiceUnavailable"
            | "KMSThrottlingException"
    )
}

fn has_retry_time_budget(deadline: SystemTime) -> bool {
    deadline
        .duration_since(SystemTime::now())
        .unwrap_or_default()
        >= Duration::from_millis(RETRY_TIME_FLOOR_MS)
}

fn retry_backoff_delay(failures: &[KinesisPublishFailure], attempt_count: usize) -> Duration {
    let exponent = attempt_count.saturating_sub(1).min(4) as u32;
    let base_delay_ms =
        (BASE_RETRY_BACKOFF_MS.saturating_mul(1_u64 << exponent)).min(MAX_RETRY_BACKOFF_MS);
    let jitter_ms = failures
        .iter()
        .map(|failure| hash_source_event_id(failure.record.source_event_id.as_str()))
        .fold(0_u64, |acc, hash| acc ^ hash)
        % 100;
    Duration::from_millis(base_delay_ms.saturating_add(jitter_ms))
}

fn hash_source_event_id(source_event_id: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    source_event_id.hash(&mut hasher);
    hasher.finish()
}

fn now_unix_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use aws_lambda_events::{
        encodings::Base64Data,
        event::{
            cloudwatch_logs::{AwsLogs, LogData, LogEntry},
            kinesis::{
                KinesisEncryptionType, KinesisEvent, KinesisRecord, KinesisTimeWindowEventResponse,
            },
        },
        time_window::{TimeWindowEventResponseProperties, TimeWindowProperties, Window},
    };
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use prost::Message;
    use serde_json::json;
    use std::{collections::VecDeque, sync::Mutex};

    fn fixture(name: &str) -> String {
        match name {
            "target" => include_str!("../tests/fixtures/completed_linkable_target.json")
                .trim()
                .to_string(),
            "decorator" => include_str!("../tests/fixtures/managed_link_decorator.json")
                .trim()
                .to_string(),
            "client" => include_str!("../tests/fixtures/normal_client_span.json")
                .trim()
                .to_string(),
            other => panic!("unknown fixture {other}"),
        }
    }

    fn build_logs_event(messages: Vec<String>, log_group: &str) -> LogsEvent {
        let log_events = messages
            .into_iter()
            .enumerate()
            .map(|(index, message)| {
                let mut entry = LogEntry::default();
                entry.id = format!("event-{index}");
                entry.timestamp = index as i64;
                entry.message = message;
                entry
            })
            .collect();

        let mut log_data = LogData::default();
        log_data.owner = "owner".to_string();
        log_data.log_group = log_group.to_string();
        log_data.log_stream = "stream".to_string();
        log_data.message_type = "DATA_MESSAGE".to_string();
        log_data.subscription_filters = vec!["subscription".to_string()];
        log_data.log_events = log_events;

        let mut aws_logs = AwsLogs::default();
        aws_logs.data = log_data;

        let mut event = LogsEvent::default();
        event.aws_logs = aws_logs;
        event
    }

    fn decode_single_span(
        item: &signals_relay_core::EncodedOtlpPayload,
    ) -> opentelemetry_proto::tonic::trace::v1::Span {
        let request = ExportTraceServiceRequest::decode(item.payload.as_slice())
            .expect("payload should decode");
        request.resource_spans[0].scope_spans[0].spans[0].clone()
    }

    fn partitioned_record(message: &str, source_event_id: &str) -> PartitionedSpanRecord {
        let record: JsonValue = serde_json::from_str(message).expect("fixture should parse");
        PartitionedSpanRecord {
            source_log_group: "aws/spans".to_string(),
            source_event_id: source_event_id.to_string(),
            trace_id: trace_id_of_record(&record).expect("trace id should exist"),
            record,
        }
    }

    fn build_kinesis_event(
        records: Vec<PartitionedSpanRecord>,
        state: HashMap<String, String>,
        is_final_invoke_for_window: bool,
    ) -> KinesisTimeWindowEvent {
        let records = records
            .into_iter()
            .enumerate()
            .map(|(index, record)| {
                let mut data = Base64Data::default();
                data.extend(
                    serde_json::to_vec(&record).expect("partitioned record should serialize"),
                );

                let mut kinesis = KinesisRecord::default();
                kinesis.approximate_arrival_timestamp =
                    serde_json::from_value(json!(0)).expect("timestamp should deserialize");
                kinesis.data = data;
                kinesis.encryption_type = KinesisEncryptionType::None;
                kinesis.partition_key = record.trace_id.clone();
                kinesis.sequence_number = format!("sequence-{index}");
                kinesis.kinesis_schema_version = Some("1.0".to_string());

                let mut event_record = KinesisEventRecord::default();
                event_record.aws_region = Some("us-east-1".to_string());
                event_record.event_id = Some(format!("kinesis-event-{index}"));
                event_record.event_name = Some("aws:kinesis:record".to_string());
                event_record.event_source = Some("aws:kinesis".to_string());
                event_record.event_source_arn =
                    Some("arn:aws:kinesis:us-east-1:123456789012:stream/test-stream".to_string());
                event_record.event_version = Some("1.0".to_string());
                event_record.kinesis = kinesis;
                event_record
            })
            .collect();

        let mut kinesis_event = KinesisEvent::default();
        kinesis_event.records = records;

        let mut time_window_properties = TimeWindowProperties::default();
        time_window_properties.window = Window::default();
        time_window_properties.state = state;
        time_window_properties.shard_id = Some("shardId-000000000000".to_string());
        time_window_properties.event_source_arn =
            Some("arn:aws:kinesis:us-east-1:123456789012:stream/test-stream".to_string());
        time_window_properties.is_final_invoke_for_window = is_final_invoke_for_window;
        time_window_properties.is_window_terminated_early = false;

        let mut event = KinesisTimeWindowEvent::default();
        event.kinesis_event = kinesis_event;
        event.time_window_properties = time_window_properties;
        event
    }

    #[derive(Clone, Debug)]
    enum FakePublishAction {
        Success,
        PartialFailure {
            failed_event_ids: Vec<String>,
            error_code: String,
            error_message: String,
            retryable: bool,
        },
        RequestFailure {
            error_code: String,
            error_message: String,
            retryable: bool,
        },
    }

    #[derive(Default)]
    struct FakePartitionerState {
        publish_actions: VecDeque<FakePublishAction>,
        publish_calls: Vec<Vec<String>>,
        failure_queue_batches: Vec<Vec<FailureQueueEnvelope>>,
        fail_failure_queue_send: bool,
    }

    #[derive(Clone, Default)]
    struct FakePartitionerIo {
        state: Arc<Mutex<FakePartitionerState>>,
    }

    impl FakePartitionerIo {
        fn with_actions(actions: Vec<FakePublishAction>) -> Self {
            Self {
                state: Arc::new(Mutex::new(FakePartitionerState {
                    publish_actions: actions.into(),
                    ..FakePartitionerState::default()
                })),
            }
        }

        fn publish_calls(&self) -> Vec<Vec<String>> {
            self.state
                .lock()
                .expect("lock should succeed")
                .publish_calls
                .clone()
        }

        fn failure_queue_batches(&self) -> Vec<Vec<FailureQueueEnvelope>> {
            self.state
                .lock()
                .expect("lock should succeed")
                .failure_queue_batches
                .clone()
        }

        fn set_failure_queue_send_failure(&self, fail: bool) {
            self.state
                .lock()
                .expect("lock should succeed")
                .fail_failure_queue_send = fail;
        }
    }

    #[async_trait]
    impl PartitionerIo for FakePartitionerIo {
        async fn put_records(
            &self,
            _stream_name: &str,
            records: &[KinesisPublishRecord],
            attempt_count: usize,
        ) -> PublishAttemptSummary {
            let mut state = self.state.lock().expect("lock should succeed");
            state.publish_calls.push(
                records
                    .iter()
                    .map(|record| record.source_event_id.clone())
                    .collect(),
            );
            let action = state
                .publish_actions
                .pop_front()
                .unwrap_or(FakePublishAction::Success);
            drop(state);

            match action {
                FakePublishAction::Success => PublishAttemptSummary {
                    successful_records: records.len(),
                    failed_records: Vec::new(),
                },
                FakePublishAction::PartialFailure {
                    failed_event_ids,
                    error_code,
                    error_message,
                    retryable,
                } => {
                    let failed_event_ids = failed_event_ids.into_iter().collect::<BTreeSet<_>>();
                    let failed_records = records
                        .iter()
                        .filter(|record| failed_event_ids.contains(&record.source_event_id))
                        .cloned()
                        .map(|record| KinesisPublishFailure {
                            record,
                            error_code: error_code.clone(),
                            error_message: error_message.clone(),
                            retryable,
                            attempt_count,
                        })
                        .collect::<Vec<_>>();

                    PublishAttemptSummary {
                        successful_records: records.len().saturating_sub(failed_records.len()),
                        failed_records,
                    }
                }
                FakePublishAction::RequestFailure {
                    error_code,
                    error_message,
                    retryable,
                } => request_level_failures(
                    records,
                    attempt_count,
                    error_code.as_str(),
                    error_message.as_str(),
                    retryable,
                ),
            }
        }

        async fn send_failure_messages(
            &self,
            _queue_url: &str,
            messages: &[FailureQueueEnvelope],
        ) -> Result<()> {
            let mut state = self.state.lock().expect("lock should succeed");
            if state.fail_failure_queue_send {
                return Err(anyhow::anyhow!("simulated SQS failure"));
            }
            state.failure_queue_batches.push(messages.to_vec());
            Ok(())
        }
    }

    fn runtime_with_fake_io(fake_io: FakePartitionerIo) -> PartitionerRuntime {
        PartitionerRuntime::new(
            "test-stream".to_string(),
            "https://sqs.us-east-1.amazonaws.com/123456789012/test".to_string(),
            Arc::new(fake_io),
        )
        .expect("runtime should build")
    }

    fn generous_deadline() -> SystemTime {
        SystemTime::now() + Duration::from_secs(60)
    }

    #[test]
    fn partitioner_builds_records_with_trace_id_partition_key() {
        let result = build_partition_batches(build_logs_event(
            vec![fixture("target")],
            "/aws/appsignals/test-group",
        ))
        .expect("partition batches should build");

        assert_eq!(result.input_records, 1);
        assert_eq!(result.published_records, 1);
        assert_eq!(result.malformed_records, 0);
        assert_eq!(result.batches.len(), 1);
        assert_eq!(result.batches[0].len(), 1);

        let wrapped: PartitionedSpanRecord =
            serde_json::from_slice(result.batches[0][0].payload.as_slice())
                .expect("wrapped partition payload should deserialize");
        assert_eq!(wrapped.source_log_group, "/aws/appsignals/test-group");
        assert_eq!(wrapped.source_event_id, "event-0");
        assert_eq!(
            result.batches[0][0].partition_key, wrapped.trace_id,
            "partition key should be traceId"
        );
    }

    #[test]
    fn partitioner_skips_malformed_records() {
        let result = build_partition_batches(build_logs_event(
            vec!["not-json".to_string(), fixture("client")],
            "/aws/appsignals/test-group",
        ))
        .expect("partition batches should build");

        assert_eq!(result.input_records, 2);
        assert_eq!(result.published_records, 1);
        assert_eq!(result.malformed_records, 1);
    }

    #[test]
    fn partitioner_splits_records_into_put_records_batches_of_five_hundred() {
        let messages = std::iter::repeat_with(|| fixture("client"))
            .take(501)
            .collect::<Vec<_>>();
        let result =
            build_partition_batches(build_logs_event(messages, "/aws/appsignals/test-group"))
                .expect("partition batches should build");

        assert_eq!(result.published_records, 501);
        assert_eq!(result.batches.len(), 2);
        assert_eq!(result.batches[0].len(), 500);
        assert_eq!(result.batches[1].len(), 1);
    }

    #[tokio::test]
    async fn partitioner_retries_only_failed_subset_without_republishing_successes() {
        let fake_io = FakePartitionerIo::with_actions(vec![
            FakePublishAction::PartialFailure {
                failed_event_ids: vec!["event-1".to_string()],
                error_code: "ProvisionedThroughputExceededException".to_string(),
                error_message: "try again".to_string(),
                retryable: true,
            },
            FakePublishAction::Success,
        ]);
        let runtime = runtime_with_fake_io(fake_io.clone());
        let build_result = build_partition_batches(build_logs_event(
            vec![fixture("client"), fixture("target"), fixture("client")],
            "/aws/appsignals/test-group",
        ))
        .expect("partition batches should build");

        let publish_result = runtime
            .publish_batches(&build_result.batches, generous_deadline())
            .await
            .expect("publish should succeed");

        assert_eq!(
            fake_io.publish_calls(),
            vec![
                vec![
                    "event-0".to_string(),
                    "event-1".to_string(),
                    "event-2".to_string()
                ],
                vec!["event-1".to_string()]
            ]
        );
        assert_eq!(publish_result.published_records, 3);
        assert_eq!(publish_result.retried_records, 1);
        assert_eq!(publish_result.recovered_on_retry, 1);
        assert_eq!(publish_result.sent_to_failure_queue, 0);
    }

    #[tokio::test]
    async fn partitioner_sends_non_retryable_failures_to_sqs_with_replay_payload() {
        let fake_io = FakePartitionerIo::with_actions(vec![FakePublishAction::PartialFailure {
            failed_event_ids: vec!["event-0".to_string()],
            error_code: "AccessDeniedException".to_string(),
            error_message: "denied".to_string(),
            retryable: false,
        }]);
        let runtime = runtime_with_fake_io(fake_io.clone());
        let build_result = build_partition_batches(build_logs_event(
            vec![fixture("client"), fixture("target")],
            "/aws/appsignals/test-group",
        ))
        .expect("partition batches should build");

        let publish_result = runtime
            .publish_batches(&build_result.batches, generous_deadline())
            .await
            .expect("publish should succeed");

        assert_eq!(publish_result.published_records, 1);
        assert_eq!(publish_result.sent_to_failure_queue, 1);
        let queued = fake_io.failure_queue_batches();
        assert_eq!(queued.len(), 1);
        assert_eq!(queued[0].len(), 1);
        let message = &queued[0][0];
        assert_eq!(message.kind, FAILURE_KIND_KINESIS_PUBLISH);
        assert_eq!(message.stream_name, "test-stream");
        assert_eq!(message.source_event_id, "event-0");
        assert!(!message.trace_id.is_empty());
        assert_eq!(message.source_log_group, "/aws/appsignals/test-group");
        assert_eq!(message.last_error_code, "AccessDeniedException");
        assert_eq!(message.last_error_message, "denied");
        assert_eq!(message.attempt_count, 1);
    }

    #[tokio::test]
    async fn partitioner_retries_request_level_failures_with_same_budget() {
        let fake_io = FakePartitionerIo::with_actions(vec![
            FakePublishAction::RequestFailure {
                error_code: "ServiceUnavailable".to_string(),
                error_message: "unavailable".to_string(),
                retryable: true,
            },
            FakePublishAction::Success,
        ]);
        let runtime = runtime_with_fake_io(fake_io.clone());
        let build_result = build_partition_batches(build_logs_event(
            vec![fixture("client")],
            "/aws/appsignals/test-group",
        ))
        .expect("partition batches should build");

        let publish_result = runtime
            .publish_batches(&build_result.batches, generous_deadline())
            .await
            .expect("publish should succeed");

        assert_eq!(
            fake_io.publish_calls(),
            vec![vec!["event-0".to_string()], vec!["event-0".to_string()]]
        );
        assert_eq!(publish_result.publish_attempts, 2);
        assert_eq!(publish_result.recovered_on_retry, 1);
    }

    #[tokio::test]
    async fn partitioner_returns_error_when_failure_queue_send_fails() {
        let fake_io = FakePartitionerIo::with_actions(vec![FakePublishAction::PartialFailure {
            failed_event_ids: vec!["event-0".to_string()],
            error_code: "AccessDeniedException".to_string(),
            error_message: "denied".to_string(),
            retryable: false,
        }]);
        fake_io.set_failure_queue_send_failure(true);
        let runtime = runtime_with_fake_io(fake_io);
        let build_result = build_partition_batches(build_logs_event(
            vec![fixture("client")],
            "/aws/appsignals/test-group",
        ))
        .expect("partition batches should build");

        let err = runtime
            .publish_batches(&build_result.batches, generous_deadline())
            .await
            .expect_err("publish should fail");
        assert!(err
            .to_string()
            .contains("Failed to enqueue partitioner publish failures"));
    }

    #[tokio::test]
    async fn partitioner_queues_retryable_failures_when_deadline_is_too_close() {
        let fake_io = FakePartitionerIo::with_actions(vec![FakePublishAction::PartialFailure {
            failed_event_ids: vec!["event-0".to_string()],
            error_code: "ProvisionedThroughputExceededException".to_string(),
            error_message: "throttled".to_string(),
            retryable: true,
        }]);
        let runtime = runtime_with_fake_io(fake_io.clone());
        let build_result = build_partition_batches(build_logs_event(
            vec![fixture("client")],
            "/aws/appsignals/test-group",
        ))
        .expect("partition batches should build");

        let publish_result = runtime
            .publish_batches(
                &build_result.batches,
                SystemTime::now() + Duration::from_millis(RETRY_TIME_FLOOR_MS.saturating_sub(1)),
            )
            .await
            .expect("publish should succeed");

        assert_eq!(fake_io.publish_calls(), vec![vec!["event-0".to_string()]]);
        assert_eq!(publish_result.retried_records, 0);
        assert_eq!(publish_result.sent_to_failure_queue, 1);
    }

    #[tokio::test]
    async fn partitioner_handles_multiple_batches_with_mixed_outcomes() {
        let fake_io = FakePartitionerIo::with_actions(vec![
            FakePublishAction::PartialFailure {
                failed_event_ids: vec!["event-499".to_string()],
                error_code: "ProvisionedThroughputExceededException".to_string(),
                error_message: "try again".to_string(),
                retryable: true,
            },
            FakePublishAction::Success,
            FakePublishAction::Success,
        ]);
        let runtime = runtime_with_fake_io(fake_io.clone());
        let messages = std::iter::repeat_with(|| fixture("client"))
            .take(501)
            .collect::<Vec<_>>();
        let build_result =
            build_partition_batches(build_logs_event(messages, "/aws/appsignals/test-group"))
                .expect("partition batches should build");

        let publish_result = runtime
            .publish_batches(&build_result.batches, generous_deadline())
            .await
            .expect("publish should succeed");

        assert_eq!(fake_io.publish_calls().len(), 3);
        assert_eq!(fake_io.publish_calls()[0].len(), 500);
        assert_eq!(fake_io.publish_calls()[1], vec!["event-499".to_string()]);
        assert_eq!(fake_io.publish_calls()[2], vec!["event-500".to_string()]);
        assert_eq!(publish_result.published_records, 501);
    }

    #[test]
    fn relay_accumulates_until_final_window_invoke() {
        let first = process_kinesis_time_window_event(build_kinesis_event(
            vec![partitioned_record(&fixture("target"), "event-1")],
            HashMap::new(),
            false,
        ))
        .expect("first relay batch should process");
        assert!(first.parsed_batch.telemetry_items.is_empty());
        assert!(!first.response_state.is_empty());

        let second = process_kinesis_time_window_event(build_kinesis_event(
            vec![partitioned_record(&fixture("decorator"), "event-2")],
            first.response_state,
            true,
        ))
        .expect("second relay batch should process");
        assert_eq!(second.parsed_batch.telemetry_items.len(), 1);
        assert_eq!(
            decode_single_span(&second.parsed_batch.telemetry_items[0])
                .links
                .len(),
            1
        );
    }

    #[test]
    fn relay_dedupes_duplicate_source_event_ids() {
        let result = process_kinesis_time_window_event(build_kinesis_event(
            vec![
                partitioned_record(&fixture("client"), "event-1"),
                partitioned_record(&fixture("client"), "event-1"),
            ],
            HashMap::new(),
            true,
        ))
        .expect("relay batch should process");

        assert_eq!(result.deduped_source_events, 1);
        assert_eq!(result.parsed_batch.telemetry_items.len(), 1);
    }

    #[test]
    fn relay_keeps_multiple_traces_isolated() {
        let mut second_client: JsonValue =
            serde_json::from_str(&fixture("client")).expect("fixture should parse");
        second_client["traceId"] =
            JsonValue::String("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string());
        second_client["spanId"] = JsonValue::String("bbbbbbbbbbbbbbbb".to_string());

        let result = process_kinesis_time_window_event(build_kinesis_event(
            vec![
                partitioned_record(&fixture("client"), "event-1"),
                PartitionedSpanRecord {
                    source_log_group: "aws/spans".to_string(),
                    source_event_id: "event-2".to_string(),
                    trace_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".to_string(),
                    record: second_client,
                },
            ],
            HashMap::new(),
            true,
        ))
        .expect("relay batch should process");

        assert_eq!(result.aggregated_traces, 2);
        assert_eq!(result.parsed_batch.telemetry_items.len(), 2);
    }

    #[test]
    fn relay_drops_late_decorators_in_later_windows() {
        let first = process_kinesis_time_window_event(build_kinesis_event(
            vec![partitioned_record(&fixture("target"), "event-1")],
            HashMap::new(),
            true,
        ))
        .expect("first relay batch should process");
        assert_eq!(first.parsed_batch.telemetry_items.len(), 1);
        assert_eq!(
            decode_single_span(&first.parsed_batch.telemetry_items[0])
                .links
                .len(),
            0
        );

        let second = process_kinesis_time_window_event(build_kinesis_event(
            vec![partitioned_record(&fixture("decorator"), "event-2")],
            HashMap::new(),
            true,
        ))
        .expect("second relay batch should process");
        assert_eq!(second.parsed_batch.telemetry_items.len(), 0);
        assert_eq!(second.late_decorators_dropped, 1);
    }

    #[test]
    fn relay_emits_unlinked_targets_when_no_decorator_arrives() {
        let result = process_kinesis_time_window_event(build_kinesis_event(
            vec![partitioned_record(&fixture("target"), "event-1")],
            HashMap::new(),
            true,
        ))
        .expect("relay batch should process");

        let span = decode_single_span(&result.parsed_batch.telemetry_items[0]);
        assert_eq!(span.links.len(), 0);
    }

    #[test]
    fn response_state_round_trips_for_non_final_window() {
        let processing = process_kinesis_time_window_event(build_kinesis_event(
            vec![partitioned_record(&fixture("client"), "event-1")],
            HashMap::new(),
            false,
        ))
        .expect("relay batch should process");

        let mut response = KinesisTimeWindowEventResponse::default();
        let mut response_properties = TimeWindowEventResponseProperties::default();
        response_properties.state = processing.response_state.clone();
        response.time_window_event_response_properties = response_properties;
        assert_eq!(
            processing.response_state,
            response.time_window_event_response_properties.state
        );
    }
}
