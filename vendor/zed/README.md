# Vendored zed crates

`crates/` holds the path-dependency closure of zed's `gpui_platform` (27
crates at the revision in `UPSTREAM.md`), copied in by `scripts/vendor-zed.py`.
The crates are rewritten to build outside zed's workspace: every
`workspace = true` inheritance is replaced by the concrete version or path it
resolved to, `[lints] workspace = true` is dropped, and `publish` is `false`.

Nothing here is published to crates.io.

## The pristine branch

Local patches and upstream snapshots have to stay distinguishable, so the
snapshot lives on its own branch:

- `zed-vendor` receives only the script's output. Never commit a hand-edit or a
  patch to it. A re-vendor is `scripts/vendor-zed.py <new-rev>` committed on
  that branch.
- The integrating branch (`gpui-upstream`, and `main` once it merges) merges
  `zed-vendor`. Patches are ordinary commits there, each recorded in
  `PATCHES.md` with the files it owns.
- To move the base: run the script at the new revision on `zed-vendor`, commit,
  and merge into a branch off the integrating branch. Conflicts should only
  appear in files `PATCHES.md` names; a conflict anywhere else means the tree
  was edited outside a recorded patch. A patch that no longer applies is
  re-ported or dropped, and the list updated.

`zed-vendor` is cut from the first snapshot, so its history is the sequence of
upstream revisions and a diff against it is exactly the local patches.

## Checking a revision without committing it

```
scripts/vendor-zed.py <rev> --zed /path/to/zed --out /tmp/gpui-vendor
scripts/vendor-zed.py <rev> --zed /path/to/zed --out /tmp/gpui-vendor  # second run: 0 files changed
```

`--dry-run` reports the crate list and what would be written without touching
the tree.
