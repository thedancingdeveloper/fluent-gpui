# Changelog

All notable changes to fluent-gpui are documented in this file.

The format follows [Keep a Changelog 1.1.0](https://keepachangelog.com/en/1.1.0/).
fluent-gpui follows [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

What is versioned: a release is one workspace release of the `fluent-*` crates
(`fluent-core`, `fluent-primitives`, `fluent-layout`, `fluent-ribbon`,
`fluent-app`, `fluent-test-utils`). They share one version number. Releases are
Git tags of the form `vX.Y.Z+zed.<sha>`, where `<sha>` is the first 12 characters
of the vendored Zed commit the release is built on (for example
`v3.2.0+zed.ba63a022c9a2`). Pin the tag, not a branch. How a release is cut is
described in [docs/maintainers/RELEASING.md](docs/maintainers/RELEASING.md).

## Unreleased (next release: 4.0.0)

<!--
How to add an entry (whoever lands a change writes its entry, in the same change):
- Put it under Added, Changed, Deprecated, Removed, Fixed or Security, below a
  short bold area label such as **Buttons and inputs**. Reuse an existing label.
- Write plain language for a developer who has never seen our tracker: what
  changed for the person using the library and, for a breaking change, what to
  do about it. One line to a short paragraph. Crate and type names in backticks.
- No work-item numbers, changeset ids or internal process words.
- A breaking change also gets a row in "Breaking changes and migration" below.
- scripts/check-changelog.sh checks the format.
-->

### Highlights

- **Keyboard focus works in real apps.** `Tab` and `Shift-Tab` are now bound when
  you start with `FluentApp`, and every control draws a two-tone focus ring, for
  keyboard focus only.
- **One size rule for rows and chrome.** Dock headers, toolbars, ribbon tabs and menu
  bars are 32 px (were 28 px). List, menu, tree and table rows are 32 px (were 28 px),
  with an explicit 28 px `RowDensity::Compact`.
- **Consistent buttons.** Every button appearance has the same 1 px-bordered box
  (32 px, or 24 px compact), and icon-to-label gaps are 8 px (4 px compact).
- **Enter and Space activate once.** They no longer fire twice in several controls,
  and Enter now works in menus and navigation.
- **New:** a Fluent 2 `Scrollbar`, a `Toolbar` "More" menu, `Button::loading`, an
  `.error(bool)` state on text-like inputs, and window position save/restore.
- **Much wider gallery:** twelve pages, plus screenshots that also prove the
  focus ring is visible.

### Breaking changes and migration

Removed public items:

| Removed | Use instead |
|---|---|
| `ComponentTokens::menu_item_height` | `ComponentTokens::list_item_height` (32 px) |
| `ComponentTokens::dropdown_option_height` | `ComponentTokens::list_item_height` |
| `ComponentTokens::settings_nav_section_height` (was 28 px) | `ComponentTokens::list_item_height` (32 px) |
| `ComponentTokens::settings_nav_item_height` (was already 32 px) | `ComponentTokens::list_item_height` (32 px, so no visible change) |
| `ComponentTokens::menu_bar_height` | `ComponentTokens::chrome_height` (32 px) |
| `ComponentTokens::dock_header_height` | `ComponentTokens::chrome_height` |
| `ComponentTokens::command_palette_footer_height` | `ComponentTokens::chrome_height` |
| `ComponentTokens::ribbon_tab_height` | `ComponentTokens::chrome_height` |

To get the old 28 px rows, call `ComponentTokens::with_row_density(RowDensity::Compact)`
rather than setting a height by hand. Chrome rows (`chrome_height`) do not change
with density.

Changed signatures and construction:

| Change | What to do |
|---|---|
| `Flyout::new` now takes the trigger's window-space `Bounds<Pixels>` as its first argument: `Flyout::new(trigger_bounds, child)`. It anchors with window snapping. | Pass the bounds of the element that opens the flyout. |
| `Flyout::trigger_size(width, height)` is removed. The flyout used to be placed relative to its parent using that size. | Pass the trigger's window-space `Bounds<Pixels>` to `Flyout::new(trigger_bounds, child)` instead. |
| `ComponentTokens` is not `#[non_exhaustive]` and 4.0 adds public fields (`chrome_height`, `list_item_height`, `icon_label_gap` and others), so code that builds it with a struct literal no longer compiles. | Start from `ComponentTokens::default()` (and `with_row_density` for compact) and change only the fields you need. |
| `SettingsNav` has private focus fields, so it can no longer be built with a struct literal. | Build it with its builder methods. |

Behaviour changes you will notice (no code change needed unless noted):

| Area | Before | After |
|---|---|---|
| `Tab` / `Shift-Tab` | Did nothing in a real app, because the key bindings were only installed in tests. | `FluentApp::run` binds them. **Apps that build their own gpui `Application` must call `fluent_primitives::install_focus_keybindings(cx)` once at startup** (safe to call more than once). With nothing focused, `Tab` enters the first tab stop and `Shift-Tab` the last. |
| Keyboard focus ring | Mixed styles; several controls had none. | Every control draws the same two-tone ring (2 px outer, 1 px inner), only for keyboard focus. Disabled controls are not tab stops. |
| Chrome rows (dock header, toolbar bar, ribbon tab strip, menu bar, command palette footer) | 28 px | 32 px |
| List, menu, dropdown option, tree, table and settings-nav section header rows | 28 px | 32 px. `RowDensity::Compact` keeps 28 px. |
| Settings-nav item rows | 32 px | 32 px (unchanged). |
| Gap between an icon and its label | 2 px or 4 px, differing by control | 8 px everywhere (4 px in compact rows). Vertical large ribbon buttons and icon-only controls are unchanged. |
| Button box (`Button`, `ToggleButton`, ribbon commands) | Height and border differed by appearance. | Every appearance has a 1 px border and the same height: 32 px (24 px compact). Horizontal padding is 12 px (8 px compact). Content is always centred. |
| Enter / Space on `SettingsNav` headers and items, `TabStrip` tabs, `Tag` and grouped-list headers | Fired twice (a header opened and closed again). | Activates once. If your own code worked around the double firing, remove the workaround. |
| Enter in `MenuBar`, `ContextMenu`, `MenuButton` and `Nav` | Did nothing. | Activates the focused item. |
| Enter in `SelectableList` and `Tree` | Did nothing. | Activates the selection. |
| Text height | Inherited gpui's 1.618x line height. | `FluentApp` sets a 13 px text with 20 px line height as the root style. |
| Several tab-like controls | Two siblings could share one tab stop and ring. | Give sibling `Tree`s and clickable `Card`s their own `id(..)`. |

### Added
- Motion tokens now include loop timings (`spinner_loop_ms`, `pulse_loop_ms`, `skeleton_loop_ms`, `progress_loop_ms`) and `toast_offset`, plus a `toast_motion_surface` helper that slides and fades a toast in.

**Buttons and inputs**
- `Button::loading(bool)`: a spinner replaces the leading icon at the same size.
  Click, Enter and Space are ignored while loading; the tab stop and focus ring stay.
- `.error(bool)` on `Textarea`, `Combobox`, `Dropdown` and `Field`: a danger-coloured
  border, like `TextInput::error`. `Field::error` also colours the helper text and
  marks the wrapped input; an explicit `validation` state wins.
- `Scrollbar`: a Fluent 2 vertical and horizontal scrollbar with overlay
  auto-hide, thumb drag, track paging and hover/pressed states. It works over a
  `ScrollHandle` or `ListState` through `ScrollbarSource`. `VirtualList::scrollbar(true)`
  shows it.
- `Tooltip::id(..)` (default `"tooltip-trigger"`), `DataTable::id(..)` (default
  `"data-table"`) and `ToastHost::id(..)` (default `"toast-host"`): give each one
  its own id when you have several in a view, otherwise they share hover state,
  tab stops or column widths, or panic with a duplicate accessibility node id in
  debug builds.
- `id(..)` on `MessageBar`, `Spinner` and `ProgressBar`, which also sets the
  accessibility node id, so two of a kind in one window no longer panic in debug
  builds. Defaults are unchanged.

**Layout and navigation**
- `Toolbar` moves the items that do not fit (by measured width, or `available_width`)
  into a "More" menu. The More button is the last stop of the toolbar's single tab
  stop; Enter, Space and Down open the menu.
- `ContextMenu` limits its height to the window and scrolls with the wheel;
  arrow, access-key and Left/Right navigation scroll the selected row into view.
  Submenus are now separate layers anchored to their trigger row, so a scrolling
  parent no longer clips them.
- `RowDensity` and `ComponentTokens::with_row_density` for an explicit compact (28 px)
  or default (32 px) density.

**Menus**
- `ContextMenuItem::shortcut` now also fills the trailing column of checkbox and
  radio items (a new `shortcut` field on those variants).

**App shell and window chrome**
- `FluentApp::window_position(x, y)` opens the window at a saved screen position,
  clamped onto the display it overlaps (centred if none). `fluent_app::window_bounds(&Window)`
  reads the current bounds so an app can save them.

**Theming and tokens**
- Colour tokens `stroke_focus_outer` and `stroke_focus_inner` in every theme, and the
  shared `fluent_primitives::focus_ring` and `is_keyboard_focused` helpers.
- New `ComponentTokens` fields: `chrome_height`, `list_item_height`, `icon_label_gap`,
  `icon_label_gap_compact`, `row_icon_label_gap`, `toggle_box` (20 px), `menu_max_width`
  (320 px), button padding and gap tokens, and size tokens for badges, the title bar,
  the command palette result list and document-viewer images.

**Gallery and screenshots**
- New gallery pages: Inputs, Status and feedback, Layout components, Data display,
  Overlays, and Menus and commands, a Scrollbar page, an Edit menu, and checkbox,
  radio and shortcut menu items. The sidebar is now a `SettingsNav` of 12 pages.
- The gallery accepts `--page <id>` (or `GALLERY_PAGE`), `--theme dark|light`,
  `--size WxH`, `--ready-file <path>` and `--list-pages`. Defaults are unchanged.
- `scripts/screenshots-xvfb.sh` sends a real Tab key, saves `focus/{dark,light}/<component>.png`
  and prints one `focus ring: visible|NOT visible|n/a` line per component. It also
  captures every gallery page at 1280x800 and a `hero-<theme>.png`.
  `--only <component>` runs one component. `scripts/screenshots.sh` gains layout,
  overlay, open-menu and error/loading/disabled captures.

**Build, CI and docs**
- Compile-tested `no_run` examples in the documentation of `Button`, `TextInput`,
  `Field`, `Dropdown`, `Checkbox`, `DataTable`, `Tree`, `Dialog`, `TabStrip` and `FluentApp`.
- The focus-ring comparison lives in `scripts/focus-ring-lib.sh`, with `scripts/test-focus-ring.sh`.

### Changed
- Toasts now slide and fade in, tree chevrons rotate when a node expands or collapses, and the normal motion duration is 167 ms (was 200 ms).
- Whole-pixel spacing and 12px dismiss and chevron glyphs across fields, badges, chips, the search box, message bars, dropdowns, the settings navigation, the command palette and the date picker; the textarea minimum height now matches its padding.

**Accessibility and keyboard**
- `Button`, `IconButton`, `ToggleButton`, `Checkbox`, `Switch`, `RadioGroup`, `Dropdown`,
  `Link`, `Slider`, `SpinButton`, `Expander`/`Accordion` headers, `ColorPicker` swatches,
  `Toolbar`, `DataTable`, `TabStrip`, `Tree`, `SelectableList`, `SettingsNav`, ribbon tabs
  and commands, the `TitleBar` caption buttons, `Dialog` actions, dismissable `Chip` and
  clickable `Card` each own a focus handle, activate on Enter and Space, and draw the
  two-tone focus ring for keyboard focus only. Disabled controls are not tab stops.
- Groups are one tab stop with arrow-key movement: `RadioGroup`, `TabStrip` (arrows move
  and select), `Tree`, `SelectableList`, `Toolbar`, `ColorPicker` swatches. `DataTable`
  is one tab stop with its ring on an unclipped wrapper; the active ribbon tab is the
  strip's tab stop.
- A `Chip` focuses its dismiss button. `Dialog` and closable-tab close buttons are now
  compact subtle `IconButton`s with hover and pressed states.
- The `ColorPicker` selection outline uses the accent colour.
- `TabStrip` draws its ring inside each tab, and `SettingsNav` keeps 2 px of room at the
  side, so the ring is no longer clipped.

**Buttons and inputs**
- `Button` and `ToggleButton` share one sizing helper, so every appearance has the same box
  (see the behaviour table). `ToggleButton` and `IconButton` read icon and square sizes
  from the `button_*` component tokens (toggle icons were fixed at 16 px at every size).
- `Checkbox`, `RadioGroup` and `Switch` share one 20 px `toggle_box` size.
- Long labels are cut to one line with an ellipsis (`TruncateExt::truncate_label`) on
  tab labels (maximum width 240 px), tree rows, table cells and headers, the title-bar
  title, message bar and toast titles, `Button` labels, menu items and the menu-button
  trigger, and command-palette rows.
- Pressed backgrounds on toggle button, checkbox, radio, chip, tabs, tree, list, toolbar,
  menus, dialogs, dock, ribbon and title bar. The `Switch` knob grows on hover and press.
  Disabled command-palette rows no longer use an opacity trick.
- Text input error state and selection, and the title-bar close hover, use theme colours.

**Layout and navigation**
- `Combobox` scrolls the highlighted option into view during keyboard navigation.
  `Tree` fills its parent's height so its list renders in a fixed-height container.
- `RibbonBar` keys each command's focus handle by its position, so the controls of a
  `Stack` no longer share one handle (one tab stop and two rings) with the plain buttons
  after it.

**Menus**
- Menu separators share one helper: equal space above and below the 1 px line, centred
  at both densities, and no longer shrinking to nothing when a long menu scrolls.
- `MenuButton` item labels truncate to one line with an ellipsis, like `ContextMenu`
  and `MenuBar`.
- The submenu chevron mirrors in right-to-left layouts.
- Submenus open beside their parent row, flip left and snap inside the window with an
  8 px margin. The `MenuBar` drop-down anchors to the trigger's bottom-left with the same
  fit. Click-away backdrops cover the whole window, and popups keep an 8 px window margin.
- A menu row waits 200 ms before closing an open submenu in favour of a sibling row.
- Popup motion offsets with `top` instead of a margin, and `fluent_core::layer` holds the
  shared stacking-priority table.

**Theming and tokens**
- Pill, badge, dock, app-dot and date-time action radii, chip and spin-button radii, and
  the tree/list badge font size now come from theme tokens. Avatar and app-dot initials use
  `on_accent`. Tree and list badge padding, list text gap and ribbon-tab padding use
  `spacing` tokens (values unchanged).
- The switch thumb is 12 px with symmetric insets (40x20 track); the checkbox is 20 px with
  `radii.md`; the radio is 20/10 px.
- Text no longer inherits gpui's 1.618x line height. `TextRampExt::text_ramp` sets a text
  step's size and whole-pixel line height together.

**Build, CI and docs**
- The README states that 3.x is released on the vendored upstream Zed gpui and names the
  latest tag.
- The screenshot workflow installs `xdotool`. The focus-ring report says `NOT visible`
  only when fewer than 30 px differ, and shows `n/a (no tab stop: ...)` for chip, card and
  title bar, which the harness renders as non-interactive.

### Fixed
- The loading spinner on a Primary (Accent) button is now drawn in the on-accent foreground colour, so it stays visible against the blue fill; other appearances are unchanged.

**Accessibility and keyboard**
- `Tab` and `Shift-Tab` did nothing in real apps (see the behaviour table). Regression
  tests now send a real Tab through the startup path and check that every text-like
  control moves focus by exactly one stop per press.
- Enter did nothing on `MenuBar`, `ContextMenu`, `MenuButton` and `Nav`, and on
  `SelectableList` and `Tree` selections. The handlers checked for the key name `"return"`
  but gpui reports `"enter"`.
- Enter and Space fired twice on `SettingsNav` headers and items, `TabStrip` tabs, `Tag`,
  `Toolbar` items and `GroupedVirtualList` group headers, so a header opened and closed
  again. They now activate once.
- Tree rows, ribbon items and tabs, and title-bar caption buttons are now tab stops; caption
  buttons activate on Enter and Space.

**Menus**
- The gallery menu bar no longer shows a fifth, highlighted "File" cell. `MenuBar` now
  ignores the `&` access-key marker when it checks whether a File menu already exists, so
  `Workspace::with_file_menu` does not add a second one holding only Exit. This also fixes
  a keyboard-opened File menu showing only Exit.

**Layout and navigation**
- Two `DataTable`s, `Tooltip`s or `ToastHost` toasts in one view no longer panic with a
  duplicate accessibility node id (see `id(..)` under Added).

**Buttons and inputs**
- Long section titles and page names in `SettingsNav` are cut off with an ellipsis instead
  of wrapping onto a second line.
- Ribbon command buttons always centre their content, so an icon-and-label button stretched
  wider than its content is no longer left-aligned.

**Build, CI and docs**
- A missing dependency from `fluent-layout` to `fluent-primitives` in `Cargo.lock` is
  restored, so locked release builds resolve without version changes.
- A headless `gpui_wgpu` regression test covers drawing a plain quad frame after the
  intermediate texture is invalidated.

### Dependencies and security

- `url` is updated to 2.5.9, which fixes the advisory about nested `blob:` URLs.
- The advisory that `chrono` is unmaintained is ignored in `deny.toml` with a dated entry
  (added 2026-11-26, review by 2026-12-31). It is not exploitable here and `chrono` only
  comes in through the vendored Zed crates; it will be revisited at the next Zed update.

## 3.2.0 - 2026-11-05

Includes `VideoSurface` (from 3.1.0), the AccessKit tree snapshot test helper and the
`test-a11y-activation` vendor patch (headless AccessKit activation in test windows) that
`rdpapp-fluent` needs. Crate versions are unchanged from 3.1.0; consumers pin the tag
`v3.2.0+zed.ba63a022c9a2`.

### Added

**Buttons and inputs**
- `Slider` (horizontal or vertical, step, range, disabled, arrow/Home/End/PageUp/PageDown
  keys) and `SpinButton` (step buttons, min/max clamp, Up/Down keys, format and parse hooks),
  with gallery stories. No pointer drag or typed text entry yet.
- `Link`, inline or standalone, with default, subtle and disabled styles, hover underline,
  keyboard activation, link accessibility semantics and URL opening.
- `Accordion`, grouping `Expander` items with single or multiple open modes, a collapsible
  control and arrow-key header navigation.
- `InfoLabel` with a click/Enter info popover and `Field` integration.
- `Field` validation states, status messages, required markers, hints and input borders.
- `CounterBadge` with configurable counts, overflow labels, zero visibility, dot mode and
  filled or ghost appearance. Presence badges over avatars and sized persona rows with
  primary and secondary text.
- `ProgressBar`: indeterminate animation, semantic status colours, medium and large
  thicknesses and clamped values.
- `DateTimePicker`: `DateTimeLocale` (field order, separator, 12/24 h), formatter and parser
  hooks, locale-ordered keyboard segment navigation with Home/End, and invalid-date messages.
  Your app owns locale and time zone.
- `fluent-app`: async native file, folder and save pickers, and a read-only `FilePickerField`
  with a Browse action.

**Layout and navigation**
- `Breadcrumb` and `Pagination` with overflow menus, current-item semantics, right-to-left
  separators and keyboard support.
- `Drawer` with inline and overlay modes, start/end placement that follows text direction,
  size presets, transitions that respect reduced motion, focus trapping and focus restored
  to the opener.
- `SelectableList` rows are virtualised: `rows(count, item_at)`, optional variable row
  heights, scroll-into-view on keyboard selection and position-in-set accessibility data, on
  a new `VirtualList` element. The gallery has a 100,000-row story.
- `Tree` rows are virtualised with keyboard-preserving navigation, lazy child loading
  states and a 50,000-node gallery story.
- `DataStatusKind` loading, empty, no-results and error states with status roles for
  `DataTable` (new `NoResults` state) and `Tree`.
- `ModalStack`: nested modals dismiss the topmost one only on Escape or backdrop click;
  non-dismissible dialogs ignore both; focus stays trapped until the stack is empty.
- Dialogs dismiss on Escape, trap focus while open, restore focus on close and keep their
  actions across re-renders.
- Keyboard: `MenuBar` and `ContextMenu` get F10 or a tap of Alt to focus the menu bar,
  underlined `&` access keys (Alt+key from anywhere), and Escape/F10 to hand focus back.
  `context_menu_key_target` and `is_context_menu_key` open a context menu on Shift+F10 or the
  Menu key. `DataTable` gets Up/Down/Home/End/PageUp/PageDown, Space to select, Shift to
  extend through `on_row_extend`, and Enter to activate. List and picker keyboard contracts
  now cover Home/End, paging, Alt+Down, selection and dismissal.
- `RadioGroup`, `TabStrip` and `Toolbar` are one tab stop with arrow-key roving focus; Home
  and End jump.

**Accessibility**
- Screen-reader roles, names and state for: `Button`, `IconButton`, `ToggleButton`,
  `Checkbox`, `Switch`, `RadioGroup`; `TextInput`, `Textarea`, `Searchbox`, `Combobox`,
  `TokenInput` (a masked `TextInput` is a password input and exposes no value);
  `Tree`, `DataTable`, `TabStrip`, `SelectableList`, `MenuBar`, `MenuButton`, `ContextMenu`;
  `Toolbar`; `RibbonBar` (tab list, tabs, panels, groups, buttons, overflow, last-command
  status); `ProgressBar` (value text), `Spinner` (label), `ConnectionBadge` and `MessageBar`
  (status and alert roles, plus a recovery `action`).
  `Checkbox`, `Switch`, `RadioGroup` and `IconButton` gain `label()`, and `Field::input`
  names its input from the label.
- `Button`, `IconButton`, `ToggleButton`, `Checkbox`, `Switch`, `RadioGroup` and `Dropdown`
  take `tab_index`, handle Tab and Shift-Tab, and show the Fluent 2 focus stroke for keyboard
  focus only.
- A text-direction API, `LayoutDirection::current/set/toggle/pick`, is the single source of
  direction. `Dock`, `MenuBar`, `TabStrip` and `Field` mirror in right-to-left layouts while
  the platform title-bar buttons keep their position. The gallery has an Arabic and Hebrew demo.
- A high-contrast theme (`ColorScheme::high_contrast`, `Theme::high_contrast`,
  `Theme::apply_high_contrast`) mapped from the system colours, with strokes in place of
  fills and 7:1 contrast tests. The gallery View group switches Dark, Light and High contrast.
- Opt-in system light/dark theme following for `FluentApp`.

**Theming and tokens**
- Complete Fluent 2 neutral foreground, background and stroke state aliases for light, dark,
  Midnight, Ember and high-contrast themes.
- `StrokeWidthTokens` (1 px thin, 2 px thick, 3 px thickest), Fluent 2 motion curves and
  durations (popup motion defaults are kept), and the `shadow2` to `shadow64` elevation tokens.
- `ControlStates` (`neutral/accent/subtle/danger_control()`) map every control state to theme
  aliases. `Button`, `Card` and `Field` read them. The danger button's pressed fill is now the
  error border colour, the disabled neutral border uses the disabled stroke, and high-contrast
  pressed foregrounds use HighlightText.

**Testing**
- `fluent-test-utils`: `assert_a11y_node`, tab-order, keyboard-outcome, WCAG-contrast
  (`assert_contrast`) and right-to-left order assertions; compact role, name and state
  snapshots of rendered accessibility trees; `FluentInteractionExt` helpers to type keys,
  click by debug selector and assert focus.
- Interaction tests for buttons, checkboxes, switches, radio groups, dropdowns, comboboxes,
  text input (grapheme-safe cursor, selection, clipboard, UTF-16 offsets), `DataTable`,
  docks, `SettingsNav`, panes, `Toolbar`, `Workspace`, modals, dialogs, flyouts, popovers and
  toasts. Every dark, light, Midnight and Ember colour token is checked for initialisation
  and WCAG contrast, and high-contrast tests cover brand, subtle and status pairs.

**Gallery and docs**
- Gallery routes are grouped by Fluent 2 component family, with state matrices for buttons,
  links, selection controls and labels, a light/dark/high-contrast/right-to-left variant row
  on every page, and success, warning and focus-ring states in the theme matrix.
- `docs/consumer-contracts.md` (frame lifetime and cadence, terminal sizing and focus, picker
  cancellation, error and platform-support contracts), `docs/platform-parity.md` (Windows and
  Linux parity matrix and the native verification gaps), and DPI-scaling and resize notes for
  Windows DirectX and Linux X11 and Wayland. `docs/tokens.md` covers token aliases and
  icon size, contrast, label and tooltip rules.

### Changed

- `fluent-primitives` denies missing public API documentation; every exported control and
  supporting type is now documented.
- An icon-only control audit: `arrow_left` and `arrow_right` resolve to specific built-in
  icons, and a test checks every referenced icon exists.
- Vendored gpui: backdrop blur (`backdrop_blur(radius, tint)`) is re-applied, with a wgpu
  renderer path and a DirectX path that is not yet verified. Upstream Zed fix 65952 is
  backported, so entities read in paint callbacks of a cached view redraw the window when
  notified. The `key-scan-code` patch gives `KeyDownEvent`, `KeyUpEvent` and `KeystrokeEvent`
  the physical `scan_code` (Windows set-1 make code, `0xE000` for extended keys; Linux
  evdev code), and `InputCapture` now reports it and derives the W3C `code` from it. Windows
  is not yet verified.

## 3.1.0 - 2026-11-01

### Added

**Buttons and inputs**
- `VideoSurface` shows caller-supplied RGBA8 frames (up to 8192 px per side) on a gpui video
  texture: whole frames or dirty-region patches, with `Fit` (letterboxed) or `Native`
  (1:1, device-pixel-snapped) scaling, a `VideoLayout` widget-to-frame coordinate mapping, a
  `Resized` event in device pixels and a `Video` accessibility role. A placeholder backend and
  `VideoRecorder` are provided for tests. `Fit` scaling is linearly filtered. The Windows
  DirectX path is not yet verified.
- `TerminalView`, a monospace cell grid over a caller-owned `TerminalGrid` (characters,
  colours, bold/italic/underline/inverse, cursor shape and blink, drag selection and
  Ctrl+Shift+C copy, bounded scrollback, dirty rows, resize-to-fit with a `Resize` event),
  with a `Terminal` accessibility role. It does no terminal escape-sequence parsing.
- `InputCapture` grabs raw key press and release events (including Tab, Alt and lone
  modifiers) and element-local pointer and wheel events. Click or Enter grabs; Escape,
  Ctrl+Alt or losing focus releases. `scan_code` is reserved and always `None` in this release
  (it arrives in 3.2.0). Not yet verified on Windows.
- `Expander`: a collapsible section with a chevron header, Enter/Space toggle, a visible
  focus ring and an accessible button role with expanded state. You own the `expanded` state.

**Layout and navigation**
- `Tree` rows can have a checkbox (`TreeItem::checkable`, `checked`), with tri-state folders
  derived from their children, `Tree::on_check` and `set_tree_checked` to propagate a folder
  toggle to its descendants. Off by default.

**Gallery and docs**
- Gallery demos for video, terminal and input capture.

### Changed

- Vendored gpui patches: video frames (`Window::alloc_video_texture`, `upload_video_frame`,
  `paint_video_frame`, `free_video_texture`) drawn by the wgpu renderer on Linux, and by the
  DirectX renderer on Windows (not yet verified there). `upload_video_frame` rejects a zero or
  over-8192 dimension. gpui also gets an opt-in `svg-font-warmup` feature (enabled by
  `fluent-core`) that loads system fonts for SVG rendering on a background thread, so the first
  SVG render no longer blocks the UI.
- `scripts/vendor-zed.py` copies the font assets the vendored Zed crates embed (IBM Plex Sans
  and Lilex, SIL OFL 1.1) into `vendor/zed/assets`, so `cargo test -p gpui` compiles.

## 3.0.0 - 2026-10-31

fluent-gpui now builds on the vendored upstream Zed gpui instead of the old in-repo copy.
Windows and Linux only.

### Changed

- **Breaking:** the workspace builds on the vendored Zed gpui (`vendor/zed`, Zed commit
  `ba63a022`). The old `crates/gpui` copy is removed, and `fluent-app` starts through
  `gpui_platform::application()`. `Popover::anchor` takes `gpui::Anchor` (it took `gpui::Corner`).
- The vendored tree drops `gpui_ios` and the `runtime_shaders` feature (Windows and Linux only).
- `cargo deny` bans now list the vendored crates that depend on `gpui` as wrappers.

### Added

- CI checks: a layering guard (no direct `gpui` use outside the `fluent-*` crates and the
  vendored closure), `cargo deny` (no GPL exception), a Zed revision check, a Windows test
  job and a summary check.
- `AGENTS.md` guidance for automated contributors, with a `CLAUDE.md` pointer.

### Fixed

- The README consumer snippet now includes the direct `gpui` dependency the quick start needs,
  and the Linux build dependencies list `libxcb1-dev` and `libfontconfig-dev`. The vendoring
  docs no longer describe the removed `crates/gpui` copy.

## 2.4.0 - 2026-05-28

A release focused on application primitives (multi-value inputs, date and time selection,
anchored suggestion surfaces), richer layout components (`SelectableList`, `DocumentViewer`,
constrained `PaneGroup` splits) and app lifecycle hooks. Reactivity, theme-toggle behaviour
and the GPUI fork pin are unchanged.

Crate versions:

| Crate | Previous | New |
|---|---|---|
| `fluent-core` | 2.3.0 | 2.4.0 |
| `fluent-primitives` | 2.3.0 | 2.4.0 |
| `fluent-ribbon` | 2.0.2 | 2.0.2 |
| `fluent-layout` | 2.1.1 | 2.2.0 |
| `fluent-app` | 2.2.0 | 2.3.0 |
| `gpui` (vendored) | 0.2.4 | 0.2.5 |

### Added

**Theming and tokens** (`fluent-core`)
- `AsyncResource<T, E>` and `AsyncResourceState` model `Idle`/`Loading`/`Ready`/`Error`
  request state with generation tokens, so views can ignore stale async responses while
  keeping cached values visible during refreshes.
- The **Midnight** colour scheme (deep navy with a vivid violet accent: `Theme::midnight`,
  `Theme::apply_midnight`) and the **Ember** colour scheme (warm dark browns with an
  amber/orange accent: `Theme::ember`, `Theme::apply_ember`).

**Buttons and inputs** (`fluent-primitives`)
- `TokenInput`: a multi-value chip input with freeform tokens, a filtered suggestion
  popover, keyboard selection and removal, and a validity state per token.
- `SuggestionPopover`: a reusable anchored list surface with keyboard roving state, shared
  by search, autocomplete and tokenised inputs.
- `DateTimePicker`: local date and time selection with validation, keyboard adjustment per
  segment, and `YYYY-MM-DDTHH:MM` parsing and formatting.
- `Textarea::set_text` and `set_placeholder`, and `Searchbox::set_value`, for updating
  state after construction.

**Layout and navigation** (`fluent-layout`)
- `SelectableList`: a lazy uniform-row list with single selection, keyboard roving,
  activation and context-menu callbacks, and loading, empty and error states.
- `DocumentViewer`: a safe rich-document model with lossy HTML and plain-text import for
  headings, paragraphs, links, code, images and tables.
- `ResizablePanelGroup`/`PaneGroup` two-panel splits, horizontal or vertical, with per-panel
  minimum and maximum pixel sizes, draggable dividers and an `on_resize` ratio callback so
  apps can save restored sizes.
- Command palette: hint text, disabled rows and callbacks owned by each entry.

**App shell and window chrome** (`fluent-app`)
- `WindowTitle` global: `set_title` and `set_status` keep platform chrome and the Fluent
  `TitleBar` in step with app state from anywhere.
- `FluentApp` lifecycle hooks at app startup, after the window opens and on theme change.
- `TitleBar` refinements aligning the leading icon, drag region and window controls.

### Changed

- `fluent-ribbon` has no source changes; it was bundled for workspace consistency.
- Vendored gpui 0.2.5: the SVG font database is warmed up while the renderer starts, so the
  first SVG render no longer blocks the UI thread.

### Fixed

- `Combobox`, `Dropdown` and `DateTimePicker` close listeners no longer stop click
  propagation, so clicks pass through to sibling widgets and app content under the popup.
- `TokenInput::set_tokens` clears transient query state and resets the highlighted
  suggestion, so an external reset does not keep earlier typing.
- `MenuBar` backdrop is anchored at the window origin so click-away covers the whole window.
  The menu bar opens on mouse-down, and a backdrop mouse-down no longer re-opens the menu
  after a click-away close.
- Context menu, menu button and command palette popups stop the backdrop click from
  reaching sibling triggers, so they cannot re-open as part of the same gesture.
- Item row clipping, image rendering and clipboard handling fixes shared between menus and
  the command palette.

## 2.3.0 - 2026-05-11

Primitives polish, GPUI 0.2.4 renderer fixes and the AndroidConnect-driven uplift. No more
detailed notes were kept for this release; see the Git history.

### Changed

- Primitives polish and GPUI 0.2.4 renderer fixes (see the Git history for details).
