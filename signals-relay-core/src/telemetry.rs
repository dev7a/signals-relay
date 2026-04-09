//! Encoded telemetry payload types passed to exporters.

/// Encoded OTLP payload plus the transport metadata needed by callers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedOtlpPayload {
    /// Logical source identifier for the record batch, such as `aws/spans`.
    pub source: String,
    /// Serialized payload bytes.
    pub payload: Vec<u8>,
    /// MIME type to send with the payload.
    pub content_type: String,
    /// Optional content encoding, for example `gzip`.
    pub content_encoding: Option<String>,
}

impl EncodedOtlpPayload {
    /// Creates a new protobuf OTLP payload wrapper with default content metadata.
    pub fn new(source: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            source: source.into(),
            payload,
            content_type: "application/x-protobuf".to_string(),
            content_encoding: None,
        }
    }
}
