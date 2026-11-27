#!/usr/bin/env bash
# Offline test for scripts/release-notes.sh over a sample changelog.
set -uo pipefail
cd "$(dirname "$0")/.." || exit 2
tmp=$(mktemp -d); trap 'rm -rf "$tmp"' EXIT
cat > "$tmp/CHANGELOG.md" <<'CL'
# Changelog

## Unreleased (next release: 4.0.0)

### Added

- unreleased-entry

## 3.2.0 - 2026-11-05

### Added

- three-two-entry

## 3.1.0 - 2026-11-01

- three-one-entry
CL
export CHANGELOG_FILE="$tmp/CHANGELOG.md"
fail=0
ok() { echo "FAIL: $1" >&2; fail=$((fail + 1)); }

for tag in 3.2.0 v3.2.0 v3.2.0+zed.ba63a022c9a2; do
  if ! scripts/release-notes.sh "$tmp/out.md" "$tag" >/dev/null 2>&1; then ok "$tag: script failed"; continue; fi
  grep -q three-two-entry "$tmp/out.md" || ok "$tag: missing 3.2.0 entry"
  grep -q -e unreleased-entry -e three-one-entry "$tmp/out.md" && ok "$tag: leaked other section"
  grep -q '^## ' "$tmp/out.md" && ok "$tag: heading included"
done

scripts/release-notes.sh "$tmp/out.md" >/dev/null 2>&1 || ok "no tag: script failed"
grep -q unreleased-entry "$tmp/out.md" || ok "no tag: missing Unreleased entry"
grep -q three-two-entry "$tmp/out.md" && ok "no tag: leaked 3.2.0"

scripts/release-notes.sh "$tmp/out.md" v9.9.9+zed.abcdef0 >/dev/null 2>&1 && ok "unknown version should fail"
scripts/release-notes.sh "$tmp/out.md" garbage >/dev/null 2>&1 && ok "bad tag should fail"

[[ $fail -eq 0 ]] && echo "release-notes tests passed"
exit $((fail > 0))
