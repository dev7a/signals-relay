#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


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
    semantic_version = require_string(manifest, "semanticVersion")
    no_vpc = require_launch_template(manifest, "noVpc")
    vpc = require_launch_template(manifest, "vpc")

    lines = [
        f"# Signals Relay {tag}",
        "",
        (
            "This release publishes the Signals Relay SAR application and versioned "
            "CloudFormation launch templates."
        ),
        "",
        "## SAR Application",
        "",
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
            f"[Launch in us-east-1]({no_vpc['quickCreateUrl']}) | "
            f"[launch-no-vpc.yaml]({no_vpc['templateUrl']}) |"
        ),
        (
            "| Existing VPC | "
            f"[Launch in us-east-1]({vpc['quickCreateUrl']}) | "
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
