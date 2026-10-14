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

## This snapshot

Pinned at `76b8ab814f0d` (2026-10-14), the tip of zed `main` on that date. It is
18 commits past `26363aabdb31` (2026-10-13), the reference SHA the migration plan
was written against; none of those commits touch the gpui crate closure or the
zed workspace manifest, so the closure matches the reference pin. The newer tip
was taken because the plan's pin rule is the newest zed commit on which
`gpui_platform` checks on Linux, and `cargo check -p gpui_platform` passes at
this pin (rustc 1.96.1).

The workspace `Cargo.toml` on `gpui-upstream` lists the crates the check needs
as members and points the `gpui` workspace dependency at
`vendor/zed/crates/gpui`. The published gpui 0.2 fork at `crates/gpui` is no
longer a member, because both packages are named `gpui`; its source stays in the
tree until the migration retires it. No file under `vendor/zed/` was patched to
make the check pass.

## Checking a revision without committing it

```
scripts/vendor-zed.py <rev> --zed /path/to/zed --out /tmp/gpui-vendor
scripts/vendor-zed.py <rev> --zed /path/to/zed --out /tmp/gpui-vendor  # second run: 0 files changed
```

`--dry-run` reports the crate list and what would be written without touching
the tree.
