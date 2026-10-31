# Contributing to fluent-gpui

This is the maintainers' manual for the fork, for people and agent sessions.
It says how changes land, how the vendored Zed code is kept, how the monthly
upstream pull runs, which checks a land must pass, and how releases are cut.
For using fluent-gpui from an app, see [README.md](README.md). For how a
worker session completes one work item, see `AGENTS.md`.

Supported platforms are **Windows and Linux**. macOS is out of scope (§9).

> **Status (3.x re-fork).** Some of what this document describes is not
> built yet. Each such item is marked **Not yet built**, with the work item
> that adds it. Until then, follow the rule by hand.

---

## 1. How changes land (graft)

This repository lands through graft, under the workspace rules
`graft-only.md` and `delivery-process.md` ("Landing"). Nothing in this
repository is exempt from graft. Endpoint, line, mirror and token scopes are
in [docs/graft.md](docs/graft.md).

### Workers

1. `g clone fluent-gpui <dir> --line backlog-1`, then `g sync`.
2. Make one small change, in one area, for one work item.
3. Run the land checks for the crates you touched (§5).
4. `g commit -m "<message>" --meta agent.work_item=WI-<n>`, `g sync`
   (exit 3 means conflicts: resolve, re-check, commit), and `g stack squash`
   if you made more than one draft.
5. `g land`. Exit 4 (contended): run it again. Exit 7 (unavailable): retry
   up to three times, then report blocked. Exit 6 (not authorised): stop and
   report blocked.
6. Post a hand-off comment on the work item and stop.

Workers do not create git branches, push, open pull requests or poll CI.

### Oversight

- All landed work on `backlog-1` goes into **one** batch PR:
  `graft-mirror pr-batch backlog-1 --base main`. One open batch at a time.
- Cut a batch when at least 6 lands are waiting, or 60 minutes after the
  previous batch merged, whichever comes first. A merge alone is not a
  trigger.
- A batch is sealed once auto-merge is on; later lands wait for the next
  batch.
- Review the batch as a whole (§10). Blockers go back to the worker: close
  the batch, the worker fixes and lands, and the fix rides the next batch.
- Merge with `land-pr --method rebase`, so each land is one commit on
  `main`, with its `Graft-Session` trailer.
- A release blocker or production fix may go in a batch of its own; say so
  in the PR body.

---

## 2. Layering rules

- Only `fluent-*` crates and the vendored tree (`vendor/zed`) may depend on
  `gpui`. Examples and tests use `fluent-*` too, as apps do. `fluent-core`
  re-exports what they need.
- **Enforced in CI** by the `layering` job: `scripts/check-layering.py`
  (manifests via `cargo metadata`, plus `gpui::` / `gpui_*::` paths in any
  `.rs` file) and `deny.toml` `[bans]` (`gpui` with `wrappers`). The six
  examples are a recorded baseline that may shrink, never grow; they still
  import `gpui` until the `fluent-core` re-export lands.
- Vendored code changes only through the patch process (§3).

---

## 3. Vendored upstream

### Layout

| Item | What it is |
|---|---|
| `vendor/zed/crates/*` | The Zed crate closure GPUI needs (`gpui`, `gpui_platform`, `gpui_linux`, `gpui_windows`, `sum_tree`, `collections`, `util`, `refineable`, `scheduler`, `http_client`, `macros`, `shared_string`, ...), filtered from Zed `main`. |
| graft line `zed-vendor` | Pristine copy of that closure. Only the vendor script's output lands on it. |
| line `backlog-1` / `main` | `zed-vendor` merged in (`g line merge zed-vendor --into backlog-1`), plus the patch series and the `fluent-*` crates. |
| `vendor/zed/UPSTREAM.md` | Zed commit, date and crate list of the current snapshot. |
| `vendor/zed/PATCHES.md` | Every local patch: name, Cargo feature, upstream files it touches, upstream PR if any. Also the current patch-budget count. |

