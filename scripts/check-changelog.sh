#!/usr/bin/env bash
# Check the format of CHANGELOG.md.
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: scripts/check-changelog.sh [CHANGELOG_FILE]

Checks a Keep a Changelog file (default: CHANGELOG.md at the repository root):
  - it has a "## Unreleased" heading
  - no line mentions an internal work-item reference ("WI-")
  - no "###" subsection is empty
  - every "## " version heading except Unreleased has a date (YYYY-MM-DD)

Exit status: 0 when the file passes, 1 when it has problems, 2 on bad usage.
USAGE
}

case "${1:-}" in
  -h|--help) usage; exit 0 ;;
esac
if [ "$#" -gt 1 ]; then usage >&2; exit 2; fi

file=${1:-"$(dirname "$0")/../CHANGELOG.md"}
if [ ! -f "$file" ]; then
  echo "error: $file not found" >&2
  exit 2
fi

awk '
  function fail(msg) { printf "%s:%d: %s\n", FILENAME, NR, msg; bad = 1 }
  function close_sub() {
    if (insub && !content) fail("empty subsection: " sub_title)
    insub = 0
  }
  /<!--/ { incomment = 1 }
  {
    if (/WI-[0-9]/) fail("work-item reference (WI-) found")
  }
  incomment { if (/-->/) incomment = 0; next }
  /^## / {
    close_sub()
    if ($0 ~ /^## Unreleased([[:space:]]|$)/) { unreleased = 1 }
    else if ($0 !~ /^## [^[:space:]].* - [0-9][0-9][0-9][0-9]-[0-9][0-9]-[0-9][0-9]$/)
      fail("version heading without a date: " $0)
    next
  }
  /^### / { close_sub(); insub = 1; content = 0; sub_title = $0; next }
  /[^[:space:]]/ { content = 1 }
  END {
    close_sub()
    if (!unreleased) { printf "%s: no \"## Unreleased\" heading\n", FILENAME; bad = 1 }
    if (!bad) printf "%s: ok\n", FILENAME
    exit bad
  }
' "$file"
