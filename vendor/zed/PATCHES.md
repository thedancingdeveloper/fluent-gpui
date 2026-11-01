# Local patches

Each local patch to this tree is a land on the working line (`backlog-1`), never on the `zed-vendor` line. List them here so a re-vendor only needs conflicts resolved in these files.

| patch | commit | files |
|---|---|---|
| `trim-ios` (WI-2728): drop the `target_os = "ios"` arm of `current_platform`, since `gpui_ios` is no longer vendored | WI-2728 land on `backlog-1` | `crates/gpui_platform/src/gpui_platform.rs` |
| `svg-font-warmup` (WI-1432): module-level `SYSTEM_FONT_DB` plus `pub fn warmup_svg_font_db()` behind the `svg-font-warmup` feature (enabled by `fluent-core`), so `main()` can prime `load_system_fonts()` on a background thread | WI-1432 land on `backlog-1` | `crates/gpui/src/svg_renderer.rs`, `crates/gpui/Cargo.toml` |
| `element-scale` (WI-1433): `Style::scale` and `Styled::scale`, `Window::with_element_scale` with `element_scale_stack` and the paint-time transforms (`transform_bounds_for_paint`, `scale_length_for_paint`, `scale_corners_for_paint`, `transform_point_for_paint`) applied to quads, shadows, paths, text, svg, images and surfaces; the `div.rs` paint wrapper; example `examples/scale.rs` | WI-1433 land on `backlog-1` | `crates/gpui/src/style.rs`, `crates/gpui/src/styled.rs`, `crates/gpui/src/window.rs`, `crates/gpui/src/elements/div.rs`, `crates/gpui/Cargo.toml`, `crates/gpui/examples/scale.rs` |

Dropped from the WI-1432 triage, upstream already has them (nothing applied): `elements/list.rs`, `interactive.rs`, `key_dispatch.rs`, `tab_stop.rs`, and the let-chain tidy in `elements/div.rs`. Upstream also has no renderer-time font-DB warm-up, so `svg-font-warmup` is still needed.

`gpui_ios` and the `runtime_shaders` feature are removed by `scripts/vendor-zed.py` itself (`EXCLUDED`, `DROPPED_FEATURES`), so they are not patches. A re-vendor keeps that output and then reapplies `trim-ios`.
