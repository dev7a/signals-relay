# Release and Packaging

This repository releases the reusable `signals-relay-core` crate artifact and the deployable SAM application together under one coordinated semantic version.

## What Gets Released

Each `vX.Y.Z` release packages both deliverables from the same repo state:

- the `signals-relay-core` crate artifact
- the deployable SAM application defined in [`template.yaml`](../template.yaml)

The release workflow packages the crate artifact for GitHub-based consumers and publishes the SAM application through AWS SAM and the Serverless Application Repository path.

## Release Flow

Releases are driven by semantic version tags using the `vX.Y.Z` pattern.

1. Update the workspace package version in [`Cargo.toml`](../Cargo.toml).
2. Create a tag such as `v0.1.0`.
3. Push the tag to GitHub.
4. The release workflow runs once for that tag, validates that the workspace versions match it, packages the crate artifact, builds and packages the SAM application, publishes the application, and uploads the release artifacts.

The tag value becomes the effective semantic version for the release flow, because:

- the Rust workspace packages are expected to match that version directly
- the workflow passes the extracted version to `sam publish --semantic-version`

The workflow expects:

- `AWS_ROLE_TO_ASSUME` for GitHub Actions OIDC authentication
- `SAR_ARTIFACT_BUCKET` for packaged template and asset uploads during release

In the current `dev7a` setup, those values come from the public-account infrastructure stack in the companion `oidc-gha-provider` project:

- `SignalsRelayPublisherRoleArn` -> `AWS_ROLE_TO_ASSUME`
- `SignalsRelaySarArtifactsBucketName` -> `SAR_ARTIFACT_BUCKET`

## Manual SAR Publish

For manual publication from a workstation, configure a dedicated `public_publish` environment in `samconfig.toml` that targets the publication account and its SAR artifacts bucket.

With that environment in place, the CLI flow is:

```bash
sam build --config-env public_publish --template-file template.yaml
sam package --config-env public_publish
sam publish --config-env public_publish --semantic-version 0.1.0
```

The `package` step writes a packaged template to `.aws-sam/publish-public.yaml`. Override `--semantic-version` for each release, and override `--s3-prefix` if you want a version-specific upload path instead of the default manual prefix.

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
