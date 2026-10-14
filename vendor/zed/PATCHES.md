# Local patches

Each local patch to this tree is a commit on the integrating branch, never on `zed-vendor`. List them here so a re-vendor only needs conflicts resolved in these files.

| patch | commit | files |
|---|---|---|
| `assets/fonts` | this commit | `vendor/zed/assets/fonts/` |
| SVG font-DB pre-warm | this commit | `vendor/zed/crates/gpui/src/svg_renderer.rs` |

## Dropped: upstream already has them

Triage against the vendored snapshot (WI-1432, 2026-10-14). Nothing to re-apply:

| fork change | verdict |
|---|---|
| `elements/list.rs` — `if let Some(handler)` instead of `unwrap()` on the scroll handler | upstream has it |
| `interactive.rs` — `#[derive(Default)]` for `MouseButton`, `NavigationDirection` | upstream has it |
| `key_dispatch.rs` — `if let Some(view_id)` push | upstream has it |
| `tab_stop.rs` — array literal in a test | upstream has it |
| `elements/div.rs` — let-chain tidy in `paint_debug_info` | upstream has it |

The SVG pre-warm keeps upstream's per-renderer lazily enriched font DB. Only the system font database (`load_system_fonts`) is hoisted to a module `LazyLock`, with `pub fn warmup_svg_font_db()` so `main()` can prime it on a background thread. `gpui.rs` already re-exports `svg_renderer::*`, so the function is public without a new `use`.
