# Release Guide

Use this document when you maintain this repository and want to publish a new
coordinated release.

For SAR installs, source deployments, and upgrade guidance, see
[install.md](./install.md).

## What Gets Released

Each `v<version>` release packages both deliverables from the same repository
state:

- the `signals-relay-core` crate artifact
- the deployable SAM application defined in [`template.yaml`](../template.yaml)

The release workflow packages the crate artifact for GitHub-based consumers and
publishes the SAM application through AWS SAM and the Serverless Application
Repository path. It also emits versioned CloudFormation artifacts for
launch-link style installs, but SAR remains the primary installation and
publication path.

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
6. After the tag exists, the workflow creates or updates the matching GitHub
   Release with SAR details, CloudFormation quick-launch links, and the
   versioned launch-template assets.
7. If you push a matching tag outside the workflow, the same publish job still
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
- `CFN_ARTIFACT_BUCKET` as a GitHub Actions repository variable for versioned
  CloudFormation launch-template artifacts

When you run the `release` workflow manually, GitHub presents a `share_scope`
choice with `account` and `organization` values. `account` keeps the published
SAR app private to the publisher account. `organization` follows `sam publish`
with `serverlessrepo put-application-policy`, discovers the current AWS
Organization ID at runtime, and shares the app privately across that
organization. For that org-wide path to work, the role assumed via
`AWS_ROLE_TO_ASSUME` must allow `organizations:DescribeOrganization`. The
tag-push release path keeps the safe default and publishes to the account only.

The `CFN_ARTIFACT_BUCKET` variable points at the separately managed S3
distribution bucket for CloudFormation launch-template artifacts. Keep this
bucket in `us-east-1`, matching the current single-Region release path. The
release workflow uploads those artifacts under an immutable versioned prefix:

```text
signals-relay/cloudformation/releases/<version>/
```

The CloudFormation distribution bucket contains only the small launch wrappers
and their manifest:

```text
signals-relay/cloudformation/releases/<version>/launch-no-vpc.yaml
signals-relay/cloudformation/releases/<version>/launch-vpc.yaml
signals-relay/cloudformation/releases/<version>/manifest.json
```

The Lambda and layer artifacts stay on the SAR publication path through
`SAR_ARTIFACT_BUCKET`; the CloudFormation bucket does not receive a duplicate
packaged SAM child template or duplicate code artifacts. The parent launch
templates are rendered after `sam publish` so their nested SAR application
resource points at the published `ApplicationId` and `SemanticVersion`.

The no-VPC launch template exposes only the common application parameters. The
VPC launch template exposes `VpcId` as `AWS::EC2::VPC::Id` and `SubnetIds` as
`List<AWS::EC2::Subnet::Id>` so the CloudFormation console can offer account
and Region-aware pickers. It passes those selections to the published SAR child
application through an `AWS::Serverless::Application` resource.

The release workflow records the S3 URI, S3 HTTPS template URL, and
CloudFormation quick-create URL for the default no-VPC launch template in
`release/<version>/cloudformation/manifest.json`. The manifest also includes
the SAR `ApplicationId`, the SAR semantic version, and separate no-VPC and VPC
launch-template URLs. Public access, bucket policy, request controls, and
billing alarms for this distribution bucket are managed outside this
repository.

After the publish job succeeds, the release workflow uses that manifest to
publish the GitHub Release notes for the tag. Those notes include plain Markdown
quick-launch links rather than image buttons, plus links to the no-VPC and VPC
template URLs. The workflow also attaches the manifest, both launch templates,
and the `signals-relay-core` crate artifact to the GitHub Release.

The role assumed through `AWS_ROLE_TO_ASSUME` must be able to call
`s3:GetBucketLocation` and `s3:PutObject` on `CFN_ARTIFACT_BUCKET`; the workflow
uses the location check to fail early if the CloudFormation distribution bucket
is not in `us-east-1`. It must also allow `serverlessrepo:GetApplication` so the
workflow can verify the newly published SAR version before writing launch links.

CloudFormation launch installs create a parent stack that creates the published
SAR app as a nested application. The target account must be allowed to deploy
that SAR application, either because it is shared privately or public. Because
the parent wrapper and child application use the SAM transform and the child app
contains IAM resources, operators should expect to acknowledge
`CAPABILITY_AUTO_EXPAND` plus the IAM capability prompts during stack creation.
In the CloudFormation console launch flow, this appears as acknowledgements for
IAM resources, IAM resources with custom names, and `CAPABILITY_AUTO_EXPAND`.
Tooling that names capabilities explicitly should include
`CAPABILITY_IAM`, `CAPABILITY_NAMED_IAM`, `CAPABILITY_RESOURCE_POLICY`, and
`CAPABILITY_AUTO_EXPAND`.

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
- The SAR-backed CloudFormation parent launch templates are additional
  versioned release artifacts for launch-link workflows. They do not replace SAR
  and do not introduce a mutable `latest` launch URL.
- GitHub Release notes are generated from the CloudFormation manifest after the
  SAR app has been published and the versioned launch templates have been
  uploaded.

This document does not claim that the application is already public in SAR. It
describes the publication path and the install paths for either shared SAR
consumers or source-based operators. Validate the org-shared install path from
another member account in `us-east-1` before introducing any future public
sharing step.
