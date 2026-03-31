# App Release and Packaging

This document covers the deployable SAM application track in this repository. The reusable core crate lives on a separate release track, so the steps below intentionally stay focused on the serverless app packaging flow.

## What Gets Released

The app release track packages and publishes the deployable SAM application defined in [`template.yaml`](../template.yaml).

- The AWS SAM template is the deployable unit.
- The release workflow produces versioned release artifacts tied to a semantic version tag.
- The workflow is also aligned with AWS Serverless Application Repository publication, so the same packaged template can be published there once the app is ready to be shared that way.

## Release Flow

Releases are driven by semantic version tags using the `app-vX.Y.Z` pattern.

1. Create a tag such as `app-v0.1.0`.
2. Push the tag to GitHub.
3. The app release workflow builds the SAM application, packages it, publishes the packaged template, and stores versioned artifacts for traceability.

The tag value becomes the effective semantic version for the release flow, because the workflow passes the extracted version to `sam publish --semantic-version`.

The workflow expects:

- `AWS_ROLE_TO_ASSUME` for GitHub Actions OIDC authentication
- `SAR_ARTIFACT_BUCKET` for packaged template and asset uploads during release

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

If the application is later shared through Serverless Application Repository, the same semantic version tag should map directly to the published app version.

## SAR Publication Direction

The repository is now structured so the app can be published to the AWS Serverless Application Repository once the publication account and sharing model are ready.

- `template.yaml` includes `AWS::ServerlessRepo::Application` metadata.
- `sam publish` can use the packaged template emitted by the release workflow.
- The release workflow keeps the semantic version explicit so SAR versions and Git tags stay aligned.

This document does not claim that the application is already public in SAR. It only describes the publication path the repo is prepared for.
