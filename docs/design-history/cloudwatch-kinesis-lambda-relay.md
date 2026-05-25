# CloudWatch Logs to Kinesis to Lambda relay

> [!NOTE]
> This is an alternative design note. It is retained for tradeoff history and
> comparison, not as the recommended deployment path.

## What it is

This alternative sends CloudWatch Logs subscription records directly to Kinesis
Data Streams. A relay Lambda then consumes the stream and emits OTLP.

For the implemented pipeline, see [Architecture](../architecture.md).

How it works:

1. CloudWatch Logs subscription sends `aws/spans` records to Kinesis.
2. Kinesis buffers records.
3. Relay Lambda consumes records through an event source mapping.
4. The relay parses spans and exports OTLP.

## Why it was considered

This approach keeps a durable stream between CloudWatch Logs and Lambda while
removing the explicit partitioner Lambda from the current design. The main
appeal is better invoke-level batching than a direct CloudWatch Logs-to-Lambda
path.

## Strengths

- Kinesis provides a durable buffer.
- Lambda event source mapping exposes `BatchSize` and
  `MaximumBatchingWindowInSeconds`.
- Relay invocations can scale through stream shards.
- The pipeline uses fewer Lambda functions than the current design.

## Weaknesses

- CloudWatch Logs still controls how source log events are packed into Kinesis
  records.
- The design does not choose `traceId` as the Kinesis partition key.
- Related spans may still land in different relay batches.
- Managed-link reconciliation would require external state or weaker correctness
  guarantees.
- Throughput planning is harder if CloudWatch Logs distribution creates hot
  partitions.

## Historical Fit

This approach fit when invoke-level batching was the main concern and
trace-based grouping or managed-link reconciliation mattered less than removing
the partitioner Lambda.

## Questions that drove the decision

- Would CloudWatch Logs subscription distribution create hot partitions for
  `aws/spans` traffic?
- Would Kinesis on-demand be enough, or would the stream need provisioned
  capacity?
- Is the batching improvement large enough to justify the stream without the
  trace-based partitioning used by the current design?
