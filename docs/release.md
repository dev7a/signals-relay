# Release and Packaging

This document covers three workflows:

- install the application from AWS Serverless Application Repository (SAR)
- deploy the application from a source checkout
- publish a new coordinated release if you maintain this repository

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

## What Gets Released

Each `v<version>` release packages both deliverables from the same repository
state:

- the `signals-relay-core` crate artifact
- the deployable SAM application defined in [`template.yaml`](../template.yaml)

The release workflow packages the crate artifact for GitHub-based consumers and
publishes the SAM application through AWS SAM and the Serverless Application
Repository path.

## Publish A Release

Releases are driven from the coordinated repository version and published from a
matching Git tag.

1. Run [`scripts/set-version.sh`](../scripts/set-version.sh) with the target
   version.
2. Run the `release` workflow manually from the branch you want to release.
3. Choose `share_scope=account` to keep the SAR app private to the publisher
   account, or `share_scope=organization` to add a private org-wide share after
   publish.
4. The workflow derives the tag as `v<version>`, fails if that tag already
   exists, and publishes the release from the checked-out commit first.
5. Only after the publish job succeeds does the workflow create and push the
   matching Git tag.
6. If you push a matching tag outside the workflow, the same publish job still
   runs on the `push.tags` trigger with account-only sharing.

Examples:

- `0.1.0` becomes `v0.1.0`
- `0.1.0-beta.1` becomes `v0.1.0-beta.1`

The tag value remains the effective semantic version for the publish job,
because:

- the Rust workspace packages are expected to match that version directly
- the SAM template `SemanticVersion` is expected to match it too
- the workflow passes the extracted version to `sam publish --semantic-version`

The workflow expects:

- `AWS_ROLE_TO_ASSUME` for GitHub Actions OIDC authentication
- `SAR_ARTIFACT_BUCKET` for packaged template and asset uploads during release

When you run the `release` workflow manually, GitHub presents a `share_scope`
choice with `account` and `organization` values. `account` keeps the published
SAR app private to the publisher account. `organization` follows `sam publish`
with `serverlessrepo put-application-policy`, discovers the current AWS
Organization ID at runtime, and shares the app privately across that
organization. For that org-wide path to work, the role assumed via
`AWS_ROLE_TO_ASSUME` must allow `organizations:DescribeOrganization`. The
tag-push release path keeps the safe default and publishes to the account only.

## Manual SAR Publish

For manual publication from a workstation, configure a dedicated
`public_publish` environment in `samconfig.toml` that targets the SAR
publishing profile and artifacts bucket you want to use.

Before publishing manually, set the coordinated repository version with
[`scripts/set-version.sh`](../scripts/set-version.sh). The script updates
[`Cargo.toml`](../Cargo.toml) and [`template.yaml`](../template.yaml), then asks
Cargo to refresh the workspace package entries in
[`Cargo.lock`](../Cargo.lock).

With that environment in place, the CLI flow is:

```bash
VERSION="$(
  python3 - <<'PY'
import json
import subprocess

metadata = json.loads(
    subprocess.check_output(
        ["cargo", "metadata", "--no-deps", "--format-version", "1"],
        text=True,
    )
)

workspace_members = set(metadata["workspace_members"])
versions = {
    package["version"]
    for package in metadata["packages"]
    if package["id"] in workspace_members
}

if len(versions) != 1:
    raise SystemExit(f"expected one coordinated workspace version, found: {sorted(versions)}")

print(next(iter(versions)))
PY
)"

sam build --config-env public_publish --template-file template.yaml
sam package --config-env public_publish
sam publish --config-env public_publish --semantic-version "$VERSION"
```

The `package` step writes a packaged template to
`.aws-sam/publish-public.yaml`. The `VERSION` shell variable keeps `sam publish`
aligned with the repository version, and you can override `--s3-prefix` if you
want a version-specific upload path instead of the default manual prefix.

## Upgrade An Existing Install

If you installed from SAR, deploy the newer SAR application version to the same
stack. If you installed from source, pull the newer tag or release artifact and
re-run `sam build` and `sam deploy` with the same stack name.

Bump `DeploymentId` if you need to force Lambda to refresh execution
environments after a secret rotation.

## Coordinated Release Policy

The repository intentionally republishes both deliverables together, even if a
given change only affects one of them.

- A core-only change still produces a fresh SAM app release.
- An app-only change still produces a fresh `signals-relay-core` crate
  artifact.

That keeps the release process simple: one tag, one workflow, one repository
version.

The example SAM config intentionally does not mirror the release version in
stack tags. The coordinated release version lives in the repository manifests
and release tag, not in deploy-time tagging defaults.

## SAR Publication Direction

The repository is structured so the app can be published to AWS Serverless
Application Repository when the desired publication and sharing configuration is
ready.

- `template.yaml` includes `AWS::ServerlessRepo::Application` metadata.
- `sam publish` uses the packaged template emitted by the release workflow.
- The release workflow keeps the semantic version explicit so SAR versions and
  Git tags stay aligned.
- When `share_scope=organization` is selected for a manual release, the
  workflow discovers the current AWS Organization ID and adds a private
  org-wide share after publish instead of making the app public.

This document does not claim that the application is already public in SAR. It
describes the publication path and the install paths for either shared SAR
consumers or source-based operators. Validate the org-shared install path from
another member account in `us-east-1` before introducing any future public
sharing step.
