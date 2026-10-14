# gpui API drift: base fork vs vendored snapshot

The 92 distinct `gpui` symbols imported by the fluent crates and examples
(`crates/fluent-*`, `examples/*`), compared between the base fork at
`crates/gpui` on `main` and the vendored snapshot at
`vendor/zed/crates` (zed `76b8ab814f0d`, 2026-10-14).

Symbols were collected from `use gpui::{...}` imports and `gpui::Name` paths.
A symbol counts as unchanged when the same kind of item is defined under the
same name in both trees; this is a location check, not a line-by-line
signature diff. The rows known to matter are called out under "Notes".

## Counts

| Status | Count |
|---|---|
| unchanged | 89 |
| moved | 1 |
| signature changed | 1 |
| removed | 1 |
| **total** | **92** |

## Removed

| Symbol | Used by | Replacement |
|---|---|---|
| `Corner` | fluent-layout, fluent-primitives | none. The `Corner` enum (`TopLeft`/`TopRight`/`BottomLeft`/`BottomRight`) is gone from `geometry.rs`. `gpui::Anchor` and `gpui::Corners<T>` survive and are the nearest equivalents. `fluent-layout`'s popover anchors with `Corner::TopLeft`. |

## Signature changed

| Symbol | Base | Vendored | Used by |
|---|---|---|---|
| `Application` | `Application::new()` at `app.rs:132` | `Application::new()` is gone. Construct with `gpui_platform::application() -> gpui::Application` (`gpui_platform.rs:13`). | fluent-app, screenshots |

## Moved

| Symbol | Base | Vendored | Used by |
|---|---|---|---|
| `SharedString` | `gpui/src/shared_string.rs:14` | `gpui_shared_string` crate (`gpui_shared_string.rs:15`), re-exported by `gpui`, so `use gpui::SharedString` still resolves. | demo_app, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon, form, gallery, screenshots, widgets |

## Unchanged

