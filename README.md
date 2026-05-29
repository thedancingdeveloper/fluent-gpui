# FluentGUI

A Rust desktop GUI framework built on a vendored [GPUI](https://gpui.rs) fork with a Fluent 2-inspired design system, app chrome, command surfaces, and reusable application layout primitives.

[![Crates.io](https://img.shields.io/crates/v/fluent-app)](https://crates.io/crates/fluent-app)
[![License: Apache 2.0](https://img.shields.io/badge/license-Apache--2.0-blue)](LICENSE)

---

## Overview

FluentGUI provides the pieces needed to build polished Rust desktop applications on top of GPUI: token-driven theming, Fluent-style widgets, a ribbon command model, menus, dock panels, modal/notification hosts, and a `FluentApp` entry point that installs client-side window chrome.

The current workspace release is **v2.4.0**. It includes a vendored GPUI **0.2.5** fork in `crates/gpui`, used by every workspace crate through the root `Cargo.toml`.

**What it is:**
- A Fluent 2-inspired application shell for GPUI desktop apps.
- A layered set of crates: tokens, primitives, ribbon, layout, and app bootstrap.
- A dogfooded framework shaped by real connection-manager and AndroidConnect-style application needs.
- An Apache-2.0 codebase suitable for proprietary and open-source applications.

**What it is not:**
- A web renderer wrapper.
- A CSS/layout engine replacement.
- A WinUI/XAML port.
- A complete GPUI upstream replacement.

---

## Getting Started

Add the crates you need to your `Cargo.toml`:

```toml
[dependencies]
gpui = "0.2"
fluent-app = "2.3.0"         # application entry point and window chrome
fluent-layout = "2.2.0"      # Workspace and application layout surfaces
fluent-ribbon = "2.0.2"      # RibbonBar and command groups
fluent-primitives = "2.4.0"  # base widgets
fluent-core = "2.4.0"        # theme, tokens, motion helpers
```

When working inside this repository, `gpui` is resolved from `crates/gpui` through the workspace dependency:

```toml
[workspace.dependencies]
gpui = { path = "crates/gpui", version = "0.2" }
```

A minimal application:

```rust
use fluent_app::FluentApp;
use fluent_layout::Workspace;

fn main() {
    FluentApp::new("My App")
        .app_id("org.example.MyApp")
        .dark_theme()
        .window_min_size(800.0, 600.0)
        .run(|cx| cx.new(|cx| Workspace::new(cx)));
}
```

`FluentApp::run` initializes `Theme`, `ModalStack`, `ToastStack`, and the global `WindowTitle`, opens a frameless GPUI window, wraps the root in resize-aware client chrome, and installs the bundled Fluent asset source. Use `WindowTitle::set_title` and `WindowTitle::set_status` from app code to keep platform chrome and Fluent title bars aligned with application state.

---

## Crates

| Crate | Current workspace version | Description |
|-------|---------------------------|-------------|
| [`fluent-core`](crates/fluent-core) | `2.4.0` | Design tokens, semantic colors, typography, motion helpers, deterministic swatches, tint helpers, Midnight/Ember schemes, `AsyncResource`, and the reactive `Theme` global |
| [`fluent-primitives`](crates/fluent-primitives) | `2.4.0` | Buttons, inputs, dropdowns, comboboxes, checkboxes, avatars, badges, cards, skeletons, `TokenInput`, `SuggestionPopover`, `DateTimePicker`, text helpers, and more |
| [`fluent-ribbon`](crates/fluent-ribbon) | `2.0.2` | `RibbonBar` with tabs, groups, large/compact/toggle/stack buttons, contextual tabs, and overflow |
| [`fluent-layout`](crates/fluent-layout) | `2.2.0` | `Workspace`, `MenuBar`, `ContextMenu`, `DockPanel`, `Pane`, `PaneGroup` (constrained splits), `Tree`, `DataTable`, `SelectableList`, `DocumentViewer`, `Dialog`, `Toast`, command palette, and overlays |
| [`fluent-app`](crates/fluent-app) | `2.3.0` | `FluentApp` (lifecycle hooks), `TitleBar`, `WindowTitle`, bundled assets, client-side resize chrome, and global app initialization |
| [`gpui`](crates/gpui) | `0.2.5` | Vendored Apache-2.0 GPUI fork with FluentGUI-required renderer fixes, SVG font-DB pre-warming, and additions |

Dependency graph:

```text
fluent-app
  -> fluent-layout
       -> fluent-ribbon
            -> fluent-primitives
                 -> fluent-core
                      -> gpui
```

Each layer only depends on layers below it. You can use `fluent-primitives` without the ribbon or full application shell.

---

## Component Inventory

### fluent-core

- `Theme` global with `apply_dark()`, `apply_light()`, `toggle()`, and per-entity reactivity through `cx.observe_global::<Theme>()`.
- `ColorScheme` with Fluent-style fill, foreground, stroke, status, ribbon, panel, tab, and `surface_blur_layer` tokens.
- `SpacingTokens`, `RadiiTokens`, `TypographyTokens`, `ComponentTokens`, and `MotionTokens`.
- `tint()`, `with_alpha()`, `hue_for()`, and `gradient_from_hue()` helpers for application-specific badges, avatars, and tinted surfaces.
- `AsyncResource` models `Idle`/`Loading`/`Ready`/`Error` request state with
  generation tokens so stale async responses can be ignored while cached values
  remain available during refreshes.

### fluent-primitives

| Widget | Notes |
|--------|-------|
| `Button`, `IconButton`, `ToggleButton` | Accent, neutral, subtle, and hyperlink appearances; compact/normal sizing |
| `TextInput`, `Textarea`, `Searchbox` | GPUI input integration; `TextInput::on_submit` handles Enter/Return and can clear after submit |
| `TokenInput` | Multi-value chip input with freeform tokens, filtered anchored suggestions, and keyboard selection |
| `SuggestionPopover` | Reusable anchored listbox surface and keyboard roving state for search/autocomplete suggestions |
| `DateTimePicker` | Local wall-clock date/time selection with validation, segment keyboard adjustment, and `YYYY-MM-DDTHH:MM` parsing/formatting |
| `Field` | Label, description, and validation wrapper for form controls |
| `Dropdown`, `Combobox` | Entity-backed select/search controls with anchored option popovers |
| `Checkbox`, `Switch`, `RadioGroup` | Standard selection primitives |
| `Tooltip` | Hover-triggered surface using `surface_blur_layer` |
| `Spinner`, `ProgressBar`, `Skeleton`, `SkeletonRow`, `Skeleton::wrap` | Loading and first-paint placeholders |
| `Card`, `SectionHeader` | Canonical dashboard/card composition helpers |
| `Avatar`, `AppDot`, `ConnectionBadge`, `Chip`, `Badge`, `Icon`, `Label`, `Divider` | Display primitives and status surfaces |
| `FluentTextExt` | Typography helpers such as `.tabular_nums()` |

### fluent-ribbon

- `RibbonBar` with tab strip and command content row.
- `RibbonTabBuilder` and `RibbonGroupBuilder` for declarative construction.
- Item types: large button, compact button, icon button, toggle button, stack, separator.
- Contextual tabs and overflow collapse into a compact "More" dropdown.

### fluent-layout

**Application shell**
- `Workspace` composes menu bar, ribbon, dock panels, pane area, and modal host.
- `MenuBar` supports application menus, shortcut labels, click-away close, and keyboard roving.
- `DockPanel`, `Pane`, `PaneGroup`, and `TabStrip` provide resizable app layouts.
- `PaneGroup`/`ResizablePanelGroup` supports horizontal or vertical two-panel
  splits, per-panel min/max pixel constraints, draggable dividers, and an
  `on_resize` ratio callback so applications can persist restored widths or
  heights without framework-specific state.
- `SelectableList` provides lazy uniform-row rendering, single selection,
  keyboard roving, activation/context-menu callbacks, and loading/empty/error
  states for app-owned list data.
- `DocumentViewer` renders a safe rich-document model with lossy HTML/plain-text
  conversion helpers for headings, paragraphs, links, code, images, and tables.

**Menus and overlays**
- `ContextMenu` supports icons, separators, disabled rows, check/radio rows, shortcuts, and cascading submenus.
- `ModalStack`, `ModalHost`, `Dialog`, `ConfirmModal`, and `Popover` handle overlay flows.
- `CommandPalette` provides a Ctrl+K-style grouped fuzzy command launcher with
  hints, disabled rows, and entry-owned action callbacks.

**Data and feedback**
- `Tree`, `DataTable`, `Toolbar`, `MessageBar`, `Toast`, and `ToastHost`.
- `SettingsNav` for sectioned settings/data-entry dialogs.

### fluent-app

- `FluentApp` builder with `.window_size()`, `.dark_theme()`, `.light_theme()`, and `.assets()`.
- `TitleBar` with optional leading icon, client-side drag, double-click maximize/restore, and min/max/close controls.
- `WindowChrome` resize-edge wrapper for frameless client-decorated windows.
- `FluentAssets` built-in icon set with app asset fallback before generic icon fallback.

---

## Vendored GPUI Fork

This repository vendors GPUI at `crates/gpui` rather than relying directly on a registry copy during local development. The fork is Apache-2.0 and is currently versioned as `0.2.5`.

Important local additions:
- `gpui::backdrop_blur(radius, tint)` background support on the Blade/WGSL renderer path.
- Dedicated `VideoTextureId` allocation, upload, paint, and free APIs for RGBA8 video-frame streaming.
- Blade and Windows/DirectX video-frame renderer paths, with macOS/Metal still unsupported.
- Renderer fixes around texture lifetime, stale video ids, and GPUI 0.2 version compatibility.
- Pre-warmed `usvg` font database so the first SVG render does not block the UI thread.
- Reqwest-using GPUI examples are feature-gated so the workspace builds cleanly by default.

See [outstanding.md](outstanding.md) for remaining validation and backend work.

---

## Examples

| Example | What it shows |
|---------|---------------|
| `hello_world` | Minimal `FluentApp` window |
| `widgets` | Primitive widget gallery |
| `form` | Form layout with `Field`, validation states, `SettingsNav`, and a `Dialog` |
| `demo_app` | Full shell: `Workspace`, `MenuBar`, `RibbonBar`, `DockPanel`, `Tree`, `TabStrip`, `ContextMenu`, `CommandPalette`, modals, toasts, and theme switching |
| `gallery` | Scrollable overview of framework components |
| `gpui` example `video_frame` | Low-level vendored-GPUI validation example for streaming RGBA video frames |

Run examples from the workspace root:

```sh
cargo run -p hello_world
cargo run -p widgets
cargo run -p form
cargo run -p demo_app
cargo run -p gallery
cargo run -p gpui --example video_frame
```

---

## Requirements

| Requirement | Version / status |
|-------------|------------------|
| Rust | MSRV documented as **1.88**; CI currently runs on `rust:1.94-bookworm` |
| GPUI | Workspace-vendored `crates/gpui` **0.2.4**, dependency range `0.2` |
| Platforms | Linux (X11/Wayland), macOS, Windows following the vendored GPUI platform backends |

Linux system dependencies:

```sh
# Debian / Ubuntu
sudo apt-get install libxkbcommon-dev libxkbcommon-x11-dev
```

---

## Building and Testing

```sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo build --workspace
```

For lower-level GPUI video-frame validation:

```sh
cargo test -p gpui video_
cargo check -p gpui --target x86_64-pc-windows-gnu
cargo test -p gpui --target x86_64-pc-windows-gnu video_ --no-run
```

---

## Design Principles

1. **Token-driven theming** - colors, spacing, radii, type, component density, and motion come from `fluent-core`.
2. **Reactive by construction** - entity-backed views that read theme data subscribe to `Theme`.
3. **Layered APIs** - apps can use small primitives directly or opt into the full shell.
4. **Ribbon-first commands** - `RibbonBar`, `MenuBar`, and `CommandPalette` are first-class command surfaces.
5. **Renderer pragmatism** - FluentGUI owns the GPUI fork where app requirements need renderer fixes before upstream catches up.
6. **Apache-2.0 throughout** - no GPL dependencies or copied GPL source.

---

## Documentation

- [CHANGELOG.md](CHANGELOG.md) - per-crate release notes and workspace release groupings.
- [docs/VISUAL_TARGETS.md](docs/VISUAL_TARGETS.md) - visual reference notes and current fulfillment status.
- [outstanding.md](outstanding.md) - remaining GPUI renderer/video-frame validation work.
- [CLAUDE.md](CLAUDE.md) - repo-local operating notes for AI agents and maintainers.

---

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

FluentGUI does not incorporate GPL-licensed code. The Fluent 2 color token structure was adapted from [aernom/fluent-ui-gpui](https://github.com/aernom/fluent-ui-gpui) (MIT).
