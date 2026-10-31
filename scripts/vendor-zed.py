#!/usr/bin/env python3
"""Vendor the path-dependency closure of zed's ``gpui_platform`` crate.

Copies the closure at a given zed commit into ``vendor/zed/crates/<name>`` and
rewrites every ``workspace = true`` inheritance so the crates build outside
zed's workspace. Idempotent: a second run at the same commit changes nothing.

The snapshot this produces is not committed by this script's own task; the
first snapshot lands separately (WI-1347). Run against a scratch directory to
check a revision:

    scripts/vendor-zed.py <rev> --zed /path/to/zed --out /tmp/scratch
    scripts/vendor-zed.py <rev> --dry-run

See ``vendor/zed/README.md`` for the ``zed-vendor`` pristine-line workflow.
"""

import argparse
import os
import re
import shutil
import subprocess
import sys
import tomllib
from pathlib import Path

DEP_SECTIONS = ("dependencies", "dev-dependencies", "build-dependencies")

# fluent-gpui supports Windows and Linux only. These platform crates are left
# out of the closure, and every dependency on them and feature forwarding to
# them is removed from the vendored manifests. The code that uses them is
# behind `cfg(target_os = "macos")` / wasm and never compiles here.
EXCLUDED = ("gpui_macos", "gpui_apple", "gpui_web")
_EXCLUDED_ALT = "|".join(EXCLUDED)
_EXCLUDED_ITEM = rf'"(?:dep:)?(?:{_EXCLUDED_ALT})(?:\??/[^"]*)?"'

def git(repo: Path, *args: str) -> str:
    """Run git in the zed checkout and return stdout. Raises on failure."""
    out = subprocess.run(
        ["git", "-C", str(repo), *args],
        check=True,
        capture_output=True,
        text=True,
    )
    return out.stdout


def resolve_rev(repo: Path, rev: str) -> tuple[str, str]:
    """Return (full sha, ISO date) for a revision, or exit if it is unknown."""
    try:
        sha = git(repo, "rev-parse", "--verify", f"{rev}^{{commit}}").strip()
        date = git(repo, "log", "-1", "--format=%cI", sha).strip()
    except subprocess.CalledProcessError as err:
        sys.exit(f"cannot resolve {rev!r} in {repo}: {err.stderr.strip()}")
    return sha, date[:10]


def read_blob(repo: Path, sha: str, path: str) -> bytes:
    return subprocess.run(
        ["git", "-C", str(repo), "cat-file", "blob", f"{sha}:{path}"],
        check=True,
        capture_output=True,
    ).stdout


def list_tree(repo: Path, sha: str, path: str) -> list[tuple[str, str]]:
    """Return (mode, relative path) for every blob under path at sha."""
    out = git(repo, "ls-tree", "-r", "-z", sha, "--", path)
    entries = []
    for rec in out.split("\0"):
        if not rec:
            continue
        meta, rel = rec.split("\t", 1)
        mode = meta.split(" ")[0]
        entries.append((mode, rel))
    return entries


def load_workspace(repo: Path, sha: str) -> dict:
    root = tomllib.loads(read_blob(repo, sha, "Cargo.toml").decode())
    workspace = root.get("workspace", {})
    deps = workspace.get("dependencies", {})
    path_of = {
        name: spec["path"]
        for name, spec in deps.items()
        if isinstance(spec, dict) and "path" in spec
    }
    return {
        "deps": deps,
        "path_of": path_of,
        "package": workspace.get("package", {}),
        "lints": workspace.get("lints", {}),
    }


def load_manifest(repo: Path, sha: str, crate_path: str) -> dict:
    return tomllib.loads(
        read_blob(repo, sha, crate_path.rstrip("/") + "/Cargo.toml").decode()
    )


