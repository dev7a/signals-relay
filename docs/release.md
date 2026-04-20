# Release and Packaging

This repository releases the reusable `signals-relay-core` crate artifact and the deployable SAM application together under one coordinated semantic version.

## What Gets Released

Each `v<version>` release packages both deliverables from the same repo state:

- the `signals-relay-core` crate artifact
- the deployable SAM application defined in [`template.yaml`](../template.yaml)

The release workflow packages the crate artifact for GitHub-based consumers and publishes the SAM application through AWS SAM and the Serverless Application Repository path.

## Release Flow

Releases are driven from the coordinated repo version and published from a matching Git tag.

1. Run [`scripts/set-version.sh`](../scripts/set-version.sh) with the target version.
2. Run the `release` workflow manually from the branch you want to release.
3. The workflow derives the tag as `v<version>`, fails if that tag already exists, and publishes the release from the checked-out commit first.
4. Only after the publish job succeeds does the workflow create and push the matching Git tag.
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

Before publishing manually, set the coordinated repo version with [`scripts/set-version.sh`](../scripts/set-version.sh). The script updates [`Cargo.toml`](../Cargo.toml) and [`template.yaml`](../template.yaml), then asks Cargo to refresh the workspace package entries in [`Cargo.lock`](../Cargo.lock).

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

The `package` step writes a packaged template to `.aws-sam/publish-public.yaml`. The `VERSION` shell variable keeps `sam publish` aligned with the repo version, and you can override `--s3-prefix` if you want a version-specific upload path instead of the default manual prefix.

## Consumer Install

For local installs from source:

```bash
export SIGNALS_RELAY_MONITORING_PROFILE="monitoring.admin"
export SIGNALS_RELAY_DEPLOYMENT_ID="replace-me"
export SIGNALS_RELAY_PUBLIC_PROFILE="public.admin"
export SIGNALS_RELAY_PUBLIC_SAR_BUCKET="your-public-sar-artifacts-bucket"
uv run ./scripts/init_samconfig.py
sam build --template-file template.yaml
sam deploy --stack-name signals-relay
```

Local source installs assume Rust `1.91` or later, `cargo-lambda` on your
`PATH`, `uv`, and AWS SAM CLI.

The generator renders the ignored local `samconfig.toml` from the checked-in
`samconfig.example.toml` template via inline PEP 723 script metadata. Set
`SIGNALS_RELAY_PUBLIC_PROFILE` and `SIGNALS_RELAY_PUBLIC_SAR_BUCKET` before
running it, because the generator writes the `public_publish` config too and
fails instead of emitting placeholder publication values. Set
`SIGNALS_RELAY_REGION` when the monitoring deployment should target a region
other than `us-east-1`. The generated `default` and `collector` deploy profiles
inherit that value, but the `public_publish` config intentionally stays pinned
to `us-east-1` for the current SAR publication path.

For collector mode:

```bash
sam build --template-file template.yaml
sam deploy --config-env collector --stack-name signals-relay
```

Direct mode is the default. Create the shared `signals-relay/secrets/collector` secret before deploying, and the app will use it in direct mode by default. Use `ExportMode=collector` together with `CollectorExtensionArn` for collector-backed export.

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

The example SAM config intentionally does not mirror the release version in stack tags. The coordinated release version lives in the repo manifests and release tag, not in deploy-time tagging defaults.

## SAR Publication Direction

The repository is structured so the app can be published to the AWS Serverless Application Repository once the publication account and sharing model are ready.

- `template.yaml` includes `AWS::ServerlessRepo::Application` metadata.
- `sam publish` uses the packaged template emitted by the release workflow.
- The release workflow keeps the semantic version explicit so SAR versions and Git tags stay aligned.

This document does not claim that the application is already public in SAR. It only describes the publication path the repo is prepared for.
