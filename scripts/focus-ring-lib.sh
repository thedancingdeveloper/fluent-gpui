#!/usr/bin/env bash
# Pure helpers for the focus-ring report in screenshots-xvfb.sh. Sourced by it
# and by scripts/test-focus-ring.sh; needs no Xvfb or ImageMagick.

# ring_label N: label for N differing pixels between the resting and the Tab
# capture. A ring on a small control (checkbox) changes only 100-140 px, so
# only an essentially unchanged frame (below RING_NONE_MAX, 30) is NOT visible.
ring_label() {
    local n="${1:-}" none_max="${RING_NONE_MAX:-30}"
    [[ "$n" =~ ^[0-9]+$ ]] || { echo "n/a (compare failed)"; return; }
    if ((n < none_max)); then echo "NOT visible ($n px)"; else echo "visible ($n px)"; fi
}