def dep_tables(manifest: dict):
    """Yield every dependency table, including per-target ones.

    ``[target.'cfg(...)'.dependencies]`` parses as a nested ``target`` table,
    not as a top-level key, so both shapes are walked.
    """
    for key in DEP_SECTIONS:
        table = manifest.get(key)
        if isinstance(table, dict):
            yield table
    for key, value in manifest.items():
        if isinstance(key, str) and key.startswith("target") and isinstance(value, dict):
            for section in DEP_SECTIONS:
                table = value.get(section)
                if isinstance(table, dict):
                    yield table
    target = manifest.get("target")
    if isinstance(target, dict):
        for sub in target.values():
            if isinstance(sub, dict):
                for section in DEP_SECTIONS:
                    table = sub.get(section)
                    if isinstance(table, dict):
                        yield table


def default_features(manifest: dict) -> set[str]:
    """Feature names enabled by default, following feature-to-feature edges."""
    features = manifest.get("features", {})
    enabled = set()
    for item in features.get("default", []):
        if isinstance(item, str):
            enabled.add(item.split("/", 1)[0].split("?", 1)[0])
    grew = True
    while grew:
        grew = False
        for name in list(enabled):
            for item in features.get(name, []):
                if (
                    isinstance(item, str)
                    and "/" not in item
                    and "?" not in item
                    and not item.startswith("dep:")
                    and item not in enabled
                ):
                    enabled.add(item)
                    grew = True
    return enabled


def activated_by(manifest: dict, enabled: set[str]) -> set[str]:
    """Dependency names switched on by the enabled features.

    Covers ``dep:foo``, a bare ``foo`` (an implicit optional dep), and
    ``foo/feature`` references.
    """
    names = set()
    for feature in enabled:
        for item in manifest.get("features", {}).get(feature, []):
            if not isinstance(item, str) or item.endswith("?"):
                continue
            token = item.removeprefix("dep:").split("/", 1)[0].split("?", 1)[0]
            names.add(token)
    return names


def closure(repo: Path, sha: str, workspace: dict) -> list[str]:
    """Crates reachable from ``gpui_platform`` through normal dependencies.

    Optional dependencies count only when a default feature enables them, and
    dev-dependencies are excluded: they pull in crates nothing links against.
    ``util`` and ``path`` are pulled in too. They sit behind http_client's
    non-default ``github-download`` feature, but they are path crates with no
    version of their own, so a manifest that names them cannot resolve them
    from crates.io.
    """
    path_of = workspace["path_of"]
    seen: set[str] = set()
    stack = ["gpui_platform", "util", "path"]
    while stack:
        name = stack.pop()
        if name in seen or name not in path_of or name in EXCLUDED:
            continue
        seen.add(name)
        manifest = load_manifest(repo, sha, path_of[name])
        declared = []
        for table in dep_tables(manifest):
            for dep, spec in table.items():
                if isinstance(spec, dict) and spec.get("workspace") is True:
                    real = spec.get("package", dep)
                    optional = bool(spec.get("optional"))
                    declared.append((dep, real, optional))
                    if real in path_of and not optional and real not in seen:
                        stack.append(real)
        switched_on = activated_by(manifest, default_features(manifest))
        for dep, real, optional in declared:
            if optional and dep in switched_on and real in path_of and real not in seen:
                stack.append(real)
    if "gpui_platform" not in seen:
        sys.exit(f"{sha}: gpui_platform is not a path dependency of the workspace")
    return sorted(seen)


def render_value(value) -> str:
    """Render a workspace-dependency value as a TOML expression."""
    if isinstance(value, str):
        return toml_string(value)
    if isinstance(value, bool):
        return "true" if value else "false"
    if isinstance(value, list):
        return "[" + ", ".join(render_value(v) for v in value) + "]"
    if isinstance(value, dict):
        parts = [f"{toml_key(k)} = {render_value(v)}" for k, v in value.items()]
        return "{ " + ", ".join(parts) + " }"
    sys.exit(f"cannot render workspace value of type {type(value).__name__}")


def toml_string(text: str) -> str:
    escaped = text.replace("\\", "\\\\").replace('"', '\\"')
    return f'"{escaped}"'


def toml_key(key: str) -> str:
    bare = key.replace("_", "a").replace("-", "a")
    return key if bare.isalnum() and not key[0].isdigit() else toml_string(key)


