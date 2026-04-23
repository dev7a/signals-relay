//! Stateful record reconciliation for the Signals Relay tumbling window.

use anyhow::Context;
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use tracing::warn;

use crate::{otlp, telemetry::EncodedOtlpPayload};

/// Maximum number of records that should be included in one `PutRecords` call.
pub const MAX_PUT_RECORDS: usize = 500;
const WINDOW_STATE_KEY: &str = "window_state";

/// Final OTLP payloads and trace bookkeeping emitted from a parsed batch.
#[derive(Debug, Default)]
pub struct ParsedBatch {
    /// Encoded OTLP payloads ready for export.
    pub telemetry_items: Vec<EncodedOtlpPayload>,
    /// Trace IDs emitted during this batch, useful for downstream accounting.
    pub emitted_trace_ids: BTreeSet<String>,
}

/// A source record after partitioning by trace ID and before relay-window accumulation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PartitionedSpanRecord {
    /// Original CloudWatch Logs log group for the source event.
    pub source_log_group: String,
    /// Original CloudWatch Logs event ID, used for deduplication.
    pub source_event_id: String,
    /// Trace ID used as the stream partition key and relay grouping key.
    pub trace_id: String,
    /// Raw span record payload.
    pub record: JsonValue,
}

/// A stored span record kept inside relay window state.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoredSpanRecord {
    /// Original CloudWatch Logs log group for the source event.
    pub source_log_group: String,
    /// Raw span record payload.
    pub record: JsonValue,
}

/// Accumulated links contributed by managed-link decorator records.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct PendingDecoratorLinks {
    /// Normalized link documents waiting to be merged into the target span.
    pub links: Vec<JsonValue>,
    /// Number of decorator records that contributed to this pending set.
    pub decorator_records: usize,
}

impl PendingDecoratorLinks {
    /// Merges one decorator record's links into the pending set.
    pub fn register_decorator_record(&mut self, links: &[JsonValue]) {
        self.decorator_records += 1;
        let mut merged = std::mem::take(&mut self.links);
        merged.extend(links.iter().cloned());
        self.links = normalize_links(merged);
    }
}

/// Per-trace aggregation state built during a relay tumbling window.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TraceAggregate {
    /// Completed spans that do not participate in managed-link target merging.
    pub ordinary_spans: Vec<StoredSpanRecord>,
    /// Completed spans that can accept managed-link decorator links.
    pub linkable_targets: BTreeMap<String, StoredSpanRecord>,
    /// Decorator links keyed by their eventual target span ID.
    pub pending_decorators: BTreeMap<String, PendingDecoratorLinks>,
}

/// Serializable relay state carried across tumbling-window invocations.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct RelayWindowState {
    /// Source event IDs already processed in this window.
    pub seen_event_ids: BTreeSet<String>,
    /// Per-trace aggregation state for the current window.
    pub traces: BTreeMap<String, TraceAggregate>,
}

/// Finalization result for a relay window.
#[derive(Debug, Default)]
pub struct RelayFinalizeResult {
    /// Parsed OTLP payloads emitted during finalization.
    pub parsed_batch: ParsedBatch,
    /// Count of decorator records that never found a target span in time.
    pub late_decorators_dropped: usize,
}

impl RelayWindowState {
    /// Restores relay window state from the Lambda tumbling-window response map.
    pub fn from_response_state(response_state: &HashMap<String, String>) -> anyhow::Result<Self> {
        let Some(serialized) = response_state.get(WINDOW_STATE_KEY) else {
            return Ok(Self::default());
        };

        serde_json::from_str(serialized).context("Failed to deserialize relay window state")
    }

    /// Serializes relay window state back into the Lambda response map.
    pub fn to_response_state(&self) -> anyhow::Result<HashMap<String, String>> {
        let mut response_state = HashMap::new();
        response_state.insert(
            WINDOW_STATE_KEY.to_string(),
            serde_json::to_string(self).context("Failed to serialize relay window state")?,
        );
        Ok(response_state)
    }

