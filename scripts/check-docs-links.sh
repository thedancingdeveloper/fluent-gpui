#!/usr/bin/env bash
# Check that every relative markdown link and every image path in README.md,
# CONTRIBUTING.md and docs/**/*.md points at a file or directory that exists.
#
# Usage:
#   scripts/check-docs-links.sh          # run from anywhere; checks the repo root
#   scripts/check-docs-links.sh --help
#
# Checked: [text](path), ![alt](path), <img src="path">, <source srcset="path">.
# Skipped: http(s):, mailto:, and pure #fragment links. A path#fragment is
# checked for the path only.
# Allow-listed (may be missing; added by other changes): LICENSE, NOTICE,
# SECURITY.md, CODE_OF_CONDUCT.md, CHANGELOG.md, docs/maintainers/RELEASING.md.
# Exit status: 0 when all links resolve, 1 otherwise.
set -euo pipefail

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" ]]; then
  sed -n '2,12p' "$0" | sed 's/^# \{0,1\}//'
  exit 0
fi

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

exec python3 -I - "$root" <<'PY'
import os, re, sys, glob

root = sys.argv[1]
ALLOW = {"LICENSE", "NOTICE", "SECURITY.md", "CODE_OF_CONDUCT.md", "CHANGELOG.md",
         "docs/maintainers/RELEASING.md"}
files = ["README.md", "CONTRIBUTING.md"] + sorted(
    glob.glob("docs/**/*.md", recursive=True))
pat = re.compile(r'''!?\[[^\]]*\]\(\s*<?([^)\s>]+)>?(?:\s+"[^"]*")?\s*\)'''
                 r'''|<img\b[^>]*?\bsrc="([^"]+)"'''
                 r'''|<source\b[^>]*?\bsrcset="([^"]+)"''', re.I)
fence = re.compile(r'^\s*(```|~~~)')
bad, checked = [], 0
for f in files:
    if not os.path.exists(f):
        continue
    in_code = False
    for n, line in enumerate(open(f, encoding="utf-8"), 1):
        if fence.match(line):
            in_code = not in_code
            continue
        if in_code:
            continue
        line = re.sub(r"`[^`]*`", "", line)
        for m in pat.finditer(line):
            t = next(g for g in m.groups() if g)
            if re.match(r"^(https?:|mailto:|#)", t):
                continue
            path = t.split("#", 1)[0]
            if not path:
                continue
            target = os.path.normpath(os.path.join(os.path.dirname(f), path))
            checked += 1
            if target in ALLOW:
                continue
            if not os.path.exists(target):
                bad.append(f"{f}:{n}: missing {t} -> {target}")
for b in bad:
    print(b)
print(f"checked {checked} links in {len(files)} files: {'FAILED, ' + str(len(bad)) + ' broken' if bad else 'ok'}")
sys.exit(1 if bad else 0)
PY