def concretize(spec: dict, dep: str, workspace: dict, vendored: set[str]) -> str:
    """The concrete dependency text that replaces one ``workspace = true`` use.

    A path dependency inside the closure becomes a relative path to the
    vendored copy. Anything else keeps the version or git source zed pinned,
    and drops the path. Keys the crate sets itself (``features``,
    ``optional``, ``default-features``) win over the workspace entry.
    """
    deps = workspace["deps"]
    real = spec.get("package", dep)
    base = deps.get(real)
    if base is None:
        sys.exit(f"workspace.dependencies has no entry for {real!r}")
    if not isinstance(base, dict):
        base = {"version": base}

    own = {k: v for k, v in spec.items() if k not in ("workspace", "package")}
    if real in EXCLUDED:
        # Placeholder; drop_excluded() removes the whole line afterwards.
        return render_value({"path": f"../{real}"})
    if "path" in base and real in vendored:
        merged = {"path": f"../{real}"}
    else:
        merged = {k: v for k, v in base.items() if k != "path"}
        if "version" not in merged and "git" not in merged:
            sys.exit(
                f"{real!r} is a path crate outside the closure with no version; "
                "add it to closure()"
            )
    for key, value in own.items():
        if key == "features" and "features" in merged:
            merged["features"] = list(dict.fromkeys([*merged["features"], *value]))
        else:
            merged[key] = value
    return render_value(merged)


def rewrite_manifest(text: str, workspace: dict, vendored: set[str]) -> str:
    """Replace workspace inheritance in one manifest, preserving formatting.

    Only the lines that carry ``workspace = true`` change; comments and
    spacing around them stay, so a re-run is byte-identical.
    """
    package = workspace["package"]
    lines = text.split("\n")
    out = []
    i = 0
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()

        if stripped == "[lints]" and i + 1 < len(lines) \
                and lines[i + 1].strip() == "workspace = true":
            # The whole table is one inherited member; drop it and one blank line.
            i += 2
            if i < len(lines) and lines[i].strip() == "":
                i += 1
            continue

        subtable = dotted_subtable(stripped)
        if subtable is not None:
            # `[....dependencies.scap]` followed by its members becomes one line.
            members = []
            i += 1
            while i < len(lines) and lines[i].strip() and not lines[i].startswith("["):
                members.append(lines[i].strip().rstrip(","))
                i += 1
            spec = tomllib.loads(f"{subtable} = {{ {', '.join(members)} }}")
            out.append(
                indent_of(line) + subtable + " = "
                + concretize(spec[subtable], subtable, workspace, vendored)
            )
            continue

        replaced = inherit_line(line, workspace, vendored, package)
        if replaced is not None:
            out.append(replaced)
            i += 1
            continue

        # A `workspace = true` held inside a multi-line inline table.
        if "workspace = true" in stripped and "{" in stripped and "}" not in stripped:
            # The table can hold nested arrays, so collect until its own
            # opening brace is closed rather than stopping at the first `}`.
            block = [line]
            i += 1
            while i < len(lines) and block_depth("\n".join(block)) > 0:
                block.append(lines[i])
                i += 1
            out.extend(rewrite_block(block, workspace, vendored))
            continue

        out.append(line)
        i += 1
    return "\n".join(drop_excluded(out))


def drop_excluded(lines: list[str]) -> list[str]:
    """Remove dependencies on, and feature forwards to, the EXCLUDED crates."""
    kept = []
    for line in lines:
        stripped = line.strip()
        if re.match(rf"(?:{_EXCLUDED_ALT})\s*[.=]", stripped):
            continue
        if not stripped.startswith("#"):
            line = re.sub(_EXCLUDED_ITEM + r"\s*,\s*", "", line)
            line = re.sub(r",\s*" + _EXCLUDED_ITEM, "", line)
            line = re.sub(_EXCLUDED_ITEM, "", line)
        kept.append(line)
    return kept


