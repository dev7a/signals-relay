# Release Guide

Use this guide to publish a coordinated release from this repository.

For operator install paths, quick-launch deployment, IaC examples, and upgrade
guidance, see [install.md](./install.md).

For website generation and local preview, see [`site/README.md`](../site/README.md).

## Release Prerequisites

The release workflow publishes from one coordinated repository version. Before
starting a release, make sure:

- `scripts/set-version.sh <semver>` has updated `Cargo.toml`, `Cargo.lock`, and
  `template.yaml`.
- The Rust workspace package versions match the SAM
  `AWS::ServerlessRepo::Application.SemanticVersion`.
- The target tag `v<semver>` does not already exist.
- `AWS_ROLE_TO_ASSUME` is configured as a GitHub Actions secret for OIDC.
- `SAR_ARTIFACT_BUCKET` is configured as a GitHub Actions secret for SAM
  package uploads.
- `CFN_ARTIFACT_BUCKET` is configured as a GitHub Actions repository variable
  for CloudFormation launch artifacts.

The current release path is pinned to a single Region, `us-east-1`.

The assumed release role must be able to build, package, publish, share, and
verify one release in that Region. At minimum, the workflow exercises:

- `sts:GetCallerIdentity`
- S3 read/write access for SAR package uploads through `SAR_ARTIFACT_BUCKET`
- `s3:GetBucketLocation` and S3 write access for `CFN_ARTIFACT_BUCKET`
- Serverless Application Repository publish and readback access, including
  `serverlessrepo:GetApplication`
- For `share_scope=organization`, include
  `serverlessrepo:GetApplicationPolicy`, `serverlessrepo:PutApplicationPolicy`, and
  `organizations:DescribeOrganization`

## Manual Workflow Dispatch

Run the `release` workflow manually from the branch or commit you want to
publish.

1. Choose `share_scope=account` to keep the SAR app private to the publisher
   account.
2. Choose `share_scope=organization` to add a private AWS Organization share
   after `sam publish` succeeds.
3. The workflow derives the tag as `v<version>`.
4. The workflow validates that Rust package versions and SAM `SemanticVersion`
   match the tag.
5. The workflow publishes SAR before creating the Git tag.
6. After publish succeeds, the workflow creates the tag and GitHub Release.

For organization sharing, the assumed AWS role must allow
`organizations:DescribeOrganization` and SAR application-policy updates. The
workflow discovers the current Organization ID at runtime and applies a SAR
application policy for that Organization.

## Tag Push Behavior

Pushing a matching `v*` tag also runs the publish workflow. This path uses
`share_scope=account`.

Prefer manual workflow dispatch for normal releases because it publishes first
and creates the tag only after the publish job succeeds. That avoids creating a
release tag for a version that did not publish.

## Artifact Outputs

Each release produces these outputs from the same repository state:

- The `signals-relay-core` crate artifact attached to the GitHub Release
- The SAM application published through SAR
- SAR package artifacts uploaded through `SAR_ARTIFACT_BUCKET`
- CloudFormation launch wrappers uploaded through `CFN_ARTIFACT_BUCKET`
- CloudFormation manifest attached to the GitHub Release
- GitHub Release notes generated from the CloudFormation manifest

SAR remains the canonical publish path. The CloudFormation launch templates are
small parent wrappers that deploy the published SAR application through
`AWS::Serverless::Application`.

The CloudFormation distribution bucket receives only versioned launch artifacts:

```text
signals-relay/cloudformation/releases/<version>/launch-no-vpc.yaml
signals-relay/cloudformation/releases/<version>/launch-vpc.yaml
signals-relay/cloudformation/releases/<version>/manifest.json
```

The workflow does not publish a mutable `latest` CloudFormation URL. Public
access, bucket policy, request controls, and billing alarms for the
CloudFormation distribution bucket are managed outside this repository.

## GitHub Release Notes

After SAR publish and CloudFormation artifact upload succeed, the workflow
creates or updates the GitHub Release for the tag.

The generated notes include:

- SAR application ID
- SAR semantic version
- source commit
- CloudFormation quick-launch badge links for no-VPC and VPC deployment paths
- versioned template URLs
- release artifact references

The badge image is a checked-in SVG referenced from the release tag through a
GitHub `blob/<tag>/...svg?raw=1` URL. The launch targets point to the versioned
CloudFormation quick-create URLs in the manifest.

## SAR Sharing Behavior

`share_scope=account` publishes the SAR app without adding an organization
policy. The application remains deployable only where the publisher account's
SAR sharing model allows it.

`share_scope=organization` follows `sam publish` with
`serverlessrepo put-application-policy`. The workflow verifies that the policy
contains the expected Organization ID before continuing to CloudFormation
artifact generation.

CloudFormation launch installs still require the target account to be allowed
to deploy the SAR application version. The launch wrappers do not bypass SAR
sharing.

## Required Capabilities

CloudFormation launch installs create a parent stack that deploys the published
SAR app as a nested application. Operators should expect prompts for IAM
resources, including resources with custom names, and `CAPABILITY_AUTO_EXPAND`.

Tooling that names capabilities explicitly should include:

```text
CAPABILITY_IAM CAPABILITY_NAMED_IAM CAPABILITY_RESOURCE_POLICY CAPABILITY_AUTO_EXPAND
```

## Failure and rerun notes

The workflow fails fast when:

- the requested tag already exists
- Rust package versions do not match the release tag
- SAM `SemanticVersion` does not match the release tag
- `CFN_ARTIFACT_BUCKET` is missing or is not in `us-east-1`
- the newly published SAR application version cannot be read back

If the workflow fails before `sam publish`, fix the issue and rerun the same
version. If it fails after SAR publish, inspect SAR, the Git tag, S3 artifacts,
and the GitHub Release before rerunning. SAR semantic versions are immutable, so
a partial post-publish failure may require a new version or a targeted manual
repair rather than a blind rerun.

## Manual SAR Publish

Manual publication from a workstation is supported for maintainers who have a
local `public_publish` SAM environment.

1. Run `scripts/set-version.sh <semver>`.
2. Set the same version for the publish command:

   ```bash
   VERSION="<semver>"
   ```

3. Build with the public publish configuration:

   ```bash
   sam build --config-env public_publish --template-file template.yaml
   ```

4. Package the application:

   ```bash
   sam package --config-env public_publish
   ```

5. Publish with the same semantic version:

   ```bash
   sam publish --config-env public_publish --semantic-version "$VERSION"
   ```

The automated GitHub Actions release remains the preferred path because it also
validates version parity, creates the tag only after successful publish, uploads
the CloudFormation launch artifacts, and creates the GitHub Release.
