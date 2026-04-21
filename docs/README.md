# Documentation

Choose the path that matches what you are trying to do.

## Try The Application

- [Root README](../README.md): start here if you want to try the shared SAR
  application or decide whether you need a source checkout
- [Install and deployment](./install.md): SAR install path, IaC examples, and
  source deployment guidance for users who want to run the application

## Deploy Or Modify From Source

- [Install and deployment](./install.md#install-from-source): source-based
  deploy path and local bootstrap guidance
- [Current architecture](./current-architecture.md): implemented pipeline,
  export modes, failure handling, and tradeoffs

## Publish A Release

- [Release guide](./release.md): coordinated release flow and manual SAR publish
  workflow for maintainers

## Design Background

- [Why the current architecture was chosen](./cloudwatch-lambda-partitioner-kinesis-tumbling-window.md)
- [CloudWatch Logs to Kinesis to Lambda relay](./cloudwatch-kinesis-lambda-relay.md)
- [Long poller with SQS for delayed work](./long-poller-sqs-delayed-task.md)
