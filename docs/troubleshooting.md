# Troubleshooting

Use this guide after deployment when you need to verify the stack, diagnose
missing spans, or understand failure queues.

Start with the shortest path that proves data is moving:

1. CloudWatch Logs writes source records to the span log group.
2. The subscription filter invokes the partitioner Lambda.
3. The partitioner writes records to Kinesis by `traceId`.
4. The relay consumes Kinesis windows.
5. The relay exports OTLP to the configured backend.

## Quick Verification

After creating or updating the stack:

1. Confirm the stack reaches `CREATE_COMPLETE` or `UPDATE_COMPLETE`.
2. Open the stack outputs and note:
   - `ProcessorPartitionerFunctionArn`
   - `ProcessorRelayFunctionArn`
   - `SpanRelayStreamArn`
   - `PartitionerPublishFailureQueueUrl`
   - `PartitionerInvocationFailureQueueUrl`
3. Confirm the source log group exists in the same account and Region.
4. Confirm the log group has a subscription filter for the partitioner Lambda.
5. Confirm the shared secret exists as `signals-relay/secrets/collector`.
6. Generate or wait for Application Signals spans.
7. Watch the partitioner and relay Lambda logs while spans are emitted.
8. Confirm both failure queues remain empty during normal traffic.

## Stack Creation Fails

### Source Log Group Does Not Exist

The stack creates a subscription filter on the source log group, but it does not
create the log group. Create the log group first or pass the correct
`SpanLogGroupName`.

Default:

```text
aws/spans
```

### Subscription filter quota or conflict

CloudWatch Logs limits the number of subscription filters on a log group. If
another relay or processor already subscribes to the same log group, remove the
conflicting subscription or choose a different source log group.

### Missing CloudFormation Capabilities

Deployments need these capabilities when the deployment tool requests them
explicitly:

```text
CAPABILITY_IAM CAPABILITY_NAMED_IAM CAPABILITY_RESOURCE_POLICY CAPABILITY_AUTO_EXPAND
```

`CAPABILITY_AUTO_EXPAND` is required because the deployment uses the SAM
transform and nested SAR application flow.

### SAR Application Is Not Deployable By This Account

Quick-launch templates deploy the published SAR application as a nested
application. They do not bypass SAR sharing. Ensure the target account can deploy
the selected application version through account, organization, or public
sharing.

### Collector mode is missing the collector layer ARN

`ExportMode=collector` requires `CollectorExtensionArn`. Use `direct` mode for
the simplest test path, or provide an OpenTelemetry Lambda collector extension
layer ARN that matches the target Region and architecture.

## No Spans Arrive

### Application Signals Is Not Writing Source Records

Signals Relay can only process records that already exist in the source log
group. Confirm the application or service emits Application Signals spans into
the expected log group.

### Wrong Account Or Region

CloudWatch Logs, the subscription filter, the Lambdas, Kinesis, SQS, and the
shared secret are Region-scoped. Confirm you are checking the same account and
Region as the deployed stack.

### Subscription Filter Is Missing

If the stack succeeded but no partitioner logs appear, inspect the source log
group's subscription filters. The destination should be the partitioner Lambda.

### Source Records Do Not Include Trace IDs

The partitioner republishes records with `partitionKey = traceId`. Records that
do not match the expected Application Signals span shape cannot be grouped into
trace-aligned Kinesis lanes.

## Relay Runs But Nothing Is Exported

### Secret Is Missing Or Invalid

The shared secret must be named:

```text
signals-relay/secrets/collector
```

The secret value must be JSON:

```json
{
  "endpoint": "https://example.com",
  "headers": {
    "authorization": "Bearer ..."
  }
}
```

Headers are optional. In direct mode, the relay validates header names and values
before sending requests.

### Endpoint Path Is Ambiguous

For direct mode, the endpoint can be a base OTLP/HTTP endpoint. Signals Relay
appends `/v1/traces` when the configured path does not already end with
`/v1/traces`.

Examples:

| Secret endpoint | Direct-mode trace endpoint |
| --- | --- |
| `https://example.com` | `https://example.com/v1/traces` |
| `https://example.com/base` | `https://example.com/base/v1/traces` |
| `https://example.com/v1/traces` | `https://example.com/v1/traces` |

### Backend rejects the request

Check relay logs for HTTP status and export errors. Common causes are missing
authorization headers, expired tokens, unsupported OTLP/HTTP paths, backend rate
limits, or a backend that expects a collector-specific configuration instead of
direct OTLP/HTTP protobuf.

### VPC Deployment Cannot Reach Dependencies

The VPC launch path passes existing VPC and subnet selections to the application.
It does not create NAT gateways, route-table entries, or VPC endpoints. Selected
subnets must reach Secrets Manager and the OTLP destination over HTTPS.

### Collector mode cannot start the collector

Confirm `CollectorExtensionArn` points to a valid collector extension layer for
the target Region and architecture. Then inspect the relay Lambda logs for
extension startup errors.

## Failure Queues

### Publish-Failure Queue

The publish-failure queue receives source records that could not be written to
Kinesis after retry handling. Inspect this queue when partitioner logs mention
non-retryable records, retry exhaustion, or Kinesis write failures.

Replaying old publish-failure messages can help with recovery, but managed-link
reconciliation is limited to the original relay window. A replay after that
window closes may export spans without same-window decorator matches.

### Invocation-Failure Queue

The invocation-failure queue receives unexpected asynchronous partitioner
invocation failures. Inspect this queue when Lambda reports async delivery
failures or when the partitioner does not run consistently despite a valid
subscription filter.

## Operational signals to watch

Before production use, add alarms or dashboards for:

| Surface | Signals |
| --- | --- |
| Partitioner Lambda | Errors, throttles, duration, async event age, dead-letter or destination failures |
| Relay Lambda | Errors, throttles, duration, iterator age, concurrent executions |
| Kinesis stream | Write throttles, read throttles, iterator age, shard utilization |
| SQS failure queues | Visible messages, oldest message age |
| OTLP backend | Authentication failures, rate limits, rejected payloads |

Also review Kinesis shard count, retention, and cost. The default application
uses a provisioned stream with one shard and 24-hour retention.

## After Secret Rotation

The relay reads the shared secret during startup in direct mode. After rotating
credentials, update the stack with a new `DeploymentId` value when you need
CloudFormation to force fresh Lambda execution environments.

## When to escalate to architecture review

If the deployment works but the shape does not fit your reliability model, read
[Current architecture](./current-architecture.md). Pay special attention to
window-bounded reconciliation, replay behavior, Kinesis operational ownership,
and the lack of durable external reconciliation state on the hot path.
