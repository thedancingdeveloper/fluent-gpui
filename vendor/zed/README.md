# Vendored zed crates

`crates/` holds the path-dependency closure of zed's `gpui_platform` (25
crates at the revision in `UPSTREAM.md`), copied in by `scripts/vendor-zed.py`.
The crates are rewritten to build outside zed's workspace: every inherited
workspace field or dependency is replaced by the concrete version or path it
resolved to, inherited `[lints]` tables are dropped, and `publish` is `false`.

fluent-gpui supports Windows and Linux only, so `gpui_macos`, `gpui_apple`, `gpui_web` are left out, and
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
