# fluent-gpui

The estate's fork of [GPUI](https://gpui.rs) (Zed's GPU UI framework) and a
Fluent 2 design system on top of it, for native Windows and Linux desktop
apps. Its consumers are rdpapp-client, rtbit-gpui, email-desktop-fluent and
any later app that wants the same look and controls.

fluent-gpui is not affiliated with, endorsed by or sponsored by Microsoft.
"Fluent" refers to the publicly documented Fluent 2 design language.

> **Status: 3.x is released** on the vendored upstream Zed gpui (latest release
> `v3.2.0+zed.ba63a022c9a2`, 2026-11-05). The 3.x line replaces the old vendored
> GPUI copy with a filtered snapshot of current Zed `main` under
> `vendor/zed/crates/*` (Zed `ba63a022c9a2`). The named estate patches are listed
> in `vendor/zed/PATCHES.md`. Backdrop blur (WI-1349) is re-applied but not yet
> visually validated (WI-1438), and video frames are not validated at runtime.
> Sections below describe the 3.x consumer contract; where something does not
> exist yet, it says so.

---

## 1. Layering

```text
  your app (rdpapp-client, rtbit-gpui, email-desktop-fluent, ...)
        │  depends on fluent-* crates only
        ▼
  fluent-app   fluent-layout   fluent-ribbon   fluent-primitives   fluent-core   fluent-test-utils
        │
        ▼
  vendored gpui closure   vendor/zed/crates/*   (gpui + the small set of Zed crates it needs,
        │                                        plus feature-gated estate patch modules)
        ▼
  upstream Zed (github.com/zed-industries/zed), pulled monthly
```

**Rule: apps depend on `fluent-*` crates only, never on `gpui` directly.**
`fluent-core` re-exports the GPUI types apps need (elements, `Context`,
`Window`, `Render`, styling helpers), so the GPUI version an app builds
against is always the one the fluent crates were tested with. (The
re-export is part of the 3.x work; until it lands, the examples still
import `gpui` from the workspace.)

---

## 2. How to reference it

fluent-gpui is distributed **by git tag only**. It is not published to
crates.io. Tags have the form:

```text
v<major>.<minor>.<patch>+zed.<short-sha>
```

where `<short-sha>` is the upstream Zed commit the vendored closure was cut
from. Pin an exact tag on every `fluent-*` crate and use the same tag for
all of them. Never pin a branch.

```toml
[dependencies]
fluent-core       = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.2.0+zed.ba63a022c9a2" }
fluent-primitives = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.2.0+zed.ba63a022c9a2" }
fluent-layout     = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.2.0+zed.ba63a022c9a2" }
fluent-ribbon     = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.2.0+zed.ba63a022c9a2" }
fluent-app        = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.2.0+zed.ba63a022c9a2" }
# Until fluent-core re-exports GPUI (see §1), the quick start imports gpui directly.
gpui              = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.2.0+zed.ba63a022c9a2" }

[dev-dependencies]
fluent-test-utils = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.2.0+zed.ba63a022c9a2" }
```

The examples use the latest 3.x release, `v3.2.0+zed.ba63a022c9a2`. To adopt a
different release, pick its tag from the repository's tags and `CHANGELOG.md`.
Take only the crates you use.

### Which crate for which need

| Need | Crate |
|------|-------|
| Theme, design tokens, colours, typography, motion, the GPUI re-exports | `fluent-core` |
| Buttons, inputs, dropdowns, checkboxes, switches, badges, cards, tooltips, date/time picker and other controls | `fluent-primitives` |
| Workspace shell, menus, context menus, dock panels, panes and splits, tree, data table, dialogs, toasts, command palette | `fluent-layout` |
| Ribbon command bar: tabs, groups, contextual tabs, overflow | `fluent-ribbon` |
| `FluentApp` entry point, title bar, client-side window chrome, bundled assets | `fluent-app` |
| Accessibility tree dumps and UI test helpers (dev-dependency) | `fluent-test-utils` |

### Date and time input

`DateTimePicker` edits a timezone-less wall-clock `DateTimeValue`. **The app owns the locale and the
timezone**: the picker never reads the system locale or zone. Pass a `DateTimeLocale` (`iso`, `en_us`,
`en_gb`, `de_de`, or your own field order, separator and 12/24-hour clock) to set display format, entry
parsing and segment order. `formatter` and `parser` hooks replace the locale for anything else.
`parse_input` reports bad text as a `DateTimeParseError` with a message (also shown in the popup).
Left/Right move between segments in display order, Home/End jump to the first/last, Up/Down adjust.
The gallery shows en-US and de-DE side by side.

### Feature flags

Estate renderer additions sit behind Cargo features so that the default
build is close to upstream GPUI:

| Feature | Gives you | When off |
|---------|-----------|----------|
| `backdrop-blur` | `backdrop_blur(radius, tint)` acrylic-style backgrounds | Fluent surfaces fall back to a solid, tinted fill of the same colour. No error, no API change |

Enable them on the crate that exposes them, for example
`fluent-core = { git = "...", tag = "...", features = ["backdrop-blur"] }`.

### Toolchain

The workspace pins its Rust toolchain in `rust-toolchain.toml` at each tag
and states the MSRV as `rust-version` in the workspace `Cargo.toml`. The
MSRV tracks what upstream GPUI needs; build your app with that toolchain or
newer. (Both files arrive with the 3.x re-fork.)

Linux build dependencies (Debian/Ubuntu):

```sh
sudo apt-get install libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev libfontconfig-dev libwayland-dev libvulkan-dev
```

Without `libfontconfig-dev`, the build script for `yeslogic-fontconfig-sys`
fails; alternatively set `RUST_FONTCONFIG_DLOPEN=on` to load fontconfig at
runtime, but the link step still needs the `-dev` packages above.

