# fluent-gpui

The estate's fork of [GPUI](https://gpui.rs) (Zed's GPU UI framework) and a
Fluent 2 design system on top of it, for native Windows and Linux desktop
apps. Its consumers are rdpapp-client, rtbit-gpui, email-desktop-fluent and
any later app that wants the same look and controls.

fluent-gpui is not affiliated with, endorsed by or sponsored by Microsoft.
"Fluent" refers to the publicly documented Fluent 2 design language.

> **Status: re-fork in progress.** The 3.x line replaces the old vendored
> GPUI copy with a filtered snapshot of current Zed `main` under
> `vendor/zed/crates/*`. That vendoring work is planned and not yet landed.
> Sections below describe the 3.x consumer contract; where something does
> not exist yet, it says so.

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
fluent-core       = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
fluent-primitives = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
fluent-layout     = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
fluent-ribbon     = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
fluent-app        = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }

[dev-dependencies]
fluent-test-utils = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
```

Replace `<short-sha>` with the value in the tag you are adopting (see the
repository's tags and `CHANGELOG.md`). The first 3.x tag does not exist
yet; this snippet is written against the integration branch and is
verified against the real tag by the first-tag work item. Take only the
crates you use.

### Which crate for which need

| Need | Crate |
|------|-------|
| Theme, design tokens, colours, typography, motion, the GPUI re-exports | `fluent-core` |
| Buttons, inputs, dropdowns, checkboxes, switches, badges, cards, tooltips, date/time picker and other controls | `fluent-primitives` |
| Workspace shell, menus, context menus, dock panels, panes and splits, tree, data table, dialogs, toasts, command palette | `fluent-layout` |
| Ribbon command bar: tabs, groups, contextual tabs, overflow | `fluent-ribbon` |
| `FluentApp` entry point, title bar, client-side window chrome, bundled assets | `fluent-app` |
| Accessibility tree dumps and UI test helpers (dev-dependency) | `fluent-test-utils` |

### Feature flags

Estate renderer additions sit behind Cargo features so that the default
build is close to upstream GPUI:

| Feature | Gives you | When off |
|---------|-----------|----------|
| `video-surface` | GPU video texture allocation, upload and paint, for streaming RGBA frames (remote desktop, media) | The video API is not compiled; code that uses it fails to build, so gate your own use behind the same feature |
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
sudo apt-get install libxkbcommon-dev libxkbcommon-x11-dev libwayland-dev libvulkan-dev
```

---

## 3. Platform support

| Platform | Renderer | Accessibility | Status |
|----------|----------|---------------|--------|
| Windows 10/11 x64 | DirectX 11 | UI Automation | **Supported** |
| Linux x64, X11 and Wayland | wgpu (Vulkan, GL fallback) | AT-SPI | **Supported** |
| macOS | Metal (upstream) | — | **Not supported, not validated.** Best-effort compile only. No video surface, no blur |
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

---

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
- [docs/VISUAL_TARGETS.md](docs/VISUAL_TARGETS.md) — visual reference
  targets.
- [CHANGELOG.md](CHANGELOG.md) — release notes and migration notes.
