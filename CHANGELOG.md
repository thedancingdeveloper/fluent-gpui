# Changelog

All notable changes to FluentGUI are documented here. Versions are per-crate; the
"workspace release" headings below group simultaneous bumps so consumers can pin
a coherent set.

The format is loosely based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/)
and FluentGUI follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html)
per crate.

---

## Unreleased

- Add a `gpui_wgpu` headless regression test that a non-blur quad frame renders after intermediate-texture invalidation, guarding the backdrop placeholder (WI-3122).
- Restore the missing layout-to-primitives lockfile dependency so locked release dry-runs resolve without dependency version changes (WI-2951).

- Popups: `Flyout::new` now takes the trigger's window-space `Bounds` and anchors with window snapping (breaking, no in-repo callers); popup motion offsets with `top` instead of a margin; `fluent_core::layer` holds the shared deferred-priority table; menu rows wait 200 ms before closing an open submenu in favour of a sibling row (WI-3255).
- Pill, badge, dock, app-dot and date-time action radii and the tree/list badge font size now come from theme tokens; avatar and app-dot initials use `on_accent` (WI-3266).
- Tree/list badge padding, list text gap and ribbon tab vertical padding now use `spacing` tokens (values unchanged) (WI-3266).
- Badge dot/counter, title-bar, command-palette result list and document-viewer image sizes now come from new `ComponentTokens` fields (WI-3266).
- Dialog and closable-tab close buttons are now compact subtle `IconButton`s, gaining hover and pressed states; dividers already use `stroke_neutral_subtle` (WI-3266).

- Pressed (active) backgrounds on toggle button, checkbox, radio, chip, tabs, tree, list, toolbar, menus, dialogs, dock, ribbon and title bar; switch knob grows on hover/press; command palette disabled rows drop the opacity hack (WI-3265).

- New `list_item_height` component token (32px, Fluent 2); tree and data table rows use it instead of `menu_item_height` (WI-3259).

- Text input error state and selection, and title-bar close hover, now use theme colours (WI-3264, WI-3266)
- Chip dismiss and spin-button radii now use theme radius tokens (WI-3266)

- Single-line ellipsis truncation (`TruncateExt::truncate_label`) on tab labels (max width 240px token), tree rows, data table cells and headers, the title bar title and message bar/toast titles (WI-3268).

Workspace release. Includes `VideoSurface` (3.1.0), the AccessKit tree snapshot test helper and the `test-a11y-activation` vendor patch (headless test-window AccessKit activation) that rdpapp-fluent needs. Crate versions are unchanged, as in v3.1.0; consumers pin the tag.

- fluent-core, fluent-primitives, fluent-layout, fluent-ribbon, fluent-app: text no longer inherits gpui's 1.618x line height. `TextRampExt::text_ramp` sets a ramp step's size and whole-pixel line height together, `FluentApp` sets the body ramp (13/20) as the root text style, and Button, ToggleButton and dialog actions share one border-box height per size, always centre their content, and take 12/8px padding and gap from new `ComponentTokens` (WI-3256, WI-3257).
- fluent-layout: submenus open beside their parent row, flip left and snap inside the window with an 8px margin; the MenuBar dropdown anchors to the trigger's bottom-left with the same fit; MenuButton and ContextMenu click-away backdrops cover the window; popups use an 8px window margin (WI-3252, WI-3253, WI-3254).

- fluent-primitives: deny missing public API documentation and document every exported control and supporting type (WI-2945).
- fluent-primitives: switch thumb 12px with symmetric insets (40x20 track); checkbox 20px with `radii.md`; radio 20/10 (WI-3258).
- Single-line ellipsis truncation (`TruncateExt::truncate_label`) on tab labels (max width 240px token), tree rows, data table cells and headers, the title bar title and message bar/toast titles (WI-3268).

