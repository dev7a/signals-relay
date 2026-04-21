# Install and Deployment

Use this document when you want to deploy or evaluate `signals-relay`.

For release and SAR publication workflows maintained in this repository, see
[release.md](./release.md).

This document covers three workflows:

- install the application from AWS Serverless Application Repository (SAR)
- deploy the application from a source checkout
- upgrade an existing install

## Install From SAR

If the application has been shared with your AWS account or AWS Organization,
this is the fastest way to try it. You do not need to clone the repository,
install Rust, install `cargo-lambda`, install `uv`, or generate a local
`samconfig.toml` for this path.

The current private/shared SAR publication path is pinned to `us-east-1`, so
use that Region when browsing or deploying the application.

Before you deploy, create the shared Secrets Manager secret that both export
modes expect at `signals-relay/secrets/collector`:

```json
{
  "endpoint": "https://example.com",
  "headers": {
    "authorization": "Bearer ...",
    "x-api-key": "..."
  }
}
```

When you deploy the SAR application, these parameters matter most:

- `SpanLogGroupName`
  - defaults to `aws/spans`
- `ExportMode`
  - `direct` is the default
  - `collector` requires `CollectorExtensionArn`
- `CollectorExtensionArn`
  - required only when `ExportMode=collector`
  - must point to an upstream OpenTelemetry Lambda collector layer ARN for your
    Region and architecture
- `DeploymentId`
  - optional no-op deployment marker
  - change it when you want CloudFormation to force a fresh Lambda rollout after
    rotating secrets
- `VpcId` and `SubnetIds`
  - optional VPC settings for the relay Lambda only

If you choose `collector` mode, the relay sends OTLP to
`http://localhost:4318` through the upstream OpenTelemetry Lambda collector
extension. If you choose `direct` mode, the relay exports to the OTLP endpoint
described by the shared secret.

### Deploy From Another SAM Template

If you want to compose the shared SAR application into a larger SAM or
CloudFormation stack, use `AWS::Serverless::Application`:

Replace the placeholder application ARN and version with the values published
for the release you want to deploy.

```yaml
AWSTemplateFormatVersion: "2010-09-09"
Transform: AWS::Serverless-2016-10-31

Resources:
  SignalsRelay:
    Type: AWS::Serverless::Application
    Properties:
      Location:
        ApplicationId: arn:aws:serverlessrepo:us-east-1:123456789012:applications/signals-relay
        SemanticVersion: <published-version>
      Parameters:
        SpanLogGroupName: aws/spans
        ExportMode: direct

Outputs:
  RelayFunctionArn:
    Value: !GetAtt SignalsRelay.Outputs.ProcessorRelayFunctionArn
```

Add `CollectorExtensionArn` when `ExportMode=collector`. Optional settings such
as `DeploymentId`, `VpcId`, and `SubnetIds` can be passed the same way. Keep
the parent stack in `us-east-1` so it can reach the currently shared SAR app.

When you deploy a parent SAM template that embeds `signals-relay`, acknowledge
the nested application plus the child app's IAM and resource-policy
requirements:

```bash
sam deploy \
  --stack-name my-signals-relay-wrapper \
  --capabilities CAPABILITY_IAM CAPABILITY_RESOURCE_POLICY CAPABILITY_AUTO_EXPAND
```

The nested app's sharing rules still apply to the parent stack. If the
`signals-relay` SAR app has only been shared with your AWS account or AWS
Organization, the parent stack can only be deployed by an account that already
has permission to deploy the child application.

### Deploy From AWS CDK

AWS CDK can synthesize the same nested SAR pattern by using the SAM L1
construct:

```ts
import * as cdk from 'aws-cdk-lib';
import * as sam from 'aws-cdk-lib/aws-sam';
import { Construct } from 'constructs';

export class SignalsRelayStack extends cdk.Stack {
  constructor(scope: Construct, id: string, props?: cdk.StackProps) {
    super(scope, id, props);

    this.templateOptions.transforms = ['AWS::Serverless-2016-10-31'];

    const app = new sam.CfnApplication(this, 'SignalsRelay', {
      location: {
        applicationId:
          'arn:aws:serverlessrepo:us-east-1:123456789012:applications/signals-relay',
        semanticVersion: '<published-version>',
      },
      parameters: {
        SpanLogGroupName: 'aws/spans',
        ExportMode: 'direct',
      },
    });

    new cdk.CfnOutput(this, 'RelayFunctionArn', {
      value: app.getAtt('Outputs.ProcessorRelayFunctionArn').toString(),
    });
  }
}
```

