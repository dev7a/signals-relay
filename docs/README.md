# Documentation

Choose the document that matches your task.

## Deploy Or Evaluate

- [Root README](../README.md): project overview, fastest install path,
  architecture summary, and documentation map
- [Install and deployment](./install.md): operator guide for GitHub Release
  quick launch, SAR installs, IaC composition, source deployment, and upgrades

## Understand The Architecture

- [Current architecture](./current-architecture.md): canonical explanation of
  the implemented pipeline, export modes, failure handling, and tradeoffs
- [Why the current architecture was chosen](./cloudwatch-lambda-partitioner-kinesis-tumbling-window.md):
  design rationale for the partitioner, Kinesis stream, and tumbling window

## Maintain The Release

- [Release guide](./release.md): maintainer-only workflow for coordinated
  versions, SAR publication, GitHub Releases, and CloudFormation launch
  artifacts

## Design Background

These pages are retained as explanation and tradeoff history. They are not the
recommended deployment path unless they explicitly describe the current design.

- [CloudWatch Logs to Kinesis to Lambda relay](./cloudwatch-kinesis-lambda-relay.md)
- [Long poller with SQS for delayed work](./long-poller-sqs-delayed-task.md)
