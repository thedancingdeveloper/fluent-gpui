#!/usr/bin/env bash
# Capture screenshots of every FluentGUI primitive.
#
# Requirements: screenshot-rs (from /home/sprooty/Working/Active/apps/sceenshot-rs), cargo
#
# Usage:
#   ./scripts/screenshots.sh                  # all components, both themes -> screenshots/
#   ./scripts/screenshots.sh --out ./out      # custom output dir
#   ./scripts/screenshots.sh --component button          # single component, both themes
#   ./scripts/screenshots.sh --component button --dark   # single component, dark only
#   ./scripts/screenshots.sh --component button --light  # single component, light only
#
# The binary opens a fullscreen window, writes /tmp/.fluent-screenshot-ready
# after its first GPU frame, then this script calls screenshot-rs --root and kills it.

set -euo pipefail

SENTINEL="/tmp/.fluent-screenshot-ready"
BINARY="./target/release/screenshots"
SCREENSHOT_RS="${SCREENSHOT_RS:-/home/sprooty/Working/Active/apps/sceenshot-rs/target/release/screenshot-rs}"
OUT_DIR="./screenshots"
FILTER_COMPONENT=""
DO_DARK=true
DO_LIGHT=true
TIMEOUT_DECISECONDS=80  # 8 seconds

ALL_COMPONENTS=(
    button
    icon-button
    toggle-button
    checkbox
    switch
    label
    badge
    chip
    divider
    progress-bar
    spinner
    skeleton
    avatar
    app-dot
    connection-badge
    text-input
    textarea
    searchbox
    dropdown
    combobox
    radio-group
    field
    card
    section-header
)

# ── Argument parsing ──────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case $1 in
        --out)      OUT_DIR="$2"; shift 2 ;;
        --component) FILTER_COMPONENT="$2"; shift 2 ;;
        --dark)     DO_LIGHT=false; shift ;;
        --light)    DO_DARK=false; shift ;;
        -h|--help)
            sed -n '/^#/!q; s/^# \?//p' "$0"
            exit 0
            ;;
        *) echo "Unknown flag: $1" >&2; exit 1 ;;
    esac
done

if [[ -n "$FILTER_COMPONENT" ]]; then
    COMPONENTS=("$FILTER_COMPONENT")
else
    COMPONENTS=("${ALL_COMPONENTS[@]}")
fi

# ── Build ─────────────────────────────────────────────────────────────────
echo "Building screenshots binary…"
cargo build -p screenshots --release 2>&1 | grep -E "^(error|warning\[|Compiling|Finished)" || true
echo ""

# ── Display / capture setup ───────────────────────────────────────────────
# GPUI uses the native Wayland backend; X11 is only used by screenshot-rs for
# monitor geometry queries.
export DISPLAY="${DISPLAY:-:0}"

# ── Window geometry ───────────────────────────────────────────────────────
# GNOME consistently places the 1000×600 logical window centred on the primary
# monitor (DP-2: 3440×1440 logical, X11 6880×2880 at offset (0,2880)).
# Measured window position in X11 physical coords:
#   offset (2440, 3683)  size (2000, 1200)
# which maps to portal (1250×750 at offset 1525,502 in the 4300×1800 DP-2 region).
WIN_X=2440; WIN_Y=3683; WIN_W=2000; WIN_H=1200

# ── Capture helpers ───────────────────────────────────────────────────────

capture() {
    local component="$1"
    local theme_flag="$2"   # "--dark" or "--light"
    local theme_name="$3"   # "dark" or "light"
    local out_path="$4"

    rm -f "$SENTINEL"

    "$BINARY" "$component" "$theme_flag" &
    local pid=$!

    # Poll for sentinel (max TIMEOUT_DECISECONDS * 0.1s)
    local t=$TIMEOUT_DECISECONDS
    while [[ ! -f "$SENTINEL" && $t -gt 0 ]]; do
        sleep 0.1
        t=$((t - 1))
    done

    if [[ ! -f "$SENTINEL" ]]; then
        echo "  TIMEOUT: $component [$theme_name]" >&2
        kill "$pid" 2>/dev/null || true
        wait "$pid" 2>/dev/null || true
        return 1
    fi

    # Wait for Wayland compositor to fully composite the first GPU frame.
    # 0.5s was too short on this machine; 3s is reliable.
    sleep 3.0

    # Capture the exact window region directly.  GNOME places the 1000×600
    # window at a fixed position on the primary monitor; no detection needed.
    "$SCREENSHOT_RS" \
        --region "$WIN_W" "$WIN_H" "$WIN_X" "$WIN_Y" \
        --delay 0 \
        "$out_path" 2>/dev/null

    kill "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true

    # Small gap so the next window opens into a clean state
    sleep 0.3
}

# ── Main loop ─────────────────────────────────────────────────────────────

total=0
ok=0

for component in "${COMPONENTS[@]}"; do
    if $DO_DARK; then
        mkdir -p "$OUT_DIR/dark"
        out="$OUT_DIR/dark/$component.png"
        printf "  [dark]  %-20s → %s\n" "$component" "$out"
        capture "$component" "--dark" "dark" "$out" && ok=$((ok + 1)) || true
        total=$((total + 1))
    fi

    if $DO_LIGHT; then
        mkdir -p "$OUT_DIR/light"
        out="$OUT_DIR/light/$component.png"
        printf "  [light] %-20s → %s\n" "$component" "$out"
        capture "$component" "--light" "light" "$out" && ok=$((ok + 1)) || true
        total=$((total + 1))
    fi
done

echo ""
echo "Done: $ok/$total screenshots saved to $OUT_DIR/"
