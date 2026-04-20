#!/usr/bin/env python3
# /// script
# requires-python = ">=3.11"
# ///
"""Generate a local samconfig.toml from the checked-in template.

Requires Python 3.11+.
"""

from __future__ import annotations

import argparse
import os
import sys
from pathlib import Path
from string import Template

if sys.version_info < (3, 11):
    raise SystemExit("scripts/init_samconfig.py requires python3.11 or newer")

import tomllib


DEFAULT_COLLECTOR_EXTENSION_ARN = (
    "arn:aws:lambda:us-east-1:184161586896:layer:"
    "opentelemetry-collector-arm64-0_21_0:1"
)
DEFAULT_DEPLOY_REGION = "us-east-1"

REQUIRED_ENV_VARS = {
    "MONITORING_PROFILE": "SIGNALS_RELAY_MONITORING_PROFILE",
    "DEPLOYMENT_ID": "SIGNALS_RELAY_DEPLOYMENT_ID",
    "PUBLIC_PROFILE": "SIGNALS_RELAY_PUBLIC_PROFILE",
    "PUBLIC_SAR_BUCKET": "SIGNALS_RELAY_PUBLIC_SAR_BUCKET",
}

PLACEHOLDER_ENV_VARS = {
    **REQUIRED_ENV_VARS,
    "DEPLOY_REGION": "SIGNALS_RELAY_REGION",
    "PUBLIC_PROFILE": "SIGNALS_RELAY_PUBLIC_PROFILE",
    "PUBLIC_SAR_BUCKET": "SIGNALS_RELAY_PUBLIC_SAR_BUCKET",
    "COLLECTOR_EXTENSION_ARN": "SIGNALS_RELAY_COLLECTOR_EXTENSION_ARN",
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
        if value is None or value == "":
            missing.append(env_name)
            continue
        values[template_key] = value

    if missing:
        missing_list = ", ".join(sorted(missing))
        raise SystemExit(f"Missing required environment variables: {missing_list}")

    values["DEPLOY_REGION"] = os.environ.get(
        "SIGNALS_RELAY_REGION",
        DEFAULT_DEPLOY_REGION,
    ) or DEFAULT_DEPLOY_REGION
    values["COLLECTOR_EXTENSION_ARN"] = os.environ.get(
        "SIGNALS_RELAY_COLLECTOR_EXTENSION_ARN",
        DEFAULT_COLLECTOR_EXTENSION_ARN,
    ) or DEFAULT_COLLECTOR_EXTENSION_ARN
    return values


def validate_rendered_toml(text: str) -> None:
    if "${" in text:
        raise SystemExit("Generated samconfig.toml still contains unresolved placeholders")

    try:
        parsed = tomllib.loads(text)
    except tomllib.TOMLDecodeError as exc:
        location = ""
        if getattr(exc, "lineno", None) is not None and getattr(exc, "colno", None) is not None:
            location = f" at line {exc.lineno}, column {exc.colno}"
        raise SystemExit(f"Generated samconfig.toml is invalid TOML{location}: {exc}") from None
    stale_overrides: list[str] = []

    for config_env in ("default", "collector"):
        overrides = (
            parsed.get(config_env, {})
            .get("deploy", {})
            .get("parameters", {})
            .get("parameter_overrides", [])
        )
        for override in overrides:
            if override.startswith("OtlpTargetSecretArn="):
                stale_overrides.append(override)
            if "collector/secrets" in override:
                stale_overrides.append(override)

    if stale_overrides:
        formatted = ", ".join(sorted(stale_overrides))
        raise SystemExit(
            "Generated samconfig.toml still contains stale secret contract overrides: "
            f"{formatted}"
        )


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
    values = load_template_values()
    try:
        rendered = template.substitute(values)
    except KeyError as exc:
        missing_placeholder = exc.args[0]
        env_name = PLACEHOLDER_ENV_VARS.get(missing_placeholder)
        if env_name is not None:
            raise SystemExit(
                "Template contains placeholder "
                f"${{{missing_placeholder}}} but environment variable {env_name} is not set"
            ) from None
        raise SystemExit(
            f"Template contains unexpected placeholder: ${{{missing_placeholder}}}"
        ) from None
    validate_rendered_toml(rendered)
    output_path.write_text(rendered)
    print(f"Wrote {output_path}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
