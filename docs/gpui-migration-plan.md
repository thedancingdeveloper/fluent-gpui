# GPUI migration plan: adopt upstream gpui (crate split + wgpu)

Status: proposal, 2026-10-14. Tracks Vogt WI-1346 and the initiative
"fluent-gpui: adopt upstream gpui (crate split + wgpu)". Design only; no
product code changes come with this document.

## 1. Where we are

- `crates/gpui` is a vendored copy of the **crates.io package `gpui` 0.2.2**
  (its `Cargo.toml` is the registry-normalised one and `Cargo.toml.orig` is the
  packaging leftover). Upstream base: zed `08d95ad9d31f`, 2025-10-22. The
  sibling crates it needs (`gpui_collections`, `gpui_util`, `gpui_http_client`,
  `gpui_refineable`, `gpui_sum_tree`, `gpui_macros`, ...) come from crates.io
  at 0.2.2. The fork's `0.2.5` is a local number.
- The base, the evidence for it and the local-patch inventory are in
  `crates/gpui/UPSTREAM.md` on PR #1 (branch `wi-1346-gpui-upstream-sync`).
- **The fluent crates use none of the fork-only APIs.** `backdrop_blur`,
  `alloc_video_texture`, `paint_video_frame`, `free_video_texture` and
  `VideoTextureId` are referenced only inside `crates/gpui`.
  `surface_blur_layer` in fluent-core is a plain colour token used with
  `.bg(...)`. The patches exist for downstream apps, not for fluent itself.
- Upstream (`zed-industries/zed`, tip `26363aabdb`, 2026-10-13) has moved
  601 commits on `crates/gpui` and restructured it:
  - `af8ea0d6c2` 2026-02-13: Blade removed, **Linux renderer rewritten on
    wgpu** (`crates/gpui_wgpu`).
  - `bc31ad4a8c` 2026-02-19: platform backends extracted into `gpui_platform`,
    `gpui_linux`, `gpui_windows`, `gpui_macos`, `gpui_apple`.
  - `14f37ed502` 2026-02-26: `gpui_web` (wasm).
  - **Windows still renders with DirectX** (`gpui_windows/src/directx_*`) and
    **macOS still renders with Metal** (`gpui_apple/src/metal_*`). Only the
    Linux backend changed renderer. This matters for the patch re-ports below.
  - `gpui::Application::new()` is gone. Binaries call
    `gpui_platform::application()` (or `headless()`), which picks the OS
    backend. Libraries keep depending on `gpui` only.
  - Toolchain `1.98.1`, edition 2024, taffy `0.13`, wgpu `29`. This host has
    1.99.0.
- **Nothing newer than gpui 0.2.2 is on crates.io** (checked 2026-10-14), and
  `gpui_platform`, `gpui_wgpu`, `gpui_linux` etc. are not published at all
  (`publish = false` via the zed workspace). There is no registry path to the
  new structure.
- Every crate in the vendoring closure (section 3) is `Apache-2.0`.

## 2. Target end state

1. The repository carries a pristine snapshot of the upstream crate closure at
   a pinned zed commit, produced by a script, plus a short list of named local
   patches on top. No hand-edited upstream code outside those patches.
2. `fluent-core`, `fluent-primitives`, `fluent-ribbon`, `fluent-layout`,
   `fluent-app` and the examples build and test against that snapshot on
   Linux, macOS and Windows in CI.
3. `fluent-app` starts the app through `gpui_platform::application()`; the
   library crates depend on `gpui` only.
4. Video frames and backdrop blur work on Linux (wgpu); their Windows and
   macOS status is whatever the operator decides in section 5, and is
   documented rather than implied.
5. `outstanding.md` no longer carries the fork-tracking items; `UPSTREAM.md`
   and `PATCHES.md` under the vendored tree do.
6. A workspace release is tagged and the changelog explains the dependency
   change to consumers.

## 3. Migration strategy

### 3.1 Vendor a snapshot, with a pristine branch (recommended)

Vendor the path-dependency closure of `gpui_platform` from a zed checkout at a
pinned commit into `vendor/zed/crates/<name>` (upstream paths preserved).
The closure at `26363aabdb` is 25 crates, about 11 MB of source:

