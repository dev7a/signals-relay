# Documentation

Choose the document that matches your task.

## Start Here

| Task | Read |
| --- | --- |
| New evaluator | [Concepts](./concepts.md), then [Installation and deployment](./install.md) |
| Deploying from a release | [Installation and deployment](./install.md) |
| Verifying or debugging an install | [Troubleshooting](./troubleshooting.md) |
| Explaining the runtime design | [Current architecture](./current-architecture.md) |
| Publishing a version | [Release guide](./release.md) |

## Deploy and operate

- [Root README](../README.md): project overview, fastest install path,
  architecture summary, and documentation map
- [Installation and deployment](./install.md): operator guide for quick launch
  from GitHub Releases, SAR installs, IaC composition, source deployment, and
  upgrades
- [Troubleshooting](./troubleshooting.md): operator runbook for post-deploy
  verification, missing spans, export failures, and failure queues

## Understand the architecture

- [Concepts](./concepts.md): glossary, export modes, secret contract, and core
  deployment inputs for readers new to the project
- [Current architecture](./current-architecture.md): canonical explanation of
  the implemented pipeline, why it uses the partitioner, Kinesis streams, and
  the tumbling window, as well as export modes, failure handling, and tradeoffs

## Maintain the release

- [Release guide](./release.md): maintainer-only workflow for coordinated
  versions, SAR publication, GitHub Releases, and CloudFormation launch
  artifacts

## Design background

These pages document alternative design and tradeoff decisions and are not the
recommended deployment path.

- [CloudWatch Logs to Kinesis to Lambda relay](./cloudwatch-kinesis-lambda-relay.md)
- [Long poller with SQS for delayed work](./long-poller-sqs-delayed-task.md)
