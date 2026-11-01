# Changelog

All notable changes to FluentGUI are documented here. Versions are per-crate; the
"workspace release" headings below group simultaneous bumps so consumers can pin
a coherent set.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and FluentGUI follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
per crate.

---

## Unreleased

- Tree: optional per-row checkbox slot (`TreeItem::checkable`, `checked`) with tri-state folders derived from leaves, `Tree::on_check`, and `set_tree_checked` for propagating a folder toggle to its descendants. Off by default; Windows and Linux (WI-2782).
- Vendor: `gpui_ios` and the `runtime_shaders` feature are trimmed from `vendor/zed` (Windows and Linux only; WI-2728).
- Vendor: `gpui` gets an opt-in `svg-font-warmup` feature (enabled by `fluent-core`) with `warmup_svg_font_db()`, which primes the usvg system font database on a background thread so the first SVG render does not block the UI (WI-1432).
- CI: layering guard, `cargo deny` (no GPL exception), zed-rev check, Windows test job and a `ci` summary check (WI-2725).
- Docs: `AGENTS.md` (how a worker session completes a work item) and a `CLAUDE.md` pointer.
- Dependency: the workspace now builds on the vendored zed gpui (`vendor/zed`, zed ba63a022) instead of `crates/gpui`, which is removed. `fluent-app` starts through `gpui_platform::application()`. Breaking: `Popover::anchor` takes `gpui::Anchor` (was `gpui::Corner`). Windows and Linux only.

- Build: `cargo deny` bans now list the vendored gpui crates that depend on `gpui` as wrappers, and the vendoring docs no longer describe the removed `crates/gpui` copy (WI-2754).
- Docs: README consumer snippet adds the direct `gpui` dependency the quick start needs, and the Linux build dependencies add `libxcb1-dev` and `libfontconfig-dev` (WI-2727).

## Workspace release v2.4.0 — 2026-05-28

Cohesive uplift focused on application primitives (multi-value inputs, date/time
selection, anchored suggestion surfaces), richer layout components
(`SelectableList`, `DocumentViewer`, constrained `PaneGroup` splits), and app
lifecycle hooks. Reactivity, theme-toggle behaviour, and the GPUI fork pin are
unchanged.

### Crate versions

| Crate              | Previous | New     |
|--------------------|----------|---------|
| `fluent-core`      | 2.3.0    | 2.4.0   |
| `fluent-primitives`| 2.3.0    | 2.4.0   |
| `fluent-ribbon`    | 2.0.2    | 2.0.2   |
| `fluent-layout`    | 2.1.1    | 2.2.0   |
| `fluent-app`       | 2.2.0    | 2.3.0   |
| `gpui` (vendored)  | 0.2.4    | 0.2.5   |

### `fluent-core` 2.4.0

Added
- `AsyncResource<T, E>` and `AsyncResourceState` for modelling
  `Idle`/`Loading`/`Ready`/`Error` request state with generation tokens. Lets
  views ignore stale async responses while keeping cached values visible during
  refreshes.
- **Midnight** colour scheme — deep navy darks with a vivid violet accent
  (`Theme::midnight`, `Theme::apply_midnight`).
- **Ember** colour scheme — warm dark browns with an amber/orange accent
  (`Theme::ember`, `Theme::apply_ember`).

### `fluent-primitives` 2.4.0

Added
- `TokenInput` — multi-value chip input with freeform tokens, filtered anchored
  suggestion popover, keyboard selection/removal, and validity state per token.
- `SuggestionPopover` — reusable anchored listbox surface and keyboard roving
  state shared by search/autocomplete and tokenised inputs.
- `DateTimePicker` — local wall-clock date and time selection with validation,
  segment keyboard adjustment, and `YYYY-MM-DDTHH:MM` parsing/formatting.
- `Textarea` runtime state setters (`set_text`, `set_placeholder`).
- `Searchbox::set_value` for programmatic value updates after construction.

Fixed
- Combobox / Dropdown / DateTimePicker close listeners no longer
  `stop_propagation`, unblocking click-through onto sibling widgets and
  application content underneath the popup.
- `TokenInput::set_tokens` clears transient query state and resets the
  highlighted suggestion index so external resets do not leak prior typing.

### `fluent-ribbon` 2.0.2

No source changes; bundled for workspace consistency.

### `fluent-layout` 2.2.0

Added
- `SelectableList` — lazy uniform-row list with single selection, keyboard
  roving, activation and context-menu callbacks, and loading/empty/error states
  for app-owned list data.
- `DocumentViewer` — safe rich-document model with lossy HTML/plain-text import
  helpers for headings, paragraphs, links, code, images, and tables.
- `ResizablePanelGroup`/`PaneGroup` constrained two-panel splits — horizontal
  or vertical, per-panel min/max pixel constraints, draggable dividers, and an
  `on_resize` ratio callback so apps can persist restored widths or heights
  without framework-specific state.
- Command palette enhancements: hint text, disabled rows, and entry-owned
  action callbacks.

Fixed
- `MenuBar` backdrop now uses `anchored()` at the window origin so
  close-on-click-away covers the full window instead of a stale region.
- Menu bar opens on mouse-down rather than click, and backdrop mouse-down no
  longer re-opens the menu trigger after a click-away close.
- Context menu, menu button, and command palette popups stop propagation of
  the backdrop click so sibling triggers cannot re-open the popup as part of
  the same gesture.
- Item row clipping, image rendering, and clipboard handling fixes shared
  between menus and the command palette.

### `fluent-app` 2.3.0

Added
- `WindowTitle` global — `set_title` and `set_status` keep platform chrome and
  the Fluent `TitleBar` aligned with application state from any code site.
- `FluentApp` lifecycle hooks — opt-in callbacks at app startup, post-window
  open, and theme change so consumers can wire status messages, keymaps, and
  background work without subclassing the entry point.
- `TitleBar` refinements aligning leading icon, drag region, and window
  controls with the new lifecycle.

### `gpui` (vendored) 0.2.5

Performance
- Pre-warm the `usvg` font database during renderer initialisation so the first
  SVG render no longer blocks the UI thread.

---

## v2.3.0 — 2026-05-11

See git history for the v2.3.0 release notes (primitives polish, GPUI 0.2.4
renderer fixes, AndroidConnect-driven uplift).
