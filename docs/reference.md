# Reference

Use this page when you need exact terms, parameters, secret formats, export
modes, or operating contracts.

## Key terms

### Application Signals

Amazon CloudWatch Application Signals can emit span records into CloudWatch
Logs. Signals Relay assumes those records are available in a source log group.

### `aws/spans`

`aws/spans` is the default CloudWatch Logs log group name used by the stack. The
stack subscribes to this log group, but it does not create it. The log group
must exist in the target account and Region before CloudFormation creates the
subscription filter.

### Span record

Each source log record is one span or one related decorator record. A single
trace can arrive as many small records spread across multiple CloudWatch Logs
delivery batches.

### Managed-link decorator

A managed-link decorator is an Application Signals record that describes a
relationship to another span. Signals Relay reconciles decorators with target
spans only while both are present in the active relay window.

### OTLP/HTTP backend

The OpenTelemetry Protocol over HTTP destination that receives exported trace
payloads. This can be a vendor endpoint such as Honeycomb, Datadog, Grafana
Tempo, or New Relic, a self-hosted OpenTelemetry Collector, or any other
OTLP-compatible receiver.

### Trace-aligned buffering

The partitioner Lambda republishes source records to Kinesis with
`partitionKey = traceId`. That moves batching from CloudWatch Logs delivery
batches to trace-based batching.

### Tumbling window

The relay Lambda consumes Kinesis with a 60-second tumbling window. It converts
and exports OTLP only on the final window invocation, after it has had a bounded
chance to reconcile related records.

## Deployment parameters

| Parameter | Required | Default | Notes |
| --- | --- | --- | --- |
| `SpanLogGroupName` | Yes | `aws/spans` | Must name an existing CloudWatch Logs log group. |
| `ExportMode` | Yes | `direct` | Use `direct` or `collector`. |
| `CollectorExtensionArn` | Collector mode only | empty | Required when `ExportMode=collector`; must match target Region and architecture. |
| `DeploymentId` | No | empty | Change to force fresh Lambda environments after secret rotation or other startup-time config changes. |
| `VpcId` | VPC launch only | empty | Existing VPC for the relay Lambda. |
| `SubnetIds` | VPC launch only | empty | Existing subnets with HTTPS egress to Secrets Manager and the OTLP destination. |

## Secret contract

Create the shared secret in the same account and Region as the stack:

```json
{
  "endpoint": "https://example.com",
  "headers": {
    "authorization": "Bearer ...",
    "x-api-key": "..."
  }
}
```

The secret name must be:

```text
signals-relay/secrets/collector
```

Headers are optional. If headers are present, they must be valid HTTP header
names and values.

In direct mode, the endpoint may be a base OTLP/HTTP endpoint such as
`https://example.com`. Signals Relay resolves the trace export URL and appends
`/v1/traces` when the configured path does not already end with `/v1/traces`.

Examples:

| Secret endpoint | Direct-mode trace endpoint |
| --- | --- |
| `https://example.com` | `https://example.com/v1/traces` |
| `https://example.com/base` | `https://example.com/base/v1/traces` |
| `https://example.com/v1/traces` | `https://example.com/v1/traces` |

## Export modes

### Direct export mode

`ExportMode=direct` is the default. The relay Lambda reads the shared Secrets
Manager secret and sends OTLP/HTTP trace payloads directly to the configured
endpoint.

### Collector export mode

`ExportMode=collector` sends OTLP/HTTP to a local OpenTelemetry Lambda collector
extension running inside the relay Lambda execution environment. Use this when
you already standardize export behavior through the collector extension.

| Topic | Direct mode | Collector mode |
| --- | --- | --- |
| Stack value | `ExportMode=direct` | `ExportMode=collector` |
| Default | Yes | No |
| Requires `CollectorExtensionArn` | No | Yes |
| Reads shared secret | Relay Lambda | Collector config |
| Local target | OTLP backend URL from the secret | `http://localhost:4318/v1/traces` |
| Endpoint normalization | Relay appends `/v1/traces` when needed | Collector configuration resolves the endpoint |
| Best fit | Evaluation and fewer moving parts | Teams that already standardize on the Lambda collector extension |

## Required CloudFormation capabilities

When you deploy through the CloudFormation console, expect prompts for:

- IAM resources
- IAM resources with custom names
- `CAPABILITY_AUTO_EXPAND`

When you deploy through tooling that names capabilities explicitly, include:

```text
CAPABILITY_IAM CAPABILITY_NAMED_IAM CAPABILITY_RESOURCE_POLICY CAPABILITY_AUTO_EXPAND
```

`CAPABILITY_AUTO_EXPAND` is required because the launch wrappers and nested SAR
application use the SAM transform. The IAM and resource-policy capabilities are
required by resources created by the child application.

## Failure queues

The stack creates two SQS queues:

- a publish-failure queue for source records that could not be written to
  Kinesis after retry handling
- an invocation-failure queue for unexpected asynchronous partitioner invocation
  failures

These queues support recovery and diagnosis. They are not a complete long-term
replay control plane.

## What Signals Relay does not do

Signals Relay does not create the source `aws/spans` log group, create a backend
collector, manage your OTLP credentials, add VPC NAT gateways or endpoints, or
provide a durable cross-window reconciliation database.

Before using Signals Relay for critical production telemetry, review the
[production-hardening checklist](./operate.md#production-hardening-checklist)
and confirm that the operational tradeoffs fit your environment.
