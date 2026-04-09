#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <yaml-scalar>" >&2
  exit 2
fi

value="$1"

value="${value#"${value%%[![:space:]]*}"}"
value="${value%"${value##*[![:space:]]}"}"

case "$value" in
  \'*)
    value="${value#\'}"
    value="${value%%\'*}"
    ;;
  \"*)
    value="${value#\"}"
    value="${value%%\"*}"
    ;;
  *)
    value="${value%%[[:space:]]#*}"
    value="${value%"${value##*[![:space:]]}"}"
    ;;
esac

printf '%s\n' "$value"
