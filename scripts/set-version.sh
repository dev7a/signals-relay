#!/usr/bin/env bash
set -euo pipefail

usage() {
  echo "Usage: ./scripts/set-version.sh <semver>" >&2
  echo "Example: ./scripts/set-version.sh 0.1.0-beta.1" >&2
  exit 1
}

if [[ $# -ne 1 ]]; then
  usage
fi

version="$1"
semver_regex='^(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)\.(0|[1-9][0-9]*)(-((0|[1-9][0-9]*|[0-9A-Za-z-][0-9A-Za-z-]*)(\.(0|[1-9][0-9]*|[0-9A-Za-z-][0-9A-Za-z-]*))*))?(\+([0-9A-Za-z-]+(\.[0-9A-Za-z-]+)*))?$'

if [[ ! "$version" =~ $semver_regex ]]; then
  echo "error: version must be valid semver, got '$version'" >&2
  exit 1
fi

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cargo_toml="$repo_root/Cargo.toml"
template_yaml="$repo_root/template.yaml"

python3 - "$version" "$cargo_toml" "$template_yaml" <<'PY'
from pathlib import Path
import re
import sys

version = sys.argv[1]
cargo_toml = Path(sys.argv[2])
template_yaml = Path(sys.argv[3])


def replace_once(path: Path, pattern: str, replacement: str, description: str) -> None:
    text = path.read_text()
    updated, count = re.subn(pattern, replacement, text, count=1, flags=re.MULTILINE)
    if count != 1:
        raise SystemExit(f"error: could not update {description} in {path}")
    path.write_text(updated)


replace_once(
    cargo_toml,
    r"(\[workspace\.package\]\nversion = \")([^\"]+)(\")",
    rf"\g<1>{version}\g<3>",
    "workspace package version",
)
replace_once(
    template_yaml,
    r"(^[ \t]*SemanticVersion:[ \t]*)([^ \t\r\n]+)([ \t]*$)",
    rf"\g<1>{version}\g<3>",
    "SAM application semantic version",
)
PY

(cd "$repo_root" && cargo update --workspace --offline >/dev/null)

echo "Updated release version to $version"
echo "Derived release tag: v$version"
