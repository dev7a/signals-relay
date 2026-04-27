# Install And Deployment

Use this guide when you want to deploy, evaluate, or upgrade Signals Relay.

For repository publication and release maintenance, see
[release.md](./release.md).

## Prerequisites

Signals Relay currently publishes a shared SAR application in `us-east-1`.
Deploy from that Region unless you are working from a source checkout and know
which parts you need to change.

Before deployment, create the shared Secrets Manager secret in the target
account and Region:

```json
{
  "endpoint": "https://example.com",
  "headers": {
    "authorization": "Bearer ...",
    "x-api-key": "..."
  }
}
```

The secret name must be `signals-relay/secrets/collector`. Both export modes
use this same secret shape. In `direct` mode, the relay Lambda reads it at
startup. In `collector` mode, the OpenTelemetry Lambda collector extension
resolves it from the collector config.

## Install From GitHub Release Quick Launch

Use this path when you want the CloudFormation console to pre-load the correct
published SAR application version.

1. Open the GitHub Release for the version you want to deploy.
2. In **CloudFormation Quick Launch**, choose:
   - **No VPC** when the relay Lambda does not need VPC networking.
   - **Existing VPC** when the relay Lambda should run in selected VPC subnets.
3. Review the pre-loaded stack in the CloudFormation console.
4. Set the application parameters:
   - `SpanLogGroupName`: defaults to `aws/spans`.
   - `ExportMode`: use `direct` unless you need the collector extension.
   - `CollectorExtensionArn`: required only when `ExportMode=collector`.
   - `DeploymentId`: optional marker for forcing fresh Lambda environments.
   - `VpcId` and `SubnetIds`: shown only by the VPC launch template.
5. Acknowledge the capability prompts and create the stack.

The quick-launch templates are versioned release artifacts. They create a
parent CloudFormation stack that deploys the published SAR application as a
nested application. The target account must already be allowed to deploy that
SAR application version.

## Install From SAR

Use this path when you prefer the AWS Serverless Application Repository console
or when you want to deploy the shared app directly from SAR.

1. Open the `signals-relay` SAR application in `us-east-1`. The target account
   must be allowed to deploy the shared application.
2. Choose the published semantic version you want to deploy.
3. Configure the same parameters described in the quick-launch section.
4. Acknowledge the required CloudFormation capabilities and deploy.

If you choose `collector` mode, use an upstream OpenTelemetry Lambda collector
layer ARN that matches your Region and architecture. In `direct` mode, the
relay exports to the OTLP endpoint from the shared secret without the collector
extension.

## Required CloudFormation Capabilities

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

## Deploy From Another SAM Template

Use `AWS::Serverless::Application` when you want to compose Signals Relay into
a larger SAM or CloudFormation stack.

Replace the placeholder application ARN and semantic version with values from
the release you want to deploy.

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

Deploy the parent stack with the required capabilities:

```bash
sam deploy \
  --stack-name my-signals-relay-wrapper \
  --capabilities CAPABILITY_IAM CAPABILITY_NAMED_IAM CAPABILITY_RESOURCE_POLICY CAPABILITY_AUTO_EXPAND
```

Add `CollectorExtensionArn` when `ExportMode=collector`. Pass optional settings
such as `DeploymentId`, `VpcId`, and `SubnetIds` the same way. Keep the parent
stack in `us-east-1` for the current shared SAR publication path.

## Deploy From AWS CDK

AWS CDK can synthesize the same nested SAR pattern by using the SAM L1
construct.

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

Add `CollectorExtensionArn` for collector mode. Pass `SubnetIds` as the
comma-separated string expected by the nested stack parameter when you deploy
the SAR application directly from CDK.

## Deploy From Terraform

The Terraform AWS provider can deploy the SAR application without a custom
CloudFormation wrapper.

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

For collector mode, add `CollectorExtensionArn` to `parameters`. If you pass
VPC settings directly to the SAR stack, provide `SubnetIds` as a
comma-separated string.

## Deploy From Source

Use this path when you want to inspect, modify, or test the repository before
deploying it. Source deployment requires more local tooling than the SAR and
quick-launch paths.

Local source installs assume:

- Python `3.11` or later
- Rust `1.91` or later
- `cargo-lambda` on your `PATH`
- `uv`
- AWS SAM CLI
- AWS credentials for the target deployment account and Region

Generate local SAM configuration:

```bash
export SIGNALS_RELAY_MONITORING_PROFILE="your-deploy-profile"
export SIGNALS_RELAY_DEPLOYMENT_ID="replace-me"
export SIGNALS_RELAY_PUBLIC_PROFILE="your-publish-profile"
export SIGNALS_RELAY_PUBLIC_SAR_BUCKET="your-sar-artifacts-bucket"
uv run ./scripts/init_samconfig.py
```

Build and deploy direct mode:

```bash
sam build --template-file template.yaml
sam deploy --stack-name signals-relay
```

Deploy collector mode:

```bash
sam build --template-file template.yaml
sam deploy --config-env collector --stack-name signals-relay
```

The generator renders the ignored local `samconfig.toml` from
`samconfig.example.toml`. Set `SIGNALS_RELAY_REGION` when local deployment
should target a Region other than `us-east-1`. The generated `default` and
`collector` profiles inherit that Region, while `public_publish` remains pinned
to `us-east-1` for the current SAR publication path.

## Upgrade An Existing Install

- GitHub Release quick launch: open the newer release and launch the same
  deployment path against the existing stack name.
- SAR console: update the existing stack to the newer SAR semantic version.
- IaC parent stack: update `SemanticVersion` in the parent template and deploy.
- Source deployment: pull the newer tag or branch, then run `sam build` and
  `sam deploy` with the same stack name.

Bump `DeploymentId` when you need CloudFormation to force fresh Lambda
execution environments, for example after rotating the shared OTLP secret.
