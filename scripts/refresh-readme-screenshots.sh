#!/usr/bin/env bash
# Refresh the screenshots shown in README.md from a "Screenshots (Xvfb)" run.
#
# Usage:
#   scripts/refresh-readme-screenshots.sh <run-id>
#   scripts/refresh-readme-screenshots.sh --dir <unpacked-artifact-dir>
#
# Downloads the `screenshots-xvfb` artifact (gh run download) and copies
#   dark/ and light/ -> screenshots/
#   gallery/         -> docs/images/gallery/
#   focus/<theme>/<component>.png (components listed in FOCUS only)
#                    -> docs/images/focus/<theme>/
# Needs `gh` (logged in) unless --dir is given. Run from anywhere.
set -euo pipefail

FOCUS="button checkbox combobox data-table date-time-picker dialog dropdown field icon-button radio-group ribbon searchbox selectable-list settings-nav switch tab-strip text-input textarea toggle-button token-input toolbar tree"

usage() { sed -n '2,12p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; }

src=""
tmp=""
case "${1:-}" in
  ""|-h|--help) usage; exit 0 ;;
  --dir) src="${2:?--dir needs a directory}" ;;
  *)
    command -v gh >/dev/null || { echo "gh not found" >&2; exit 1; }
    tmp="$(mktemp -d)"
    trap 'rm -rf "$tmp"' EXIT
    gh run download "$1" -n screenshots-xvfb -D "$tmp"
    src="$tmp"
    ;;
esac

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
[ -d "$src/dark" ] && [ -d "$src/light" ] || { echo "no dark/ and light/ in $src" >&2; exit 1; }

mkdir -p "$root/screenshots/dark" "$root/screenshots/light" "$root/docs/images/gallery"
cp "$src"/dark/*.png "$root/screenshots/dark/"
cp "$src"/light/*.png "$root/screenshots/light/"
[ -d "$src/gallery" ] && cp "$src"/gallery/*.png "$root/docs/images/gallery/"

for theme in dark light; do
  mkdir -p "$root/docs/images/focus/$theme"
  for c in $FOCUS; do
    f="$src/focus/$theme/$c.png"
    if [ -f "$f" ]; then cp "$f" "$root/docs/images/focus/$theme/"; else echo "missing focus capture: $theme/$c" >&2; fi
  done
done
echo "Refreshed. Optimise PNGs (oxipng/pngquant) if sizes grew, then review git status."
