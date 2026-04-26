#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path
from urllib.parse import quote


PLACEHOLDER_TEMPLATE_URL = (
    "https://s3.us-east-1.amazonaws.com/signals-relay-placeholder-bucket/"
    "signals-relay/cloudformation/releases/0.0.0/packaged.yaml"
)

LAUNCH_TEMPLATES = {
    "noVpc": {
        "source": "launch-no-vpc.yaml",
        "output": "launch-no-vpc.yaml",
        "label": "No VPC launch template",
        "stack_name": "signals-relay",
    },
    "vpc": {
        "source": "launch-vpc.yaml",
        "output": "launch-vpc.yaml",
        "label": "VPC launch template",
        "stack_name": "signals-relay-vpc",
    },
}


def s3_uri(bucket: str, key: str) -> str:
    return f"s3://{bucket}/{key}"


def template_url(region: str, bucket: str, key: str) -> str:
    return f"https://s3.{region}.amazonaws.com/{bucket}/{key}"


def quick_create_url(region: str, url: str, stack_name: str) -> str:
    return (
        f"https://{region}.console.aws.amazon.com/cloudformation/home"
        f"?region={region}#/stacks/create/review"
        f"?templateURL={quote(url, safe='')}"
        f"&stackName={quote(stack_name, safe='')}"
    )


def render_template(source: Path, output: Path, application_template_url: str) -> None:
    text = source.read_text()
    if PLACEHOLDER_TEMPLATE_URL not in text:
        raise SystemExit(f"error: {source} does not contain the expected TemplateURL placeholder")
    output.write_text(text.replace(PLACEHOLDER_TEMPLATE_URL, application_template_url))


def main() -> None:
    parser = argparse.ArgumentParser(description="Render versioned CloudFormation launch wrapper templates.")
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--output-dir", type=Path, required=True)
    parser.add_argument("--tag", required=True)
    parser.add_argument("--version", required=True)
    parser.add_argument("--commit", required=True)
    parser.add_argument("--bucket", required=True)
    parser.add_argument("--prefix", required=True)
    parser.add_argument("--region", required=True)
    parser.add_argument("--summary-file", type=Path, required=True)
    args = parser.parse_args()

    args.output_dir.mkdir(parents=True, exist_ok=True)

    application_key = f"{args.prefix}/packaged.yaml"
    application_template_url = template_url(args.region, args.bucket, application_key)
    application = {
        "s3Uri": s3_uri(args.bucket, application_key),
        "templateUrl": application_template_url,
    }

    launch_templates = {}
    for name, config in LAUNCH_TEMPLATES.items():
        output_name = config["output"]
        source_path = args.source_dir / config["source"]
        output_path = args.output_dir / output_name
        render_template(source_path, output_path, application_template_url)

        key = f"{args.prefix}/{output_name}"
        url = template_url(args.region, args.bucket, key)
        launch_templates[name] = {
            "s3Uri": s3_uri(args.bucket, key),
            "templateUrl": url,
            "quickCreateUrl": quick_create_url(args.region, url, config["stack_name"]),
        }

    default_launch = launch_templates["noVpc"]
    manifest = {
        "tag": args.tag,
        "version": args.version,
        "commit": args.commit,
        "bucket": args.bucket,
        "prefix": args.prefix,
        "s3Uri": default_launch["s3Uri"],
        "templateUrl": default_launch["templateUrl"],
        "quickCreateUrl": default_launch["quickCreateUrl"],
        "application": application,
        "launchTemplates": launch_templates,
    }

    manifest_path = args.output_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n")

    lines = [
        "## CloudFormation launch templates",
        "",
        f"- Packaged SAM child template: {application['templateUrl']}",
    ]
    for name, config in LAUNCH_TEMPLATES.items():
        rendered = launch_templates[name]
        lines.extend(
            [
                f"- {config['label']}: {rendered['templateUrl']}",
                f"- {config['label']} quick-create URL: {rendered['quickCreateUrl']}",
            ]
        )
    args.summary_file.write_text("\n".join(lines) + "\n")


if __name__ == "__main__":
    main()
