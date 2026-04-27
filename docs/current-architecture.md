# Current Architecture

This is the canonical architecture explanation for the implementation in this
repository.

## Problem

CloudWatch Application Signals writes span records to the `aws/spans` log group.
In practice, CloudWatch Logs often delivers those records in very small batches,
and each log record represents a single span. Directly exporting each small
batch as OTLP would create many Lambda invocations and many small downstream
requests.

Signals Relay adds a partitioning and windowing layer so related span records
can be grouped by trace before OTLP export.

## Runtime Data Flow

1. CloudWatch Logs sends `aws/spans` records to the partitioner Lambda through a
   subscription filter.
2. The partitioner parses each source record and republishes it to Kinesis with
   `partitionKey = traceId`.
3. Kinesis buffers and groups records by partition key.
4. The relay Lambda consumes the stream with a 60-second tumbling window.
5. Managed-link decorators are reconciled with linkable target spans inside the
   active window.
6. On the final window invoke, completed spans are converted to OTLP.
7. The relay exports OTLP either directly or through the upstream OpenTelemetry
   Lambda collector extension.

The Kinesis stream is not just a buffer. It is the point where the repository
takes control of grouping by `traceId` instead of depending on CloudWatch Logs
delivery shape.

## Why This Architecture Was Chosen

The main observed problem was not just buffering. CloudWatch Logs often delivers
`aws/spans` records in small batches, and managed-link decorators need to be
near their target spans long enough for link reconciliation.

The partitioner, Kinesis stream, and tumbling-window relay solve that problem
directly:

- the partitioner chooses `traceId` as the Kinesis partition key
- Kinesis gives the relay trace-based grouping instead of source-log delivery
  grouping
- the relay gets a bounded 60-second window for managed-link reconciliation
- OTLP export happens once on the final tumbling-window invoke
- same-window reconciliation state stays inside Lambda response state instead
  of an external state store

This keeps the hot path event driven while still giving the relay enough local
context to reconcile managed links. It also makes the main tradeoff explicit:
correctness is bounded by the active window.

## Export Modes

### Direct

`ExportMode=direct` is the default. The relay Lambda reads the shared
`signals-relay/secrets/collector` secret from Secrets Manager during startup
and exports OTLP HTTP/protobuf directly to the configured endpoint.

The secret uses this JSON shape:

```json
{
  "endpoint": "https://example.com",
  "headers": {
    "authorization": "Bearer ...",
    "x-api-key": "..."
  }
}
```

Direct mode enables gzip compression for outbound OTLP trace requests.

### Collector

`ExportMode=collector` sends OTLP HTTP/protobuf to
`http://localhost:4318` inside the relay Lambda execution environment. This
mode requires `CollectorExtensionArn`, an upstream OpenTelemetry Lambda
collector extension layer ARN for the target Region and architecture.

The collector reads `config/collector.yaml` from the deployed layer at
`/opt/collector.yaml`. That config resolves the same
`signals-relay/secrets/collector` secret:

- `${secretsmanager:signals-relay/secrets/collector#endpoint}`
- `${secretsmanager:signals-relay/secrets/collector#headers}`

Collector mode keeps export behavior under the collector extension while
preserving the same stack parameters and secret contract as direct mode.

## Failure Handling

The partitioner retries retryable `PutRecords` failures in process and retries
only the failed subset. Non-retryable records and retry-exhausted records go to
the publish-failure SQS queue with replay-complete payloads.

Unexpected asynchronous partitioner invocation failures go to a separate
invocation-failure SQS queue.

The relay keeps tumbling-window state in Lambda response state. It does not use
DynamoDB or another external state store for same-window managed-link
reconciliation.

## Tradeoffs And Fit

This design accepts window-bounded correctness:

- managed-link reconciliation is limited to the active tumbling window
- late decorators are dropped and counted
- replaying publish-failure messages after the original window closes may miss
  same-window reconciliation

The tradeoff is intentional. It adds a repartitioning Lambda and a permanent
Kinesis stream, but it avoids a custom polling control plane and avoids durable
external reconciliation state on the hot path.

Use this architecture when you want a standalone AWS pipeline that converts
Application Signals spans into OTLP and you are willing to accept
window-bounded link reconciliation in exchange for predictable trace-based
batching.

## Design Background

These pages are retained as alternative design and tradeoff history. They are
not the recommended deployment path.

- [CloudWatch Logs to Kinesis to Lambda relay](./cloudwatch-kinesis-lambda-relay.md)
- [Long poller with SQS for delayed work](./long-poller-sqs-delayed-task.md)
