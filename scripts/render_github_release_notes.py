#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from urllib.parse import quote


DEFAULT_REPOSITORY = "dev7a/signals-relay"
LAUNCH_BADGE_PATH = "docs/assets/cloudformation-launch-badge.svg"


def require_string(data: dict, key: str) -> str:
    value = data.get(key)
    if not isinstance(value, str) or not value:
        raise SystemExit(f"error: manifest is missing non-empty string field '{key}'")
    return value


def require_launch_template(data: dict, name: str) -> dict:
    launch_templates = data.get("launchTemplates")
    if not isinstance(launch_templates, dict):
        raise SystemExit("error: manifest is missing launchTemplates object")

    template = launch_templates.get(name)
    if not isinstance(template, dict):
        raise SystemExit(f"error: manifest is missing launchTemplates.{name} object")

    for key in ("templateUrl", "quickCreateUrl", "s3Uri"):
        value = template.get(key)
        if not isinstance(value, str) or not value:
            raise SystemExit(f"error: manifest is missing launchTemplates.{name}.{key}")

    return template


def github_blob_raw_url(repository: str, ref: str, path: str) -> str:
    return (
        "https://github.com/"
        f"{quote(repository, safe='/')}/blob/{quote(ref, safe='')}/"
        f"{quote(path, safe='/')}?raw=1"
    )


def sar_application_url(application_id: str) -> str:
    parts = application_id.split(":", 5)
    if len(parts) != 6:
        raise SystemExit(f"error: invalid SAR application ID '{application_id}'")

    _, _, service, region, account_id, resource = parts
    if service != "serverlessrepo":
        raise SystemExit(f"error: application ID is not a SAR ARN: '{application_id}'")
    if not region or not account_id:
        raise SystemExit(
            f"error: SAR application ID is missing region or account: '{application_id}'"
        )
    if not resource.startswith("applications/"):
        raise SystemExit(
            f"error: SAR application ID is missing applications resource: '{application_id}'"
        )

    application_name = resource.removeprefix("applications/")
    if not application_name:
        raise SystemExit(
            f"error: SAR application ID is missing application name: '{application_id}'"
        )

    return (
        "https://serverlessrepo.aws.amazon.com/applications/"
        f"{quote(region, safe='')}/"
        f"{quote(account_id, safe='')}/"
        f"{quote(application_name, safe='')}"
    )


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Render GitHub Release notes from a release manifest."
    )
    parser.add_argument("--manifest", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()

    try:
        manifest = json.loads(args.manifest.read_text())
    except OSError as exc:
        raise SystemExit(
            f"error: failed to read manifest '{args.manifest}': {exc}"
        )
    except json.JSONDecodeError as exc:
        raise SystemExit(
            f"error: failed to parse manifest '{args.manifest}': {exc}"
        )
    if not isinstance(manifest, dict):
        raise SystemExit(
            f"error: manifest '{args.manifest}' root must be a JSON object"
        )
    tag = require_string(manifest, "tag")
    version = require_string(manifest, "version")
    commit = require_string(manifest, "commit")
    application_id = require_string(manifest, "applicationId")
    application_url = sar_application_url(application_id)
    semantic_version = require_string(manifest, "semanticVersion")
    no_vpc = require_launch_template(manifest, "noVpc")
    vpc = require_launch_template(manifest, "vpc")
    repository = os.environ.get("GITHUB_REPOSITORY", DEFAULT_REPOSITORY)
    launch_badge_url = github_blob_raw_url(repository, tag, LAUNCH_BADGE_PATH)
    launch_badge = (
        "![Launch stack in AWS CloudFormation]"
        f"({launch_badge_url})"
    )

    lines = [
        (
            "This release publishes the Signals Relay SAR application and versioned "
            "CloudFormation launch templates."
        ),
        "",
        "## SAR Application",
        "",
        f"- SAR application: {application_url}",
        f"- Application ID: `{application_id}`",
        f"- Semantic version: `{semantic_version}`",
        f"- Source commit: `{commit}`",
        "",
        "## CloudFormation Quick Launch",
        "",
        "| Deployment path | Quick launch | Template |",
        "| --- | --- | --- |",
        (
            "| No VPC | "
            f"[{launch_badge}]({no_vpc['quickCreateUrl']}) | "
            f"[launch-no-vpc.yaml]({no_vpc['templateUrl']}) |"
        ),
        (
            "| Existing VPC | "
            f"[{launch_badge}]({vpc['quickCreateUrl']}) | "
            f"[launch-vpc.yaml]({vpc['templateUrl']}) |"
        ),
        "",
        "Before creating the stack, acknowledge the CloudFormation prompts for IAM resources, "
        "IAM resources with custom names, and `CAPABILITY_AUTO_EXPAND`. Tooling that names "
        "capabilities explicitly should include `CAPABILITY_IAM`, `CAPABILITY_NAMED_IAM`, "
        "`CAPABILITY_RESOURCE_POLICY`, and `CAPABILITY_AUTO_EXPAND`.",
        "",
        "The CloudFormation templates deploy the SAR application version shown above. The target "
        "AWS account must be allowed to deploy that SAR application version.",
        "",
        "## Release Artifacts",
        "",
        f"- CloudFormation manifest: `{no_vpc['s3Uri'].rsplit('/', 1)[0]}/manifest.json`",
        f"- No-VPC launch template: `{no_vpc['s3Uri']}`",
        f"- VPC launch template: `{vpc['s3Uri']}`",
        "",
    ]

    if version != semantic_version:
        raise SystemExit(
            "error: manifest version "
            f"'{version}' does not match semanticVersion '{semantic_version}'"
        )

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
