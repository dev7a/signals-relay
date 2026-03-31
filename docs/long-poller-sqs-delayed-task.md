# Long Poller With SQS For Delayed Work

> [!NOTE]
> This document captures an alternative considered during design. It is retained for tradeoff history and comparison, not as the recommended deployment path for this repository.

## Summary

Flow:

1. An optional heartbeat Lambda, invoked on a schedule, checks whether the queue is idle and starts the poller chain if needed
2. A poller Lambda reads pages from CloudWatch Logs APIs
3. The poller processes the current page
4. If more work is available, the poller posts follow-up work to SQS with the next cursor and delay
5. If there is no continuation token, the poller can schedule a quiet-period wake-up, for example 30 seconds later
6. The poller adapts delay based on recent page fullness or queue state

This model replaces the push subscription path with a pull loop managed by Lambda and SQS.

## Strengths

- Strong control over page size, polling cadence, and aggregation.
- The system can adapt polling frequency during busy and quiet periods.
- Batching can be much larger than direct CloudWatch subscription delivery.
- SQS provides a clean handoff for delayed follow-up work.
- A single active poller chain is conceptually simple if cursor state is carried in the SQS payload.

## Weaknesses

- This creates a custom control plane around CloudWatch Logs pagination and self-scheduling.
- `nextToken` may be sufficient for a short-delay single-chain design, but it is not enough for every recovery scenario by itself.
- Duplicate handling and cursor correctness become application responsibilities.
- Even a single-chain design must handle overlap from SQS at-least-once delivery, visibility timeout expiry, or heartbeat races.
- Lambda recursion safeguards and SQS delay limits still shape the design.
- This is less event-native than the subscription or Kinesis paths.

## Cost Considerations

Main billed components:

- poller Lambda duration
- SQS sends and receives
- CloudWatch Logs API requests
- checkpoint storage, only if a separate durable cursor store is required
- downstream collector ingest

This model can reduce:

- collector request fan-out from tiny push batches
- dependence on CloudWatch subscription delivery shape

This model can increase:

- Lambda runtime spent waiting on and processing CloudWatch Logs pages
- application complexity around cursor handling, retries, and optional checkpoint persistence

Actual costs will depend on traffic patterns and configuration.

## Best Fit

Use this approach when explicit control over polling cadence matters more than keeping a push-based event pipeline.

## Open Questions

- What documented CloudWatch Logs API limits matter most here, including page size, rate limits, and token lifetime?
- Is SQS payload state enough for recovery, or is a separate durable checkpoint needed?
- Can a single active poller keep up, and how should the heartbeat bootstrap logic recover if the chain goes idle?
- Is the extra control worth the control-plane complexity compared with the Kinesis partitioner path?
