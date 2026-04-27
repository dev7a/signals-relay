# Signals Relay

Signals Relay is an experimental AWS serverless pipeline that converts
CloudWatch Application Signals `aws/spans` log records into OTLP trace payloads
and exports them to an OTLP/HTTP backend.

> [!NOTE]
> This repository is experimental and is not recommended for production use
> without additional hardening.

## Who Should Use This

Use Signals Relay when you want to evaluate a standalone AWS path for exporting
Application Signals spans to an OTLP backend, and you are comfortable operating
an event-driven pipeline built from CloudWatch Logs, Lambda, Kinesis, SQS, and
Secrets Manager.

The fastest evaluation path is the AWS Serverless Application Repository (SAR)
application or the versioned CloudFormation quick-launch links published with
each GitHub Release. The target account must be allowed to deploy the selected
SAR version through account, organization, or public sharing. You only need a
source checkout when you want to inspect, modify, or publish the application
yourself.

## Fastest Install Path

1. Open the GitHub Release for the version you want to deploy.
2. Use the CloudFormation quick-launch link for either:
   - no VPC configuration
   - an existing VPC and subnet selection, if those subnets have outbound HTTPS
     access to Secrets Manager and your OTLP destination
3. Confirm that the source CloudWatch Logs log group already exists in the
   target account and Region. The default is `aws/spans`; the stack subscribes
   to it but does not create it.
4. Create the shared Secrets Manager secret at
   `signals-relay/secrets/collector` before the stack runs:

   ```json
   {
     "endpoint": "https://example.com",
     "headers": {
       "authorization": "Bearer ...",
       "x-api-key": "..."
     }
   }
   ```

5. In the CloudFormation console, acknowledge the prompts for IAM resources,
   IAM resources with custom names, and `CAPABILITY_AUTO_EXPAND`.

The current SAR publication path is pinned to `us-east-1`. Full install
instructions, IaC examples, source deployment, and upgrade guidance live in
[docs/install.md](./docs/install.md).

## Architecture At A Glance

The application deploys:

1. a CloudWatch Logs subscription on `aws/spans`
2. a partitioner Lambda that republishes records to Kinesis with
   `partitionKey = traceId`
3. a Kinesis stream that buffers and groups records
4. a relay Lambda that consumes the stream with a 60-second tumbling window
5. OTLP conversion and export, either directly or through the upstream
   OpenTelemetry Lambda collector extension

<div align="center">
<picture>
  <source media="(prefers-color-scheme: dark)" srcset="./svgs/readme-1-dark.svg">
  <img src="./svgs/readme-1-light.svg" alt="Signals Relay architecture diagram">
</picture>
</div>

This shape gives the relay control over trace-based grouping and batching
instead of depending on the small batches CloudWatch Logs often delivers for
`aws/spans`. Managed-link decorators are reconciled inside the active tumbling
window before OTLP export.

Read [docs/current-architecture.md](./docs/current-architecture.md) for the
implemented architecture, export modes, failure handling, and tradeoffs.

## Deliverables

Each coordinated release includes:

- the reusable `signals-relay-core` crate artifact
- the deployable SAM application published through SAR
- versioned SAR-backed CloudFormation launch templates for no-VPC and VPC
  console deployments
- GitHub Release notes with CloudFormation quick-launch badge links and
  versioned template links

SAR remains the canonical publish path. The CloudFormation launch templates are
a console-friendly release surface for deploying the published SAR application;
they do not replace SAR and do not use a mutable `latest` URL.

## Documentation Map

- [Install and deployment](./docs/install.md): deploy from GitHub Release
  quick launch, SAR, IaC, or source
- [Current architecture](./docs/current-architecture.md): runtime data flow,
  export modes, failure handling, and tradeoffs
- [Release guide](./docs/release.md): maintainer-only release and publication
  workflow
- [Design background](./docs/README.md#design-background): architecture
  decisions and alternatives considered

## Build And Validate From Source

Use these commands after cloning the repository:

```bash
cargo test --workspace --locked
sam validate --template-file template.yaml
sam build --template-file template.yaml
uv run ./scripts/init_samconfig.py --help
```