| group | crates |
|---|---|
| gpui proper | `gpui`, `gpui_platform`, `gpui_macros`, `gpui_shared_string`, `gpui_util` |
| backends | `gpui_linux`, `gpui_wgpu`, `gpui_windows`, `gpui_macos`, `gpui_apple`, `gpui_web` |
| support | `collections`, `util`, `util_macros`, `http_client`, `refineable`, `derive_refineable`, `sum_tree`, `scheduler`, `bench_metrics`, `path`, `perf` (tooling), `zlog`, `ztracing`, `ztracing_macro` |

`gpui_web` is only reachable through a `wasm32` target block, but Cargo
resolves every target's dependencies into the lockfile, so the path has to
exist; vendoring it is cheaper than patching the manifest.

A script, `scripts/vendor-zed.py <rev>`, does the copy and the mechanical
rewrites: replace `foo.workspace = true` with the concrete version or path from
zed's root `[workspace.dependencies]`, replace `[lints] workspace = true` with
nothing (or add an equivalent `[workspace.lints]` to our root), set
`publish = false`, and write `vendor/zed/UPSTREAM.md` with the rev, date and
crate list. Runs are idempotent so a re-vendor is a clean diff.

Rebaseability comes from git, not from patch files:

- Branch `zed-vendor` only ever receives the script's output ("pristine").
- `main` merges `zed-vendor`. Local patches are ordinary commits on `main`
  that touch `vendor/zed/**`, each named in `vendor/zed/PATCHES.md` with the
  files it owns.
- To move the base: run the script at the new rev on `zed-vendor`, commit,
  merge into a branch off `main`, resolve conflicts only in files `PATCHES.md`
  lists, run CI. Patches that no longer apply are either re-ported or dropped
  and the list updated.

Trade-offs against the alternatives:

| option | for | against |
|---|---|---|
| **Vendored snapshot + pristine branch** | builds offline; one repo for workers; patches are normal commits; licence-clean (all Apache-2.0); merge-based rebase | ~11 MB of upstream source in the repo; lockfile grows (wgpu, cosmic-text, accesskit, wayland/x11/windows/objc2 stacks); script has to be maintained |
| Git dependencies on a zed fork (`git = ..., rev = ...`) | rebase is `git rebase` on the fork | every build and CI run clones the whole zed monorepo (GBs); patches live in a second repo; **crates.io refuses git dependencies**, so nothing could be published |
| Git submodule of a zed fork | same as above, slightly more explicit pin | same clone cost; submodules are a known foot-gun for parallel worker sessions and worktrees |
| Keep the current patched fork and cherry-pick | smallest diff today | the files the patches live in were deleted upstream; cherry-picking 601 commits onto a crates.io-normalised tree is not a workable process |
| Wait for upstream to publish gpui 0.3 to crates.io | no vendoring at all | no release since 2025-10-22, the new crates are marked unpublishable, and there is no announced timeline |

### 3.2 Big-bang vendor, staged everything else (recommended)

The gpui tree cannot be half-replaced, so the vendor swap is one change. The
rest is staged so `main` stays releasable on the current fork until the end:

1. Long-lived integration branch `gpui-upstream` off `main`. All migration
   items land there as reviewed PRs. Nothing touches `main` until the branch
   is green on three platforms.
2. Order inside the branch: vendor script and first snapshot; CI matrix;
   symbol drift report; fluent crates bottom-up (core, primitives, ribbon,
   layout, app, examples); screenshot baselines; patch re-ports; validation;
   docs; merge; release.
3. Patches are re-ported Linux first, because that is the only backend that
   changed renderer and the only one this host can run. Windows and macOS
   patch work waits for CI to prove the vendored tree builds there.

