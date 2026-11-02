#!/usr/bin/env bash
# Extract the curated Unreleased section used for a draft GitHub release.
set -euo pipefail
cd "$(dirname "$0")/.."
output=${1:-release-notes.md}

awk '
  /^## Unreleased([[:space:]]|$)/ { found=1; next }
  found && /^## / { exit }
  found { print }
' CHANGELOG.md > "$output"

if ! grep -q '[^[:space:]]' "$output"; then
  echo "error: CHANGELOG.md has no release notes under ## Unreleased" >&2
  exit 1
fi
printf 'release notes ready: %s\n' "$output"
