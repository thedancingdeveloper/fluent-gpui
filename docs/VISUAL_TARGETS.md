# FluentGUI Visual Targets

Last updated: 2026-05-25

---

## Purpose

This document records the visual references that shape FluentGUI. It is not a pixel-perfect product spec. The goal is to keep reusable framework primitives aligned with Fluent-style professional desktop applications while avoiding app-specific screens in the framework.

Primary references:

- Fluent 2-style remote connection manager screenshots in `ExternalExamples/` when available locally.
- Downstream rdpapp requirements for connection-manager shells.
- AndroidConnect-style dashboard and device-control requirements that drove the recent v2.3.0 primitive polish.

---

## Framework Visual Direction

FluentGUI should feel:

- dense enough for repeated desktop work;
- calm and predictable rather than decorative;
- token-driven, with light/dark parity;
- compatible with ribbon/menu command models;
- suitable for remote connection, device management, admin, and productivity tools.

Default surfaces should avoid one-off app colors. Accent, status, tint, spacing, radius, and typography should come from `fluent-core`.

---

## Reference Observations

### App shell

Target:

- Thin app title/chrome area.
- Menu/ribbon command surfaces below title/chrome.
- Resizable dock/sidebar regions.
- Central pane area with tabs or split panes.
- Modal and popover overlays that sit above the full workspace.

Current implementation:

- `FluentApp` installs client-side decorated GPUI windows.
- `WindowChrome` provides resize-edge hitboxes.
- `TitleBar` supports icon, drag, double-click maximize/restore, and window controls.
- `Workspace` composes menu bar, ribbon, docks, pane content, modal host, and status bar support.

### Ribbon

Target:

- Text tab strip with active accent indicator.
- Content row grouped by command intent.
- Large buttons for primary actions and compact/stacked buttons for secondary actions.
- Contextual tabs for active session/object modes.
- Overflow handling for narrow widths.

Current implementation:

- `RibbonBar` supports tabs, groups, large/compact/icon/toggle/stack items, contextual tabs, and More overflow.
- Contextual tab treatment exists but can still be visually refined if downstream apps need closer Fluent 2 matching.

### Menus and context menus

Target:

- Application menu bar for classic desktop workflows.
- Flat context menus with icons, shortcuts, separators, disabled rows, and cascading submenus.
- Menus open by click, not hover-switching between top-level menus.

Current implementation:

- `MenuBar` supports click-open menus, shortcut hints, click-away close, and keyboard roving.
- `ContextMenu` supports the required row types, cascading submenus, backdrop close, and keyboard navigation.

### Dock/sidebar navigation

Target:

- Resizable navigation sidebars.
- Search/filter controls when supplied by the application.
- Tree rows with chevrons, icons, disabled states, and app-owned selection.

Current implementation:

- `DockPanel` handles edge docking, resizing, collapse, and content hosting.
- `Tree` handles hierarchy, expansion, selection, disabled rows, callbacks, and corrected key model.
- App-specific search boxes and row adornments remain in downstream apps.

### Tab and pane content

Target:

- Central content panes with optional tab strip.
- Split panes for dual-pane workflows.
- Tab rows with active/hover states and close affordances.

Current implementation:

- `Pane`, `PaneGroup`, and `TabStrip` cover these patterns.
- Scrollable/overflowing tab-strip behavior can be refined if a downstream app hits the limit.

### Data entry and settings

Target:

- Modal dialogs with stable sizing.
- Left section nav for complex settings/data-entry flows.
- Right content region with labeled controls and validation messaging.
- Accent-left active item treatment in settings navigation.

Current implementation:

- `ModalStack`, `ModalHost`, `Dialog`, and `ConfirmModal` provide modal flow.
- `SettingsNav`, `Field`, `TextInput`, `Textarea`, `Dropdown`, `Combobox`, `Checkbox`, `Switch`, and `RadioGroup` cover common dialog controls.
- `Field` and inputs remain framework-level; specific 30+ dialog forms belong downstream.

