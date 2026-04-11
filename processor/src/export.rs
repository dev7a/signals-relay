use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use aws_sdk_secretsmanager::Client as SecretsManagerClient;
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue, CONTENT_ENCODING, CONTENT_TYPE},
    Url,
};
use serde::{Deserialize, Serialize};
use serverless_otlp_forwarder_core::{
    http_sender::{send_telemetry_batch, HttpOtlpForwarderClient},
    TelemetryData,
};
use std::{
    collections::{BTreeMap, HashMap},
    env,
    str::FromStr,
    time::Duration,
};
use tracing::warn;

const COLLECTOR_LOCAL_OTLP_ENDPOINT: &str = "http://localhost:4318/v1/traces";
const DEFAULT_OTLP_EXPORT_TIMEOUT: Duration = Duration::from_secs(10);
const DEFAULT_OTLP_TARGET_SECRET_ID: &str = "signals-relay/secrets/collector";
const OTLP_TRACES_PATH: &str = "/v1/traces";
const OTLP_TARGET_SECRET_ID_ENV: &str = "OTLP_TARGET_SECRET_ID";
const COLLECTOR_CONFIG_URI_ENV: &str = "OPENTELEMETRY_COLLECTOR_CONFIG_URI";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelayExportTarget {
    Collector,
    Direct(ResolvedOtlpTarget),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedOtlpTarget {
    endpoint: String,
    headers: BTreeMap<String, String>,
}

impl ResolvedOtlpTarget {
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }

    pub fn header_map_for_exporter(&self) -> HashMap<String, String> {
        self.headers.clone().into_iter().collect()
    }

    fn header_map_for_request(&self) -> Result<HeaderMap> {
        let mut headers = HeaderMap::new();
        for (name, value) in &self.headers {
            let header_name = HeaderName::from_str(name)
                .with_context(|| format!("Invalid OTLP header name '{name}'"))?;
            let header_value = HeaderValue::from_str(value)
                .with_context(|| format!("Invalid OTLP header value for '{name}'"))?;
            headers.append(header_name, header_value);
        }
        Ok(headers)
    }
}

#[derive(Debug, Deserialize, Serialize)]
struct OtlpTargetSecretDocument {
    endpoint: String,
    #[serde(default)]
    headers: BTreeMap<String, String>,
}

#[async_trait]
pub trait OtlpTargetSecretProvider: Send + Sync {
    async fn get_secret_string(&self, secret_id: &str) -> Result<String>;
}

pub struct AwsOtlpTargetSecretProvider {
    client: SecretsManagerClient,
}

impl AwsOtlpTargetSecretProvider {
    pub fn new(client: SecretsManagerClient) -> Self {
        Self { client }
    }
}

#[async_trait]
impl OtlpTargetSecretProvider for AwsOtlpTargetSecretProvider {
    async fn get_secret_string(&self, secret_id: &str) -> Result<String> {
        let response = self
            .client
            .get_secret_value()
            .secret_id(secret_id)
            .send()
            .await
            .with_context(|| format!("Failed to fetch OTLP target secret '{secret_id}'"))?;

        response
            .secret_string()
            .map(str::to_string)
            .ok_or_else(|| anyhow!("Secret '{secret_id}' did not contain a SecretString value"))
    }
}

pub async fn resolve_relay_export_target() -> Result<RelayExportTarget> {
    if collector_mode_enabled()? {
        return Ok(RelayExportTarget::Collector);
    }

    let aws_config = aws_config::load_from_env().await;
    let provider = AwsOtlpTargetSecretProvider::new(SecretsManagerClient::new(&aws_config));
    resolve_relay_export_target_with_provider(&provider).await
}

pub async fn resolve_relay_export_target_with_provider(
    provider: &impl OtlpTargetSecretProvider,
) -> Result<RelayExportTarget> {
    if collector_mode_enabled()? {
        return Ok(RelayExportTarget::Collector);
    }

    let secret_id = optional_env(OTLP_TARGET_SECRET_ID_ENV)?
        .unwrap_or_else(|| DEFAULT_OTLP_TARGET_SECRET_ID.to_string());
    let target = resolve_direct_otlp_target(provider, &secret_id).await?;
    Ok(RelayExportTarget::Direct(target))
}

