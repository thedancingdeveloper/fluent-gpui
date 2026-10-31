#!/usr/bin/env bash
# WI-2725: every fluent-* crate records [package.metadata.estate-gpui] zed-rev,
# and it must equal the zed commit in vendor/zed/UPSTREAM.md.
set -euo pipefail
cd "$(dirname "$0")/.."
want=$(sed -n 's/^- zed commit: `\([0-9a-f]\{40\}\)`$/\1/p' vendor/zed/UPSTREAM.md)
if [ -z "$want" ]; then echo "error: no zed commit in vendor/zed/UPSTREAM.md" >&2; exit 1; fi
fail=0
for toml in crates/fluent-*/Cargo.toml; do
  got=$(awk '/^\[package\.metadata\.estate-gpui\]/{s=1;next} /^\[/{s=0} s && /^zed-rev *=/{gsub(/[" ]/,"",$0); sub(/^zed-rev=/,""); print}' "$toml")
  if [ "$got" != "$want" ]; then
    echo "error: $toml zed-rev='${got:-<missing>}', vendor/zed/UPSTREAM.md has $want" >&2
    fail=1
  fi
done
[ "$fail" -eq 0 ] && echo "zed-rev ok: $want"
exit "$fail"
