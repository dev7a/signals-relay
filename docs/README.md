# Signals Relay docs

Signals Relay exports AWS Application Signals traces from CloudWatch Logs to an
OTLP/HTTP destination. It is a serverless AWS pipeline for teams that want to
keep Application Signals as the source of truth while sending traces to their
observability backend or their own OpenTelemetry Collector.

## For a first deployment

Start with [Deploy](./deploy.md). It covers the release quick-launch flow, SAR
deployment, IaC examples, source deployment, and upgrades.

After the stack is running, use [Operate](./operate.md) to prove trace export is
working, diagnose missing spans, and review the production checks.

Use [Architecture](./architecture.md) when you want to understand why the relay
uses Lambda, Kinesis, and a tumbling window. Use [Reference](./reference.md) when
you need exact parameters, secret formats, export modes, or runtime contracts.

## What you need

Signals Relay assumes:

- the source CloudWatch Logs log group already exists; the default is
  `aws/spans`
- the target account can deploy the selected Serverless Application Repository
  version
- the shared Secrets Manager secret exists in the target account and Region
- your destination accepts OTLP/HTTP trace export

The fastest evaluation path is the versioned CloudFormation quick-launch link
from a GitHub Release. If you need IaC examples, source deployment, or upgrade
steps, start with [Deploy](./deploy.md).

## Maintainer and background notes

These pages are public because they are useful when contributing to the project
or comparing the current design with alternatives:

- [Release guide](./maintainers/release.md)
- [CloudWatch Logs to Kinesis to Lambda relay](./design-history/cloudwatch-kinesis-lambda-relay.md)
- [Long poller with SQS for delayed work](./design-history/long-poller-sqs-delayed-task.md)