If you need `collector` mode, add `CollectorExtensionArn` to the `parameters`
map. Use the same approach for optional `DeploymentId`, `VpcId`, and
comma-separated `SubnetIds` values.

### Deploy From Terraform

The Terraform AWS provider has native SAR resources and data sources, so you
can deploy the application without wrapping your own CloudFormation stack:

Replace the placeholder application ARN and version defaults with the values
published for the release you want to deploy.

```hcl
variable "signals_relay_application_id" {
  type    = string
  default = "arn:aws:serverlessrepo:us-east-1:123456789012:applications/signals-relay"
}

variable "signals_relay_version" {
  type    = string
  default = "<published-version>"
}

provider "aws" {
  region = "us-east-1"
}

data "aws_serverlessapplicationrepository_application" "signals_relay" {
  application_id   = var.signals_relay_application_id
  semantic_version = var.signals_relay_version
}

resource "aws_serverlessapplicationrepository_cloudformation_stack" "signals_relay" {
  name             = "signals-relay"
  application_id   = var.signals_relay_application_id
  semantic_version = var.signals_relay_version
  capabilities     = data.aws_serverlessapplicationrepository_application.signals_relay.required_capabilities

  parameters = {
    SpanLogGroupName = "aws/spans"
    ExportMode       = "direct"
  }
}

output "relay_function_arn" {
  value = aws_serverlessapplicationrepository_cloudformation_stack.signals_relay.outputs["ProcessorRelayFunctionArn"]
}
```

For `collector` mode, add `CollectorExtensionArn` to the `parameters` map. If
you pass VPC settings, `SubnetIds` should be provided as the comma-separated
string form expected by the nested stack parameter.

## Install From Source

Use this path when you want to inspect, modify, or test the repository before
deploying it. This path is more operationally involved than the SAR install
path because it uses the local SAM/bootstrap tooling and also prepares the local
`public_publish` profile used for SAR publication.

Local source installs assume:

- Python `3.11` or later
- Rust `1.91` or later
- `cargo-lambda` on your `PATH`
- `uv`
- AWS SAM CLI
- AWS credentials for the deploy and publish roles you want to use

For local installs from source:

```bash
export SIGNALS_RELAY_MONITORING_PROFILE="your-deploy-profile"
export SIGNALS_RELAY_DEPLOYMENT_ID="replace-me"
export SIGNALS_RELAY_PUBLIC_PROFILE="your-publish-profile"
export SIGNALS_RELAY_PUBLIC_SAR_BUCKET="your-sar-artifacts-bucket"
uv run ./scripts/init_samconfig.py
sam build --template-file template.yaml
sam deploy --stack-name signals-relay
```

These variables describe two logical roles. `SIGNALS_RELAY_MONITORING_PROFILE`
is the AWS profile for deploying and operating the relay stack. The
`SIGNALS_RELAY_PUBLIC_PROFILE` and `SIGNALS_RELAY_PUBLIC_SAR_BUCKET` values are
for the `public_publish` configuration used to package and publish the app to
SAR. Some teams keep deployment and publication in separate AWS accounts, but
that split is optional. If the same account handles both duties in your setup,
use the same AWS profile for both variables and point the SAR artifacts bucket
at that account.

The generator renders the ignored local `samconfig.toml` from the checked-in
`samconfig.example.toml` template. The script includes inline PEP 723 metadata,
so `uv run` enforces the Python `3.11`+ requirement. Set
`SIGNALS_RELAY_PUBLIC_PROFILE` and `SIGNALS_RELAY_PUBLIC_SAR_BUCKET` before
running it, because the generator writes the `public_publish` config too and
fails instead of emitting placeholder publication values. Set
`SIGNALS_RELAY_REGION` when the local deployment should target a Region other
than `us-east-1`. The generated `default` and `collector` deploy profiles
inherit that value, but the `public_publish` config intentionally stays pinned
to `us-east-1` for the current SAR publication path.

For collector mode:

```bash
sam build --template-file template.yaml
sam deploy --config-env collector --stack-name signals-relay
```

Collector mode requires `CollectorExtensionArn` from the upstream
`open-telemetry/opentelemetry-lambda` release set.

## Upgrade An Existing Install

If you installed from SAR, deploy the newer SAR application version to the same
stack. If you installed from source, pull the newer tag or release artifact and
re-run `sam build` and `sam deploy` with the same stack name.
