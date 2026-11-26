#!/usr/bin/env bash
# Headless capture of the screenshots harness and the gallery under Xvfb
# (workflow `screenshots.yml`, also fine to run by hand on any Linux box with
# xvfb, xdotool, imagemagick and a Vulkan driver, e.g. mesa-vulkan-drivers /
# lavapipe).
#
# Unlike scripts/screenshots.sh (GNOME + screenshot-rs, fixed monitor
# geometry) this needs no desktop: it starts one Xvfb server and runs the
# `screenshots` and `gallery` binaries against it (gpui falls back to X11 when
# there is no Wayland). There is no window manager: the script gives the
# window keyboard focus itself with `xdotool windowfocus` and checks it took
# (`xdotool getwindowfocus`); when it did not, the focus captures are skipped
# and report.txt says so, instead of capturing an unfocused frame.
#
# Output (all under --out DIR):
#   dark/<component>.png, light/<component>.png          resting, 1000x600
#   focus/dark/<component>.png, focus/light/...          after a real Tab key
#                                                        (<component>-2.png ... for later stops)
#   gallery/<page>-<theme>.png, gallery/hero-<theme>.png whole gallery, 1280x800
#   report.txt                                           one `focus ring:` line per
#                                                        component and theme
# The report is also appended to $GITHUB_STEP_SUMMARY as a table when set.
#
# Usage:
#   scripts/screenshots-xvfb.sh                  # everything -> screenshots-xvfb/
#   scripts/screenshots-xvfb.sh --out DIR
#   scripts/screenshots-xvfb.sh --only button    # one component (--component is an alias);
#                                                # skips the gallery unless --gallery is given
#   scripts/screenshots-xvfb.sh --dark | --light
#   scripts/screenshots-xvfb.sh --no-focus       # resting captures only
#   scripts/screenshots-xvfb.sh --no-gallery | --gallery
#
# Environment: SCREENSHOTS_BIN, GALLERY_BIN (binaries), SETTLE_SECONDS,
# XVFB_SCREEN, RING_MIN_PIXELS (differing pixels that count as a ring, 150).
#
# Exit status: 0 when every capture succeeded, 1 when any failed (a summary
# line says how many). A missing focus ring is reported, never a failure.
# Component names come from ALL_COMPONENTS in scripts/screenshots.sh so the two
# scripts cannot drift; gallery pages come from `gallery --list-pages`.

set -uo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT" || exit 2

SENTINEL="/tmp/.fluent-screenshot-ready"
BINARY="${SCREENSHOTS_BIN:-./target/debug/screenshots}"
GALLERY="${GALLERY_BIN:-./target/debug/gallery}"
OUT_DIR="./screenshots-xvfb"
FILTER=""
DO_DARK=true
DO_LIGHT=true
DO_FOCUS=true
DO_GALLERY=""            # "" = default (on unless --only), true/false = forced
TIMEOUT_DECISECONDS=300  # software rendering: first frame can take a while
SETTLE_SECONDS="${SETTLE_SECONDS:-1.5}"
SCREEN="${XVFB_SCREEN:-1280x800x24}"
RING_MIN_PIXELS="${RING_MIN_PIXELS:-150}"
GALLERY_SIZE="1280x800"

while [[ $# -gt 0 ]]; do
    case $1 in
        --out) OUT_DIR="${2:?--out needs a directory}"; shift 2 ;;
        --only|--component) FILTER="${2:?$1 needs a component}"; shift 2 ;;
        --dark) DO_LIGHT=false; shift ;;
        --light) DO_DARK=false; shift ;;
        --no-focus) DO_FOCUS=false; shift ;;
        --no-gallery) DO_GALLERY=false; shift ;;
        --gallery) DO_GALLERY=true; shift ;;
        -h|--help) sed -n '/^#/!q; s/^# \?//p' "$0"; exit 0 ;;
        *) echo "Unknown flag: $1" >&2; exit 2 ;;
    esac
done
if [[ -z "$DO_GALLERY" ]]; then
    if [[ -n "$FILTER" ]]; then DO_GALLERY=false; else DO_GALLERY=true; fi
fi

need=(Xvfb import convert compare)
$DO_FOCUS && need+=(xdotool)
for tool in "${need[@]}"; do
    command -v "$tool" >/dev/null || { echo "missing tool: $tool (apt: xvfb xdotool imagemagick)" >&2; exit 2; }
done
[[ -x "$BINARY" ]] || { echo "missing binary $BINARY (cargo build -p screenshots)" >&2; exit 2; }
if $DO_GALLERY; then
    [[ -x "$GALLERY" ]] || { echo "missing binary $GALLERY (cargo build -p gallery)" >&2; exit 2; }
fi