pub async fn send_compacted_telemetry_batch(
    client: &impl HttpOtlpForwarderClient,
    telemetry_data: TelemetryData,
    export_target: &RelayExportTarget,
) -> Result<()> {
    match export_target {
        RelayExportTarget::Collector => send_telemetry_batch(client, telemetry_data).await,
        RelayExportTarget::Direct(target) => {
            send_direct_telemetry_batch(client, telemetry_data, target).await
        }
    }
}

pub fn collector_local_otlp_endpoint() -> &'static str {
    COLLECTOR_LOCAL_OTLP_ENDPOINT
}

async fn resolve_direct_otlp_target(
    provider: &impl OtlpTargetSecretProvider,
    secret_id: &str,
) -> Result<ResolvedOtlpTarget> {
    let secret_string = provider.get_secret_string(secret_id).await?;
    parse_target_secret_document(&secret_string)
}

fn parse_target_secret_document(secret_string: &str) -> Result<ResolvedOtlpTarget> {
    let document: OtlpTargetSecretDocument =
        serde_json::from_str(secret_string).context("Failed to parse OTLP target secret JSON")?;
    let endpoint = normalize_otlp_traces_endpoint(document.endpoint.trim())?;
    let resolved = ResolvedOtlpTarget {
        endpoint: endpoint.to_string(),
        headers: document.headers,
    };
    resolved.header_map_for_request()?;
    Ok(resolved)
}

fn normalize_otlp_traces_endpoint(raw_endpoint: &str) -> Result<Url> {
    if raw_endpoint.is_empty() {
        return Err(anyhow!("OTLP target secret endpoint must not be empty"));
    }

    let mut url = Url::parse(raw_endpoint)
        .with_context(|| format!("Invalid OTLP target endpoint '{raw_endpoint}'"))?;
    let current_path = url.path();
    if !current_path.ends_with(OTLP_TRACES_PATH) {
        let new_path = if current_path == "/" || current_path.is_empty() {
            OTLP_TRACES_PATH.to_string()
        } else {
            format!("{}{}", current_path.trim_end_matches('/'), OTLP_TRACES_PATH)
        };
        url.set_path(&new_path);
    }
    Ok(url)
}

