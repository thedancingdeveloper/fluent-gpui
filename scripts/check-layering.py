#!/usr/bin/env python3
"""WI-2725 layering guard.

Only crates/fluent-* (and gpui itself: crates/gpui, vendor/zed) may depend on
gpui or any gpui_*/gpui-* crate, or name `gpui::` / `gpui_*::` paths in Rust
source. Covers every workspace member plus any .rs file under the repo
(examples/, tests/, ...). The allowed list matches deny.toml [bans].
"""
import json
import pathlib
import re
import subprocess
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
# BASELINE: examples that predate the rule. Shrink, never grow (see deny.toml).
BASELINE = {
    "examples/demo_app",
    "examples/form",
    "examples/gallery",
    "examples/hello_world",
    "examples/screenshots",
    "examples/widgets",
}
SKIP_DIRS = {"target", ".git", ".graft"}
PATH_RE = re.compile(r"(?<![A-Za-z0-9_])gpui(?:_[a-z_]+)?::")


def allowed(rel: pathlib.PurePosixPath) -> bool:
    parts = rel.parts
    if len(parts) >= 2 and parts[0] == "crates" and (parts[1].startswith("fluent-") or parts[1] == "gpui"):
        return True
    if parts[:2] == ("vendor", "zed"):
        return True
    return any(str(rel) == b or str(rel).startswith(b + "/") for b in BASELINE)


def is_gpui(name: str) -> bool:
    return name == "gpui" or name.startswith(("gpui_", "gpui-"))


def main() -> int:
    errors = []
    meta = json.loads(
        subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"],
            cwd=ROOT, check=True, capture_output=True, text=True,
        ).stdout
    )
    for pkg in meta["packages"]:
        rel = pathlib.PurePosixPath(pathlib.Path(pkg["manifest_path"]).parent.relative_to(ROOT).as_posix())
        for dep in pkg["dependencies"]:
            if is_gpui(dep["name"]) and not allowed(rel):
                errors.append(f"{rel}/Cargo.toml depends on {dep['name']}")
    for path in ROOT.rglob("*.rs"):
        rel = path.relative_to(ROOT)
        if SKIP_DIRS & set(rel.parts) or allowed(pathlib.PurePosixPath(rel.as_posix())):
            continue
        for n, line in enumerate(path.read_text(encoding="utf-8", errors="replace").splitlines(), 1):
            if PATH_RE.search(line):
                errors.append(f"{rel.as_posix()}:{n}: gpui path outside fluent-*: {line.strip()}")
    for e in errors:
        print(f"error: {e}", file=sys.stderr)
    if errors:
        print("layering guard failed: only crates/fluent-* may use gpui (CONTRIBUTING.md §2)", file=sys.stderr)
        return 1
    print("layering ok")
    return 0


if __name__ == "__main__":
    sys.exit(main())