def inherit_line(line: str, workspace: dict, vendored: set[str], package: dict):
    """Rewrite one single-line inheritance, or return None to keep it."""
    stripped = line.strip()
    indent = line[: len(line) - len(line.lstrip())]

    if stripped == "workspace = true":
        # A bare member, only valid inside [lints], which is dropped wholesale.
        return None

    field = dotted_field(stripped)
    if field in ("edition", "publish", "license", "rust-version"):
        if field not in package:
            sys.exit(f"workspace.package has no {field!r} to fill in")
        return f"{indent}{field} = {render_value(package[field])}"

    dep = dependency_name(stripped)
    if dep is None:
        # A table opened but not closed on this line belongs to the caller.
        if stripped.endswith("["):
            return None
        if "workspace = true" in stripped:
            sys.exit(f"unrecognised workspace inheritance: {stripped}")
        return None
    spec = dependency_spec(stripped, dep)
    return indent + dep + " = " + concretize(spec, dep, workspace, vendored)


def dotted_field(stripped: str):
    """``edition.workspace = true`` -> ``edition``; else None."""
    suffix = ".workspace = true"
    if stripped.endswith(suffix) and "." not in stripped[: -len(suffix)]:
        return stripped[: -len(suffix)]
    return None


def dependency_name(stripped: str):
    """The dependency key of a single-line ``workspace = true`` use, or None.

    A table that opens on this line but closes on a later one is left for the
    multi-line handler, so it only matches when the ``}`` is on this line.
    """
    if stripped.endswith(".workspace = true"):
        return stripped[: -len(".workspace = true")]
    marker = " = {"
    if (
        marker in stripped
        and "workspace = true" in stripped
        and stripped.endswith("}")
    ):
        return stripped.split(marker, 1)[0].strip()
    return None


def dependency_spec(stripped: str, dep: str) -> dict:
    """Parse the dependency as a table, however it was spelled."""
    if stripped.endswith(".workspace = true"):
        body = f"{dep}.workspace = true"
    else:
        body = stripped
    parsed = tomllib.loads(body)
    spec = parsed[dep]
    if not isinstance(spec, dict):
        sys.exit(f"expected a table for dependency {dep!r}: {stripped}")
    return spec


def dotted_subtable(stripped: str):
    """The dependency named by a `[<target>.dependencies.foo]` header, or None."""
    for marker in (".dependencies.", ".dev-dependencies.", ".build-dependencies."):
        if marker in stripped and stripped.startswith("[") and stripped.endswith("]"):
            return stripped[stripped.rfind(".") + 1:-1]
    return None


def indent_of(line: str) -> str:
    return line[: len(line) - len(line.lstrip())]


def block_depth(text: str) -> int:
    """How many more ``{`` than ``}`` a fragment holds, ignoring braces in strings."""
    depth = 0
    in_string = False
    escaped = False
    for char in text:
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
    return depth
    """How many more ``{`` than ``}`` a fragment holds, ignoring braces in strings."""
    depth = 0
    in_string = False
    escaped = False
    for char in text:
        if in_string:
            if escaped:
                escaped = False
            elif char == "\\":
                escaped = True
            elif char == '"':
                in_string = False
            continue
        if char == '"':
            in_string = True
        elif char == "{":
            depth += 1
        elif char == "}":
            depth -= 1
    return depth


def rewrite_block(block: list[str], workspace: dict, vendored: set[str]) -> list[str]:
    """Rewrite a multi-line inline dependency table to a single concrete line."""
    first = block[0]
    indent = first[: len(first) - len(first.lstrip())]
    dep = first.split("=", 1)[0].strip()
    spec = tomllib.loads("\n".join(block))[dep]
    if not isinstance(spec, dict) or spec.get("workspace") is not True:
        sys.exit(f"cannot rewrite multi-line dependency {dep!r}")
    return [indent + dep + " = " + concretize(spec, dep, workspace, vendored)]


def crate_dest(name: str) -> str:
    """Where a crate lands relative to ``vendor/zed``: always ``crates/<name>``."""
    return f"crates/{name}"


def write_link(repo: Path, sha: str, rel: str, source: str, dest: Path) -> None:
    """Copy a symlink, materialising it when it points outside its crate.

    The licence files are links to the zed repo root, which is not part of the
    snapshot, so they are written as the file they name. A link that stays
    inside the crate is kept as a link.
    """
    link = read_blob(repo, sha, rel).decode()
    resolved = os.path.normpath((Path(rel).parent / link).as_posix())
    if not resolved.startswith(source + "/"):
        dest.write_bytes(read_blob(repo, sha, resolved))
    else:
        dest.symlink_to(link)


