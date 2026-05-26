# Signals Relay

Signals Relay is an AWS SAM application that converts CloudWatch Application
Signals `aws/spans` log records into OTLP trace payloads and sends them to your
observability backend or OpenTelemetry collector.

Use this Serverless Application Repository page to deploy the published
application. Use the GitHub project for source code, design docs, and release
artifacts.

## Before You Deploy

- The source CloudWatch Logs log group must already exist in the target account
  and Region. The default log group name is `aws/spans`.
- Create the collector configuration secret in AWS Secrets Manager before the
  stack runs. The default secret name is `signals-relay/secrets/collector`.
- The collector secret must contain an `endpoint` value and any required request
  `headers`.
- Deployment creates IAM resources and resource policies. Review the permissions
  and acknowledge the CloudFormation capability prompts.
- Existing VPC deployments need private subnets with outbound HTTPS access to
  Secrets Manager and your OTLP destination.

## Deployment Options

- Use the Deploy button on this page for SAR deployment.
- Use the versioned CloudFormation quick-launch templates from the GitHub
  Releases page if you want a guided CloudFormation console launch.
- Use the full deploy guide for source deploys, upgrades, Terraform, CDK, and
  operational checks.

## What It Deploys

Signals Relay deploys:

- a CloudWatch Logs subscription for Application Signals spans
- a partitioner Lambda that groups span records by trace ID
- a Kinesis stream for buffering and ordering work
- a relay Lambda with a 60-second tumbling window
- OTLP conversion and export to your configured destination
- alarms and failure handling resources for the event-driven pipeline

This design fits teams that need a native AWS path for exporting Application
Signals traces and can operate Kinesis, Lambda, SQS, CloudWatch Logs, and
Secrets Manager.

## When Signals Relay Is Not a Fit

Signals Relay may not be the right tool if you need a fully managed trace
export service, cannot operate the supporting AWS services, cannot tolerate
60-second window-bounded reconciliation, or have an OTLP backend that cannot
absorb batched trace export.

## Links

- [GitHub project](https://github.com/dev7a/signals-relay)
- [Releases and CloudFormation quick-launch templates](https://github.com/dev7a/signals-relay/releases)
- [Full deploy guide](https://dev7a.github.io/signals-relay/docs/deploy/)
- [Operations guide](https://dev7a.github.io/signals-relay/docs/operate/)
- [Architecture](https://dev7a.github.io/signals-relay/docs/architecture/)
