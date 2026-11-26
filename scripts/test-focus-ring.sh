#!/usr/bin/env bash
# Test for ring_label (scripts/focus-ring-lib.sh); no Xvfb needed.
set -uo pipefail
cd "$(dirname "$0")" || exit 2
# shellcheck source=focus-ring-lib.sh
source ./focus-ring-lib.sh
fail=0
check() {  # check N EXPECTED
    local got
    got="$(ring_label "$1")"
    if [[ "$got" != "$2" ]]; then echo "FAIL ring_label '$1': got '$got', want '$2'" >&2; fail=$((fail + 1)); fi
}
check 0 "NOT visible (0 px)"
check 29 "NOT visible (29 px)"
check 30 "visible (30 px)"
check 100 "visible (100 px)"   # checkbox, light
check 140 "visible (140 px)"   # checkbox, dark
check 168 "visible (168 px)"
check "" "n/a (compare failed)"
check abc "n/a (compare failed)"
((fail == 0)) && echo "ok"
((fail == 0))
