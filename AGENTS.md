# Instructions for agents

fluent-gpui is the estate's fork of GPUI (Zed's GPU UI framework) plus a Fluent 2 design
system on top of it, for native desktop apps on **Windows and Linux only**. Apps depend on the
`fluent-*` crates; only those crates and the vendored Zed closure touch `gpui`.
`README.md` is the consumer contract; `CONTRIBUTING.md` is how the repo is maintained.

| Path | What it is |
|---|---|
| `crates/fluent-*` | The library: `fluent-core`, `fluent-primitives`, `fluent-layout`, `fluent-ribbon`, `fluent-app`, `fluent-test-utils` |
| `vendor/zed/crates/*` | Vendored upstream Zed crate closure (gpui and what it needs), plus named estate patches. The only gpui source; the old `crates/gpui` copy is gone |
| `examples/` | Demo, gallery and sample apps |
| `scripts/` | Dev scripts (screenshots, `vendor-zed.py` for vendoring) |
| `docs/` | `graft.md` (how changes land), `VISUAL_TARGETS.md` |

Read `CONTRIBUTING.md` §2 and §3 (vendoring and patches) before touching anything under
`vendor/zed`.

## How a work item is done

1. **Read the Vogt item fully.** It names the files, the checks and the line. Bind your session
   to it (`session_bind_work`).
2. **Get the code through graft, on line `backlog-1`.** `g clone fluent-gpui <dir> --line backlog-1`,
   or `g sync` an existing checkout first. Setup (endpoint, token file, author identity) is in
   the worker setup your brief points to. Never create a git branch, never open a PR, never
   push to GitHub.
3. **Touch only the files the item lists**, plus `CHANGELOG.md` (one line under Unreleased). If
   you need more, or a design choice the item and `CONTRIBUTING.md` don't settle, stop and ask
   on the item. Do not guess.
4. **Run the crate-scoped gates** on the pinned toolchain (`rust-toolchain.toml`), for each crate
   you touched:

   ```sh
   export CARGO_BUILD_JOBS=2   # shared hosts; run under nice -n 10
   cargo fmt --all -- --check
   nice -n 10 cargo clippy -p <crate> --all-targets -- -D warnings
   nice -n 10 cargo test -p <crate>
   ```

   Docs-only items skip cargo. The workspace-wide suite and the Windows/Linux matrix are CI's job.
5. **Land and stop.** `g commit -m "<message>" --meta agent.work_item=WI-<n>`, `g sync`, then
   `g land` with your session token. Post a `handoff` comment on the item (changeset id, gates
   run, each "done when" point mapped to a file or test). Do not poll CI. Print `AUTOPILOT: DONE`.

## Rules that are never broken

- **Layering:** no `gpui` dependency or `use gpui::` outside `crates/fluent-*` and `vendor/zed`,
  examples and tests included. CI fails it.
- **Vendored code:** no hand edits inside `vendor/zed` except in files `vendor/zed/PATCHES.md`
  lists for a named patch. The pristine branch `zed-vendor` is written only by
  `scripts/vendor-zed.py`. Patch procedure: `CONTRIBUTING.md`.
- **No upstream Zed rev change inside a feature land.** The monthly pull is its own item.
- **No version bumps** in a feature land.
- **No macOS work:** no Metal paths, no macOS validation claims, no macOS-only fixes. macOS is
  out of scope (operator, 2026-10-31). Web and wasm are out of scope too.
- **No `unsafe` and no panics** in `fluent-*` library code without an item that says so.
- **Assets:** no Microsoft fonts or Office artwork in the repo or binaries; icons only through
  the generated catalogue; every asset has an attribution row.
- **Generated files** (tokens, icon catalogue, vendored tree) change only by re-running their
  generator.
- **No secrets, tokens or transcripts** committed. Command output that may hold a token goes
  through `redact`.

## Reviews

A land is reviewed as part of the batch PR that oversight cuts, not by the worker. If a
reviewer sends blockers, the fix is a new land on `backlog-1`, not an amended one.

## When blocked

Call `session_report_blocked` with the exact question, and comment on the item. Do not guess at
policy. Graft exit codes: 4 (contended) retry `g land`; 7 (unavailable) retry up to 3 times,
then report blocked; 6 (not authorized) report blocked.

## Pointers

- `CONTRIBUTING.md`: maintenance, upstream pull, patches, licences, releases
- `README.md`: consumer contract
- `docs/graft.md`: how changes land
- `vendor/zed/PATCHES.md`: the estate patch list
- `docs/VISUAL_TARGETS.md`: visual targets
