pub mod otlp;
pub mod state;
pub mod telemetry;

pub use otlp::convert_span_to_otlp_payload;
pub use state::{
    decorator_links, decorator_target_span_id, is_completed_span, is_linkable_target,
    is_managed_link_decorator, merge_links_into_target, record_richness, span_id_of_record,
    trace_id_of_record, ParsedBatch, PartitionedSpanRecord, PendingDecoratorLinks,
    RelayFinalizeResult, RelayWindowState, StoredSpanRecord, MAX_PUT_RECORDS,
};
pub use telemetry::EncodedOtlpPayload;
