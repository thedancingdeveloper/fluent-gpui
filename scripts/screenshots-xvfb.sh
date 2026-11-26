#!/usr/bin/env bash
# Headless capture of the screenshots harness under Xvfb (merge-queue job
# `screenshots-xvfb`, also fine to run by hand on any Linux box with xvfb,
# imagemagick and a Vulkan driver, e.g. mesa-vulkan-drivers / lavapipe).
#
# Unlike scripts/screenshots.sh (GNOME + screenshot-rs, fixed monitor
# geometry) this needs no desktop: it starts one Xvfb server, runs the
# `screenshots` binary against it (gpui falls back to X11 when there is no
# Wayland), and grabs the 1000x600 window at the screen origin.
#
# Usage:
#   scripts/screenshots-xvfb.sh                       # all components, both themes -> screenshots-xvfb/
#   scripts/screenshots-xvfb.sh --out DIR
#   scripts/screenshots-xvfb.sh --component button    # one component
#   scripts/screenshots-xvfb.sh --dark | --light
#
# Exit status: 0 when every capture succeeded, 1 when any failed (a summary
# line says how many). Component names come from ALL_COMPONENTS in
# scripts/screenshots.sh so the two scripts cannot drift.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

SENTINEL="/tmp/.fluent-screenshot-ready"
BINARY="${SCREENSHOTS_BIN:-./target/debug/screenshots}"
OUT_DIR="./screenshots-xvfb"
FILTER=""
DO_DARK=true
DO_LIGHT=true
TIMEOUT_DECISECONDS=300   # software rendering: first frame can take a while
SETTLE_SECONDS="${SETTLE_SECONDS:-1.5}"
SCREEN="${XVFB_SCREEN:-1280x800x24}"

while [[ $# -gt 0 ]]; do
    case $1 in
        --out) OUT_DIR="$2"; shift 2 ;;
        --component) FILTER="$2"; shift 2 ;;
        --dark) DO_LIGHT=false; shift ;;
        --light) DO_DARK=false; shift ;;
        -h|--help) sed -n '/^#/!q; s/^# \?//p' "$0"; exit 0 ;;
        *) echo "Unknown flag: $1" >&2; exit 2 ;;
    esac
done

for tool in Xvfb import convert; do
    command -v "$tool" >/dev/null || { echo "missing tool: $tool (apt: xvfb imagemagick)" >&2; exit 2; }
done
[[ -x "$BINARY" ]] || { echo "missing binary $BINARY (cargo build -p screenshots)" >&2; exit 2; }

# ALL_COMPONENTS=( ... ) from the GNOME script, one name per line.
mapfile -t ALL < <(sed -n '/^ALL_COMPONENTS=(/,/^)/p' scripts/screenshots.sh | sed '1d;$d' | tr -d ' ')
if [[ -n "$FILTER" ]]; then COMPONENTS=("$FILTER"); else COMPONENTS=("${ALL[@]}"); fi
((${#COMPONENTS[@]} > 0)) || { echo "no components found" >&2; exit 2; }

DISPLAY_NUM=99
export DISPLAY=":$DISPLAY_NUM"
# Force the X11 backend and a software-capable Vulkan stack.
unset WAYLAND_DISPLAY
Xvfb "$DISPLAY" -screen 0 "$SCREEN" -nolisten tcp >/tmp/xvfb.log 2>&1 &
XVFB_PID=$!
trap 'kill "$XVFB_PID" 2>/dev/null || true' EXIT
for _ in $(seq 1 50); do
    [[ -S "/tmp/.X11-unix/X$DISPLAY_NUM" ]] && break
    sleep 0.1
done
[[ -S "/tmp/.X11-unix/X$DISPLAY_NUM" ]] || { echo "Xvfb did not start:" >&2; cat /tmp/xvfb.log >&2; exit 1; }

capture() {
    local component="$1" theme_flag="$2" out="$3"
    rm -f "$SENTINEL"
    "$BINARY" "$component" "$theme_flag" >"$out.log" 2>&1 &
    local pid=$!
    local t=$TIMEOUT_DECISECONDS
    while [[ ! -f "$SENTINEL" && $t -gt 0 ]] && kill -0 "$pid" 2>/dev/null; do
        sleep 0.1; t=$((t - 1))
    done
    if [[ ! -f "$SENTINEL" ]]; then
        echo "  FAIL (no first frame; see $out.log): $component $theme_flag" >&2
        kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true
        return 1
    fi
    sleep "$SETTLE_SECONDS"
    # The window is 1000x600 at the origin; there is no window manager.
    if ! import -window root -crop 1000x600+0+0 +repage "$out"; then
        echo "  FAIL (capture): $component $theme_flag" >&2
        kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true
        return 1
    fi
    kill "$pid" 2>/dev/null || true; wait "$pid" 2>/dev/null || true
    rm -f "$out.log"
}

total=0; ok=0
for component in "${COMPONENTS[@]}"; do
    for theme in dark light; do
        [[ $theme == dark && $DO_DARK == false ]] && continue
        [[ $theme == light && $DO_LIGHT == false ]] && continue
        mkdir -p "$OUT_DIR/$theme"
        printf "  [%-5s] %s\n" "$theme" "$component"
        total=$((total + 1))
        capture "$component" "--$theme" "$OUT_DIR/$theme/$component.png" && ok=$((ok + 1))
    done
done

echo "Done: $ok/$total screenshots saved to $OUT_DIR/"
((ok == total))