| Symbol | Vendored location | Used by |
|---|---|---|
| `Animation` | `gpui/src/elements/animation.rs:15` | fluent-core, fluent-primitives |
| `AnimationExt` | `gpui/src/elements/animation.rs:82` | fluent-core, fluent-primitives |
| `AnyElement` | `gpui/src/element.rs:650` | fluent-core, fluent-layout, fluent-primitives, fluent-ribbon, form, screenshots |
| `AnyView` | `gpui/src/view.rs:24` | fluent-layout |
| `App` | `gpui/src/app.rs:816` | demo_app, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon, form, screenshots, widgets |
| `AppContext` | `gpui/src/gpui.rs:222` | fluent-layout, fluent-primitives |
| `AssetSource` | `gpui/src/assets.rs:13` | fluent-app |
| `AsyncApp` | `gpui/src/app/async_context.rs:22` | fluent-layout, fluent-primitives |
| `Background` | `gpui/src/color.rs:779` | fluent-core |
| `Bounds` | `gpui/src/geometry.rs:723` | fluent-app, fluent-layout, fluent-primitives, screenshots |
| `ClickEvent` | `gpui/src/interactive.rs:290` | demo_app, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon, form, hello_world |
| `ClipboardItem` | `gpui/src/platform.rs:2969` | fluent-primitives |
| `Context` | `gpui/src/app/context.rs:21` | demo_app, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon, fluent-test-utils, form, gallery, hello_world, screenshots, widgets |
| `CursorStyle` | `gpui/src/platform.rs:2881` | fluent-app, fluent-layout, fluent-primitives |
| `Decorations` | `gpui/src/platform.rs:793` | fluent-app |
| `Div` | `gpui/src/elements/div.rs:1805` | fluent-layout, screenshots |
| `DragMoveEvent` | `gpui/src/elements/div.rs:67` | fluent-layout |
| `Element` | `gpui/src/view.rs:105` | fluent-primitives |
| `ElementId` | `gpui/src/window.rs:7591` | fluent-core, fluent-layout, fluent-primitives |
| `ElementInputHandler` | `gpui/src/input.rs:128` | fluent-primitives |
| `Entity` | `gpui/src/app/entity_map.rs:435` | demo_app, fluent-app, fluent-layout, fluent-primitives, fluent-test-utils, form, gallery, screenshots, widgets |
| `EntityInputHandler` | `gpui/src/input.rs:13` | fluent-primitives |
| `FocusHandle` | `gpui/src/window.rs:538` | fluent-layout, fluent-primitives |
| `Focusable` | `gpui/src/window.rs:712` | fluent-primitives |
| `FontFeatures` | `gpui/src/text_system/font_features.rs:8` | fluent-primitives |
| `FontWeight` | `gpui/src/text_system.rs:1128` | demo_app, fluent-layout, fluent-primitives, fluent-ribbon |
| `Global` | `gpui/src/global.rs:22` | fluent-app, fluent-core, fluent-layout |
| `GlobalElementId` | `gpui/src/element.rs:220` | fluent-primitives |
| `HitboxBehavior` | `gpui/src/window.rs:890` | fluent-app |
| `Hsla` | `gpui/src/color.rs:334` | fluent-app, fluent-core, fluent-layout, fluent-primitives |
| `InspectorElementId` | `gpui/src/inspector.rs:3` | fluent-primitives |
| `IntoElement` | `gpui/src/element.rs:146` | demo_app, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon, form, gallery, hello_world, screenshots, widgets |
| `KeyBinding` | `gpui/src/keymap/binding.rs:10` | demo_app, fluent-primitives |
| `KeyDownEvent` | `gpui/src/interactive.rs:25` | fluent-layout, fluent-primitives |
| `LayoutId` | `gpui/src/taffy.rs:385` | fluent-primitives |
| `Length` | `gpui/src/geometry.rs:3623` | fluent-layout |
| `MouseButton` | `gpui/src/interactive.rs:453` | demo_app, fluent-app, fluent-layout, fluent-primitives |
| `MouseDownEvent` | `gpui/src/interactive.rs:148` | demo_app, fluent-app, fluent-layout, fluent-primitives |
| `MouseMoveEvent` | `gpui/src/interactive.rs:494` | fluent-primitives |
| `MouseUpEvent` | `gpui/src/interactive.rs:185` | fluent-primitives |
| `ObjectFit` | `gpui/src/style.rs:29` | fluent-layout, fluent-primitives |
| `PaintQuad` | `gpui/src/window.rs:7763` | fluent-primitives |
| `Pixels` | `gpui/src/geometry.rs:2689` | fluent-app, fluent-layout, fluent-primitives |
| `Point` | `gpui/src/geometry.rs:85` | fluent-app, fluent-layout, fluent-primitives, screenshots |
| `Render` | `gpui/src/element.rs:164` | demo_app, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon, fluent-test-utils, form, gallery, hello_world, widgets |
| `RenderOnce` | `gpui/src/element.rs:180` | fluent-layout, fluent-primitives |
| `ResizeEdge` | `gpui/src/platform.rs:762` | fluent-app |
| `Result` | `gpui/src/gpui.rs:312` | fluent-app |
| `ShapedLine` | `gpui/src/text_system/line.rs:43` | fluent-primitives |
| `Size` | `gpui/src/geometry.rs:396` | fluent-app, screenshots |
| `Stateful` | `gpui/src/elements/div.rs:4067` | fluent-layout |
| `Style` | `gpui/src/style.rs:180` | fluent-primitives |
| `Styled` | `gpui/src/styled.rs:22` | fluent-primitives |
| `TestAppContext` | `gpui/src/app/test_context.rs:21` | fluent-layout, fluent-primitives, fluent-test-utils |
| `TextRun` | `gpui/src/text_system.rs:1228` | fluent-primitives |
| `TitlebarOptions` | `gpui/src/platform.rs:2692` | fluent-app, screenshots |
| `Transformation` | `gpui/src/elements/svg.rs:214` | fluent-primitives |
| `UTF16Selection` | `gpui/src/platform.rs:2227` | fluent-primitives |
| `UnderlineStyle` | `gpui/src/style.rs:828` | fluent-primitives |
| `VisualTestContext` | `gpui/src/app/test_context.rs:857` | fluent-test-utils |
| `WeakEntity` | `gpui/src/app/entity_map.rs:765` | fluent-layout, fluent-primitives |
| `Window` | `gpui/src/window.rs:1216` | demo_app, fluent-app, fluent-layout, fluent-primitives, fluent-ribbon, fluent-test-utils, form, gallery, hello_world, screenshots, widgets |
| `WindowBounds` | `gpui/src/platform.rs:2629` | fluent-app, screenshots |
| `WindowDecorations` | `gpui/src/platform.rs:783` | fluent-app, screenshots |
| `WindowOptions` | `gpui/src/platform.rs:2491` | fluent-app, screenshots |
| `actions` | `gpui/src/action.rs:24` | demo_app, fluent-primitives |
| `anchored` | `gpui/src/elements/anchored.rs:27` | fluent-layout, fluent-primitives |
| `canvas` | `gpui/src/elements/canvas.rs:10` | fluent-app, fluent-layout, fluent-primitives |
| `deferred` | `gpui/src/elements/deferred.rs:7` | fluent-layout, fluent-primitives |
| `div` | `gpui/src/geometry.rs:269` | demo_app, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon, form, gallery, hello_world, screenshots, widgets |
| `ease_out_quint` | `gpui/src/elements/animation.rs:526` | fluent-core |
| `fill` | `gpui/src/window.rs:7832` | fluent-primitives |
| `hsla` | `gpui/src/color.rs:424` | fluent-core |
| `img` | `gpui/src/elements/img.rs:201` | fluent-layout, fluent-primitives |
| `linear_color_stop` | `gpui/src/color.rs:893` | fluent-core |
| `linear_gradient` | `gpui/src/color.rs:865` | fluent-core |
| `percentage` | `gpui/src/geometry.rs:2636` | fluent-primitives |
| `point` | `gpui/src/geometry.rs` (`pub const fn point`) | fluent-app, fluent-layout, fluent-primitives |
| `prelude` | `gpui/src/gpui.rs:36` | demo_app, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon, form, gallery, hello_world, screenshots, widgets |
| `pulsating_between` | `gpui/src/elements/animation.rs:542` | fluent-primitives |
| `px` | `gpui/src/geometry.rs:3748` | demo_app, fluent-app, fluent-core, fluent-layout, fluent-primitives, fluent-ribbon, form, hello_world, screenshots, widgets |
| `relative` | `gpui/src/geometry.rs:3717` | fluent-layout, fluent-primitives |
| `rgb` | `gpui/src/color.rs:14` | demo_app, fluent-app, fluent-core |
| `rgba` | `gpui/src/color.rs:20` | fluent-primitives |
| `size` | `gpui/src/assets.rs:79` | fluent-app, fluent-primitives |
| `svg` | `gpui/src/elements/mod.rs:25` | fluent-app, fluent-layout, fluent-primitives, fluent-ribbon |
| `test` | `gpui/src/platform.rs:174` | fluent-layout, fluent-primitives |
| `uniform_list` | `gpui/src/elements/uniform_list.rs:22` | fluent-layout |
| `white` | `gpui/src/color.rs:474` | fluent-primitives |

