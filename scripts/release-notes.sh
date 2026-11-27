#!/usr/bin/env bash
# Extract the curated notes for a draft GitHub release.
#
# Usage: scripts/release-notes.sh [OUTPUT] [TAG]
#   TAG    release tag (vX.Y.Z+zed.<sha>) or bare version (X.Y.Z). The leading
#          "v" and the "+zed.<sha>" suffix are stripped and the text under the
#          matching "## X.Y.Z - YYYY-MM-DD" heading is written.
#          With no TAG (or an empty one, as on a dry run) the text under
#          "## Unreleased" is written instead.
#   OUTPUT defaults to release-notes.md.
# CHANGELOG_FILE overrides the changelog path (used by the test).
set -euo pipefail
cd "$(dirname "$0")/.."
output=${1:-release-notes.md}
tag=${2:-}
changelog=${CHANGELOG_FILE:-CHANGELOG.md}

if [[ -n "$tag" ]]; then
  version=${tag#v}
  version=${version%%+*}
  if [[ ! "$version" =~ ^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$ ]]; then
    echo "error: cannot read a version from tag '$tag'" >&2
    exit 1
  fi
  heading="## $version - "
  awk -v h="$heading" '
    index($0, h) == 1 { found=1; next }
    found && /^## / { exit }
    found { print }
  ' "$changelog" > "$output"
  where="$heading"
else
  awk '
    /^## Unreleased([[:space:]]|$)/ { found=1; next }
    found && /^## / { exit }
    found { print }
  ' "$changelog" > "$output"
  where="## Unreleased"
fi

if ! grep -q '[^[:space:]]' "$output"; then
  echo "error: $changelog has no release notes under $where" >&2
  [[ -n "$tag" ]] && echo "hint: rename ## Unreleased to '## <version> - <date>' before tagging" >&2
  exit 1
fi
printf 'release notes ready: %s\n' "$output"
