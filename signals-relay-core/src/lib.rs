//! Reusable span-reconciliation and OTLP encoding logic for Signals Relay.
//!
//! The crate is split into three small surfaces:
//!
//! - [`state`] manages tumbling-window accumulation and managed-link decorator merging
//! - [`otlp`] converts reconciled span records into OTLP protobuf payloads
//! - [`telemetry`] defines the encoded payload container consumed by callers

/// OTLP payload conversion helpers.
pub mod otlp;
/// Relay window state and span-reconciliation helpers.
pub mod state;
/// Encoded telemetry payload types returned by the conversion layer.
pub mod telemetry;

pub use otlp::convert_span_to_otlp_payload;
pub use state::{
    decorator_links, decorator_target_span_id, is_completed_span, is_linkable_target,
    is_managed_link_decorator, merge_links_into_target, record_richness, span_id_of_record,
    trace_id_of_record, ParsedBatch, PartitionedSpanRecord, PendingDecoratorLinks,
    RelayFinalizeResult, RelayWindowState, StoredSpanRecord, MAX_PUT_RECORDS,
};
pub use telemetry::EncodedOtlpPayload;