# ALL_COMPONENTS=( ... ) from the GNOME script, one name per line.
mapfile -t ALL < <(sed -n '/^ALL_COMPONENTS=(/,/^)/p' scripts/screenshots.sh | sed '1d;$d' | tr -d ' ')
if [[ -n "$FILTER" ]]; then COMPONENTS=("$FILTER"); else COMPONENTS=("${ALL[@]}"); fi
((${#COMPONENTS[@]} > 0)) || { echo "no components found" >&2; exit 2; }

# Components with a keyboard Tab stop, and how many Tab presses to capture
# (one capture per stop). Everything else gets `focus ring: n/a`.
declare -A STOPS=(
    [button]=1 [icon-button]=1 [toggle-button]=1 [checkbox]=1 [switch]=1
    [radio-group]=1 [dropdown]=1 [chip]=1 [card]=1 [tab-strip]=1 [tree]=1
    [selectable-list]=1 [settings-nav]=1 [toolbar]=1 [data-table]=1
    [ribbon]=1 [title-bar]=1 [dialog]=1
    # Text-like controls (WI-4840): text-input shows three enabled inputs and
    # field wraps two; `states` has only disabled controls, so no tab stop.
    [text-input]=3 [textarea]=1 [searchbox]=1 [combobox]=1 [token-input]=1
    [date-time-picker]=1 [field]=2
)

DISPLAY_NUM=99
export DISPLAY=":$DISPLAY_NUM"
# Force the X11 backend and a software-capable Vulkan stack.
unset WAYLAND_DISPLAY

WORK="$(mktemp -d)"
XVFB_PID=""
APP_PID=""
cleanup() {
    stop_app
    [[ -n "$XVFB_PID" ]] && kill "$XVFB_PID" 2>/dev/null
    rm -rf "$WORK"
    rm -f "$SENTINEL"
}
stop_app() {
    if [[ -n "$APP_PID" ]]; then
        kill "$APP_PID" 2>/dev/null
        wait "$APP_PID" 2>/dev/null
        APP_PID=""
    fi
}
trap cleanup EXIT
trap 'exit 130' INT TERM

Xvfb "$DISPLAY" -screen 0 "$SCREEN" -nolisten tcp >"$WORK/xvfb.log" 2>&1 &
XVFB_PID=$!
for _ in $(seq 1 50); do
    [[ -S "/tmp/.X11-unix/X$DISPLAY_NUM" ]] && break
    sleep 0.1
done
[[ -S "/tmp/.X11-unix/X$DISPLAY_NUM" ]] || { echo "Xvfb did not start:" >&2; cat "$WORK/xvfb.log" >&2; exit 1; }

mkdir -p "$OUT_DIR"
REPORT="$OUT_DIR/report.txt"
: >"$REPORT"
# Per component: "<component> <dark cell> <light cell>" rows for the summary table.
declare -A RING_DARK=() RING_LIGHT=()

# start_app LOG ARGS...: run a binary in the background and wait for the
# first-frame sentinel. Sets APP_PID; returns 1 when no frame appeared.
start_app() {
    local log="$1"; shift
    rm -f "$SENTINEL"
    "$@" >"$log" 2>&1 &
    APP_PID=$!
    local t=$TIMEOUT_DECISECONDS
    while [[ ! -f "$SENTINEL" && $t -gt 0 ]] && kill -0 "$APP_PID" 2>/dev/null; do
        sleep 0.1; t=$((t - 1))
    done
    [[ -f "$SENTINEL" ]]
}

# grab OUT GEOMETRY: capture the root window region (no WM: the app window
# sits at the origin).
grab() {
    timeout 30 import -window root -crop "$2+0+0" +repage "$1"
}

# focus_window NAME: give the app window keyboard focus. Prints nothing; returns
# 0 only when `getwindowfocus` then reports that window.
focus_window() {
    local name="$1" wid=""
    wid="$(timeout 10 xdotool search --pid "$APP_PID" 2>/dev/null | tail -n 1)"
    [[ -n "$wid" ]] || wid="$(timeout 10 xdotool search --name "$name" 2>/dev/null | tail -n 1)"
    [[ -n "$wid" ]] || return 1
    # Park the pointer outside the 1000x600 window so it cannot hover anything.
    timeout 10 xdotool mousemove 1270 790 2>/dev/null
    timeout 10 xdotool windowfocus --sync "$wid" 2>/dev/null
    sleep 0.3
    [[ "$(timeout 10 xdotool getwindowfocus 2>/dev/null)" == "$wid" ]]
}

# ring_verdict REST FOCUS: prints "visible (N px)" or "NOT visible (N px)".
ring_verdict() {
    local n
    n="$(compare -metric AE -fuzz 5% "$1" "$2" null: 2>&1 >/dev/null | awk '{print int($1)}')"
    [[ "$n" =~ ^[0-9]+$ ]] || { echo "n/a (compare failed)"; return; }
    if ((n >= RING_MIN_PIXELS)); then echo "visible ($n px)"; else echo "NOT visible ($n px)"; fi
}

record() {  # record COMPONENT THEME VERDICT
    local component="$1" theme="$2" verdict="$3"
    echo "focus ring: $component [$theme]: $verdict" | tee -a "$REPORT"
    if [[ $theme == dark ]]; then RING_DARK[$component]="$verdict"; else RING_LIGHT[$component]="$verdict"; fi
}

failed=0
total=0

capture_component() {
    local component="$1" theme="$2"
    local rest="$OUT_DIR/$theme/$component.png"
    local focusdir="$OUT_DIR/focus/$theme"
    local log="$WORK/$component-$theme.log"
    total=$((total + 1))
    printf "  [%-5s] %s\n" "$theme" "$component"
    if ! start_app "$log" "$BINARY" "$component" "--$theme"; then
        echo "  FAIL (no first frame; see log below): $component $theme" >&2
        tail -n 20 "$log" >&2
        stop_app; failed=$((failed + 1)); return
    fi
    local stops="${STOPS[$component]:-0}" focused=false
    if $DO_FOCUS && ((stops > 0)); then
        focus_window "FluentGUI-Screenshots" && focused=true
    fi
    sleep "$SETTLE_SECONDS"
    if ! grab "$rest" 1000x600; then
        echo "  FAIL (capture): $component $theme" >&2
        stop_app; failed=$((failed + 1)); return
    fi
    if $DO_FOCUS; then
        if ((stops == 0)); then
            record "$component" "$theme" "n/a (no tab stop)"
        elif ! $focused; then
            record "$component" "$theme" "n/a (window focus not established; no focus capture)"
        else
            mkdir -p "$focusdir"
            local i out first="" verdict
            for ((i = 1; i <= stops; i++)); do
                timeout 10 xdotool key --clearmodifiers Tab 2>/dev/null
                sleep "$SETTLE_SECONDS"
                if ((i == 1)); then out="$focusdir/$component.png"; else out="$focusdir/$component-$i.png"; fi
                if grab "$out" 1000x600; then
                    ((i == 1)) && first="$out"
                else
                    echo "  FAIL (focus capture): $component $theme stop $i" >&2
                    failed=$((failed + 1))
                fi
            done
            if [[ -n "$first" ]]; then
                verdict="$(ring_verdict "$rest" "$first")"
                record "$component" "$theme" "$verdict"
            else
                record "$component" "$theme" "n/a (focus capture failed)"
            fi
        fi
    fi
    stop_app
}

for component in "${COMPONENTS[@]}"; do
    for theme in dark light; do
        [[ $theme == dark && $DO_DARK == false ]] && continue
        [[ $theme == light && $DO_LIGHT == false ]] && continue
        mkdir -p "$OUT_DIR/$theme"
        capture_component "$component" "$theme"
    done
done

capture_gallery() {  # capture_gallery NAME THEME [PAGE]
    local name="$1" theme="$2" page="${3:-}"
    local out="$OUT_DIR/gallery/$name-$theme.png"
    local args=(--theme "$theme" --size "$GALLERY_SIZE" --ready-file "$SENTINEL")
    [[ -n "$page" ]] && args+=(--page "$page")
    total=$((total + 1))
    printf "  [%-5s] gallery %s\n" "$theme" "$name"
    if ! start_app "$WORK/gallery-$name-$theme.log" "$GALLERY" "${args[@]}"; then
        echo "  FAIL (no first frame): gallery $name $theme" >&2
        tail -n 20 "$WORK/gallery-$name-$theme.log" >&2
        stop_app; failed=$((failed + 1)); return
    fi
    sleep "$SETTLE_SECONDS"
    if ! grab "$out" "$GALLERY_SIZE"; then
        echo "  FAIL (capture): gallery $name $theme" >&2
        failed=$((failed + 1))
    fi
    stop_app
}

if $DO_GALLERY; then
    mkdir -p "$OUT_DIR/gallery"
    mapfile -t PAGES < <("$GALLERY" --list-pages 2>/dev/null)
    if ((${#PAGES[@]} == 0)); then
        echo "  FAIL: gallery --list-pages printed nothing" >&2
        failed=$((failed + 1))
    fi
    for theme in dark light; do
        [[ $theme == dark && $DO_DARK == false ]] && continue
        [[ $theme == light && $DO_LIGHT == false ]] && continue
        capture_gallery hero "$theme"
        for page in "${PAGES[@]}"; do
            capture_gallery "$page" "$theme" "$page"
        done
    done
fi

# Markdown table for the job summary.
if $DO_FOCUS && [[ -n "${GITHUB_STEP_SUMMARY:-}" ]]; then
    {
        echo "### Keyboard focus ring (real Tab key via xdotool)"
        echo
        echo "| component | dark | light |"
        echo "|---|---|---|"
        for component in "${COMPONENTS[@]}"; do
            echo "| $component | ${RING_DARK[$component]:--} | ${RING_LIGHT[$component]:--} |"
        done
        echo
        echo "A ring counts as visible when the focus capture differs from the resting capture by at least $RING_MIN_PIXELS pixels."
    } >>"$GITHUB_STEP_SUMMARY"
fi

echo "Done: $((total - failed))/$total captures ok, output in $OUT_DIR/ (report: $REPORT)"
((failed == 0))