def write_tree(repo: Path, sha: str, date: str, names: list[str], workspace: dict, out: Path) -> None:
    vendored = set(names)
    crates_root = out / "crates"
    crates_root.mkdir(parents=True, exist_ok=True)

    # A crate that lives inside another (derive_refineable under refineable)
    # is copied first, and the parent's copy skips it, so neither clobbers the
    # other.
    order = sorted(names, key=lambda n: workspace["path_of"][n].count("/"), reverse=True)
    nested = tuple(workspace["path_of"][n].rstrip("/") + "/" for n in order)
    for name in order:
        source = workspace["path_of"][name].rstrip("/")
        dest = out / crate_dest(name)
        if dest.exists():
            shutil.rmtree(dest)
        dest.mkdir(parents=True)
        skip = tuple(p for p in nested if p.startswith(source + "/") and p != source + "/")
        for mode, rel in list_tree(repo, sha, source):
            if rel.startswith(skip):
                continue
            leaf = rel[len(source):].lstrip("/")
            target = dest / leaf
            target.parent.mkdir(parents=True, exist_ok=True)
            if mode == "120000":
                write_link(repo, sha, rel, source, dest / leaf)
            elif mode == "160000":
                sys.exit(f"{rel} is a submodule; the closure must be self-contained")
            else:
                data = read_blob(repo, sha, rel)
                if leaf == "Cargo.toml":
                    data = rewrite_manifest(data.decode(), workspace, vendored).encode()
                target.write_bytes(data)
                if mode == "100755":
                    target.chmod(0o755)

    write_docs(out, sha, date, names, workspace)
    remove_stale(crates_root, names)
    assert_closed(out)


def write_docs(out: Path, sha: str, date: str, names: list[str], workspace: dict) -> None:
    listing = "\n".join(
        f"- `{name}` (from `{workspace['path_of'][name]}`)" for name in names
    )
    upstream = (
        "# Upstream\n\n"
        "Generated by `scripts/vendor-zed.py`. Do not edit by hand.\n\n"
        f"- zed commit: `{sha}`\n"
        f"- zed commit date: {date}\n"
        f"- vendored: {date}\n"
        f"- crates: {len(names)}\n\n"
        f"{listing}\n"
    )
    (out / "UPSTREAM.md").write_text(upstream)

    if not (out / "PATCHES.md").exists():
        (out / "PATCHES.md").write_text(
            "# Local patches\n\n"
            "Each local patch to this tree is a land on the working line "
            "(`backlog-1`), never on the `zed-vendor` line. List them here so a re-vendor only "
            "needs conflicts resolved in these files.\n\n"
            "| patch | commit | files |\n"
            "|---|---|---|\n"
            "| _none yet_ | | |\n"
        )
    (out / "README.md").write_text(readme(len(names)))


