# fluent-gpui

[![CI](https://github.com/thedancingdeveloper/fluent-gpui/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/thedancingdeveloper/fluent-gpui/actions/workflows/ci.yml)
[![Latest release](https://img.shields.io/github/v/release/thedancingdeveloper/fluent-gpui)](https://github.com/thedancingdeveloper/fluent-gpui/releases/latest)
[![Licence: Apache-2.0](https://img.shields.io/badge/licence-Apache--2.0-blue)](LICENSE)

**A Fluent 2 design system for native Windows and Linux desktop apps, built on [GPUI](https://gpui.rs), the GPU UI framework from Zed.**

<picture>
  <source media="(prefers-color-scheme: dark)" srcset="docs/images/gallery/hero-dark.png">
  <source media="(prefers-color-scheme: light)" srcset="docs/images/gallery/hero-light.png">
  <img src="docs/images/gallery/hero-light.png" width="800" alt="The fluent-gpui component gallery with the Fluent title bar, a navigation sidebar and a page of controls">
</picture>

## What is it

fluent-gpui gives GPUI applications a complete, themed set of desktop controls
and application chrome in the style of Fluent 2: buttons, inputs, menus,
dialogs, trees, data tables, a ribbon command bar and a custom title bar. You
write ordinary GPUI views; the `fluent-*` crates supply the look, the tokens
and the behaviour. It is aimed at productivity-style desktop apps, not at the
web.

## Highlights

- **Fluent 2 look on GPUI.** Light, dark, high-contrast and right-to-left
  variants of every control, driven by design tokens in `fluent-core`.
- **Ribbon-centric layout.** A ribbon with tabs, groups, contextual tabs and
  overflow, plus dock panels, panes and splits, a command palette, context
  menus, a tree and a data table.
- **A frameless Fluent window.** `FluentApp` opens a window with the Fluent
  title bar, resize chrome, theme, modal and toast hosts and bundled assets.
- **Keyboard and accessibility.** Visible focus, a keyboard path for every
  action, and roles, names and actions exposed through AccessKit (UI Automation
  on Windows, AT-SPI on Linux).
- **Test utilities.** `fluent-test-utils` dumps the accessibility tree and
  helps write in-process UI tests.
- **Native integration surfaces.** A video surface, a terminal grid and view,
  and file pickers, with [documented contracts](docs/consumer-contracts.md).
- **A vendored GPUI.** A filtered snapshot of upstream Zed `main` is vendored,
  so every release builds against one known GPUI.

## Install

fluent-gpui is distributed by git tag. The latest release is
[`v4.0.0+zed.ba63a022c9a2`](https://github.com/thedancingdeveloper/fluent-gpui/releases/latest) (the part after `+zed.` is the
Zed commit the vendored GPUI was cut from). Pin one exact tag on every
`fluent-*` crate you use:

```toml
[dependencies]
fluent-core       = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v4.0.0+zed.ba63a022c9a2" }
fluent-primitives = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v4.0.0+zed.ba63a022c9a2" }
fluent-app        = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v4.0.0+zed.ba63a022c9a2" }

[dev-dependencies]
fluent-test-utils = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v4.0.0+zed.ba63a022c9a2" }
```

> **Not on crates.io (for 4.x).** The names `fluent-core`, `fluent-app`,
> `fluent-primitives`, `fluent-layout` and `fluent-ribbon` exist on crates.io,
> but they are an older 2.x line that does not match this repository. Use the
> git tags.

Build requirements, the Linux packages and feature flags are in
[docs/architecture.md](docs/architecture.md#toolchain-and-linux-build-dependencies).
On Debian or Ubuntu:

```sh
sudo apt-get install libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev libfontconfig-dev libwayland-dev libvulkan-dev
```

## Quick start

A window with the Fluent title bar and a counter. This is
[`examples/hello_world`](examples/hello_world/src/main.rs):

```rust
use fluent_app::FluentApp;
use fluent_core::gpui::{div, prelude::*, px, ClickEvent, Context, IntoElement, Render, Window};
use fluent_core::ThemeProvider as _;
use fluent_primitives::{Button, ButtonAppearance, Label, LabelSize};

struct Counter {
    count: u32,
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(24.0))
            .bg(colors.surface)
            .child(Label::new(format!("Count: {}", self.count)).size(LabelSize::Display))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.0))
                    .child(
                        Button::new("increment")
                            .label("Increment")
                            .appearance(ButtonAppearance::Accent)
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.count += 1;
                                cx.notify();
                            })),
                    )
                    .child(Button::new("reset").label("Reset").on_click(cx.listener(
                        |this, _: &ClickEvent, _, cx| {
                            this.count = 0;
                            cx.notify();
                        },
                    ))),
            )
    }
}

fn main() {
    FluentApp::new("Hello FluentGUI")
        .window_size(480.0, 320.0)
        .run(|cx| cx.new(|_| Counter { count: 0 }));
}
```

`FluentApp::run` installs the theme, the modal and toast hosts and the window
title, opens a frameless window with the Fluent title bar and resize chrome, and
loads the bundled assets. Apps use GPUI through `fluent_core::gpui`, so they
never depend on `gpui` directly. For forms, a ribbon and a data table, read the
[quick start guide](docs/quick-start.md).

## Examples

Clone the repository and run any example with Cargo:

| Command | What it shows |
|---|---|
| `cargo run -p hello_world` | The smallest app, as above |
| `cargo run -p gallery` | Every control, by component family, with light, dark, high-contrast and RTL variants ([guide](docs/gallery.md)) |
| `cargo run -p widgets` | A set of widgets in one window |
| `cargo run -p form` | A form built from `Field` and input controls |
| `cargo run -p demo_app` | A project-tracker style app with a ribbon, settings navigation and a data table |
| `cargo run -p screenshots -- <component>` | One component in a small window, used by the [screenshots harness](CONTRIBUTING.md#screenshots) |

## Screenshots

Every image on this page is a real render of the gallery or of one component, captured headlessly under Xvfb at 1000x600 (components) or 1280x800 (gallery).

### Component gallery

#### Buttons and toggles

<table>
<tr><th>Dark</th><th>Light</th></tr>
<tr>
<td><img src="screenshots/dark/button.png" width="480" alt="Button (dark theme)"></td>
<td><img src="screenshots/light/button.png" width="480" alt="Button (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/icon-button.png" width="480" alt="Icon button (dark theme)"></td>
<td><img src="screenshots/light/icon-button.png" width="480" alt="Icon button (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/toggle-button.png" width="480" alt="Toggle button (dark theme)"></td>
<td><img src="screenshots/light/toggle-button.png" width="480" alt="Toggle button (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/switch.png" width="480" alt="Switch (dark theme)"></td>
<td><img src="screenshots/light/switch.png" width="480" alt="Switch (light theme)"></td>
</tr>
</table>

#### Selection and text input

<table>
<tr><th>Dark</th><th>Light</th></tr>
<tr>
<td><img src="screenshots/dark/checkbox.png" width="480" alt="Checkbox (dark theme)"></td>
<td><img src="screenshots/light/checkbox.png" width="480" alt="Checkbox (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/radio-group.png" width="480" alt="Radio group (dark theme)"></td>
<td><img src="screenshots/light/radio-group.png" width="480" alt="Radio group (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/combobox.png" width="480" alt="Combobox (dark theme)"></td>
<td><img src="screenshots/light/combobox.png" width="480" alt="Combobox (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/dropdown.png" width="480" alt="Dropdown (dark theme)"></td>
<td><img src="screenshots/light/dropdown.png" width="480" alt="Dropdown (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/text-input.png" width="480" alt="Text input (dark theme)"></td>
<td><img src="screenshots/light/text-input.png" width="480" alt="Text input (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/textarea.png" width="480" alt="Textarea (dark theme)"></td>
<td><img src="screenshots/light/textarea.png" width="480" alt="Textarea (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/searchbox.png" width="480" alt="Search box (dark theme)"></td>
<td><img src="screenshots/light/searchbox.png" width="480" alt="Search box (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/token-input.png" width="480" alt="Token input (dark theme)"></td>
<td><img src="screenshots/light/token-input.png" width="480" alt="Token input (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/date-time-picker.png" width="480" alt="Date and time picker (dark theme)"></td>
<td><img src="screenshots/light/date-time-picker.png" width="480" alt="Date and time picker (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/field.png" width="480" alt="Field (dark theme)"></td>
<td><img src="screenshots/light/field.png" width="480" alt="Field (light theme)"></td>
</tr>
</table>

#### Status and feedback

<table>
<tr><th>Dark</th><th>Light</th></tr>
<tr>
<td><img src="screenshots/dark/badge.png" width="480" alt="Badge (dark theme)"></td>
<td><img src="screenshots/light/badge.png" width="480" alt="Badge (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/message-bar.png" width="480" alt="Message bar (dark theme)"></td>
<td><img src="screenshots/light/message-bar.png" width="480" alt="Message bar (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/progress-bar.png" width="480" alt="Progress bar (dark theme)"></td>
<td><img src="screenshots/light/progress-bar.png" width="480" alt="Progress bar (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/spinner.png" width="480" alt="Spinner (dark theme)"></td>
<td><img src="screenshots/light/spinner.png" width="480" alt="Spinner (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/skeleton.png" width="480" alt="Skeleton (dark theme)"></td>
<td><img src="screenshots/light/skeleton.png" width="480" alt="Skeleton (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/toast.png" width="480" alt="Toast (dark theme)"></td>
<td><img src="screenshots/light/toast.png" width="480" alt="Toast (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/tooltip.png" width="480" alt="Tooltip (dark theme)"></td>
<td><img src="screenshots/light/tooltip.png" width="480" alt="Tooltip (light theme)"></td>
</tr>
</table>

#### Layout and navigation

<table>
<tr><th>Dark</th><th>Light</th></tr>
<tr>
<td><img src="screenshots/dark/ribbon.png" width="480" alt="Ribbon (dark theme)"></td>
<td><img src="screenshots/light/ribbon.png" width="480" alt="Ribbon (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/tab-strip.png" width="480" alt="Tab strip (dark theme)"></td>
<td><img src="screenshots/light/tab-strip.png" width="480" alt="Tab strip (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/tree.png" width="480" alt="Tree (dark theme)"></td>
<td><img src="screenshots/light/tree.png" width="480" alt="Tree (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/settings-nav.png" width="480" alt="Settings navigation (dark theme)"></td>
<td><img src="screenshots/light/settings-nav.png" width="480" alt="Settings navigation (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/toolbar.png" width="480" alt="Toolbar (dark theme)"></td>
<td><img src="screenshots/light/toolbar.png" width="480" alt="Toolbar (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/title-bar.png" width="480" alt="Title bar (dark theme)"></td>
<td><img src="screenshots/light/title-bar.png" width="480" alt="Title bar (light theme)"></td>
</tr>
</table>

#### Overlays and menus

<table>
<tr><th>Dark</th><th>Light</th></tr>
<tr>
<td><img src="screenshots/dark/dialog.png" width="480" alt="Dialog (dark theme)"></td>
<td><img src="screenshots/light/dialog.png" width="480" alt="Dialog (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/flyout.png" width="480" alt="Flyout (dark theme)"></td>
<td><img src="screenshots/light/flyout.png" width="480" alt="Flyout (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/popover.png" width="480" alt="Popover (dark theme)"></td>
<td><img src="screenshots/light/popover.png" width="480" alt="Popover (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/context-menu.png" width="480" alt="Context menu (dark theme)"></td>
<td><img src="screenshots/light/context-menu.png" width="480" alt="Context menu (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/command-palette.png" width="480" alt="Command palette (dark theme)"></td>
<td><img src="screenshots/light/command-palette.png" width="480" alt="Command palette (light theme)"></td>
</tr>
</table>

#### Data

<table>
<tr><th>Dark</th><th>Light</th></tr>
<tr>
<td><img src="screenshots/dark/data-table.png" width="480" alt="Data table (dark theme)"></td>
<td><img src="screenshots/light/data-table.png" width="480" alt="Data table (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/selectable-list.png" width="480" alt="Selectable list (dark theme)"></td>
<td><img src="screenshots/light/selectable-list.png" width="480" alt="Selectable list (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/card.png" width="480" alt="Card (dark theme)"></td>
<td><img src="screenshots/light/card.png" width="480" alt="Card (light theme)"></td>
</tr>
<tr>
<td><img src="screenshots/dark/avatar.png" width="480" alt="Avatar (dark theme)"></td>
<td><img src="screenshots/light/avatar.png" width="480" alt="Avatar (light theme)"></td>
</tr>
</table>

#### Whole-gallery pages

The gallery's pages, one per component family, in dark and light.

<details>
<summary>Show all 12 gallery pages</summary>

<table>
<tr><th>Dark</th><th>Light</th></tr>
<tr>
<td><img src="docs/images/gallery/buttons-and-links-dark.png" width="480" alt="Gallery page: Buttons and links (dark theme)"></td>
<td><img src="docs/images/gallery/buttons-and-links-light.png" width="480" alt="Gallery page: Buttons and links (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/selection-dark.png" width="480" alt="Gallery page: Selection (dark theme)"></td>
<td><img src="docs/images/gallery/selection-light.png" width="480" alt="Gallery page: Selection (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/text-dark.png" width="480" alt="Gallery page: Text (dark theme)"></td>
<td><img src="docs/images/gallery/text-light.png" width="480" alt="Gallery page: Text (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/status-and-feedback-dark.png" width="480" alt="Gallery page: Status and feedback (dark theme)"></td>
<td><img src="docs/images/gallery/status-and-feedback-light.png" width="480" alt="Gallery page: Status and feedback (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/data-and-lists-dark.png" width="480" alt="Gallery page: Data and lists (dark theme)"></td>
<td><img src="docs/images/gallery/data-and-lists-light.png" width="480" alt="Gallery page: Data and lists (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/data-display-dark.png" width="480" alt="Gallery page: Data display (dark theme)"></td>
<td><img src="docs/images/gallery/data-display-light.png" width="480" alt="Gallery page: Data display (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/date-and-input-dark.png" width="480" alt="Gallery page: Date and input (dark theme)"></td>
<td><img src="docs/images/gallery/date-and-input-light.png" width="480" alt="Gallery page: Date and input (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/layout-dark.png" width="480" alt="Gallery page: Layout (dark theme)"></td>
<td><img src="docs/images/gallery/layout-light.png" width="480" alt="Gallery page: Layout (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/media-dark.png" width="480" alt="Gallery page: Media (dark theme)"></td>
<td><img src="docs/images/gallery/media-light.png" width="480" alt="Gallery page: Media (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/menus-dark.png" width="480" alt="Gallery page: Menus (dark theme)"></td>
<td><img src="docs/images/gallery/menus-light.png" width="480" alt="Gallery page: Menus (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/overlays-dark.png" width="480" alt="Gallery page: Overlays (dark theme)"></td>
<td><img src="docs/images/gallery/overlays-light.png" width="480" alt="Gallery page: Overlays (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/gallery/theme-and-direction-dark.png" width="480" alt="Gallery page: Theme and direction (dark theme)"></td>
<td><img src="docs/images/gallery/theme-and-direction-light.png" width="480" alt="Gallery page: Theme and direction (light theme)"></td>
</tr>
</table>

</details>

### Keyboard focus

Each image is the component after a real Tab key press, showing its focus ring (dark theme on top, light below).

<table>
<tr>
<td><img src="docs/images/focus/dark/button.png" width="230" alt="Button with keyboard focus (dark theme)"></td>
<td><img src="docs/images/focus/dark/text-input.png" width="230" alt="Text input with keyboard focus (dark theme)"></td>
<td><img src="docs/images/focus/dark/tab-strip.png" width="230" alt="Tab strip with keyboard focus (dark theme)"></td>
</tr>
<tr>
<td><img src="docs/images/focus/dark/tree.png" width="230" alt="Tree with keyboard focus (dark theme)"></td>
<td><img src="docs/images/focus/dark/data-table.png" width="230" alt="Data table with keyboard focus (dark theme)"></td>
<td><img src="docs/images/focus/dark/dropdown.png" width="230" alt="Dropdown with keyboard focus (dark theme)"></td>
</tr>
</table>

<table>
<tr>
<td><img src="docs/images/focus/light/button.png" width="230" alt="Button with keyboard focus (light theme)"></td>
<td><img src="docs/images/focus/light/text-input.png" width="230" alt="Text input with keyboard focus (light theme)"></td>
<td><img src="docs/images/focus/light/tab-strip.png" width="230" alt="Tab strip with keyboard focus (light theme)"></td>
</tr>
<tr>
<td><img src="docs/images/focus/light/tree.png" width="230" alt="Tree with keyboard focus (light theme)"></td>
<td><img src="docs/images/focus/light/data-table.png" width="230" alt="Data table with keyboard focus (light theme)"></td>
<td><img src="docs/images/focus/light/dropdown.png" width="230" alt="Dropdown with keyboard focus (light theme)"></td>
</tr>
</table>


## Crates

| Crate | What it provides |
|---|---|
| [`fluent-core`](crates/fluent-core) | Theme, design tokens, colours, typography, motion, and the `fluent_core::gpui` re-export |
| [`fluent-primitives`](crates/fluent-primitives) | Buttons, inputs, dropdowns, checkboxes, switches, badges, cards, tooltips, expander, scrollbar, date and time picker, input capture, video surface, terminal view |
| [`fluent-layout`](crates/fluent-layout) | Menus, context menus, toolbar with overflow, dock panels, panes and splits, tree, data table, dialogs, toasts, command palette |
| [`fluent-ribbon`](crates/fluent-ribbon) | The ribbon command bar: tabs, groups, contextual tabs, overflow |
| [`fluent-app`](crates/fluent-app) | `FluentApp`, the title bar, client-side window chrome, bundled assets, file pickers |
| [`fluent-test-utils`](crates/fluent-test-utils) | Accessibility tree dumps and UI test helpers (use as a dev-dependency) |

## Platform support and status

| Platform | Renderer | Accessibility | Status |
|---|---|---|---|
| Windows 10/11 x64 | DirectX 11 | UI Automation | Supported |
| Linux x64, X11 and Wayland | wgpu (Vulkan, GL fallback) | AT-SPI | Supported |
| macOS, web / wasm | | | Not a target |

fluent-gpui is at version 4.x. Parts of it have had limited runtime validation.
Plainly:

- Everything is built and tested on Windows and Linux in CI, including
  in-process GPUI tests. Screenshots are captured headlessly on Linux.
- On Linux the example apps have been launched under a headless Wayland
  compositor with software rendering. Runs on real GPU hardware and on a full
  X11 desktop session are still outstanding.
- On Windows the examples launch and the window states (minimise, maximise,
  restore, move, resize) behave on a real desktop. Rendered output could not be
  captured automatically, and pointer snap gestures and DirectX device-loss
  recovery have not been exercised.
- The video surface and backdrop blur are applied on both platforms but not yet
  validated at runtime. Without the `backdrop-blur` feature, blur surfaces fall
  back to a solid tinted fill.
- Screen reader testing is not yet part of the published validation record.

Details and the full list of known gaps:
[docs/maintainers/status.md](docs/maintainers/status.md).

## Keyboard and accessibility

Every control is reachable and operable from the keyboard: Tab and Shift+Tab move
focus, with a visible focus ring; Enter and Space activate; arrow keys,
Home/End and PageUp/PageDown move within lists, trees, tables and menus; Escape
closes popups; dialogs trap focus. Each control exposes a role, an accessible
name and its actions through AccessKit. The [component index](docs/components.md)
lists the keyboard contract of every component, and `fluent-test-utils` lets you
test your own app's accessibility tree.

## Documentation

The full list is in [docs/README.md](docs/README.md). Start with:

- [Quick start](docs/quick-start.md): dependency setup, a themed window, a form, a ribbon and a data table.
- [Component index](docs/components.md): every component with its states, keyboard contract and gallery page.
- [Design tokens](docs/tokens.md): colours, typography, spacing and motion.
- [Architecture and consumer contract](docs/architecture.md): layering, versions, feature flags.
- [Native integration contracts](docs/consumer-contracts.md): video, terminal and pickers.
- [Changelog](CHANGELOG.md): release notes and migration notes.

### Upgrading

The 4.0.0 release changes row and chrome sizes and a few signatures. The
migration steps are in the [4.0.0 section of the changelog](CHANGELOG.md#unreleased-next-release-400).
It also adds `Scrollbar`, a `Toolbar` "More" menu, `Button::loading`, an
`.error(bool)` state on text-like inputs, window position save and restore, and
`RowDensity`.

## Contributing, security and licence

- Contributions are welcome: see [CONTRIBUTING.md](CONTRIBUTING.md) and the
  [code of conduct](CODE_OF_CONDUCT.md).
- Report vulnerabilities as described in [SECURITY.md](SECURITY.md), not in a public issue.
- The `fluent-*` crates are licensed under the [Apache License, Version 2.0](LICENSE);
  see also [NOTICE](NOTICE). The vendored Zed crates under `vendor/zed/` keep Zed's
  Apache-2.0 licence and notices; their origin is in
  [`vendor/zed/UPSTREAM.md`](vendor/zed/UPSTREAM.md) and the changes made to them in
  [`vendor/zed/PATCHES.md`](vendor/zed/PATCHES.md).
- The Fluent 2 colour token structure was adapted from
  [aernom/fluent-ui-gpui](https://github.com/aernom/fluent-ui-gpui) (MIT).

## Trademarks and affiliation

fluent-gpui is an independent project. It is not affiliated with, endorsed by or
sponsored by Microsoft. "Fluent" refers to the publicly documented Fluent 2
design language; no Microsoft fonts or Office artwork are included. It is built on
[GPUI](https://gpui.rs) from [Zed](https://github.com/zed-industries/zed), which is
licensed under Apache-2.0. Zed and GPUI belong to their respective owners.
