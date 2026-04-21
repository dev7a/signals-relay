# Documentation

Choose the path that matches what you are trying to do.

## Try The Application

- [Root README](../README.md): start here if you want to try the shared SAR
  application or decide whether you need a source checkout
- [Release and packaging](./release.md#install-from-sar): SAR install path for
  users who only want to deploy the application

## Deploy Or Modify From Source

- [Root README](../README.md#deploy-from-source): source-based deploy path and
  operator-oriented configuration notes
- [Current architecture](./current-architecture.md): implemented pipeline,
  export modes, failure handling, and tradeoffs

## Publish A Release

- [Release and packaging](./release.md): coordinated release flow, manual SAR
  publish flow, and upgrade guidance

## Design Background

- [Why the current architecture was chosen](./cloudwatch-lambda-partitioner-kinesis-tumbling-window.md)
- [CloudWatch Logs to Kinesis to Lambda relay](./cloudwatch-kinesis-lambda-relay.md)
- [Long poller with SQS for delayed work](./long-poller-sqs-delayed-task.md)
