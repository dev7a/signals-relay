pub use signals_relay_core::state::{
    build_encoded_otlp_payload, decorator_links, decorator_target_span_id, is_completed_span,
    is_linkable_target, is_managed_link_decorator, merge_links_into_target, record_richness,
    span_id_of_record, trace_id_of_record, ParsedBatch, PartitionedSpanRecord,
    PendingDecoratorLinks, RelayFinalizeResult, RelayWindowState, StoredSpanRecord, TraceAggregate,
    MAX_PUT_RECORDS,
};
