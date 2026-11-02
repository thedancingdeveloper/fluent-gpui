#!/usr/bin/env bash
# Check fluent crate metadata against the vendored Zed revision. When given a
# release tag, also check that its version and +zed suffix describe this tree.
set -euo pipefail
cd "$(dirname "$0")/.."

upstream_file=vendor/zed/UPSTREAM.md
want=$(sed -n 's/^- zed commit: `\([0-9a-f]\{40\}\)`$/\1/p' "$upstream_file")
if [ -z "$want" ]; then
  echo "error: no zed commit in $upstream_file" >&2
  exit 1
fi

fail=0
for toml in crates/fluent-*/Cargo.toml; do
  got=$(awk '/^\[package\.metadata\.estate-gpui\]/{s=1;next} /^\[/{s=0} s && /^zed-rev *=/{gsub(/[" ]/,"",$0); sub(/^zed-rev=/,""); print}' "$toml")
  if [ "$got" != "$want" ]; then
    echo "error: $toml zed-rev='${got:-<missing>}', $upstream_file has $want" >&2
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  exit "$fail"
fi

tag=${1:-}
if [ -z "$tag" ]; then
  echo "zed-rev ok: $want"
  exit 0
fi

if [[ ! "$tag" =~ ^v([0-9]+\.[0-9]+\.[0-9]+)\+zed\.([0-9a-f]{7,40})$ ]]; then
  echo "error: release tag '$tag' must match v<major>.<minor>.<patch>+zed.<7-40 hex sha>" >&2
  exit 1
fi
version=${BASH_REMATCH[1]}
tag_rev=${BASH_REMATCH[2]}
if [[ "$want" != "$tag_rev"* ]]; then
  echo "error: tag zed revision '$tag_rev' does not match $upstream_file revision '$want'" >&2
  exit 1
fi

for toml in crates/fluent-*/Cargo.toml; do
  # Private support crates are part of the build but not the workspace release.
  if grep -Eq '^publish[[:space:]]*=[[:space:]]*false([[:space:]]*#.*)?$' "$toml"; then
    continue
  fi
  crate_version=$(sed -n 's/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p' "$toml" | head -n 1)
  if [ "$crate_version" != "$version" ]; then
    echo "error: $toml version '${crate_version:-<missing>}' does not match tag version '$version'" >&2
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  exit "$fail"
fi
echo "release tag ok: $tag (zed $want)"
