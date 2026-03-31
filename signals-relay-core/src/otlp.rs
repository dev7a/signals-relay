use anyhow::{Context, Result};
use opentelemetry_proto::tonic::{
    collector::trace::v1::ExportTraceServiceRequest,
    common::v1::{any_value, AnyValue, ArrayValue, KeyValue, KeyValueList},
    resource::v1::Resource,
    trace::v1::{
        span::{Event, Link, SpanKind},
        status::StatusCode,
        ResourceSpans, ScopeSpans, Span, Status,
    },
};
use prost::Message;
use serde_json::{Map, Value};

use crate::telemetry::EncodedOtlpPayload;

/// Decodes a hex string to bytes.
fn decode_hex(value: &str) -> Result<Vec<u8>> {
    let normalized = value.trim_start_matches("0x");
    let normalized: String = normalized
        .chars()
        .filter(|ch| ch.is_ascii_hexdigit())
        .collect();

    let mut bytes = Vec::with_capacity(normalized.len() / 2);
    for idx in (0..normalized.len()).step_by(2) {
        if idx + 2 <= normalized.len() {
            let byte = u8::from_str_radix(&normalized[idx..idx + 2], 16)
                .map_err(|err| anyhow::anyhow!("Invalid hex string: {err}"))?;
            bytes.push(byte);
        } else if idx + 1 == normalized.len() {
            let byte = u8::from_str_radix(&format!("{}0", &normalized[idx..idx + 1]), 16)
                .map_err(|err| anyhow::anyhow!("Invalid hex string: {err}"))?;
            bytes.push(byte);
        }
    }

    Ok(bytes)
}

fn map_status_code(code: &str) -> StatusCode {
    match code.to_uppercase().as_str() {
        "OK" => StatusCode::Ok,
        "ERROR" => StatusCode::Error,
        _ => StatusCode::Unset,
    }
}

fn map_span_kind(kind: &str) -> SpanKind {
    match kind.to_uppercase().as_str() {
        "INTERNAL" => SpanKind::Internal,
        "SERVER" => SpanKind::Server,
        "CLIENT" => SpanKind::Client,
        "PRODUCER" => SpanKind::Producer,
        "CONSUMER" => SpanKind::Consumer,
        _ => SpanKind::Unspecified,
    }
}

fn convert_value(value: &Value) -> AnyValue {
    match value {
        Value::Bool(boolean) => AnyValue {
            value: Some(any_value::Value::BoolValue(*boolean)),
        },
        Value::Number(number) => {
            if let Some(int_value) = number.as_i64() {
                AnyValue {
                    value: Some(any_value::Value::IntValue(int_value)),
                }
            } else {
                AnyValue {
                    value: Some(any_value::Value::DoubleValue(
                        number.as_f64().unwrap_or_default(),
                    )),
                }
            }
        }
        Value::Array(values) => AnyValue {
            value: Some(any_value::Value::ArrayValue(ArrayValue {
                values: values.iter().map(convert_value).collect(),
            })),
        },
        Value::Object(object) => AnyValue {
            value: Some(any_value::Value::KvlistValue(KeyValueList {
                values: convert_attributes(object),
            })),
        },
        Value::String(text) => AnyValue {
            value: Some(any_value::Value::StringValue(text.clone())),
        },
        Value::Null => AnyValue {
            value: Some(any_value::Value::StringValue("null".to_string())),
        },
    }
}

fn convert_attributes(attrs: &Map<String, Value>) -> Vec<KeyValue> {
    attrs
        .iter()
        .map(|(key, value)| KeyValue {
            key: key.clone(),
            value: Some(convert_value(value)),
        })
        .collect()
}

