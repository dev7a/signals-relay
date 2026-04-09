# Release and Packaging

This repository releases the reusable `signals-relay-core` crate artifact and the deployable SAM application together under one coordinated semantic version.

## What Gets Released

Each `v<version>` release packages both deliverables from the same repo state:

- the `signals-relay-core` crate artifact
- the deployable SAM application defined in [`template.yaml`](../template.yaml)

The release workflow packages the crate artifact for GitHub-based consumers and publishes the SAM application through AWS SAM and the Serverless Application Repository path.

## Release Flow

Releases are driven from the coordinated repo version and published from a matching Git tag.

1. Update [`Cargo.toml`](../Cargo.toml) `workspace.package.version`.
2. Update [`template.yaml`](../template.yaml) `Metadata.AWS::ServerlessRepo::Application.SemanticVersion` to the same value.
3. Run the `release` workflow manually from the branch you want to release.
4. The workflow derives the tag as `v<version>`, fails if that tag already exists, creates it, and publishes the release from that tagged commit in the same run.
5. If you push a matching tag outside the workflow, the same publish job still runs on the `push.tags` trigger.

Examples:

- `0.1.0` becomes `v0.1.0`
- `0.1.0-beta.1` becomes `v0.1.0-beta.1`

The tag value remains the effective semantic version for the publish job, because:

- the Rust workspace packages are expected to match that version directly
- the SAM template `SemanticVersion` is expected to match it too
- the workflow passes the extracted version to `sam publish --semantic-version`

The workflow expects:

- `AWS_ROLE_TO_ASSUME` for GitHub Actions OIDC authentication
- `SAR_ARTIFACT_BUCKET` for packaged template and asset uploads during release

In the current `dev7a` setup, those values come from the public-account infrastructure stack in the companion `oidc-gha-provider` project:

- `SignalsRelayPublisherRoleArn` -> `AWS_ROLE_TO_ASSUME`
- `SignalsRelaySarArtifactsBucketName` -> `SAR_ARTIFACT_BUCKET`

## Manual SAR Publish

For manual publication from a workstation, configure a dedicated `public_publish` environment in `samconfig.toml` that targets the publication account and its SAR artifacts bucket.

Before publishing manually, update the coordinated repo version first:

- set [`Cargo.toml`](../Cargo.toml) `workspace.package.version`
- set [`template.yaml`](../template.yaml) `Metadata.AWS::ServerlessRepo::Application.SemanticVersion`

Those values should match the version you plan to publish.

With that environment in place, the CLI flow is:

```bash
sam build --config-env public_publish --template-file template.yaml
sam package --config-env public_publish
sam publish --config-env public_publish --semantic-version 0.1.0
```

The `package` step writes a packaged template to `.aws-sam/publish-public.yaml`. Keep `--semantic-version` aligned with the repo version above, and override `--s3-prefix` if you want a version-specific upload path instead of the default manual prefix.

## Consumer Install

For local installs from source:

```bash
cp samconfig.example.toml samconfig.toml
sam build --template-file template.yaml
sam deploy --stack-name signals-relay
```

For collector mode:

```bash
sam build --template-file template.yaml
sam deploy --config-env collector --stack-name signals-relay
```

Direct mode is the default. Set `OtlpTargetSecretArn` in `samconfig.toml` for a secret-backed direct export path, or use `ExportMode=collector` together with `CollectorExtensionArn` for collector-backed export.

## Consumer Upgrade

- Pull the newer tag or release artifact.
- Re-run `sam build` and `sam deploy` with the same stack name.
- Bump `DeploymentId` if you need to force Lambda to refresh execution environments after a secret rotation.

If the application is later shared through Serverless Application Repository, the same `vX.Y.Z` tag should map directly to the published app version.

## Coordinated Release Policy

The repository intentionally republishes both deliverables together, even if a given change only affects one of them.

- A core-only change still produces a fresh SAM app release.
- An app-only change still produces a fresh `signals-relay-core` crate artifact.

That keeps the release process simple: one tag, one workflow, one repo version.

## SAR Publication Direction

The repository is structured so the app can be published to the AWS Serverless Application Repository once the publication account and sharing model are ready.

- `template.yaml` includes `AWS::ServerlessRepo::Application` metadata.
- `sam publish` uses the packaged template emitted by the release workflow.
- The release workflow keeps the semantic version explicit so SAR versions and Git tags stay aligned.

This document does not claim that the application is already public in SAR. It only describes the publication path the repo is prepared for.
