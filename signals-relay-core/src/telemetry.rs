#[derive(Clone, Debug, PartialEq, Eq)]
pub struct EncodedOtlpPayload {
    pub source: String,
    pub payload: Vec<u8>,
    pub content_type: String,
    pub content_encoding: Option<String>,
}

impl EncodedOtlpPayload {
    pub fn new(source: impl Into<String>, payload: Vec<u8>) -> Self {
        Self {
            source: source.into(),
            payload,
            content_type: "application/x-protobuf".to_string(),
            content_encoding: None,
        }
    }
}