fn convert_events(record: &Map<String, Value>) -> Vec<Event> {
    record
        .get("events")
        .and_then(Value::as_array)
        .map(|events| {
            events
                .iter()
                .filter_map(|event| {
                    let event = event.as_object()?;
                    Some(Event {
                        time_unix_nano: event
                            .get("timeUnixNano")
                            .and_then(Value::as_u64)
                            .unwrap_or_default(),
                        name: event
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        attributes: event
                            .get("attributes")
                            .and_then(Value::as_object)
                            .map(convert_attributes)
                            .unwrap_or_default(),
                        dropped_attributes_count: 0,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn convert_link_attributes(link: &Map<String, Value>) -> Vec<KeyValue> {
    let mut attributes = link
        .get("attributes")
        .and_then(Value::as_object)
        .map(convert_attributes)
        .unwrap_or_default();

    if let Some(reference_type) = link
        .get("_aws")
        .and_then(|value| value.get("xray"))
        .and_then(|value| value.get("reference_type"))
        .and_then(Value::as_str)
    {
        attributes.push(KeyValue {
            key: "aws.xray.reference_type".to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(reference_type.to_string())),
            }),
        });
    }

    attributes
}

fn convert_links(record: &Map<String, Value>) -> Vec<Link> {
    record
        .get("links")
        .and_then(Value::as_array)
        .map(|links| {
            links
                .iter()
                .filter_map(|link| {
                    let link = link.as_object()?;
                    let trace_id = decode_hex(link.get("traceId")?.as_str()?).ok()?;
                    let span_id = decode_hex(link.get("spanId")?.as_str()?).ok()?;

                    Some(Link {
                        trace_id,
                        span_id,
                        trace_state: String::new(),
                        attributes: convert_link_attributes(link),
                        dropped_attributes_count: 0,
                        flags: link
                            .get("flags")
                            .and_then(Value::as_u64)
                            .unwrap_or_default() as u32,
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}

fn encode_span_to_otlp_protobuf(record: Value) -> Result<Vec<u8>> {
    let record = record.as_object().context("Record is not an object")?;
    let empty_map = Map::new();

    let end_time = record
        .get("endTimeUnixNano")
        .and_then(|value| {
            if value.is_null() {
                None
            } else {
                value.as_u64()
            }
        })
        .context("Missing or invalid endTimeUnixNano")?;

    let span_attrs = record
        .get("attributes")
        .and_then(Value::as_object)
        .unwrap_or(&empty_map)
        .clone();

    let mut resource_attrs = record
        .get("resource")
        .and_then(|value| value.get("attributes"))
        .and_then(Value::as_object)
        .unwrap_or(&empty_map)
        .clone();

    if !resource_attrs.contains_key("service.name") {
        if let Some(service_name) = span_attrs
            .get("aws.local.service")
            .and_then(Value::as_str)
            .or_else(|| span_attrs.get("service.name").and_then(Value::as_str))
        {
            resource_attrs.insert(
                "service.name".to_string(),
                Value::String(service_name.to_string()),
            );
        }
    }

    let scope = record
        .get("scope")
        .and_then(Value::as_object)
        .unwrap_or(&empty_map);

    let trace_id = record
        .get("traceId")
        .and_then(Value::as_str)
        .map(decode_hex)
        .transpose()
        .context("Invalid traceId format")?
        .unwrap_or_default();

    let span_id = record
        .get("spanId")
        .and_then(Value::as_str)
        .map(decode_hex)
        .transpose()
        .context("Invalid spanId format")?
        .unwrap_or_default();

    let parent_span_id = record
        .get("parentSpanId")
        .and_then(Value::as_str)
        .map(decode_hex)
        .transpose()
        .context("Invalid parentSpanId format")?
        .unwrap_or_default();

    let span_name = record
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.is_empty())
        .or_else(|| {
            record
                .get("_aws")
                .and_then(|value| value.get("xray"))
                .and_then(|value| value.get("name"))
                .and_then(Value::as_str)
                .filter(|name| !name.is_empty())
        })
        .unwrap_or("UnnamedSpan")
        .to_string();

    let span = Span {
        trace_id,
        span_id,
        parent_span_id,
        name: span_name,
        kind: record
            .get("kind")
            .and_then(Value::as_str)
            .map(map_span_kind)
            .unwrap_or(SpanKind::Unspecified) as i32,
        start_time_unix_nano: record
            .get("startTimeUnixNano")
            .and_then(Value::as_u64)
            .unwrap_or_default(),
        end_time_unix_nano: end_time,
        attributes: convert_attributes(&span_attrs),
        status: Some(Status {
            code: record
                .get("status")
                .and_then(|value| value.get("code"))
                .and_then(Value::as_str)
                .map(map_status_code)
                .unwrap_or(StatusCode::Unset) as i32,
            message: record
                .get("status")
                .and_then(|value| value.get("message"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_string(),
        }),
        events: convert_events(record),
        links: convert_links(record),
        dropped_attributes_count: 0,
        dropped_events_count: 0,
        dropped_links_count: 0,
        flags: record
            .get("flags")
            .and_then(Value::as_u64)
            .unwrap_or_default() as u32,
        ..Default::default()
    };

    let request = ExportTraceServiceRequest {
        resource_spans: vec![ResourceSpans {
            resource: Some(Resource {
                attributes: convert_attributes(&resource_attrs),
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            }),
            scope_spans: vec![ScopeSpans {
                scope: Some(
                    opentelemetry_proto::tonic::common::v1::InstrumentationScope {
                        name: scope
                            .get("name")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        version: scope
                            .get("version")
                            .and_then(Value::as_str)
                            .unwrap_or_default()
                            .to_string(),
                        attributes: Vec::new(),
                        dropped_attributes_count: 0,
                    },
                ),
                spans: vec![span],
                schema_url: String::new(),
            }],
            schema_url: String::new(),
        }],
    };

    Ok(request.encode_to_vec())
}

pub fn convert_span_to_otlp_payload(
    record: Value,
    source: impl Into<String>,
) -> Result<EncodedOtlpPayload> {
    let payload = encode_span_to_otlp_protobuf(record)?;
    Ok(EncodedOtlpPayload::new(source, payload))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn decode_payload(record: Value) -> ExportTraceServiceRequest {
        let payload =
            convert_span_to_otlp_payload(record, "aws/spans").expect("record should convert");
        ExportTraceServiceRequest::decode(payload.payload.as_slice())
            .expect("payload should decode")
    }

    fn linkable_target_fixture() -> Value {
        serde_json::from_str(include_str!(
            "../tests/fixtures/completed_linkable_target.json"
        ))
        .expect("target fixture should parse")
    }

    fn managed_link_decorator_fixture() -> Value {
        serde_json::from_str(include_str!(
            "../tests/fixtures/managed_link_decorator.json"
        ))
        .expect("decorator fixture should parse")
    }

    fn client_span_fixture() -> Value {
        serde_json::from_str(include_str!("../tests/fixtures/normal_client_span.json"))
            .expect("client span fixture should parse")
    }

    #[test]
    fn test_decode_hex() {
        let result = decode_hex("0123456789abcdef").unwrap();
        assert_eq!(result, vec![0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef]);

        let prefixed = decode_hex("0x0123456789abcdef").unwrap();
        assert_eq!(prefixed, result);

        let with_separators = decode_hex("01-23-45-67-89-ab-cd-ef").unwrap();
        assert_eq!(with_separators, result);
    }

    #[test]
    fn test_convert_span_to_otlp_payload_missing_endtime() {
        let span = json!({
            "name": "test-span",
            "traceId": "0123456789abcdef0123456789abcdef",
            "spanId": "0123456789abcdef"
        });

        assert!(convert_span_to_otlp_payload(span, "aws/spans").is_err());
    }

    #[test]
    fn test_convert_span_to_otlp_payload_complete() {
        let span = json!({
            "name": "test-span",
            "traceId": "0123456789abcdef0123456789abcdef",
            "spanId": "0123456789abcdef",
            "parentSpanId": "fedcba9876543210",
            "kind": "SERVER",
            "startTimeUnixNano": 1619712000000000000_u64,
            "endTimeUnixNano": 1619712001000000000_u64,
            "attributes": {
                "http.method": "GET",
                "http.url": "https://example.com",
                "http.status_code": 200
            },
            "status": {
                "code": "OK",
                "message": "ok"
            },
            "resource": {
                "attributes": {
                    "service.name": "test-service",
                    "service.version": "1.0.0"
                }
            },
            "scope": {
                "name": "test-scope",
                "version": "1.0.0"
            }
        });

        let request = decode_payload(span);
        let span = &request.resource_spans[0].scope_spans[0].spans[0];

        assert_eq!(span.name, "test-span");
        assert_eq!(span.kind, SpanKind::Server as i32);
        assert_eq!(span.status.as_ref().unwrap().message, "ok");
        assert_eq!(
            span.trace_id,
            vec![
                0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab,
                0xcd, 0xef
            ]
        );
    }

    #[test]
    fn test_converts_nested_attributes_to_kvlist() {
        let request = decode_payload(linkable_target_fixture());
        let span = &request.resource_spans[0].scope_spans[0].spans[0];
        let internal_attr = span
            .attributes
            .iter()
            .find(|attr| attr.key == "aws.internal")
            .expect("aws.internal should exist");

        match internal_attr
            .value
            .as_ref()
            .and_then(|value| value.value.as_ref())
            .expect("aws.internal should have a value")
        {
            any_value::Value::KvlistValue(list) => {
                assert_eq!(list.values.len(), 1);
                assert_eq!(list.values[0].key, "linking");
            }
            other => panic!("expected kvlist value, got {other:?}"),
        }
    }

    #[test]
    fn test_preserves_flags_from_input_span() {
        let request = decode_payload(client_span_fixture());
        let span = &request.resource_spans[0].scope_spans[0].spans[0];
        assert_eq!(span.flags, 256);
    }

    #[test]
    fn test_projects_aws_local_service_to_resource_service_name() {
        let span = json!({
            "name": "test-span",
            "traceId": "0123456789abcdef0123456789abcdef",
            "spanId": "0123456789abcdef",
            "kind": "SERVER",
            "startTimeUnixNano": 1619712000000000000_u64,
            "endTimeUnixNano": 1619712001000000000_u64,
            "resource": { "attributes": {} },
            "attributes": {
                "aws.local.service": "demo-service"
            }
        });

        let request = decode_payload(span);
        let resource = request.resource_spans[0].resource.as_ref().unwrap();
        let service_name = resource
            .attributes
            .iter()
            .find(|attr| attr.key == "service.name")
            .expect("resource service.name should be projected");

        assert_eq!(
            service_name
                .value
                .as_ref()
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::StringValue("demo-service".to_string()))
        );
    }

    #[test]
    fn test_converts_links_from_merged_record() {
        let mut target = linkable_target_fixture();
        let decorator = managed_link_decorator_fixture();
        target
            .as_object_mut()
            .expect("fixture should be an object")
            .insert("links".to_string(), decorator["links"].clone());

        let request = decode_payload(target);
        let span = &request.resource_spans[0].scope_spans[0].spans[0];
        assert_eq!(span.links.len(), 1);
        assert_eq!(
            span.links[0]
                .attributes
                .iter()
                .find(|attr| attr.key == "aws.xray.reference_type")
                .and_then(|attr| attr.value.as_ref())
                .and_then(|value| value.value.as_ref()),
            Some(&any_value::Value::StringValue("parent".to_string()))
        );
    }

    #[test]
    fn test_span_name_fallback() {
        let span = json!({
            "name": "",
            "traceId": "0123456789abcdef0123456789abcdef",
            "spanId": "0123456789abcdef",
            "startTimeUnixNano": 1619712000000000000_u64,
            "endTimeUnixNano": 1619712001000000000_u64,
            "_aws": {
                "xray": {
                    "name": "XRay Span Name"
                }
            }
        });

        let request = decode_payload(span);
        assert_eq!(
            request.resource_spans[0].scope_spans[0].spans[0].name,
            "XRay Span Name"
        );
    }
}
