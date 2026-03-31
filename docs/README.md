# Documentation

Start with the root [README](../README.md) if you want to evaluate or deploy this repository.

## Start Here

- [Current architecture](./current-architecture.md): the implemented pipeline, export modes, failure handling, and tradeoffs
- [Why the current architecture was chosen](./cloudwatch-lambda-partitioner-kinesis-tumbling-window.md): rationale for the Kinesis repartitioning and tumbling-window design

## Alternatives Considered

These documents are retained for tradeoff history and comparison. They are not the recommended deployment path for this repository.

- [CloudWatch Logs to Kinesis to Lambda relay](./cloudwatch-kinesis-lambda-relay.md)
- [Long poller with SQS for delayed work](./long-poller-sqs-delayed-task.md)
