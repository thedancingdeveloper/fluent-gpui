# Architecture and consumer contract

How fluent-gpui is layered, how apps depend on it, and what it promises.

## Layering

```text
  your app
        │  depends on fluent-* crates only
        ▼
  fluent-app   fluent-layout   fluent-ribbon   fluent-primitives   fluent-core   fluent-test-utils
        │
        ▼
  vendored gpui closure   vendor/zed/crates/*   (gpui and the small set of Zed crates it needs,
        │                                        plus feature-gated patch modules)
        ▼
  upstream Zed (github.com/zed-industries/zed), pulled monthly
```

**Rule: apps depend on `fluent-*` crates only, never on `gpui` directly.**
`fluent-core` re-exports the GPUI types apps need (elements, `Context`,
`Window`, `Render`, styling helpers) as `fluent_core::gpui`, so the GPUI version
an app builds against is always the one the fluent crates were tested with.
CI enforces the rule for this repository (`scripts/check-layering.py` and the
`[bans]` section of `deny.toml`).

## Which crate for which need

| Need | Crate |
|------|-------|
| Theme, design tokens, colours, typography, motion, the GPUI re-exports | `fluent-core` |
| Buttons, inputs, dropdowns, checkboxes, switches, badges, cards, tooltips, date/time picker and other controls | `fluent-primitives` |
| Workspace shell, menus, context menus, dock panels, panes and splits, tree, data table, dialogs, toasts, command palette | `fluent-layout` |
| Ribbon command bar: tabs, groups, contextual tabs, overflow | `fluent-ribbon` |
| `FluentApp` entry point, title bar, client-side window chrome, bundled assets | `fluent-app` |
| Accessibility tree dumps and UI test helpers (dev-dependency) | `fluent-test-utils` |

## Distribution and versions

fluent-gpui is distributed **by git tag**. Tags look like
`v<major>.<minor>.<patch>+zed.<short-sha>`, where `<short-sha>` is the upstream
Zed commit the vendored closure was cut from. Pin an exact tag on every
`fluent-*` crate and use the same tag for all of them. Never pin a branch.

```toml
[dependencies]
fluent-core       = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v4.0.0+zed.ba63a022c9a2" }
fluent-primitives = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v4.0.0+zed.ba63a022c9a2" }
fluent-app        = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v4.0.0+zed.ba63a022c9a2" }

[dev-dependencies]
fluent-test-utils = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v4.0.0+zed.ba63a022c9a2" }
```

Take only the crates you use. The crates.io names `fluent-core`, `fluent-app`,
`fluent-primitives`, `fluent-layout` and `fluent-ribbon` hold an older 2.x line
that predates the vendored-gpui 3.x line. Do not mix the two; use git tags.

- Every tag records the Zed commit it was cut from.
- A new tag follows each monthly upstream pull, and each patch release (bug or
  security fix) in between.
- **Breaking** means your app needs a source change to build or behaves
  differently after bumping the tag: a removed or renamed public item in a
  `fluent-*` crate, a changed signature, a changed feature flag, or a GPUI API
  change that leaks through the re-exports. Breaking changes bump the major
  version.
- Every tag has a [CHANGELOG.md](../CHANGELOG.md) entry with migration notes for
  anything breaking. Read it before bumping.

## Feature flags

Renderer additions sit behind Cargo features so the default build stays close
to upstream GPUI:

| Feature | Gives you | When off |
|---------|-----------|----------|
| `backdrop-blur` | `backdrop_blur(radius, tint)` acrylic-style backgrounds | Fluent surfaces fall back to a solid, tinted fill of the same colour. No error, no API change |

Enable a feature on the crate that exposes it, for example
`fluent-core = { git = "...", tag = "...", features = ["backdrop-blur"] }`.

## Toolchain and Linux build dependencies

The workspace pins its Rust toolchain in `rust-toolchain.toml` and states the
minimum supported version as `rust-version` in the workspace `Cargo.toml`. Build
your app with that toolchain or newer.

Linux build dependencies (Debian/Ubuntu):

```sh
sudo apt-get install libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev libfontconfig-dev libwayland-dev libvulkan-dev
```

Without `libfontconfig-dev`, the build script for `yeslogic-fontconfig-sys`
fails. Alternatively set `RUST_FONTCONFIG_DLOPEN=on` to load fontconfig at
runtime; the link step still needs the `-dev` packages above.

## Date and time input

`DateTimePicker` edits a timezone-less wall-clock `DateTimeValue`. **The app owns
the locale and the timezone**: the picker never reads the system locale or zone.
Pass a `DateTimeLocale` (`iso`, `en_us`, `en_gb`, `de_de`, or your own field
order, separator and 12/24-hour clock) to set display format, entry parsing and
segment order. `formatter` and `parser` hooks replace the locale for anything
else. `parse_input` reports bad text as a `DateTimeParseError` with a message
(also shown in the popup). Left/Right move between segments in display order,
Home/End jump to the first/last, Up/Down adjust. The gallery shows en-US and
de-DE side by side.

## Accessibility

Every control exposes a role, an accessible name and its actions through
[AccessKit](https://accesskit.dev), which maps to UI Automation on Windows and
AT-SPI on Linux. To check your own app, dump its accessibility tree in a test
with `fluent-test-utils` and compare it with a committed snapshot (roles and
names, not pixels). The gallery's own tree dump test is the worked example.

## More

- [Native integration contracts](consumer-contracts.md): video, terminal and
  picker behaviour that apps can rely on.
- [Vendored upstream](vendoring.md): what is under `vendor/zed` and how patches
  are tracked.