The vendor script is `scripts/vendor-zed.py <rev> --zed <zed checkout>`
(`--dry-run` lists the closure). It vendors Windows and Linux only: the
`gpui_macos`, `gpui_apple` and `gpui_web` crates are left out.
The first snapshot is in `vendor/zed` (its zed commit is in `vendor/zed/UPSTREAM.md`),
and the `zed-vendor` line was created from it (WI-1347). The old GPUI copy in
`crates/gpui` is gone. Don't hand-copy Zed code
into the repository.

`zed-vendor` is a graft line, not a git branch, and not a work line: it
is written only by the vendor script, during an upstream pull, and is never
edited by hand.

### Patches

- A patch is an additive module behind a Cargo feature where possible
  (`video-surface`, `backdrop-blur`, `svg-font-warmup` if upstream has not
  fixed it). Where a hook into a hot path (`scene.rs`, renderers, shaders)
  can't be avoided, it is one commented call site.
- Accessibility completions (disabled state, scroll clipping in the tree,
  live regions, OS text scale and reduced motion) are part of the base, not
  a feature.
- Not in the patch set: a general custom-shader API, a web platform,
  anything Metal-only.
- **Patch budget: 10 upstream files or fewer.** Growth is an operator
  decision, recorded in `PATCHES.md` with the date.
- No hand edits under `vendor/zed` outside the files `PATCHES.md` lists.
  `g diff` of `vendor/zed` between the `zed-vendor` tip and `backlog-1` must touch only those files.
  Reviewers run it on any batch that touches `vendor/zed` (it becomes a CI
  check with WI-1441 / WI-2725).
- A new patch: add the module and feature, its example and test, and its
  `PATCHES.md` line, in the same land.

---

## 4. Monthly upstream pull

