# Why This Architecture Was Chosen

> [!NOTE]
> This document explains why the current architecture was chosen. For deployment and configuration, start with the root README and [current architecture](./current-architecture.md).

## Summary

The current implementation uses CloudWatch Logs to partitioner Lambda to Kinesis to relay Lambda with a tumbling window because it solves the main observed problem directly:

- CloudWatch Logs tends to deliver `aws/spans` in very small batches
- each `aws/spans` log record represents a single span
- direct delivery therefore produces many Lambda invokes and many collector requests
- managed-link decorators need to be co-located with their target spans long enough to reconcile links correctly

This design adds a repartitioning stage so batching is based on `traceId` instead of on CloudWatch Logs delivery behavior. The relay then aggregates those records inside a 60-second tumbling window and emits OTLP once per final window invoke.

## Why It Beat The Other Options

- Compared with direct CloudWatch Logs to Lambda, it gives direct control over partitioning and batching.
- Compared with CloudWatch Logs to Kinesis directly, it avoids relying on CloudWatch subscription distribution behavior and lets the repo choose `traceId` as the Kinesis partition key.
- Compared with the earlier DynamoDB and SQS parking approach, it keeps the steady-state runtime simpler and removes external reconciliation state from the hot path.
- Compared with a long-poller design, it stays event-driven and avoids building a custom checkpointing and scheduling control plane.

## Tradeoff Accepted

The trade accepted by this repository is `window-only` correctness:

- link reconciliation is bounded by the tumbling window
- late decorators are dropped and counted
- publish-failure queue replays may miss same-window reconciliation if they are replayed later

That is a narrower correctness contract than durable external state, but it buys much stronger batching control with less steady-state operational surface.
