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
cargo_lock="$repo_root/Cargo.lock"
template_yaml="$repo_root/template.yaml"

if backup_dir="$(mktemp -d "${TMPDIR:-/tmp}/set-version.XXXXXX" 2>/dev/null)"; then
  :
elif backup_dir="$(mktemp -d -t set-version 2>/dev/null)"; then
  :
else
  echo "error: failed to create temporary backup directory" >&2
  exit 1
fi

restore_needed=1

cleanup() {
  rm -rf "$backup_dir"
}

restore_release_files() {
  cp "$backup_dir/Cargo.toml" "$cargo_toml"
  cp "$backup_dir/Cargo.lock" "$cargo_lock"
  cp "$backup_dir/template.yaml" "$template_yaml"
}

on_exit() {
  status=$?
  if [[ $status -ne 0 && $restore_needed -eq 1 ]]; then
    restore_release_files
  fi
  cleanup
  exit "$status"
}

trap on_exit EXIT

cp "$cargo_toml" "$backup_dir/Cargo.toml"
cp "$cargo_lock" "$backup_dir/Cargo.lock"
cp "$template_yaml" "$backup_dir/template.yaml"

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


def replace_workspace_version(path: Path, version: str) -> None:
    lines = path.read_text().splitlines(keepends=True)
    in_workspace_package = False

    for index, line in enumerate(lines):
        stripped = line.strip()
        if stripped.startswith("[") and stripped.endswith("]"):
            in_workspace_package = stripped == "[workspace.package]"
            continue

        if in_workspace_package and re.match(r"^[ \t]*version[ \t]*=", line):
            updated_line, count = re.subn(
                r"^([ \t]*version[ \t]*=[ \t]*\")([^\"]+)(\".*)$",
                rf"\g<1>{version}\g<3>",
                line,
                count=1,
            )
            if count != 1:
                raise SystemExit(f"error: could not update workspace package version in {path}")
            lines[index] = updated_line
            path.write_text("".join(lines))
            return

    raise SystemExit(f"error: could not update workspace package version in {path}")


replace_workspace_version(cargo_toml, version)
replace_once(
    template_yaml,
    r"(^[ \t]*SemanticVersion:[ \t]*)([^ \t\r\n]+)([ \t]*$)",
    rf"\g<1>{version}\g<3>",
    "SAM application semantic version",
)
PY

refresh_lockfile() {
  (
    cd "$repo_root"
    if ! cargo check --workspace --offline --quiet >/dev/null 2>&1; then
      cargo check --workspace --quiet >/dev/null
    fi
  )
}

if ! refresh_lockfile; then
  echo "error: failed to refresh Cargo.lock after updating the release version" >&2
  exit 1
fi

restore_needed=0

echo "Updated release version to $version"
echo "Derived release tag: v$version"