fn optional_env(name: &str) -> Result<Option<String>> {
    match env::var(name) {
        Ok(value) => {
            if value.trim().is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(anyhow!(err)).context(format!("Failed to read {name}")),
    }
}

fn collector_mode_enabled() -> Result<bool> {
    match env::var(COLLECTOR_CONFIG_URI_ENV) {
        Ok(value) => {
            if value.trim().is_empty() {
                return Err(anyhow!("{COLLECTOR_CONFIG_URI_ENV} must not be empty"));
            }
            Ok(true)
        }
        Err(env::VarError::NotPresent) => Ok(false),
        Err(err) => Err(anyhow!(err)).context(format!("Failed to read {COLLECTOR_CONFIG_URI_ENV}")),
    }
}

fn resolve_otlp_timeout() -> Duration {
    for env_name in [
        "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT",
        "OTEL_EXPORTER_OTLP_TIMEOUT",
    ] {
        if let Ok(value) = env::var(env_name) {
            if value.is_empty() {
                continue;
            }

            match value.parse::<u64>() {
                Ok(millis) => return Duration::from_millis(millis),
                Err(err) => warn!(
                    invalid_value = %value,
                    %env_name,
                    error = %err,
                    "Invalid OTLP timeout override; using default"
                ),
            }
        }
    }

    DEFAULT_OTLP_EXPORT_TIMEOUT
}

async fn send_direct_telemetry_batch(
    client: &impl HttpOtlpForwarderClient,
    telemetry_data: TelemetryData,
    target: &ResolvedOtlpTarget,
) -> Result<()> {
    let mut headers = target.header_map_for_request()?;
    headers.insert(
        CONTENT_TYPE,
        HeaderValue::from_str(&telemetry_data.content_type)
            .context("Invalid Content-Type in TelemetryData")?,
    );

    if let Some(encoding) = &telemetry_data.content_encoding {
        headers.insert(
            CONTENT_ENCODING,
            HeaderValue::from_str(encoding).context("Invalid Content-Encoding in TelemetryData")?,
        );
    } else {
        headers.remove(CONTENT_ENCODING);
    }

    let response = client
        .post_telemetry(
            Url::parse(&target.endpoint)
                .with_context(|| format!("Invalid resolved OTLP endpoint '{}'", target.endpoint))?,
            headers,
            telemetry_data.payload.into(),
            resolve_otlp_timeout(),
        )
        .await?;

    if response.status().is_success() {
        return Ok(());
    }

    let status = response.status();
    let body = response.into_body();
    Err(anyhow!(
        "OTLP export failed to {}. Status: {}. Body: {}",
        target.endpoint,
        status,
        body
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc, Mutex, OnceLock,
    };

    static TEST_MUTEX: OnceLock<Mutex<()>> = OnceLock::new();

    struct EnvGuard {
        previous: Vec<(&'static str, Option<String>)>,
    }

    impl EnvGuard {
        fn set(pairs: &[(&'static str, &'static str)]) -> Self {
            let tracked = [
                OTLP_TARGET_SECRET_ID_ENV,
                COLLECTOR_CONFIG_URI_ENV,
                "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT",
                "OTEL_EXPORTER_OTLP_TIMEOUT",
            ];

            let previous = tracked
                .into_iter()
                .map(|name| (name, env::var(name).ok()))
                .collect::<Vec<_>>();

            for (name, value) in pairs {
                unsafe { env::set_var(name, value) };
            }

            Self { previous }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            for (name, previous) in self.previous.drain(..) {
                match previous {
                    Some(value) => unsafe { env::set_var(name, value) },
                    None => unsafe { env::remove_var(name) },
                }
            }
        }
    }

    #[derive(Clone)]
    struct FakeSecretProvider {
        secret_string: String,
        fetch_count: Arc<AtomicUsize>,
        requested_secret_ids: Arc<Mutex<Vec<String>>>,
    }

    #[async_trait]
    impl OtlpTargetSecretProvider for FakeSecretProvider {
        async fn get_secret_string(&self, secret_id: &str) -> Result<String> {
            self.fetch_count.fetch_add(1, Ordering::SeqCst);
            self.requested_secret_ids
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .push(secret_id.to_string());
            Ok(self.secret_string.clone())
        }
    }

    #[tokio::test]
    async fn parses_direct_secret_document_with_headers() {
        let _guard = TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _env = EnvGuard::set(&[(
            OTLP_TARGET_SECRET_ID_ENV,
            "arn:aws:secretsmanager:us-east-1:111111111111:secret:test",
        )]);
        let provider = FakeSecretProvider {
            secret_string: json!({
                "endpoint": "https://example.com",
                "headers": {
                    "authorization": "Bearer token",
                    "x-api-key": "secret"
                }
            })
            .to_string(),
            fetch_count: Arc::new(AtomicUsize::new(0)),
            requested_secret_ids: Arc::new(Mutex::new(Vec::new())),
        };

        let target = resolve_relay_export_target_with_provider(&provider)
            .await
            .unwrap();

        let RelayExportTarget::Direct(target) = target else {
            panic!("expected direct target");
        };
        assert_eq!(target.endpoint(), "https://example.com/v1/traces");
        assert_eq!(target.header_map_for_exporter().len(), 2);
    }

    #[tokio::test]
    async fn rejects_missing_endpoint_in_direct_secret() {
        let _guard = TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _env = EnvGuard::set(&[(
            OTLP_TARGET_SECRET_ID_ENV,
            "arn:aws:secretsmanager:us-east-1:111111111111:secret:test",
        )]);
        let provider = FakeSecretProvider {
            secret_string: json!({
                "headers": {
                    "authorization": "Bearer token"
                }
            })
            .to_string(),
            fetch_count: Arc::new(AtomicUsize::new(0)),
            requested_secret_ids: Arc::new(Mutex::new(Vec::new())),
        };

        let err = resolve_relay_export_target_with_provider(&provider)
            .await
            .unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[tokio::test]
    async fn allows_empty_headers_in_direct_secret() {
        let _guard = TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _env = EnvGuard::set(&[(
            OTLP_TARGET_SECRET_ID_ENV,
            "arn:aws:secretsmanager:us-east-1:111111111111:secret:test",
        )]);
        let provider = FakeSecretProvider {
            secret_string: json!({
                "endpoint": "https://example.com/base"
            })
            .to_string(),
            fetch_count: Arc::new(AtomicUsize::new(0)),
            requested_secret_ids: Arc::new(Mutex::new(Vec::new())),
        };

        let target = resolve_relay_export_target_with_provider(&provider)
            .await
            .unwrap();

        let RelayExportTarget::Direct(target) = target else {
            panic!("expected direct target");
        };
        assert!(target.header_map_for_exporter().is_empty());
        assert_eq!(target.endpoint(), "https://example.com/base/v1/traces");
    }

    #[tokio::test]
    async fn fetches_direct_secret_once_per_execution_environment() {
        let _guard = TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _env = EnvGuard::set(&[(
            OTLP_TARGET_SECRET_ID_ENV,
            "arn:aws:secretsmanager:us-east-1:111111111111:secret:test",
        )]);
        let fetch_count = Arc::new(AtomicUsize::new(0));
        let provider = FakeSecretProvider {
            secret_string: json!({
                "endpoint": "https://example.com"
            })
            .to_string(),
            fetch_count: Arc::clone(&fetch_count),
            requested_secret_ids: Arc::new(Mutex::new(Vec::new())),
        };

        let _ = resolve_relay_export_target_with_provider(&provider)
            .await
            .unwrap();

        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn collector_mode_skips_secret_lookup_when_config_uri_is_set() {
        let _guard = TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _env = EnvGuard::set(&[(COLLECTOR_CONFIG_URI_ENV, "/opt/collector.yaml")]);
        let fetch_count = Arc::new(AtomicUsize::new(0));
        let provider = FakeSecretProvider {
            secret_string: json!({
                "endpoint": "https://example.com"
            })
            .to_string(),
            fetch_count: Arc::clone(&fetch_count),
            requested_secret_ids: Arc::new(Mutex::new(Vec::new())),
        };

        let target = resolve_relay_export_target_with_provider(&provider)
            .await
            .unwrap();
        assert!(matches!(target, RelayExportTarget::Collector));
        assert_eq!(fetch_count.load(Ordering::SeqCst), 0);
    }

    #[tokio::test]
    async fn defaults_to_shared_secret_when_secret_is_not_set() {
        let _guard = TEST_MUTEX
            .get_or_init(|| Mutex::new(()))
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let _env = EnvGuard::set(&[]);
        let fetch_count = Arc::new(AtomicUsize::new(0));
        let requested_secret_ids = Arc::new(Mutex::new(Vec::new()));
        let provider = FakeSecretProvider {
            secret_string: json!({
                "endpoint": "https://example.com"
            })
            .to_string(),
            fetch_count: Arc::clone(&fetch_count),
            requested_secret_ids: Arc::clone(&requested_secret_ids),
        };

        let target = resolve_relay_export_target_with_provider(&provider)
            .await
            .unwrap();

        let RelayExportTarget::Direct(target) = target else {
            panic!("expected direct target");
        };
        assert_eq!(target.endpoint(), "https://example.com/v1/traces");
        assert_eq!(fetch_count.load(Ordering::SeqCst), 1);
        assert_eq!(
            requested_secret_ids
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .as_slice(),
            &[DEFAULT_OTLP_TARGET_SECRET_ID.to_string()]
        );
    }
}
