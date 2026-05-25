# Architecture

Signals Relay adds a trace-aligned buffering layer between CloudWatch
Application Signals and OTLP export. The goal is to avoid forwarding whatever
small batches CloudWatch Logs happens to deliver and instead export bounded,
trace-grouped OTLP payloads.

For exact deployment inputs, parameters, and export-mode contracts, see
[Reference](./reference.md).

## Problem

CloudWatch Application Signals writes span records to the `aws/spans` log group,
where each log record represents a single span. Exporting each CloudWatch Logs
delivery batch directly as OTLP would create many Lambda invocations and many
small downstream requests.

Signals Relay adds a partitioning and windowing layer so related span records
can be grouped by trace and exported in bounded batches.

<div align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="../svgs/architecture-flow-dark.svg">
  <img src="../svgs/architecture-flow-light.svg" alt="Animated schematic showing small aws/spans batches fanning out through the partitioner into Kinesis lanes, then aggregating in the relay before OTLP export">
</picture>
</div>

## Runtime data flow

1. CloudWatch Logs sends `aws/spans` records to the partitioner Lambda through a
   subscription filter.
2. The partitioner parses each source record and republishes it to Kinesis with
   `partitionKey = traceId`.
3. Kinesis buffers and groups records by partition key.
4. The relay Lambda consumes the stream with a 60-second tumbling window.
5. Managed-link decorators are reconciled against linkable target spans within
   the active window.
6. On the final window invoke, completed spans are converted to OTLP.
7. The relay exports OTLP either directly or through the upstream OpenTelemetry
   Lambda collector extension.

The Kinesis stream is not just a buffer; it is where the application takes
control of grouping by `traceId` rather than relying on the CloudWatch Logs
delivery shape.

## Why this architecture was chosen

Buffering alone is not enough. Signals Relay also needs trace-based grouping
and a short reconciliation period for managed-link decorators.

The partitioner, Kinesis stream, and tumbling-window relay provide that shape:

- the partitioner chooses `traceId` as the Kinesis partition key
- Kinesis gives the relay trace-based grouping instead of source-log delivery
  grouping
- the relay gets a bounded 60-second window for managed-link reconciliation
- OTLP export happens once on the final tumbling-window invoke
- same-window reconciliation state stays in Lambda response state instead of in
  an external state store

This keeps the hot path event-driven while still giving the relay enough local
context to reconcile managed links. It also makes the tradeoff explicit:
correctness is limited to the active window.

## Export shape

Signals Relay supports two export modes:

| Mode | Best fit | Destination |
| --- | --- | --- |
| `direct` | Evaluation and fewer moving parts | OTLP/HTTP backend URL from the shared secret |
| `collector` | Teams that standardize on the Lambda collector extension | Local collector extension at `http://localhost:4318/v1/traces` |

Both modes use the same stack parameters and the same
`signals-relay/secrets/collector` secret. See [Reference](./reference.md) for
the full mode comparison and secret contract.

## Failure handling

The partitioner retries only retryable `PutRecords` failures, and only the
failed subset of records. Non-retryable records and retry-exhausted records go
to the publish-failure SQS queue with replay-complete payloads.

Unexpected asynchronous partitioner invocation failures go to a separate
invocation-failure SQS queue.

The relay keeps tumbling-window state in Lambda response state. It does not use
DynamoDB or another external state store for same-window managed-link
reconciliation.

## Tradeoffs and fit

This design is intentionally window-bounded:

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

## When not to use this architecture

Signals Relay may not be a good fit when:

- you need a fully managed trace export service with no operational ownership
- you cannot tolerate 60-second window-bounded reconciliation
- you need durable reconciliation state across windows
- you cannot operate Kinesis, Lambda failure queues, and CloudWatch alarms
- your OTLP backend cannot absorb batched trace export
- your source spans arrive too late or too sparsely for tumbling-window
  reconciliation to be useful

## Design history

These pages are retained as alternative design and tradeoff history. They are
not the recommended deployment path.

- [CloudWatch Logs to Kinesis to Lambda relay](./design-history/cloudwatch-kinesis-lambda-relay.md)
- [Long poller with SQS for delayed work](./design-history/long-poller-sqs-delayed-task.md)