The pull is a standing, dated Vogt work item with an owner ("Upstream pull
YYYY-MM"). It is never folded into feature work. Steps:

1. **Refresh `zed-vendor`** at a new Zed `main` commit with
   `scripts/vendor-zed.py <rev>`. Record the commit, date and crate list in
   `vendor/zed/UPSTREAM.md`.
2. **Merge `zed-vendor` into `backlog-1`** (`g line merge zed-vendor --into backlog-1`) **and re-apply the patch series**, patch
   by patch. A patch that no longer applies cleanly is rewritten, never
   force-merged. Update `PATCHES.md` and the budget count.
3. **Build on Windows and Linux.** Run the GPUI examples, the fork's
   validation examples `video_frame` and `backdrop_blur`, and the
   fluent-gpui gallery.
4. **Licences.** `cargo deny check licenses bans` (the CI `deny` job) with the
   `deny.toml` allow-list (Apache-2.0, MIT, BSD, ISC, Zlib, Unicode and other
   permissive licences) and **no GPL, LGPL or AGPL exception**. MPL-2.0 is not
   allowed globally; per-crate exceptions cover only `cbindgen`, `dwrote` and
   `option-ext`, which the gpui tree forces; the operator accepted these on
   2026-10-31. Then bump
   `[package.metadata.estate-gpui] zed-rev` in every `crates/fluent-*/Cargo.toml`
   to the new commit; the CI `zed-rev` job (`scripts/check-zed-rev.sh`) fails
   if it differs from `vendor/zed/UPSTREAM.md`. Spot-check
   `cargo tree -e normal` for any licence outside the list. A contaminated
   pull fails; it is not recorded as debt. Record the Zed commit and the
   result in `CHANGELOG.md`.
5. **API change notes**: public renames, trait changes, dependency bumps
   (Taffy and similar), in `CHANGELOG.md`.
6. **Tag** (§7) and open the follow-on items for consuming apps with the
   change notes.
7. **Accessibility smoke test**: dump the gallery's AccessKit tree and
   compare it with the previous tag's dump (roles and names, not bounds).
   Record differences on the pull item.

Between pulls, an upstream bug fix may be cherry-picked as its own patch
release. Two missed pulls in a row, or a pull that takes more than two
weeks, trigger the operator's review of the whole plan.

The pull lands through graft like any change (§1), in a batch of its own.

**Not yet built:** the pull tooling (WI-1422).
The first pull is rehearsed in WI-2726 from this document alone; gaps found
there are fixed here.

---

## 5. Checks a land must pass

| Check | Command |
|---|---|
| Format | `cargo fmt --all --check` |
| Lint | `cargo clippy --workspace --all-targets -- -D warnings` |
| Tests | `cargo test --workspace` |
| Licences and advisories | `cargo deny check licenses bans` (CI job `deny`) |
| Layering | `python3 scripts/check-layering.py` (CI job `layering`); `scripts/check-zed-rev.sh` (CI job `zed-rev`) |
| Visuals | screenshot baselines (`scripts/screenshots.sh`) for any change to `fluent-*` rendering |

Where they run:

- **Workers** run the subset for the crates they touched
  (`-p <crate>` instead of `--workspace`), with `CARGO_BUILD_JOBS` capped on
  shared hosts.
- **CI** runs the workspace on Windows and Linux. The `test` job is a matrix
  (`ubuntu-latest`, `windows-latest`); the `ci` summary job needs `test`,
  `layering`, `deny` and `zed-rev` and is the one required check. No macOS.
- **GPU runtime validation** (WI-1438, WI-1439) is manual, on the approved
  hosts (desktop-tlk1gff for DirectX 11), and recorded on the work item.

### Toolchain

The Rust toolchain is pinned in `rust-toolchain.toml` (channel, plus
`rustfmt` and `clippy`). Everyone and CI use that version. Changing it is a
land of its own, which bumps the file and fixes any new lints in the same
change; it is not mixed with feature work.

---

## 6. Accessibility gate

Every control has:

- a role, a name and its actions in the AccessKit tree;
- visible focus;
- a keyboard path for every action;
- status changes announced (live regions).

Before a release tag, the gallery passes an NVDA run on Windows and an Orca
run on Linux. Findings are recorded on the release work item.

---

## 7. Versioning and releases

- Releases are git tags on `main` of the form
  `v<major>.<minor>.<patch>+zed.<short-sha>` (first: `v3.0.0+zed.<sha>`,
  WI-2727), where `<short-sha>` is the Zed commit in `UPSTREAM.md`.
- **Git tags only.** Nothing is published to crates.io.
- Each `fluent-*` crate records the Zed commit in
  `[package.metadata.estate-gpui] zed-rev = "<sha>"`.
- No version bumps in feature lands. Versions and `zed-rev` move in the
  release land.
- `CHANGELOG.md`: one line per land under `## Unreleased`, so lands in the
  same batch don't conflict. The release land turns `Unreleased` into the
  version heading.
- The release land carries `CHANGELOG.md`, the version bumps and the
  accessibility findings; after it merges, the tag is cut on that commit.
  The release build is dry-run before a tag is first cut.

---

## 8. Upstream etiquette and attribution

- Bug fixes may be offered to Zed under its CLA by a human author who
  understands the change, with LLM assistance disclosed.
- Renderer features (video surfaces, backdrop blur) are not offered. Zed
  closed its custom-shader PR as out of scope.
- Keep Zed's `LICENSE-APACHE` and NOTICE in `vendor/zed` and in every
  release artefact (Apache-2.0 §4).

---

## 9. Platform scope

- **Supported:** Windows (DirectX 11, UI Automation) and Linux (wgpu, X11
  and Wayland, AT-SPI).
- **Out of scope:** macOS: no validation, no artefacts, no Metal-only work.
  A best-effort, non-blocking `cargo check` for macOS may run. Web and wasm
  are out of scope too.

---

## 10. Review and who decides

Batch review (by oversight):

- **Opus** if any land touches `vendor/zed`, licences or `deny.toml`, CI
  workflows, or security.
- **Sonnet, low effort** otherwise.
- Formatting-only and docs-only batches skip review.

Needs the **operator**:

- growing the patch budget past 10 upstream files;
- changing the pull cadence;
- changing platform scope;
- any licence exception (there are none, and the default answer is no);
- distribution changes (for example publishing to crates.io).

**Oversight** decides everything else: batch timing, review routing,
worker assignment, cherry-picks between pulls.
