# CloudWatch Logs To Kinesis To Lambda Relay

> [!NOTE]
> This document captures an alternative considered during design. It is retained for tradeoff history and comparison, not as the recommended deployment path for this repository.

## Summary

Flow:

1. CloudWatch Logs subscription sends records to Kinesis Data Streams
2. Lambda consumes from Kinesis with an event source mapping
3. The relay Lambda parses spans and emits OTLP

This approach introduces a stream between CloudWatch Logs and the processor Lambda. The main benefit is better control over Lambda batching through Kinesis event source mapping settings.

## Strengths

- Better invoke-level batching than direct CloudWatch Logs to Lambda.
- Lambda event source mapping gives direct controls for `BatchSize` and `MaximumBatchingWindowInSeconds`.
- Kinesis adds a durable buffer between CloudWatch Logs and the processor.
- The intake Lambda can scale through shards rather than CloudWatch subscription fan-out alone.

## Weaknesses

- CloudWatch Logs still controls how many source log events are packed into each Kinesis record.
- This does not solve the semantic grouping problem by itself. Related spans may still land in different batches.
- Managed-link reconciliation still needs external state unless another mechanism is added.
- Throughput planning is harder if `ByLogStream` creates hot partitions.

## Cost Considerations

Main billed components:

- Kinesis stream-hours and ingest
- relay Lambda invokes and duration
- downstream collector ingest
- external state, if DynamoDB and SQS are retained

This model can reduce:

- the number of relay Lambda invokes
- the number of local collector POSTs

This model adds:

- Kinesis as a permanent cost center
- operational attention around shard behavior or on-demand stream scaling

Actual costs will depend on traffic patterns and configuration.

## Best Fit

Use this approach when the main problem is CloudWatch Logs batching, and when it is acceptable to keep external reconciliation state or accept weaker link handling.

## Open Questions

- Would `Random` or `ByLogStream` distribution be better for `aws/spans` throughput?
- Would Kinesis on-demand be sufficient, or would provisioned streams be needed?
- Is the batching improvement large enough to justify the extra stream layer?