- docs: document DPI scaling conversion and resize behavior across Windows DirectX, Linux X11, and Linux Wayland, with native-run evidence requirements (WI-3061).
- Layout: add Fluent 2 Breadcrumb and Pagination controls with overflow menus, current-item semantics, RTL separators, and keyboard-ready controls (WI-3036).
- fluent-layout, gallery: Drawer with inline and overlay modes, RTL-aware start/end placement, size presets, reduced-motion-aware transitions, focus trapping and opener restoration (WI-3038).
- Add the Windows/Linux platform parity matrix and native verification gap list ([WI-3043](https://vogt.sprooty.com/work/WI-3043)).
- fluent-layout, gallery: virtualised expanded tree rows with keyboard-preserving navigation, lazy child loading states, and a 50,000-node gallery story (WI-3035).
- fluent-layout, gallery: virtualised `SelectableList` rows (`rows(count, item_at)`, optional variable row heights, scroll-into-view on keyboard selection, AccessKit position-in-set/set-size) on a new `VirtualList` element, and a 100,000-row gallery story (WI-3033).
- fluent-primitives, gallery: `Slider` (horizontal/vertical, step, range, disabled, Slider role with value text, arrow/Home/End/PageUp/PageDown keys) and `SpinButton` (step buttons, min/max clamp, Up/Down keys, format and parse hooks, SpinButton role) with gallery stories; no pointer drag or typed text entry yet (WI-3032).
- fluent-core, fluent-primitives, docs: `ControlStates` (`neutral/accent/subtle/danger_control()`) map every control state to theme aliases; Button, Card and Field read them instead of raw tokens; Danger button pressed fill is now the error border colour, the disabled neutral border uses the disabled stroke; control-state completeness test; high-contrast pressed foregrounds now use HighlightText; docs/tokens.md updated (WI-3021).
- gallery, docs: routes grouped by Fluent 2 component family with state matrices for buttons, links, selection controls and labels, a light/dark/high-contrast/RTL variant row on every page, and README guidance for using the gallery as a regression matrix (WI-3018).
- docs: `docs/consumer-contracts.md` states the frame lifetime and cadence, terminal sizing and focus, picker cancellation, error and platform-support contracts for rtbit-fluent, rdpapp and email-desktop-fluent (WI-3020).
- fluent-test-utils: reusable assertions for accessibility role/name/state (`assert_a11y_node`), tab order, keyboard outcomes, WCAG contrast (`assert_contrast`) and RTL inline order; Button, Switch and Dialog tests migrated (WI-3019).
- fluent-app, fluent-primitives, docs: icon-only control audit; arrow_left/arrow_right resolve to specific builtins, referenced-icon test, icon size/contrast/label/tooltip rules in docs/tokens.md (WI-3017).
- fluent-primitives, fluent-layout, gallery: ProgressBar value text (default percentage) and Spinner label for assistive tech, ConnectionBadge Status role, MessageBar Alert/Status roles plus a recovery `action`, and an rtbit download-progress gallery story; animations already honour `reduce_motion` (WI-3016).
- fluent-ribbon: RibbonBar exposes TabList, Tab, TabPanel, Group and Button roles with names, selected/toggled/expanded state, arrow/Home/End tab navigation, Enter/Space command activation, overflow semantics and a last-command status; gallery gains an rdpapp-style Connection tab (WI-3015).
- fluent-layout, gallery: DataStatusKind loading/empty/no-results/error states with Status/Alert roles for DataTable (new NoResults state) and Tree (`status`, auto empty and no-match), plus rtbit and rdpapp gallery stories (WI-3012).
- fluent-core, fluent-primitives, fluent-layout, gallery: add `LayoutDirection::current/set/toggle/pick` as the single direction API, route components through it, and add an Arabic/Hebrew RTL demo to the gallery (WI-3010).
- fluent-primitives: DateTimePicker gains DateTimeLocale (field order, separator, 12/24h), formatter/parser hooks, locale-ordered keyboard segment navigation with Home/End, and invalid-date messages; app owns locale and timezone (WI-3013).
- fluent-layout: nested modals — Escape and backdrop click dismiss the topmost modal only, non-dismissible dialogs ignore both, focus stays trapped until the stack empties; contract documented on ModalStack (WI-3014).
- fluent-layout: Toolbar exposes Toolbar and Button roles with accessible names, toggled state and overflow expansion (WI-3007).
- fluent-primitives: add TextInput interaction tests for grapheme-safe cursor movement, selection and UTF-16 offsets (WI-3011).
- fluent-core, gallery: extend high-contrast AAA token tests to brand, subtle and status pairs; add success, warning and focus-ring states to the gallery theme matrix (WI-3009).
- fluent-primitives, fluent-layout: complete list and picker keyboard contracts for Home/End, paging, Alt+Down, selection, dismissal, and focus-preserving navigation (WI-3008).
- Primitives: add `Accordion`, grouping `Expander` items with single/multiple open modes, collapsible control and arrow-key header navigation (WI-2933).

- fluent-primitives: add presence badges over avatars and sized persona rows with primary and secondary text (WI-2934).
- fluent-primitives: add `InfoLabel` with a click/Enter info popover and `Field` integration (WI-2936).
- fluent-primitives: add indeterminate ProgressBar animation, semantic status colours, medium/large thicknesses, clamped values, and progress accessibility semantics (WI-2937).
- fluent-primitives: add configurable `CounterBadge` counts, overflow labels, zero visibility, dot mode and filled/ghost appearance (WI-2935).
- Add Field validation states, status messages, required markers, hints, and input borders.
- fluent-app: add async native file/folder/save picker helpers and a read-only `FilePickerField` with Browse action (WI-2940).
- fluent-primitives: add inline and standalone `Link` with default/subtle and disabled styles, hover underline, keyboard activation, accessible link semantics, and URL-opening support (WI-2930).
- fluent-core: add complete Fluent 2 neutral foreground, background and stroke state aliases for light, dark, Midnight, Ember and high-contrast themes (WI-2981).
- fluent-core: add public Fluent 2 `StrokeWidthTokens` for 1px thin, 2px thick and 3px thickest logical-pixel strokes (WI-2983).
- fluent-core: add Fluent 2 motion curves and duration tokens while preserving popup motion defaults (WI-2984).
- fluent-core: add public Fluent 2 `shadow2`, `shadow4`, `shadow8`, `shadow16`, `shadow28` and `shadow64` two-layer elevation tokens (WI-2982).
- fluent-core: audit Fluent 2 alias token coverage in `docs/tokens.md`, mapping existing neutral, brand, status, radius and popup-motion names and recording missing state, shadow, stroke-width and motion families (WI-2925).
- fluent-layout, fluent-app, fluent-primitives: mirror Dock, MenuBar, TabStrip and Field layouts for RTL while preserving the platform title-bar control position (WI-2927).
- Add opt-in system light/dark theme following for FluentApp.
- fluent-core: high-contrast theme (`ColorScheme::high_contrast`, `Theme::high_contrast`, `Theme::apply_high_contrast`) mapped from the system colours Canvas, CanvasText, Highlight, GrayText and ButtonFace, with strokes in place of fills; covered by the token and contrast tests (7:1 checks); the gallery ribbon View group switches Dark, Light and High contrast (WI-2923).
- fluent-layout: AccessKit semantics for Tree (tree/treeitem with level, expanded, selected), DataTable (grid/row/columnheader/cell), TabStrip (tablist/tab), SelectableList (list/listitem), MenuBar, MenuButton and ContextMenu (menubar/menu/menuitem); snapshot test per widget (WI-2918).
- fluent-layout: keyboard access for MenuBar and ContextMenu: F10 or a tap of Alt focuses the menu bar, `&` access keys (underlined) open menus and pick items (Alt+key from anywhere), Escape and F10 hand focus back; `context_menu_key_target` and `is_context_menu_key` open a context menu on Shift+F10 or the Menu key at the focused element (WI-2922).
- fluent-primitives: AccessKit semantics for TextInput, Textarea, Searchbox, Combobox and TokenInput (textbox/searchbox/combobox roles with value; masked TextInput is a password input and exposes no value); `Field::input` sets the input's accessible name from the label and description from the validation message (WI-2917).
- fluent-primitives: Button, IconButton, ToggleButton, Checkbox, Switch and RadioGroup expose AccessKit roles, names and toggled state; Checkbox, Switch, RadioGroup and IconButton gain `label()` (WI-2916)
- fluent-layout: DataTable interaction tests: header click sort asc/desc, ctrl/shift click multi-select, selection kept by row key across a data refresh (WI-2910).
- fluent-layout: interaction tests for DockPanel collapse, SettingsNav selection, Pane tab switching, Toolbar dispatch and Workspace docks; `debug_selector`s on dock close and settings-nav items (WI-2912).
- fluent-layout: interaction tests for ModalStack/ModalHost, Dialog, Flyout, Popover and Toast (backdrop dismissal, input blocking, toast timeout on the fake clock); `debug_selector`s on dialog and modal buttons (WI-2911).
- fluent-layout: dialogs dismiss on Escape, trap focus while open, restore focus on close, and retain actions across rerenders (WI-2911).
- fluent-layout: DataTable keyboard navigation: Up/Down/Home/End/PageUp/PageDown move the active row and scroll it into view, Space selects, Shift extends via `on_row_extend`, Enter activates (WI-2921).
- fluent-primitives, fluent-layout: RadioGroup, TabStrip and Toolbar are one tab stop with arrow-key roving focus (arrows also select in RadioGroup and TabStrip), Home/End jump (WI-2920).
- fluent-primitives: Button, IconButton, ToggleButton, Checkbox, Switch, RadioGroup and Dropdown take `tab_index`, handle Tab/Shift-Tab, and show the Fluent 2 focus stroke for keyboard focus only (WI-2919).
- Add interaction coverage for text-input editing and clipboard shortcuts, textarea newlines, and Escape-cleared searchboxes (WI-2909).
- fluent-test-utils: add compact role, accessible-name and state snapshots of rendered AccessKit trees, with Expander coverage (WI-2915).
- Add click, keyboard, dismissal, callback, and filtering interaction coverage for radio groups, dropdowns, and comboboxes.
- Add headless click and keyboard interaction coverage for buttons, checkboxes, and switches.
- fluent-core: check every dark, light, Midnight and Ember colour token for initialization and enforce WCAG contrast for text, brand and status pairs (WI-2914).
- fluent-test-utils: `FluentInteractionExt` helpers on `VisualTestContext` to type keys, click an element by debug selector and assert focus (WI-2906).
- gpui: re-apply backdrop blur (`backdrop_blur(radius, tint)`) on the vendored gpui, with a wgpu renderer path and an unverified DirectX path; drop the vendored patch budget from CONTRIBUTING (WI-1349).
- gpui: backport upstream zed fix #65952 so entities read in paint callbacks or deferred draws inside a reused cached view still redraw the window when notified (WI-1346).
- Docs: outstanding.md, README and VISUAL_TARGETS state the vendored-gpui status: video frames applied on wgpu and DirectX, not runtime-validated; blur not re-applied (WI-1437).
- Primitives: `VideoSurface` shows caller-supplied RGBA8 frames (up to 8192 per side) on a gpui video texture: whole frames or dirty-region patches, texture allocated, reallocated on size change and freed on release; `Fit` (letterboxed) or `Native` 1:1 device-pixel-snapped scaling; `VideoLayout` widget-to-frame coordinate mapping and a `Resized` event in device pixels; `Video` accessibility role; a placeholder backend and `VideoRecorder` for tests. Fit scaling is linearly filtered (nearest needs a renderer patch). Gallery demo animates a generated frame. Windows DirectX path unverified (WI-2803).
- Primitives: `TerminalView` monospace cell-grid element over a caller-owned `TerminalGrid` (char, fg/bg, bold/italic/underline/inverse cells; cursor shape/blink; drag selection + Ctrl+Shift+C copy; bounded scrollback viewport; dirty rows; resize-to-fit with a `Resize` event); input via `InputCapture`, rows shaped once and cached by line version; `Terminal` accessibility role with visible text as value. No VT parsing. Gallery demo in the split-pane first panel (WI-2806).
- Primitives: `InputCapture` element: grabbed raw key press/release (incl. Tab/Alt, synthesised lone modifiers), element-local pointer/wheel, click or Enter to grab, Escape / Ctrl+Alt / blur to release; best-effort W3C key codes, `scan_code` reserved (always None; needs a vendored-gpui patch). Gallery demo in the split-pane detail. Windows and Linux; Windows unverified (WI-2804).
- Primitives: `Expander`, a collapsible section with a chevron header, Enter/Space keyboard toggle, a visible focus ring and an accessible button role with expanded state; caller-owned `expanded` state, instant toggle; widgets example entry (WI-2805).
- Vendor: `key-scan-code` patch: `KeyDownEvent`, `KeyUpEvent` and `KeystrokeEvent` carry the physical `scan_code` (Windows set-1 make code, `0xE000` for extended keys; Linux X11/Wayland evdev code). `InputCapture` reports it, derives the W3C `code` from it (`code_for_scan_code`), and releases keys held at ungrab with their scancode. Windows unverified (WI-2804).
- Vendor: `video-frames-directx` patch: the DirectX renderer draws video frames from a standalone RGBA8 texture per id, with a `video_frame` HLSL pass. Unverified on Windows (WI-1434).
- Vendor: `video-frames` patch: `Window::alloc_video_texture` / `upload_video_frame` / `paint_video_frame` / `free_video_texture` with packed RGBA8 frames, drawn by the wgpu renderer from a standalone texture per id, plus a `video_frame` example. `upload_video_frame` rejects a zero or over-8192 dimension and the wgpu renderer skips a frame shorter than `w*h*4`. Linux only; Windows is WI-1434 (WI-1348).
- Vendor: the font assets the vendored zed crates `include_bytes!` (IBM Plex Sans, Lilex; SIL OFL 1.1) are copied to `vendor/zed/assets` by `scripts/vendor-zed.py`, so `cargo test -p gpui` compiles (WI-2786).
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
