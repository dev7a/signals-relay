#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 1 ]]; then
  echo "usage: $0 <yaml-scalar>" >&2
  exit 2
fi

value="$1"

value="${value%%[[:space:]]#*}"
value="${value#"${value%%[![:space:]]*}"}"
value="${value%"${value##*[![:space:]]}"}"

if [[ ${#value} -ge 2 ]]; then
  first_char="${value:0:1}"
  last_char="${value: -1}"
  if [[ "$first_char" == "$last_char" ]] && { [[ "$first_char" == "'" ]] || [[ "$first_char" == '"' ]]; }; then
    value="${value:1:${#value}-2}"
  fi
fi

printf '%s\n' "$value"
