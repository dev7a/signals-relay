# Concepts

Start here to learn the vocabulary and mental model before deploying or reading
the architecture guide.

Signals Relay sits between CloudWatch Application Signals and an OTLP/HTTP
trace backend. It is intentionally small: it subscribes to span records,
regroups them by trace, waits for a bounded reconciliation window, and exports
OTLP trace payloads.

## Reader Path

If you are new to this project, read in this order:

| Step | Page | Why it matters |
| --- | --- | --- |
| 1 | Concepts | Learn the terms used by the deploy and architecture guides. |
| 2 | [Installation and deployment](./install.md) | Choose an install path and deploy the stack. |
| 3 | [Troubleshooting](./troubleshooting.md) | Verify the install and debug export failures. |
| 4 | [Current architecture](./current-architecture.md) | Understand the runtime design and tradeoffs. |

Maintainers should also read the [release guide](./release.md) before
publishing a version.

## Key Terms

### Application Signals

Amazon CloudWatch Application Signals can emit span records into CloudWatch
Logs. Signals Relay assumes those records are available in a source log group.

### `aws/spans`

`aws/spans` is the default CloudWatch Logs log group name used by the stack. The
stack subscribes to this log group, but it does not create it. The log group must
exist in the target account and Region before CloudFormation creates the
subscription filter.

### Span Record

Each source log record is one span or one related decorator record. A single
trace can arrive as many small records spread across multiple CloudWatch Logs
delivery batches.

### Managed-Link Decorator

A managed-link decorator is an Application Signals record that describes a
relationship to another span. Signals Relay reconciles decorators with target
spans only while both are present in the active relay window.

### OTLP/HTTP Backend

The OpenTelemetry Protocol over HTTP destination that receives exported trace
payloads. This can be a vendor endpoint such as Honeycomb, Datadog, Grafana
Tempo, or New Relic, a self-hosted OpenTelemetry Collector, or any other
OTLP-compatible receiver.

### Direct Export Mode

`ExportMode=direct` is the default. The relay Lambda reads the shared Secrets
Manager secret and sends OTLP/HTTP trace payloads directly to the configured
endpoint.

### Collector Export Mode

`ExportMode=collector` sends OTLP/HTTP to a local OpenTelemetry Lambda collector
extension running inside the relay Lambda execution environment. Use this when
you already standardize export behavior through the collector extension.

### Trace-Aligned Buffering

The partitioner Lambda republishes source records to Kinesis with
`partitionKey = traceId`. That moves batching from CloudWatch Logs delivery
batches to trace-based batching.

### Tumbling Window

The relay Lambda consumes Kinesis with a 60-second tumbling window. It converts
and exports OTLP only on the final window invocation, after it has had a bounded
chance to reconcile related records.

### Failure Queues

The stack creates two SQS queues:

- a publish-failure queue for source records that could not be written to
  Kinesis after retry handling
- an invocation-failure queue for unexpected asynchronous partitioner invocation
  failures

These queues support recovery and diagnosis. They are not a complete long-term
replay control plane.

## Deployment Inputs

The install flow has a few inputs that appear in several docs:

| Input | Meaning |
| --- | --- |
| `SpanLogGroupName` | Source CloudWatch Logs log group. Defaults to `aws/spans`. |
| `ExportMode` | `direct` or `collector`. Defaults to `direct`. |
| `CollectorExtensionArn` | Required only for collector mode. |
| `DeploymentId` | Optional marker to force fresh Lambda environments after changes such as secret rotation. |
| `VpcId` and `SubnetIds` | Optional existing VPC networking for the relay Lambda. |
| `signals-relay/secrets/collector` | Shared OTLP destination secret used by both export modes. |

## Secret Contract

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

Headers are optional. If headers are present, they must be valid HTTP header
names and values.

In direct mode, the endpoint may be a base OTLP/HTTP endpoint such as
`https://example.com`. Signals Relay resolves the trace export URL and appends
`/v1/traces` when the configured path does not already end with `/v1/traces`.

## Export Mode Comparison

| Topic | Direct mode | Collector mode |
| --- | --- | --- |
| Stack value | `ExportMode=direct` | `ExportMode=collector` |
| Default | Yes | No |
| Requires `CollectorExtensionArn` | No | Yes |
| Reads shared secret | Relay Lambda | Collector config |
| Local target | OTLP backend URL from the secret | `http://localhost:4318/v1/traces` |
| Endpoint normalization | Relay appends `/v1/traces` when needed | Collector configuration resolves the endpoint |
| Best fit | Evaluation and fewer moving parts | Teams that already standardize on the Lambda collector extension |

## What Signals Relay Does Not Do

Signals Relay does not create the source `aws/spans` log group, create a backend
collector, manage your OTLP credentials, add VPC NAT gateways or endpoints, or
provide a durable cross-window reconciliation database.

Signals Relay is in beta. Run it in dev or staging today, then review the
[production-hardening checklist](./install.md#production-hardening-checklist)
before you ship.