    /// Returns the serialized state size in bytes.
    pub fn serialized_size_bytes(&self) -> anyhow::Result<usize> {
        Ok(serde_json::to_vec(self)
            .context("Failed to serialize relay window state for sizing")?
            .len())
    }

    /// Records a source event ID and returns `true` when it was not seen before.
    pub fn record_is_new(&mut self, source_event_id: String) -> bool {
        self.seen_event_ids.insert(source_event_id)
    }

    /// Applies one partitioned span record to the in-memory window state.
    pub fn apply_record(
        &mut self,
        partitioned_record: PartitionedSpanRecord,
    ) -> anyhow::Result<()> {
        let trace = self
            .traces
            .entry(partitioned_record.trace_id.clone())
            .or_default();

        let record = partitioned_record.record;
        if is_managed_link_decorator(&record) {
            let Some(target_span_id) = decorator_target_span_id(&record) else {
                warn!("Managed-link decorator missing target_id, skipping");
                return Ok(());
            };

            trace
                .pending_decorators
                .entry(target_span_id)
                .or_default()
                .register_decorator_record(&decorator_links(&record));
            return Ok(());
        }

        if !is_completed_span(&record) {
            return Ok(());
        }

        let stored = StoredSpanRecord {
            source_log_group: partitioned_record.source_log_group,
            record,
        };

        if is_linkable_target(&stored.record) {
            let Some(span_id) = span_id_of_record(&stored.record) else {
                warn!("Linkable completed span missing spanId, skipping");
                return Ok(());
            };
            upsert_target_record(&mut trace.linkable_targets, span_id, stored);
        } else {
            trace.ordinary_spans.push(stored);
        }

        Ok(())
    }

    /// Finalizes all accumulated traces into encoded OTLP payloads.
    pub fn finalize(self) -> RelayFinalizeResult {
        let mut finalized = RelayFinalizeResult::default();

        for (trace_id, aggregate) in self.traces {
            for ordinary_span in aggregate.ordinary_spans {
                push_encoded_payload(
                    &mut finalized.parsed_batch,
                    trace_id.as_str(),
                    ordinary_span.record,
                    ordinary_span.source_log_group.as_str(),
                );
            }

            let mut pending_decorators = aggregate.pending_decorators;
            for (span_id, target) in aggregate.linkable_targets {
                let mut record = target.record;
                if let Some(pending_links) = pending_decorators.remove(&span_id) {
                    merge_links_into_target(&mut record, &pending_links.links);
                }

                push_encoded_payload(
                    &mut finalized.parsed_batch,
                    trace_id.as_str(),
                    record,
                    target.source_log_group.as_str(),
                );
            }

            finalized.late_decorators_dropped += pending_decorators
                .values()
                .map(|pending| pending.decorator_records)
                .sum::<usize>();
        }

        finalized
    }
}

fn push_encoded_payload(
    parsed_batch: &mut ParsedBatch,
    trace_id: &str,
    record: JsonValue,
    source: &str,
) {
    match build_encoded_otlp_payload(record, source) {
        Ok(item) => {
            parsed_batch.emitted_trace_ids.insert(trace_id.to_string());
            parsed_batch.telemetry_items.push(item);
        }
        Err(err) => warn!(error = %err, "Failed to convert span to OTLP, skipping record"),
    }
}

/// Converts one span record into the encoded payload wrapper used by exporters.
pub fn build_encoded_otlp_payload(
    record: JsonValue,
    source: &str,
) -> anyhow::Result<EncodedOtlpPayload> {
    otlp::convert_span_to_otlp_payload(record, source.to_string())
}

/// Returns the record trace ID, if present.
pub fn trace_id_of_record(record: &JsonValue) -> Option<String> {
    record.get("traceId")?.as_str().map(str::to_string)
}