## Notes

- `hsla` changed from `pub fn` to `pub const fn` with the same four `f32`
  arguments. Call sites are unaffected; it stays in "unchanged".
- `Window::start_window_resize(&self, edge: ResizeEdge)` and
  `Window::window_decorations(&self) -> Decorations` keep their signatures.
  `Decorations`, `ResizeEdge` and `HitboxBehavior` all still exist.
- `text_system().shape_line` and `ShapedLine` both still exist
  (`text_system/line.rs`). `EntityInputHandler` and `UTF16Selection` still
  exist (`input.rs`, `platform.rs`). `uniform_list` still exists.
- `TestAppContext` and `VisualTestContext` still exist in `app/test_context.rs`.
- taffy moves from `=0.9.0` (base) to `=0.13.0` (vendored). That is a
  behavioural change in layout, not an API change for these symbols.
- `Window::alloc_video_texture`, `paint_video_frame`, `free_video_texture` and
  `VideoTextureId` exist only in the base fork. They are not among the 92
  symbols the fluent crates import; re-applying them is WI-1348.

## Port status

| Crate | Item | Result |
|---|---|---|
| `fluent-core` | WI-1425 | No source change. Every symbol it imports is in "unchanged" above, and `cargo check`, `cargo test` (22 unit tests) and `cargo clippy --no-deps -D warnings` pass against `vendor/zed` with the public API untouched. |
| `fluent-primitives` | WI-1426 | Compiles, 27 unit tests pass, `cargo clippy --all-targets --no-deps -D warnings` is clean. The location check above missed five signature changes, all forced by the vendored gpui and each a mechanical adaptation: `Window::focus`, `focus_next` and `focus_prev` now take `&mut App`; `ShapedLine::paint` takes a `TextAlign` and an optional align width (passed `Left` and `None`, the previous implicit behaviour); `Styled::text_style()` returns the refinement directly instead of an `Option`; `ElementId::NamedChild` holds an `Arc` instead of a `Box`; and `Corner` is gone, replaced by `gpui::Anchor`. The crate re-exports its own four-variant `Corner` that converts into `Anchor`, so the popover anchor API the later crates use does not change. |
| `fluent-layout` | WI-1428 | Compiles, 37 unit tests pass, `cargo clippy --all-targets --no-deps -D warnings` is clean. Three changes, all forced by the vendored gpui: `Window::focus` takes `&mut App`; `AsyncApp::update` returns its value directly instead of a `Result`, so the toast timeout's `.ok()` is gone; and `gpui::Corner` is gone. `Popover::anchor` now takes `gpui::Anchor` instead of `Corner`, and the menu button anchors its popup at `Anchor::TopLeft`. |
| `fluent-ribbon` | WI-1427 | No source change. Every gpui symbol it uses (`App`, `AnyElement`, `ClickEvent`, `Context`, `FontWeight`, `IntoElement`, `Render`, `SharedString`, `Window`, `div`, `prelude`, `px`, `svg`) is in "unchanged" above, and `cargo check`, `cargo test` (no unit tests; one ignored doc-test) and `cargo clippy --all-targets --no-deps -D warnings` pass against `vendor/zed` with the public API untouched. |
