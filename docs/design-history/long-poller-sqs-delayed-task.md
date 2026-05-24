# Long poller with SQS for delayed work

> [!NOTE]
> This is an alternative design note. It is retained for tradeoff history and
> comparison, not as the recommended deployment path.

## What it is

This alternative replaces the push subscription path with a pull loop managed
by Lambda and SQS.

For the implemented pipeline, see [Architecture](../architecture.md).

How it works:

1. A scheduled heartbeat Lambda starts or resumes the poller chain.
2. A poller Lambda reads pages from CloudWatch Logs APIs.
3. The poller processes the current page and exports or buffers spans.
4. If more work is available, it sends follow-up work to SQS with the next
   cursor and a delay.
5. If no work is available, it schedules a quiet-period wake-up.

## Why it was considered

The long-poller model gives the application explicit control over CloudWatch
Logs page size, polling cadence, and delayed follow-up work. It can smooth out
small push batches by letting the application choose when to poll and how much
to pull.

## Strengths

- Strong control over page size and polling cadence.
- Adaptive behavior during busy and quiet periods.
- Potentially larger batches than direct CloudWatch Logs subscription delivery.
- SQS provides a simple delayed handoff between poller invocations.

## Weaknesses

- Introduces a custom control plane around CloudWatch Logs pagination.
- Cursor correctness, duplicate handling, and recovery become application
  responsibilities.
- SQS at-least-once delivery and visibility timeouts can create overlapping
  poller work.
- Lambda recursion safeguards and SQS delay limits shape the design.
- It is less event-driven than the subscription and Kinesis paths.

## Historical Fit

This approach fit the case where explicit control over polling cadence mattered
more than keeping the pipeline event-driven.

## Questions that drove the decision

- How do CloudWatch Logs API limits affect page size, request rate, and token
  lifetime?
- Is SQS payload state sufficient for recovery, or is a durable checkpoint store
  required?
- Can a single active poller keep up with expected `aws/spans` volume?
- Is the control-plane complexity worth it compared with the current
  partitioner and Kinesis design?