Per-platform staging of the gpui tree itself (e.g. "Linux on upstream, Windows
on the old fork") is not viable: the platform code is split by `cfg`, and two
gpui versions cannot coexist in one dependency graph.

### 3.3 Where the snapshot is pinned

Pin to the newest zed commit at the time the first snapshot is made where
`cargo check -p gpui_platform` passes on Linux, not necessarily `26363aabdb`.
Record it in `UPSTREAM.md`. Afterwards, re-vendor on demand (an upstream fix
we need, or a quarter has passed), never automatically.

### 3.4 What breaks for consumers

The fluent crates published on crates.io today depend on `gpui = "0.2"`, and
that resolves to upstream 0.2.2 from the registry. After the migration,
`fluent-app` needs `gpui_platform`, which is not on crates.io. Until upstream
publishes the split, **the fluent crates cannot be published to crates.io
against the new tree**. Consumers would take the workspace as a git dependency
(tag) instead. This is operator decision 1.

## 4. How each local patch survives

| patch | lines | plan |
|---|---|---|
| Video frames: API (`window.rs`, `scene.rs`, `platform.rs`, test platform, example) | ~320 | **Keep as a patch.** Re-apply onto the vendored `gpui`; the API is additive and the files still exist. |
| Video frames: Linux renderer (blade atlas/renderer/shaders) | ~650 | **Rewrite on wgpu** in `gpui_wgpu` (atlas texture slot, RGBA8 upload via `queue.write_texture`, a `VideoFrame` draw in the WGSL shader). Blade is gone; nothing to merge. |
| Video frames: Windows DirectX | ~330 | **Re-apply with conflicts** onto `gpui_windows/src/directx_*` and `shaders.hlsl`; the renderer survived the split. Verified by CI build on `windows-latest` plus a manual run on the estate Windows box. |
| Video frames: macOS Metal | 12 (stub) | Implement the Metal path in `gpui_apple` (closes the outstanding gap) **or** keep the "unsupported" stub. Operator decision 3. |
| Backdrop blur: API (`style.rs`, `styled.rs`, `color.rs`) | ~70 | **Keep as a patch**, re-apply. |
| Backdrop blur: renderer | ~160 | **Rewrite on wgpu** (framebuffer snapshot + single-pass blur, as today). Windows/macOS: tinted-solid fallback by default; operator decision 4. |
| SVG font-DB pre-warming (`svg_renderer.rs`) | +16/−5 | Re-apply if upstream has not already changed how the `usvg` font database is built, and **open an upstream PR**; it is small and general. |
| Small element/input fixes (`div.rs`, `list.rs`, `interactive.rs`, `key_dispatch.rs`, `tab_stop.rs`, `gpui.rs`, `build.rs`) | ~150 | Classified one by one in section 4.1. Four of five are already upstream and are dropped. |
| Element visual scale (`Style::scale`, `Styled::scale`, `with_element_scale`, `transform_bounds_for_paint`) | ~200 | Found during triage; unused by any fluent crate but in scope (operator, 2026-10-14). **Keep as a named patch** and re-apply by hand onto the vendored `gpui` (`style.rs`, `styled.rs`, `elements/div.rs`, `window.rs`); renderer-independent, so Linux verification covers it. |
| `README.md` in the crate, `Cargo.toml.orig`, "reqwest examples behind a feature" | — | **Drop.** `UPSTREAM.md`/`PATCHES.md` replace the README notes; the packaging artefact goes away with the registry-normalised manifest; upstream examples are gated their own way. |

### 4.1 Small fixes, classified

Triage of the fork against the base and against upstream tip (diffs taken
2026-10-14):

| file | what the fork changed | upstream tip | plan |
|---|---|---|---|
| `elements/list.rs` | `if let Some(handler)` instead of `unwrap()` on the scroll handler | has the equivalent (`list.rs:942`) | **drop** |
| `interactive.rs` | `#[derive(Default)]` for `MouseButton`, `NavigationDirection` | identical | **drop** |
| `key_dispatch.rs` | `if let Some(view_id)` push | identical | **drop** |
| `tab_stop.rs` | array literal in a test | identical | **drop** |
| `elements/div.rs` (cleanup part) | let-chain tidy in `paint_debug_info` | has it | **drop** |
| `elements/div.rs` (feature part) + `style.rs`/`styled.rs` + `window.rs` | **element visual scale**: `Style::scale`, `Styled::scale(f32)`, `window.with_element_scale(...)` wrapping the paint closure, shadow blur scaled too. Not in the earlier inventory; it was counted inside the video-frame `window.rs` delta | nothing like it; the paint body has grown (focus registration inside `with_tab_group`), so a re-apply conflicts | **keep, as its own named patch** (operator decision, 2026-10-14). No fluent crate calls `.scale(...)`, so it is verified through the gpui example that the item adds. Re-applied by hand: one wrapper line in `div.rs` plus the `window.rs` scale stack and `Style`/`Styled` additions |
| `gpui.rs` | `pub use svg_renderer::warmup_svg_font_db` | — | re-apply with the SVG patch |
| `build.rs` | adds `video_frame` to the HLSL entry-point list | that list moved to `gpui_windows/build.rs` | re-apply there, as part of the Windows video item |
| `svg_renderer.rs` | hoists the usvg font DB to a module `LazyLock` and adds `pub fn warmup_svg_font_db()` so `main()` can prime it on a background thread | restructured: a function-local system DB plus a per-renderer "enriched" DB, deliberately lazy; no warm-up entry point | **re-apply with conflicts, then upstream** (small, self-contained) |
| `color.rs`/`style.rs`/`styled.rs` (blur part) | `BackgroundTag::BackdropBlur`, `backdrop_blur(radius, tint)`, `needs_backdrop()` | `BackgroundTag` unchanged; additive arms re-apply at moved offsets | keep; renderer rewritten on wgpu |

Upstream has no element-level blur at all: every `blur` hit is box-shadow
`blur_radius`, every `backdrop` hit is an OS window material
(`WindowBackgroundAppearance::{Blurred, MicaBackdrop}`). For video, upstream's
`paint_surface` is still macOS/iOS-only (`CVPixelBuffer` through
`CVMetalTextureCache` in `gpui_apple/src/metal_renderer.rs:1183`), and the
wgpu renderer says so in a comment (`wgpu_renderer.rs:1612`, "Surfaces are
macOS-only ... not implemented by the WGPU renderer"); DirectX has an empty
`draw_surfaces`. So the fork's `VideoFrame` primitive has no upstream
equivalent on Linux or Windows, and a cross-platform RGBA surface is a
plausible upstream contribution once it works on all three backends.

## 5. API-drift triage for the five fluent crates

The fluent crates import 78 distinct `gpui` symbols. Most are stable element
and styling names (`div`, `px`, `IntoElement`, `Render`, `SharedString`,
`svg`, `anchored`, `deferred`, `uniform_list`, mouse events). The ones most
likely to have moved are:

- `Application::new()` in `fluent-app/src/app_builder.rs:162` and
  `examples/screenshots/src/main.rs:577`: replaced by
  `gpui_platform::application()`. `fluent-app` and the example binaries gain a
  `gpui_platform` dependency.
- Client-side decorations in `fluent-app/src/chrome.rs` (`window_decorations`,
  `Decorations::Client`, `start_window_resize`, `ResizeEdge`,
  `insert_hitbox`/`HitboxBehavior`) and `title_bar.rs` (`start_window_move`,
  `zoom_window`, `minimize_window`): a year of Wayland/X11/Windows CSD work.
- Text: `text_system().shape_line`, `ShapedLine`, `TextRun` in
  `fluent-primitives/src/text_input.rs` (lines 27, 557, 640-683).
- Input: `EntityInputHandler`, `ElementInputHandler`, `UTF16Selection`,
  `KeyBinding`, the `actions!` macro; `gestures.rs` is new upstream.
- Tests: `#[gpui::test]` (13 uses), `TestAppContext`, `VisualTestContext`,
  `cx.executor().advance_clock`; `fluent-test-utils` re-exports them.
- Layout: taffy 0.10 to 0.13 can change measured sizes without any API change.

Approach, in order:

1. **Symbol drift report** (one worker, script-driven): for each of the 78
   symbols, locate it in the base and in the snapshot, diff the signature, and
   write `docs/gpui-api-drift.md` with a status per symbol: unchanged, moved,
   signature changed, removed (with the replacement). This is the map the
   per-crate workers follow, and it is the measure of how big the job is.
2. **Bottom-up crate ports** on the integration branch, one worker each:
   `fluent-core` (1.3k lines) → `fluent-primitives` (7.1k) → `fluent-ribbon`
   (1k) → `fluent-layout` (9k) → `fluent-app` + `fluent-test-utils` (0.9k) →
   examples. Each item's gate is `cargo fmt --check`, `cargo clippy -p <crate>
   --all-targets -- -D warnings` and `cargo test -p <crate>` on Linux, and the
   crate below it already merged.
3. **Behavioural check**: regenerate the screenshot set with
   `scripts/screenshots.sh` and compare against `docs/VISUAL_TARGETS.md` and
   the committed `screenshots/`. Layout and text rasterisation changes show up
   here, not in the compiler.
4. Edition: the fluent crates are edition 2021; the vendored tree is 2024.
   Mixed editions are fine in one workspace. Do not bump the fluent crates in
   the same change.

## 6. Platform verification

This host is Linux only (Wayland/X11, AMD Vulkan). The repository is public
under a personal GitHub account, so **GitHub-hosted `macos-latest` and
`windows-latest` runners are free**. The estate's own runner stack
(`github-runner` on node-b) is Linux-only, has no Rust toolchain preinstalled,
and its policy of rejecting `ubuntu-latest` applies to the
`TheDancingDeveloper-org` organisation, not to this personal repo. Decision 6
asks the operator to confirm hosted runners are acceptable here.

| what | Linux | Windows | macOS |
|---|---|---|---|
| build, clippy, unit tests (test platform, no GPU) | CI `ubuntu-latest` + this host | CI `windows-latest` | CI `macos-latest` (arm64) |
| run the examples, video frame example, blur | this host (wgpu/Vulkan) | estate Windows dev box `desktop-tlk1gff` over SSH (DirectX; the box has a GPU and a desktop session) | **no estate Mac.** Options in decision 2. |
| screenshot baselines | this host (`scripts/screenshots.sh`, GNOME-specific) | not attempted | not attempted |

CI changes, as one item: a three-OS matrix for fmt/clippy/test, a `--no-run`
test build for every crate, and a job that re-runs the vendor script at the
pinned rev and fails if `vendor/zed` differs (drift guard). Windows needs
`cargo test --workspace` to stay off the GUI (the test platform already does),
and the `windows-manifest` feature pulls `embed-resource`, which is fine on the
hosted image.

Runtime GPU validation stays manual and is recorded per platform in the
validation items; it is not a CI gate.

## 7. Risks

- **Dependency and build-time growth.** wgpu 29, cosmic-text, accesskit,
  the wayland/x11 and `windows` crates, objc2. Expect a cold `cargo build` of
  the workspace to roughly double. Mitigation: CI cache keyed on `Cargo.lock`;
  trim backend features we do not use (`screen-capture`, `inspector`,
  `bench-support`) in the vendor script.
- **Visual regressions** from taffy 0.13 and the wgpu text path. Mitigation:
  the screenshot comparison item is a gate before merge, not a follow-up.
- **The renderer rewrites cannot be verified on real Windows/macOS GPUs from
  CI.** Mitigation: CI builds; the Windows box for a manual run; macOS stays
  build-only unless decision 2 gives us a machine. Document the matrix
  honestly in `outstanding.md`'s successor.
- **crates.io publishing is blocked** after migration (section 3.4).
- **Worker-shaped risk:** the vendor script and the two renderer rewrites are
  design work; everything else is bounded. The item model labels reflect
  that.
- **Upstream keeps moving.** The pinned rev is a snapshot; a re-vendor is a
  merge with conflicts in a handful of listed files. Keeping `PATCHES.md`
  accurate is what keeps that cheap.
- **AMD/GNOME fullscreen crash** noted in `examples/screenshots/src/main.rs`:
  the wgpu backend may behave differently; re-check during Linux validation.

## 8. Open questions for the operator

1. **crates.io after the migration.** (a) Pause registry releases and ship
   via git tags (recommended: consumers are our own apps; revisit when
   upstream publishes). (b) Publish the vendored closure under our own crate
   names and keep publishing fluent (25 renamed crates to maintain). (c) Hold
   the migration until upstream publishes gpui 0.3 (no timeline).
2. **macOS runtime verification.** (a) Build and test on hosted runners only;
   mark macOS "build-verified" (recommended for now). (b) Operator runs the
   examples on a personal Mac per release. (c) Rent a hosted Mac.
3. **Video frames on macOS.** (a) Implement the Metal path in `gpui_apple`
   (recommended; the renderer is there and the gap is the oldest outstanding
   item). (b) Keep returning "unsupported" and document it.
4. **Backdrop blur on Windows/macOS.** (a) Tinted-solid fallback, blur is
   Linux-only (recommended). (b) Implement the blur on DirectX and Metal
   too. (c) Drop the blur variant.
5. **Vendor location.** (a) `vendor/zed/crates/*` with the pristine branch
   (recommended; the boundary is visible). (b) Keep the crates under
   `crates/` beside the fluent crates.
6. **CI runners.** (a) GitHub-hosted three-OS matrix (recommended; free for a
   public personal repo). (b) Estate self-hosted runners (Linux only, no Rust
   image, no macOS).
7. **Release numbering.** (a) Workspace release v3.0.0 (recommended: the gpui
   dependency and the `fluent-app` entry point change are breaking). (b) v2.5.0.
8. **Upstreaming.** (a) Open zed PRs for the SVG pre-warm and any small fix
   that classifies as upstreamable (recommended; shrinks the patch list). (b)
   Keep everything local.
9. ~~**Element visual scale.**~~ Decided 2026-10-14: the operator brought
   `Styled::scale(f32)` into scope. It is kept as a named patch with its own
   re-port item.

## 9. Backlog

Vogt initiative `fluent-gpui-adopt-upstream-gpui-crate-split-wgpu`
("fluent-gpui: adopt upstream gpui (crate split + wgpu)"). Every item is a
child of WI-1346. Model labels: haiku for mechanical work, grok for bounded
edits, opus-low for design, investigation, review and platform-risky work.

| # | ref | item | pri | effort | model | depends on |
|---|---|---|---|---|---|---|
| 1 | WI-1422 | Vendor script + pristine branch mechanism | p2 | m | opus-low | — |
| 2 | WI-1423 | CI three-OS matrix + vendor drift guard | p2 | s | grok | — |
| 3 | WI-1347 | First vendored snapshot on `gpui-upstream` | p2 | l | grok | 1422 |
| 4 | WI-1350 | Symbol drift report (78 symbols) | p2 | s | haiku | 1347 |
| 5 | WI-1424 | Snapshot review: licences, deny/audit, feature trims, build time | p2 | s | opus-low | 1347 |
| 6 | WI-1425 | Port fluent-core | p2 | s | grok | 1347, 1350 |
| 7 | WI-1426 | Port fluent-primitives | p2 | m | grok | 1425 |
| 8 | WI-1427 | Port fluent-ribbon | p2 | s | haiku | 1426 |
| 9 | WI-1428 | Port fluent-layout | p2 | m | grok | 1427 |
| 10 | WI-1429 | Port fluent-app + fluent-test-utils (`gpui_platform`, CSD) | p2 | m | opus-low | 1428 |
| 11 | WI-1430 | Port the examples | p2 | s | haiku | 1429 |
| 12 | WI-1431 | Screenshot baselines + visual regression review | p2 | s | opus-low | 1430 |
| 13 | WI-1432 | SVG font-DB pre-warm re-apply; drop 4 superseded fixes | p2 | s | grok | 1347 |
| 14 | WI-1433 | Element visual scale re-port | p2 | m | grok | 1347 |
| 15 | WI-1348 | Video frames: API + Linux/wgpu renderer | p2 | l | opus-low | 1347 |
| 16 | WI-1434 | Video frames: Windows/DirectX re-apply | p2 | m | opus-low | 1348, 1423 |
| 17 | WI-1435 | Video frames: macOS/Metal (or document unsupported) | p3 | m | opus-low | 1348, 1423 |
| 18 | WI-1349 | Backdrop blur: API + wgpu renderer + Win/mac fallback | p3 | m | opus-low | 1347 |
| 19 | WI-1436 | Upstream PRs to zed (SVG warm-up; later video surface) | p3 | s | opus-low | 1432 |
| 20 | WI-1437 | Rewrite fork docs: PATCHES/UPSTREAM/outstanding/README/VISUAL_TARGETS | p2 | s | haiku | 1432, 1433, 1348, 1349, 1434, 1435 |
| 21 | WI-1438 | Linux runtime validation (wgpu) | p2 | s | opus-low | 1430, 1348, 1349, 1433 |
| 22 | WI-1439 | Windows runtime validation (estate dev box) | p2 | m | opus-low | 1430, 1434 |
| 23 | WI-1440 | macOS verification (CI build; runtime per decision 2) | p3 | s | opus-low | 1430, 1435 |
| 24 | WI-1441 | Review and merge `gpui-upstream` into `main` | p2 | s | opus-low | 1431, 1437, 1438, 1439, 1440, 1424 |
| 25 | WI-1442 | Bump fluent versions + release (v3.0.0) | p2 | s | grok | 1441 |

Recommended order: 1 and 2 in parallel; 3; then 4, 5, 13, 14, 15 and 18 in
parallel; 6 through 11 strictly in sequence; 12; 16 and 17 once 15 and 2 are
done; 19 and 20 when their inputs land; 21, 22, 23 in parallel; 24; 25.

WI-1347, WI-1348, WI-1349 and WI-1350 are the four original children,
re-scoped in place rather than superseded. Model labels could not be attached
to those four (label edits are refused on this unlinked project), so their
bodies carry the suggestion instead.
