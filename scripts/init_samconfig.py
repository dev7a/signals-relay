#!/usr/bin/env python3
"""Generate a local samconfig.toml from the checked-in template."""

from __future__ import annotations

import argparse
import os
import sys
import tomllib
from pathlib import Path
from string import Template


DEFAULT_COLLECTOR_EXTENSION_ARN = (
    "arn:aws:lambda:us-east-1:184161586896:layer:"
    "opentelemetry-collector-arm64-0_21_0:1"
)

REQUIRED_ENV_VARS = {
    "MONITORING_PROFILE": "SIGNALS_RELAY_MONITORING_PROFILE",
    "PUBLIC_PROFILE": "SIGNALS_RELAY_PUBLIC_PROFILE",
    "PUBLIC_SAR_BUCKET": "SIGNALS_RELAY_PUBLIC_SAR_BUCKET",
    "DEPLOYMENT_ID": "SIGNALS_RELAY_DEPLOYMENT_ID",
}


def parse_args() -> argparse.Namespace:
    root = Path(__file__).resolve().parent.parent
    parser = argparse.ArgumentParser(
        description="Generate a local samconfig.toml from environment variables."
    )
    parser.add_argument(
        "--template",
        type=Path,
        default=root / "samconfig.example.toml",
        help="Template file to render",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=root / "samconfig.toml",
        help="Destination file to write",
    )
    parser.add_argument(
        "--force",
        action="store_true",
        help="Overwrite an existing output file",
    )
    return parser.parse_args()


def load_template_values() -> dict[str, str]:
    missing: list[str] = []
    values: dict[str, str] = {}

    for template_key, env_name in REQUIRED_ENV_VARS.items():
        value = os.environ.get(env_name)
        if not value:
            missing.append(env_name)
            continue
        values[template_key] = value

    if missing:
        missing_list = ", ".join(sorted(missing))
        raise SystemExit(f"Missing required environment variables: {missing_list}")

    values["COLLECTOR_EXTENSION_ARN"] = os.environ.get(
        "SIGNALS_RELAY_COLLECTOR_EXTENSION_ARN",
        DEFAULT_COLLECTOR_EXTENSION_ARN,
    )
    return values


def validate_rendered_toml(text: str) -> None:
    if "${" in text:
        raise SystemExit("Generated samconfig.toml still contains unresolved placeholders")

    if "OtlpTargetSecretArn=" in text:
        raise SystemExit("Generated samconfig.toml still contains removed OtlpTargetSecretArn")

    if "collector/secrets" in text:
        raise SystemExit("Generated samconfig.toml still contains stale collector/secrets path")

    if "signals-relay/secrets/collector" not in text:
        raise SystemExit("Generated samconfig.toml is missing the shared secret contract")

    tomllib.loads(text)


def main() -> int:
    args = parse_args()

    template_path = args.template.resolve()
    output_path = args.output.resolve()

    if not template_path.is_file():
        raise SystemExit(f"Template not found: {template_path}")

    if output_path.exists() and not args.force:
        raise SystemExit(
            f"Refusing to overwrite existing file: {output_path}\n"
            "Pass --force if you want to regenerate it."
        )

    template = Template(template_path.read_text())
    rendered = template.substitute(load_template_values())
    validate_rendered_toml(rendered)
    output_path.write_text(rendered)
    print(f"Wrote {output_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
