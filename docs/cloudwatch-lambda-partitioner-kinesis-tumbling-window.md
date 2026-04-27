# Why The Current Architecture Was Chosen

> [!NOTE]
> This is the design rationale for the current implementation. For deployment
> steps, start with [install.md](./install.md). For the canonical runtime
> architecture, start with [current-architecture.md](./current-architecture.md).

## What It Is

Signals Relay uses this current data path:

1. CloudWatch Logs subscription on `aws/spans`
2. partitioner Lambda
3. Kinesis stream partitioned by `traceId`
4. relay Lambda with a 60-second tumbling window
5. OTLP export from the relay, either direct or through the collector extension

The partitioner exists so Kinesis partitioning is based on trace identity, not
on CloudWatch Logs delivery behavior.

## Why It Was Chosen

The main observed problem was not just buffering. CloudWatch Logs tends to
deliver `aws/spans` records in small batches, and managed-link decorators need
to be near their target spans long enough for link reconciliation.

This design solves that problem directly:

- it groups records by `traceId`
- it gives the relay a bounded window for managed-link reconciliation
- it emits OTLP once per final tumbling-window invoke
- it keeps same-window state inside Lambda response state

## Strengths

- Better trace-based grouping than direct CloudWatch Logs delivery.
- Direct control over Kinesis partition keys.
- Bounded reconciliation without DynamoDB on the hot path.
- Event-driven processing without a custom polling loop.
- One export contract for both direct mode and collector mode.

## Weaknesses

- Adds a partitioner Lambda and Kinesis stream to the steady-state path.
- Link reconciliation is limited to the 60-second tumbling window.
- Late managed-link decorators are dropped and counted.
- Replaying publish-failure queue messages after the original window may miss
  same-window reconciliation.

## Best Fit

This is the right fit when trace-based batching and bounded managed-link
reconciliation matter more than minimizing the number of AWS services in the
pipeline.
