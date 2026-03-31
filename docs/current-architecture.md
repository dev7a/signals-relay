# Current Architecture

This document describes the architecture implemented in this repository today.

## Summary

Current data flow:

1. CloudWatch Logs subscription on `aws/spans`
2. Partitioner Lambda parses each log record and republishes it to Kinesis with `partitionKey = traceId`
3. A provisioned Kinesis stream buffers and re-groups those repartitioned records
4. Relay Lambda consumes the stream with a 60-second tumbling window
5. Managed-link decorators are reconciled with linkable target spans inside the active window
6. Completed spans are converted to OTLP only on the final window invoke
7. The relay exports OTLP to the backend either directly or through the upstream OpenTelemetry Lambda collector extension

This repository is solving one specific problem: convert Application Signals `aws/spans` records into valid OTLP trace payloads with more predictable batching than direct CloudWatch Logs delivery.

## Export Modes

### Direct

- `ExportMode=direct`
- Default deployment mode
- The relay fetches an OTLP target secret from Secrets Manager at startup
- The target secret uses this JSON shape:

```json
{
  "endpoint": "https://example.com",
  "headers": {
    "authorization": "Bearer ...",
    "x-api-key": "..."
  }
}
```

- If `OtlpTargetSecretArn` is not set, direct mode falls back to standard `OTEL_EXPORTER_OTLP_*` environment variables
- Relay export payloads are gzip-compressed by default

### Collector

- `ExportMode=collector`
- Requires `CollectorExtensionArn` from the upstream `open-telemetry/opentelemetry-lambda` release set
- The relay sends OTLP HTTP/protobuf to `http://localhost:4318`
- The collector config comes from the checked-in `config/collector.yaml` layer at `/opt/collector.yaml`
- The fixed `collector/secrets` secret uses the same `{endpoint, headers}` JSON shape as direct mode
- That config resolves the fixed `collector/secrets` secret via `${secretsmanager:collector/secrets#endpoint}` and `${secretsmanager:collector/secrets#headers}`
- Collector egress is gzip-compressed
- Collector internal telemetry is looped back through `localhost:4318` so it reuses the normal exporter pipeline

## Failure Handling

- The partitioner retries retryable `PutRecords` failures in-process on the failed subset only
- Non-retryable or retry-exhausted records are written to a publish-failure SQS queue with replay-complete payloads
- Unexpected async invocation failures are written to a separate invocation-failure SQS queue
- The relay keeps tumbling-window state in Lambda response state, not in DynamoDB or another external store

## Why This Shape Exists

- CloudWatch Logs tends to deliver `aws/spans` in very small batches
- Each `aws/spans` log record represents a single span
- Direct delivery therefore creates many small OTLP requests
- Managed-link decorators need to co-reside with their target spans long enough to reconcile links correctly

The repartitioning Lambda and Kinesis stream exist so batching is controlled by `traceId`, not by CloudWatch Logs delivery shape.

## Known Tradeoffs

- The architecture adds a repartitioning Lambda and a permanent Kinesis stream
- Link reconciliation is bounded by the tumbling window rather than by durable external state
- Late decorators that arrive after the target window has emitted are dropped
- Replay of publish-failure queue messages after the original window closes may miss same-window reconciliation
- Large OTLP batches can still spend multiple seconds waiting on downstream backend ingest

## Best Fit

Use this architecture when you want a standalone AWS pipeline that converts Application Signals spans into OTLP and you are willing to accept window-bounded link reconciliation in exchange for better batching control.
