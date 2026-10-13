# GPUI upstream tracking

`crates/gpui` is a vendored fork of [zed-industries/zed](https://github.com/zed-industries/zed)
`crates/gpui`. The fluent-gpui repository was imported as one squashed commit
(`8371136`, "Initial commit — FluentGUI v2.4.0", 2026-05-29), so there is no
shared git history to merge along. This file records where the fork was taken
from and what we changed, which is the tracking outstanding.md asks for.

## Upstream base

**`08d95ad9d31f616a43dacda8416568d658dca6ae`** — zed-industries/zed, 2025-10-22,
"chore: Bump gpui to 0.2.2 (#40856)".

The fork's own `Cargo.toml` says `0.2.5`. That version is local. Upstream still
reads `0.2.2` on main, so the version number says nothing about how far behind
we are.

### How the base was found

Every `.rs`, `.wgsl`, `.hlsl` and `.md` file in this crate was hashed as a git
blob and compared with `git ls-tree` of zed across 2025-04 to 2026-06. The
number of byte-identical files peaks sharply at this commit:

| zed commit | date | files identical to the fork |
|---|---|---|
| `6267a147ba45` | 2025-04-16 | 21 of 145 |
| `f000dfebd2aa` | 2025-07-02 | 42 of 156 |
| `707d0e6ebd75` | 2025-09-09 | 91 of 168 |
| `677d6acc9d49` | 2025-10-13 | 117 of 169 |
| `e468edd3893c` | 2025-10-22 | 143 of 169 |
| **`08d95ad9d31f`** | **2025-10-22** | **145 of 169** |
| `738e2481096c` | 2025-10-23 | 137 of 169 |
| `6f6f652cf2b6` | 2025-11-06 | 112 of 169 |
| `42d5f7e73e85` | 2025-12-19 | 80 of 168 |

`e468edd3893c` is the parent of the base and differs from it only by the
version bump in `Cargo.toml`. The 24 files that differ at the base, plus
`examples/video_frame.rs` which exists only here, are the local patches below.
Confidence is high: the match is both the maximum and a sharp peak.

Compared against upstream tip `26363aabdb` (2026-10-13).

## Local patches

Diff against the base is roughly 3,000 lines. `Cargo.toml.orig` accounts for
326 of those and is a packaging artifact, not a change. The rest groups as:

### Video frames (~1,400 lines)

`Window::alloc_video_texture`, `Window::paint_video_frame` and
`Window::free_video_texture`, with generation-checked texture ids and a tightly
packed RGBA8 contract.

- `src/window.rs` (+189/−15)
- `src/scene.rs` (+79/−4)
- `src/platform.rs` (+25/−14)
- `src/platform/blade/blade_atlas.rs` (+232), `blade_renderer.rs` (+253), `shaders.wgsl` (+161)
- `src/platform/windows/directx_atlas.rs` (+228), `directx_renderer.rs` (+50), `shaders.hlsl` (+49)
- `src/platform/mac/metal_atlas.rs` (+11), `metal_renderer.rs` (+1) — allocation returns unsupported
- `src/platform/test/window.rs` (+14)
- `examples/video_frame.rs` (new, 117)

Status and the validation still to do are in outstanding.md.

### Backdrop blur (~230 lines)

`gpui::backdrop_blur(radius, tint)` as a `Background` variant. Blade only: a
single-pass cross-pattern blur sampled from a framebuffer snapshot.

- `src/style.rs` (+18), `src/styled.rs` (+8), `src/color.rs` (+42)
- the Blade shader and renderer changes are counted in the video-frame files above

`fluent-core`'s `surface_blur_layer` token is the consumer. Windows and macOS
are undecided; outstanding.md tracks that decision.

### SVG font database pre-warming

- `src/svg_renderer.rs` (+16/−5)

Named in the README's renderer-pragmatism note. Avoids the first SVG paying the
cost of building the font database.

### Small element and input fixes

Not renderer work, and small enough to read in full:

- `src/elements/div.rs` (+63/−55)
- `src/elements/list.rs` (+11/−9)
- `src/interactive.rs` (+4/−14)
- `src/key_dispatch.rs` (+2/−2)
- `src/tab_stop.rs` (+1/−1)
- `src/gpui.rs` (+1), `build.rs` (+1), `README.md` (+26)

## Upstream since the base, and why it is not merged here

601 commits touch `crates/gpui` between the base and `26363aabdb`. Upstream then
did two things that make a straight merge meaningless:

1. Extracted the platform traits into `crates/gpui_platform`
   (`bc31ad4a8c61`, 2026-02-19, #49277).
2. Replaced the Blade, Metal and DirectX renderers with `crates/gpui_wgpu`
   (#50228, 2026-02-26) and moved the backends to `crates/gpui_linux`,
   `crates/gpui_windows`, `crates/gpui_macos` and `crates/gpui_apple`.

`git diff --shortstat 08d95ad9..26363aabdb` is +54,985/−44,077 for `crates/gpui`
alone, and the renderer files these patches edit are deleted upstream. Both
renderer patches have to be rewritten against wgpu, not merged.

That work is tracked as child items of WI-1346:

- WI-1347 — adopt the crate split and the wgpu renderer. Blocks the other three.
- WI-1348 — re-port video frames onto wgpu, including the missing macOS path.
- WI-1349 — re-port backdrop blur onto wgpu and decide Windows/macOS support.
- WI-1350 — triage the remaining API drift against fluent-core, fluent-primitives,
  fluent-layout, fluent-ribbon and fluent-app.

## Keeping this current

When the base moves, update the commit above and re-check the patch list with a
blob-hash comparison rather than by eye. Drop a patch from this list only once
it is either upstream or deleted.