/// Returns the record span ID, if present.
pub fn span_id_of_record(record: &JsonValue) -> Option<String> {
    record.get("spanId")?.as_str().map(str::to_string)
}

/// Computes a rough richness score for choosing the better duplicate target span.
pub fn record_richness(record: &JsonValue) -> usize {
    let Some(record) = record.as_object() else {
        return 0;
    };

    record
        .get("attributes")
        .and_then(JsonValue::as_object)
        .map(|attrs| attrs.len())
        .unwrap_or_default()
        + record
            .get("resource")
            .and_then(|value| value.get("attributes"))
            .and_then(JsonValue::as_object)
            .map(|attrs| attrs.len())
            .unwrap_or_default()
        + record
            .get("links")
            .and_then(JsonValue::as_array)
            .map(|links| links.len())
            .unwrap_or_default()
        + record
            .get("events")
            .and_then(JsonValue::as_array)
            .map(|events| events.len())
            .unwrap_or_default()
}

/// Merges normalized links into a target span record.
pub fn merge_links_into_target(target: &mut JsonValue, links: &[JsonValue]) {
    let Some(target_object) = target.as_object_mut() else {
        return;
    };

    let existing = target_object
        .remove("links")
        .and_then(|value| value.as_array().cloned())
        .unwrap_or_default();

    let mut merged = existing;
    merged.extend(links.iter().cloned());
    let normalized = normalize_links(merged);
    target_object.insert("links".to_string(), JsonValue::Array(normalized));
}

/// Returns `true` when the record contains an end timestamp.
pub fn is_completed_span(record: &JsonValue) -> bool {
    record
        .get("endTimeUnixNano")
        .map(|value| !value.is_null())
        .unwrap_or(false)
}

/// Returns `true` when the record can receive managed-link decorator links.
pub fn is_linkable_target(record: &JsonValue) -> bool {
    record
        .get("_aws")
        .and_then(|value| value.get("xray"))
        .and_then(|value| value.get("linking"))
        .and_then(JsonValue::as_bool)
        .unwrap_or(false)
        || record
            .get("attributes")
            .and_then(JsonValue::as_object)
            .and_then(|attrs| attrs.get("aws.internal"))
            .and_then(JsonValue::as_object)
            .and_then(|internal| internal.get("linking"))
            .and_then(JsonValue::as_bool)
            .unwrap_or(false)
}

/// Returns `true` when the record is a managed-link decorator span.
pub fn is_managed_link_decorator(record: &JsonValue) -> bool {
    record
        .get("_aws")
        .and_then(|value| value.get("xray"))
        .and_then(|value| value.get("decorator_type"))
        .and_then(JsonValue::as_str)
        == Some("managed_link")
}

/// Extracts the managed-link target span ID from a decorator record.
pub fn decorator_target_span_id(record: &JsonValue) -> Option<String> {
    record
        .get("_aws")
        .and_then(|value| value.get("xray"))
        .and_then(|value| value.get("target_id"))
        .and_then(JsonValue::as_str)
        .map(str::to_string)
}

/// Extracts the links carried by a decorator record.
pub fn decorator_links(record: &JsonValue) -> Vec<JsonValue> {
    record
        .get("links")
        .and_then(JsonValue::as_array)
        .cloned()
        .unwrap_or_default()
}

fn upsert_target_record(
    targets: &mut BTreeMap<String, StoredSpanRecord>,
    span_id: String,
    record: StoredSpanRecord,
) {
    match targets.get(&span_id) {
        Some(existing) if record_richness(&existing.record) >= record_richness(&record.record) => {}
        _ => {
            targets.insert(span_id, record);
        }
    }
}

