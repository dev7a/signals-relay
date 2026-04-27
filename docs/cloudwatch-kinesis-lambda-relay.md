# CloudWatch Logs To Kinesis To Lambda Relay

> [!NOTE]
> This is an alternative design note. It is retained for tradeoff history and
> comparison, not as the recommended deployment path.

## What It Is

This alternative sends CloudWatch Logs subscription records directly to Kinesis
Data Streams. A relay Lambda then consumes the stream and emits OTLP.

Flow:

1. CloudWatch Logs subscription sends `aws/spans` records to Kinesis.
2. Kinesis buffers records.
3. Relay Lambda consumes records through an event source mapping.
4. The relay parses spans and exports OTLP.

## Why It Was Considered

This approach keeps a durable stream between CloudWatch Logs and Lambda while
removing the explicit partitioner Lambda from the current design. The main
appeal is better invoke-level batching than direct CloudWatch Logs to Lambda.

## Strengths

- Kinesis provides a durable buffer.
- Lambda event source mapping exposes `BatchSize` and
  `MaximumBatchingWindowInSeconds`.
- Relay invocations can scale through stream shards.
- The pipeline has fewer Lambda functions than the current design.

## Weaknesses

- CloudWatch Logs still controls how source log events are packed into Kinesis
  records.
- The design does not choose `traceId` as the Kinesis partition key.
- Related spans may still land in different relay batches.
- Managed-link reconciliation would need external state or weaker correctness.
- Throughput planning is harder if CloudWatch Logs distribution creates hot
  partitions.

## Best Fit

Use this approach when the main problem is invoke-level batching, and when
trace-based grouping or managed-link reconciliation is less important than
removing the partitioner Lambda.

## Open Questions

- Would CloudWatch Logs subscription distribution create hot partitions for
  `aws/spans` traffic?
- Would Kinesis on-demand be enough, or would the stream need provisioned
  capacity?
- Is the batching improvement large enough to justify the stream without the
  trace-based partitioning used by the current design?