### Dashboard and app-card surfaces

Target:

- Reusable neutral cards and compact section headers.
- Status pills and identity indicators for apps/devices/connections.
- First-paint loading placeholders.
- Stable numeric text for device metrics.

Current implementation:

- `Card` defines the canonical neutral card recipe.
- `SectionHeader` handles title/action rows inside cards.
- `AppDot` provides deterministic gradient app/contact identity.
- `ConnectionBadge` covers disconnected, pairing, paired, and error states.
- `Skeleton`, `SkeletonRow`, and `Skeleton::wrap` cover loading placeholders.
- `FluentTextExt::tabular_nums()` stabilizes changing numbers.
- `tint`, `with_alpha`, `hue_for`, and `gradient_from_hue` live in `fluent-core`.

### Blur and glass

Target:

- Subtle frosted surfaces for modern overlay/dashboard affordances when the backend supports it.
- Clear fallback behavior on unsupported renderers.

Current implementation:

- `surface_blur_layer` exists as a theme token.
- `gpui::backdrop_blur(radius, tint)` exists in the vendored GPUI fork.
- Blade/WGSL implements the current backdrop blur path.
- Cross-backend support is not complete; do not rely on it as a universal visual primitive yet.

### Remote display / video surfaces

Target:

- Efficient streaming frame surfaces for remote display and mirrored device use cases.
- Correct texture lifetime and stale-id behavior.

Current implementation:

- Vendored GPUI exposes `Window::alloc_video_texture`, `Window::paint_video_frame`, and `Window::free_video_texture`.
- Blade and Windows/DirectX paths exist.
- macOS/Metal video-frame support is still outstanding.
- `crates/gpui/examples/video_frame.rs` is the validation example.

---

## Fulfillment Matrix

| Visual need | Framework support | Status |
|-------------|-------------------|--------|
| Title/app chrome | `FluentApp`, `WindowChrome`, `TitleBar` | Implemented |
| Ribbon command surface | `RibbonBar` | Implemented |
| Classic menu bar | `MenuBar` | Implemented |
| Right-click/cascade menus | `ContextMenu` | Implemented |
| Dock/sidebar shell | `DockPanel`, `Workspace` | Implemented |
| Split/content panes | `Pane`, `PaneGroup`, `TabStrip` | Implemented |
| Modal dialogs | `ModalStack`, `ModalHost`, `Dialog`, `ConfirmModal` | Implemented |
| Settings navigation | `SettingsNav` | Implemented |
| Dense tables | `DataTable` | Implemented |
| Navigation trees | `Tree` | Implemented |
| Cards/section headers | `Card`, `SectionHeader` | Implemented |
| App identity dots | `AppDot` | Implemented |
| Connection/status pill | `ConnectionBadge` | Implemented |
| Loading skeletons | `Skeleton`, `SkeletonRow`, `Skeleton::wrap` | Implemented |
| Tabular numbers | `FluentTextExt::tabular_nums()` | Implemented |
| Backdrop blur | GPUI fork `backdrop_blur` | Blade implemented; cross-backend incomplete |
| Video-frame surfaces | GPUI fork `VideoTextureId` APIs | Blade/DirectX implemented; runtime validation pending |
| macOS video-frame renderer | Metal backend | Not implemented |
| Web visual parity | Web backend | Future |

---

## Design Rules for Future Work

- Add framework primitives only when at least one real app needs the pattern and the abstraction is reusable.
- Keep app-specific screens, business models, and workflow copy downstream.
- Use `fluent-core` tokens for colors, spacing, radii, type, component geometry, and motion.
- Use status tokens for info/success/warning/error states instead of hardcoded colors.
- Keep density close to desktop productivity apps: avoid oversized marketing-style sections in framework examples.
- Prefer small composable surfaces over nested card hierarchies.
- Document renderer/backend limitations before promoting a visual effect as generally supported.