fn normalize_links(mut links: Vec<JsonValue>) -> Vec<JsonValue> {
    links.retain(|link| {
        link.get("traceId").and_then(JsonValue::as_str).is_some()
            && link.get("spanId").and_then(JsonValue::as_str).is_some()
    });

    links.sort_by(|left, right| {
        let left_trace = left
            .get("traceId")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let left_span = left
            .get("spanId")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let right_trace = right
            .get("traceId")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let right_span = right
            .get("spanId")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();

        (left_trace, left_span).cmp(&(right_trace, right_span))
    });

    let mut deduped = Vec::new();
    let mut seen = BTreeSet::new();
    for link in links {
        let trace_id = link
            .get("traceId")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        let span_id = link
            .get("spanId")
            .and_then(JsonValue::as_str)
            .unwrap_or_default();
        if seen.insert(format!("{trace_id}:{span_id}")) {
            deduped.push(link);
        }
    }

    deduped
}

#[cfg(test)]
mod tests {
    use super::*;
    use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
    use prost::Message;
    use serde_json::json;

    fn linkable_target_fixture() -> JsonValue {
        serde_json::from_str(include_str!(
            "../tests/fixtures/completed_linkable_target.json"
        ))
        .expect("target fixture should parse")
    }

    fn decorator_fixture() -> JsonValue {
        serde_json::from_str(include_str!(
            "../tests/fixtures/managed_link_decorator.json"
        ))
        .expect("decorator fixture should parse")
    }

    #[test]
    fn finalize_merges_links_into_linkable_target() {
        let mut state = RelayWindowState::default();
        let trace_id =
            trace_id_of_record(&linkable_target_fixture()).expect("trace id should exist");

        state.record_is_new("event-1".to_string());
        state
            .apply_record(PartitionedSpanRecord {
                source_log_group: "aws/spans".to_string(),
                source_event_id: "event-1".to_string(),
                trace_id: trace_id.clone(),
                record: linkable_target_fixture(),
            })
            .expect("target record should apply");
        state.record_is_new("event-2".to_string());
        state
            .apply_record(PartitionedSpanRecord {
                source_log_group: "aws/spans".to_string(),
                source_event_id: "event-2".to_string(),
                trace_id: trace_id.clone(),
                record: decorator_fixture(),
            })
            .expect("decorator record should apply");

        let finalized = state.finalize();

        assert_eq!(finalized.late_decorators_dropped, 0);
        assert_eq!(finalized.parsed_batch.telemetry_items.len(), 1);

        let request = ExportTraceServiceRequest::decode(
            finalized.parsed_batch.telemetry_items[0].payload.as_slice(),
        )
        .expect("payload should decode");
        assert_eq!(
            request.resource_spans[0].scope_spans[0].spans[0]
                .links
                .len(),
            1
        );
    }

    #[test]
    fn finalize_counts_late_decorators_without_target() {
        let decorator = decorator_fixture();
        let trace_id = trace_id_of_record(&decorator).expect("trace id should exist");
        let mut state = RelayWindowState::default();

        state.record_is_new("event-1".to_string());
        state
            .apply_record(PartitionedSpanRecord {
                source_log_group: "aws/spans".to_string(),
                source_event_id: "event-1".to_string(),
                trace_id,
                record: decorator,
            })
            .expect("decorator record should apply");

        let finalized = state.finalize();
        assert_eq!(finalized.parsed_batch.telemetry_items.len(), 0);
        assert_eq!(finalized.late_decorators_dropped, 1);
    }

    #[test]
    fn merge_links_into_target_deduplicates_by_trace_and_span() {
        let mut target = json!({});
        let links = vec![
            json!({"traceId": "t1", "spanId": "s1"}),
            json!({"traceId": "t1", "spanId": "s1", "flags": 1}),
            json!({"traceId": "t2", "spanId": "s2"}),
        ];

        merge_links_into_target(&mut target, &links);

        let stored_links = target
            .get("links")
            .and_then(JsonValue::as_array)
            .expect("links should exist");
        assert_eq!(stored_links.len(), 2);
    }
}