def readme(count: int) -> str:
    excluded = ", ".join(f"`{name}`" for name in EXCLUDED)
    return f"""\
# Vendored zed crates

`crates/` holds the path-dependency closure of zed's `gpui_platform` ({count}
crates at the revision in `UPSTREAM.md`), copied in by `scripts/vendor-zed.py`.
The crates are rewritten to build outside zed's workspace: every inherited
workspace field or dependency is replaced by the concrete version or path it
resolved to, inherited `[lints]` tables are dropped, and `publish` is `false`.

fluent-gpui supports Windows and Linux only, so {excluded} are left out, and
the dependencies on them and feature forwards to them are removed from the
vendored manifests. The code that uses them is behind macOS or wasm `cfg`s.

`gpui_ios` is still vendored: WI-2728 trims it from the tree.

Nothing here is published to crates.io.

## The pristine line

Local patches and upstream snapshots have to stay distinguishable, so the
snapshot lives on its own graft line, `zed-vendor` (not a git branch):

- `zed-vendor` receives only the script's output. Never land a hand-edit or a
  patch on it. A re-vendor is `scripts/vendor-zed.py <new-rev> --zed <zed>`
  run in a clone of `zed-vendor`, then `g commit` and `g land`.
- The working line (`backlog-1`, which batches to `main`) takes the snapshot
  with `g line merge zed-vendor --into backlog-1`. Patches are ordinary lands
  there, each recorded in `PATCHES.md` with the files it owns.
- Conflicts on that merge should only appear in files `PATCHES.md` names; a
  conflict anywhere else means the tree was edited outside a recorded patch.
  A patch that no longer applies is re-ported or dropped, and the list
  updated.

`zed-vendor` is created once, with the first snapshot (WI-1347):
`g line create zed-vendor --from <changeset>`, where the changeset is the
first snapshot's land. Its history is then the sequence of upstream
revisions, and a diff of `vendor/zed` between `zed-vendor` and the working
line is exactly the local patches.

## Checking a revision without committing it

```
scripts/vendor-zed.py <rev> --zed /path/to/zed --dry-run
scripts/vendor-zed.py <rev> --zed /path/to/zed --out /tmp/gpui-vendor
scripts/vendor-zed.py <rev> --zed /path/to/zed --out /tmp/gpui-vendor  # second run: 0 files changed
```

`--dry-run` lists the crate closure and what would be written without
touching the tree.
"""


def remove_stale(crates_root: Path, names: list[str]) -> None:
    """Drop vendored crates the closure no longer reaches."""
    keep = set(names)
    if not crates_root.exists():
        return
    for child in crates_root.iterdir():
        if child.is_dir() and child.name not in keep:
            shutil.rmtree(child)


def assert_closed(out: Path) -> None:
    """Fail if any workspace inheritance survived the rewrite."""
    leftovers = []
    for manifest in (out / "crates").rglob("Cargo.toml"):
        for lineno, line in enumerate(manifest.read_text().splitlines(), 1):
            if "workspace = true" in line:
                leftovers.append(f"{manifest}:{lineno}: {line.strip()}")
        for lineno, line in enumerate(manifest.read_text().splitlines(), 1):
            if not line.lstrip().startswith("#") and re.search(rf"\b({_EXCLUDED_ALT})\b", line):
                leftovers.append(f"{manifest}:{lineno}: excluded crate: {line.strip()}")
    if leftovers:
        sys.exit("workspace inheritance left behind:\n" + "\n".join(leftovers))


def snapshot(path: Path) -> dict:
    """Byte content of every file, so two runs can be compared."""
    files = {}
    for dirpath, dirnames, filenames in os.walk(path):
        dirnames.sort()
        for name in sorted(filenames):
            file = Path(dirpath) / name
            rel = str(file.relative_to(path))
            files[rel] = file.read_bytes() if not file.is_symlink() else os.readlink(file)
    return files


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("rev", help="zed commit to vendor")
    parser.add_argument("--zed", type=Path, required=True, help="path to a zed checkout")
    parser.add_argument(
        "--out",
        type=Path,
        default=Path("vendor/zed"),
        help="where to write the tree (default: vendor/zed)",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="resolve and report what would be written, changing nothing",
    )
    args = parser.parse_args()

    repo = args.zed.resolve()
    if not (repo / ".git").exists() and not repo.is_dir():
        sys.exit(f"{repo} is not a directory")

    sha, date = resolve_rev(repo, args.rev)
    workspace = load_workspace(repo, sha)
    names = closure(repo, sha, workspace)

    print(f"{sha[:12]} ({date}): {len(names)} crates")
    for name in names:
        print(f"  {name:24} {workspace['path_of'][name]}")

    if args.dry_run:
        print(f"dry run: would write {args.out}/" + ", ".join(crate_dest(n) for n in names))
        return

    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    before = snapshot(out) if any(out.iterdir()) else {}
    write_tree(repo, sha, date, names, workspace, out)
    after = snapshot(out)
    changed = sorted(set(before) ^ set(after) | {k for k in after if before.get(k) != after[k]})
    print(f"wrote {out} ({len(changed)} files changed)")


if __name__ == "__main__":
    main()