---

## 3. Platform support

| Platform | Renderer | Accessibility | Status |
|----------|----------|---------------|--------|
| Windows 10/11 x64 | DirectX 11 | UI Automation | **Supported** |
| Linux x64, X11 and Wayland | wgpu (Vulkan, GL fallback) | AT-SPI | **Supported** |
| Web / wasm | — | — | Not a target |

---

## 4. Quick start

The smallest app: a window with the Fluent title bar and one button.

```rust
use fluent_app::FluentApp;
use fluent_core::ThemeProvider as _;
use fluent_primitives::{Button, ButtonAppearance};
use gpui::{div, prelude::*, ClickEvent, Context, IntoElement, Render, Window};

struct Hello;

impl Render for Hello {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(cx.theme().colors.surface)
            .child(
                Button::new("hello")
                    .label("Say hello")
                    .appearance(ButtonAppearance::Accent)
                    .on_click(|_: &ClickEvent, _, _| println!("hello")),
            )
    }
}

fn main() {
    FluentApp::new("Hello")
        .window_size(480.0, 320.0)
        .run(|cx| cx.new(|_| Hello));
}
```

`FluentApp::run` installs the theme, modal and toast hosts and the window
title global, opens a frameless window with the Fluent title bar and resize
chrome, and loads the bundled assets. Once `fluent-core` re-exports GPUI,
the `gpui::` import becomes `fluent_core::gpui::`.

To see every control, clone the repository and run the gallery:

```sh
cargo run -p gallery
```

Other examples: `hello_world`, `widgets`, `form`, `demo_app`.

### The gallery as a regression matrix

The gallery groups its routes by Fluent 2 component family (buttons and
links, selection controls, text and labels, status and feedback, data and
lists, date and input, terminal and video, theme and direction). Pick a
family in the left sidebar. Each page shows the state variants of its
controls side by side (default, selected, disabled, indeterminate, error).
The variant row on every page switches light, dark, high contrast and RTL, so
one pass through the families covers every state in every variant. After a
change to a primitive or to theme tokens, walk the families in all four
variants and compare against `docs/VISUAL_TARGETS.md`. A new public
primitive gets its states added to the matching family page in
`examples/gallery/src/families.rs`.

---

## Screenshots

<table>
<tr><th>Dark</th><th>Light</th></tr>
<tr>
<td><img src="docs/images/gallery/hero-dark.png" width="480" alt="The fluent-gpui gallery, default page (dark theme)"></td>
<td><img src="docs/images/gallery/hero-light.png" width="480" alt="The fluent-gpui gallery, default page (light theme)"></td>
</tr>
</table>

Every image below is a real render of the gallery or of one component, captured headlessly under Xvfb at 1000x600 (components) or 1280x800 (gallery).

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

These images are generated by the manual "Screenshots (Xvfb)" workflow (`.github/workflows/screenshots.yml`, `workflow_dispatch`) or locally with `scripts/screenshots-xvfb.sh`; `scripts/refresh-readme-screenshots.sh <run-id>` copies a run's artifact into place.

---

## Consumer guides

- [Quick start](docs/quick-start.md) — dependency setup, themed window, form, ribbon, and data table.
- [Component index](docs/components.md) — public exports, states, keyboard contracts, and gallery routes.

## 5. Versioning and cadence

- Every tag records the Zed commit it was cut from (`+zed.<short-sha>`).
- A new tag follows each **monthly** upstream pull, and each patch release
  (bug or security fix) in between.
- **Breaking**, for a consumer, means your app needs a source change to
  build or behaves differently after bumping the tag: a removed or renamed
  public item in a `fluent-*` crate, a changed signature, a changed
  feature flag, or a GPUI API change that leaks through the re-exports.
  Breaking changes bump the major version.
- Every tag has a `CHANGELOG.md` entry with migration notes for anything
  breaking. Read it before bumping.

---

## 6. Accessibility

Every control exposes a role, an accessible name and its actions through
[AccessKit](https://accesskit.dev), which maps to UI Automation on Windows
and AT-SPI on Linux. A control without them does not pass the
accessibility gate described in `CONTRIBUTING.md`.

To check your own app, dump its accessibility tree in a test with
`fluent-test-utils` and compare it with a committed snapshot (roles and
names, not pixels). The gallery's own tree dump test is the worked example.

---

## 7. Licence and attribution

- The `fluent-*` crates are licensed under the
  [Apache License, Version 2.0](LICENSE).
- The vendored Zed crates under `vendor/zed/` keep Zed's Apache-2.0
  `LICENSE-APACHE` and NOTICE files. Their origin and the upstream commit
  are recorded in `vendor/zed/UPSTREAM.md`; the estate's changes to them
  are listed in `vendor/zed/PATCHES.md`. Only Apache-2.0-compatible Zed
  crates are vendored; no GPL or AGPL code is included.
- The Fluent 2 colour token structure was adapted from
  [aernom/fluent-ui-gpui](https://github.com/aernom/fluent-ui-gpui) (MIT).

---

## 8. Pointers

- `CONTRIBUTING.md` — for maintainers: upstream pulls, the patch set, the
  accessibility gate (coming with the 3.x work).
- `AGENTS.md` — for worker sessions (coming with the 3.x work).
- [docs/graft.md](docs/graft.md) — how changes land in this repository.
- [docs/consumer-contracts.md](docs/consumer-contracts.md) — video, terminal and
  picker contracts for native consumers.
- [docs/VISUAL_TARGETS.md](docs/VISUAL_TARGETS.md) — visual reference
  targets.
- [CHANGELOG.md](CHANGELOG.md) — release notes and migration notes.
