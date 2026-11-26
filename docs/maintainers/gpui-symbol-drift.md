# gpui symbol drift: fluent crates and examples vs vendored gpui

Phase 0 groundwork for WI-1350 (fluent-gpui: adopt upstream gpui). Docs only.

- Vendored zed rev: `ba63a022c9a22ca66d68ef2a8f0bb9b5b0a9984d` (`vendor/zed/UPSTREAM.md`)
- Date: 2026-11-01
- Method: scan `crates/**/*.rs` and `examples/**/*.rs` for `gpui::X` paths and `use gpui::{...}` imports (the `git grep -ho gpui::` alone misses braced imports); check each name for a `pub` definition under `vendor/zed/crates/`.
- Distinct symbols: 91
- Status counts:
  - defined: 84
  - moved: 1
  - re-export: 5
  - module: 1
  - defined elsewhere: 0
  - not found: 0

Status meanings: `defined` = `pub` item found in the `gpui` crate source; `moved` = still in `gpui` but the entry point changed; `re-export` = reached through a `pub use` chain, no direct definition; `defined elsewhere` = lives in another vendored crate; `not found` = no match, needs a human check.

| Symbol | Status | Vendored location (rev above) | Uses | Crates using it |
|---|---|---|---|---|
| `actions` | defined | `gpui/src/action.rs`, `gpui/src/profiler.rs` | 2 | examples/demo_app, fluent-primitives |
| `Anchor` | defined | `gpui/src/geometry.rs`, `gpui/src/platform/layer_shell.rs` | 6 | fluent-layout, fluent-primitives |
| `anchored` | defined | `gpui/src/elements/anchored.rs`, `gpui/src/elements/mod.rs` | 8 | fluent-layout, fluent-primitives |
| `Animation` | defined | `gpui/src/elements/animation.rs` | 4 | fluent-core, fluent-primitives |
| `AnimationExt` | defined | `gpui/src/elements/animation.rs` | 4 | fluent-core, fluent-primitives |
| `AnyElement` | defined | `gpui/src/element.rs` | 19 | examples/form, examples/screenshots, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon |
| `AnyView` | defined | `gpui/src/view.rs` | 5 | fluent-layout |
| `App` | defined | `gpui/src/app.rs` | 57 | examples/demo_app, examples/form, examples/screenshots, examples/widgets, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon |
| `AssetSource` | defined | `gpui/src/assets.rs` | 2 | fluent-app |
| `AsyncApp` | defined | `gpui/src/app/async_context.rs` | 6 | fluent-layout, fluent-primitives |
| `Background` | defined | `gpui/src/color.rs` | 1 | fluent-core |
| `Bounds` | defined | `gpui/src/geometry.rs` | 10 | examples/screenshots, fluent-app, fluent-layout, fluent-primitives |
| `canvas` | defined | `gpui/src/elements/canvas.rs`, `gpui/src/elements/mod.rs` | 7 | fluent-app, fluent-layout, fluent-primitives |
| `ClickEvent` | defined | `gpui/src/interactive.rs` | 33 | examples/demo_app, examples/form, examples/hello_world, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon |
| `ClipboardItem` | defined | `gpui/src/platform.rs` | 2 | fluent-primitives |
| `Context` | defined | `gpui/src/app/context.rs` | 32 | examples/demo_app, examples/form, examples/gallery, examples/hello_world, examples/screenshots, examples/widgets, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon, fluent-test-utils |
| `CursorStyle` | defined | `gpui/src/platform.rs` | 9 | fluent-app, fluent-layout, fluent-primitives |
| `Decorations` | defined | `gpui/src/platform.rs` | 1 | fluent-app |
| `deferred` | defined | `gpui/src/elements/deferred.rs`, `gpui/src/elements/mod.rs` | 10 | fluent-layout, fluent-primitives |
| `div` | defined | `gpui/src/elements/div.rs`, `gpui/src/elements/mod.rs` | 59 | examples/demo_app, examples/form, examples/gallery, examples/hello_world, examples/screenshots, examples/widgets, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon |
| `Div` | defined | `gpui/src/elements/div.rs` | 3 | examples/screenshots, fluent-layout |
| `DragMoveEvent` | defined | `gpui/src/elements/div.rs` | 2 | fluent-layout |
| `ease_out_quint` | defined | `gpui/src/elements/animation.rs` | 1 | fluent-core |
| `Element` | defined | `gpui/src/element.rs`, `gpui/src/elements/anchored.rs` | 1 | fluent-primitives |
| `ElementId` | defined | `gpui/src/window.rs` | 17 | fluent-core, fluent-layout, fluent-primitives |
| `ElementInputHandler` | defined | `gpui/src/input.rs` | 1 | fluent-primitives |
| `Entity` | defined | `gpui/src/app/entity_map.rs` | 12 | examples/demo_app, examples/form, examples/gallery, examples/screenshots, examples/widgets, fluent-app, fluent-layout, fluent-primitives, fluent-test-utils |
| `EntityInputHandler` | defined | `gpui/src/input.rs` | 1 | fluent-primitives |
| `fill` | defined | `gpui/src/path_builder.rs`, `gpui/src/window.rs` | 1 | fluent-primitives |
| `Focusable` | defined | `gpui/src/window.rs` | 6 | fluent-primitives |
| `FocusHandle` | defined | `gpui/src/window.rs` | 10 | fluent-layout, fluent-primitives |
| `FontFeatures` | defined | `gpui/src/text_system/font_features.rs` | 1 | fluent-primitives |
| `FontWeight` | defined | `gpui/src/text_system.rs` | 18 | examples/demo_app, fluent-layout, fluent-primitives, fluent-ribbon |
| `Global` | defined | `gpui/src/global.rs` | 4 | fluent-app, fluent-core, fluent-layout |
| `GlobalElementId` | defined | `gpui/src/element.rs` | 1 | fluent-primitives |
| `HitboxBehavior` | defined | `gpui/src/window.rs` | 1 | fluent-app |
| `hsla` | defined | `gpui/src/color.rs` | 2 | fluent-core |
| `Hsla` | defined | `gpui/src/color.rs` | 9 | fluent-app, fluent-core, fluent-layout, fluent-primitives |
| `img` | defined | `gpui/src/elements/img.rs`, `gpui/src/elements/mod.rs` | 2 | fluent-layout, fluent-primitives |
| `InspectorElementId` | defined | `gpui/src/inspector.rs` | 1 | fluent-primitives |
| `IntoElement` | re-export | trait in `gpui/src/element.rs`; the derive is re-exported from `gpui_macros` via `gpui/src/gpui.rs:127` | 61 | examples/demo_app, examples/form, examples/gallery, examples/hello_world, examples/screenshots, examples/widgets, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon |
| `KeyBinding` | defined | `gpui/src/keymap/binding.rs` | 2 | examples/demo_app, fluent-primitives |
| `KeyDownEvent` | defined | `gpui/src/interactive.rs` | 14 | fluent-layout, fluent-primitives |
| `LayoutId` | re-export | `pub use taffy::{AvailableSpace, LayoutId}` in `gpui/src/gpui.rs`; defined in `gpui/src/taffy.rs` | 1 | fluent-primitives |
| `Length` | defined | `gpui/src/geometry.rs` | 1 | fluent-layout |
| `linear_color_stop` | defined | `gpui/src/color.rs` | 1 | fluent-core |
| `linear_gradient` | defined | `gpui/src/color.rs` | 1 | fluent-core |
| `MouseButton` | defined | `gpui/src/interactive.rs` | 18 | examples/demo_app, fluent-app, fluent-layout, fluent-primitives |
| `MouseDownEvent` | defined | `gpui/src/interactive.rs` | 13 | examples/demo_app, fluent-app, fluent-layout, fluent-primitives |
| `MouseMoveEvent` | defined | `gpui/src/interactive.rs` | 1 | fluent-primitives |
| `MouseUpEvent` | defined | `gpui/src/interactive.rs` | 1 | fluent-primitives |
| `ObjectFit` | defined | `gpui/src/style.rs` | 2 | fluent-layout, fluent-primitives |
| `PaintQuad` | defined | `gpui/src/window.rs` | 1 | fluent-primitives |
| `percentage` | defined | `gpui/src/geometry.rs` | 1 | fluent-primitives |
| `Pixels` | defined | `gpui/src/geometry.rs` | 11 | fluent-app, fluent-layout, fluent-primitives |
| `Point` | defined | `gpui/src/geometry.rs` | 11 | examples/screenshots, fluent-app, fluent-layout, fluent-primitives |
| `point` | defined | `gpui/src/geometry.rs` | 9 | fluent-app, fluent-layout, fluent-primitives |
| `prelude` | module | `gpui/src/prelude.rs` (module path `gpui::prelude`) | 62 | examples/demo_app, examples/form, examples/gallery, examples/hello_world, examples/screenshots, examples/widgets, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon |
| `pulsating_between` | defined | `gpui/src/elements/animation.rs` | 2 | fluent-primitives |
| `px` | defined | `gpui/src/geometry.rs` | 59 | examples/demo_app, examples/form, examples/hello_world, examples/screenshots, examples/widgets, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon |
| `relative` | defined | `gpui/src/geometry.rs` | 4 | fluent-layout, fluent-primitives |
| `Render` | re-export | trait in `gpui/src/element.rs`; the `Render` derive is re-exported from `gpui_macros` via `gpui/src/gpui.rs:127` | 31 | examples/demo_app, examples/form, examples/gallery, examples/hello_world, examples/widgets, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon, fluent-test-utils |
| `RenderOnce` | defined | `gpui/src/element.rs` | 31 | fluent-layout, fluent-primitives |
| `ResizeEdge` | defined | `gpui/src/platform.rs` | 1 | fluent-app |
| `Result` | re-export | `pub use anyhow::Result` in `gpui/src/gpui.rs` | 1 | fluent-app |
| `rgb` | defined | `gpui/src/color.rs` | 3 | examples/demo_app, fluent-app, fluent-core |
| `rgba` | defined | `gpui/src/color.rs` | 1 | fluent-primitives |
| `ShapedLine` | defined | `gpui/src/text_system/line.rs` | 1 | fluent-primitives |
| `SharedString` | moved | `gpui_shared_string/gpui_shared_string.rs` (in the separate `gpui_shared_string` crate; re-exported via `pub use gpui_shared_string::*` in `gpui/src/gpui.rs`) | 52 | examples/demo_app, examples/form, examples/gallery, examples/screenshots, examples/widgets, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon |
| `size` | defined | `gpui/src/assets.rs`, `gpui/src/elements/list.rs` | 2 | fluent-app, fluent-primitives |
| `Size` | defined | `gpui/src/geometry.rs` | 2 | examples/screenshots, fluent-app |
| `Stateful` | defined | `gpui/src/elements/div.rs` | 1 | fluent-layout |
| `Style` | defined | `gpui/src/style.rs` | 1 | fluent-primitives |
| `Styled` | defined | `gpui/src/styled.rs` | 1 | fluent-primitives |
| `svg` | defined | `gpui/src/elements/mod.rs`, `gpui/src/elements/svg.rs` | 26 | fluent-app, fluent-layout, fluent-primitives, fluent-ribbon |
| `test` | re-export | `#[gpui::test]` attribute: proc macro `test` in `vendor/zed/crates/gpui_macros/src/gpui_macros.rs:189`, re-exported from `gpui/src/gpui.rs:127` | 5 | fluent-layout, fluent-primitives |
| `TestAppContext` | defined | `gpui/src/app/test_context.rs` (the brief names `TestAppContext`/`VisualTestContext` as changed; confirm in the per-crate ports) | 6 | fluent-layout, fluent-primitives, fluent-test-utils |
| `TextAlign` | defined | `gpui/src/style.rs` | 1 | fluent-primitives |
| `TextRun` | defined | `gpui/src/text_system.rs` | 1 | fluent-primitives |
| `TitlebarOptions` | defined | `gpui/src/platform.rs` | 2 | examples/screenshots, fluent-app |
| `Transformation` | defined | `gpui/src/elements/svg.rs` | 1 | fluent-primitives |
| `UnderlineStyle` | defined | `gpui/src/style.rs` | 1 | fluent-primitives |
| `uniform_list` | defined | `gpui/src/elements/uniform_list.rs`, re-exported via `gpui/src/elements/mod.rs` | 1 | fluent-layout |
| `UTF16Selection` | defined | `gpui/src/platform.rs` | 1 | fluent-primitives |
| `VisualTestContext` | defined | `gpui/src/app/test_context.rs` | 1 | fluent-test-utils |
| `WeakEntity` | defined | `gpui/src/app/entity_map.rs` | 6 | fluent-layout, fluent-primitives |
| `white` | defined | `gpui/src/color.rs` | 2 | fluent-primitives |
| `Window` | defined | `gpui/src/window.rs` | 64 | examples/demo_app, examples/form, examples/gallery, examples/hello_world, examples/screenshots, examples/widgets, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon, fluent-test-utils |
| `WindowBounds` | defined | `gpui/src/platform.rs` | 2 | examples/screenshots, fluent-app |
| `WindowDecorations` | defined | `gpui/src/platform.rs` | 2 | examples/screenshots, fluent-app |
| `WindowOptions` | defined | `gpui/src/platform.rs` | 2 | examples/screenshots, fluent-app |

## Notes

- `Application::new()` does not exist; use `gpui_platform::application()` (used in `crates/fluent-app/src/app_builder.rs:162` and `examples/screenshots/src/main.rs:577`).
- `Corner` is not a `gpui` symbol in this rev; `fluent-primitives` defines its own `Corner` for button rounding.
- `taffy` moves from 0.10 to 0.13 upstream: an API-behaviour change to check in the layout port, not a rename in this table.
- `test`, `Render`, `IntoElement` are the macro-crate re-exports; the `Render` and `IntoElement` traits are still `gpui` items.
- Counts are a symbol-name scan, not a type-check. Items reported as `defined` are not yet checked for signature changes; the signature diff is a separate step.
