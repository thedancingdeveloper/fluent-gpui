# Native Chrome and System Theme Parity (WI-3055)

Scope: Windows (DirectX), Linux X11, Linux Wayland. macOS is out of scope.

Status legend: **Code-verified** = confirmed by reading the code paths on a Linux
dev host; **Manual pending** = needs a person on that platform. No manual runtime
evidence has been collected yet; the matrix records what is known and what is open.

## Parity matrix

| Behaviour | Windows / DirectX | Linux X11 | Linux Wayland |
|-----------|-------------------|-----------|---------------|
| Drag to move (`TitleBar` title area) | `start_window_move`; Manual pending | `start_window_move` (`x11/window.rs`); Manual pending | `start_window_move` (`wayland/window.rs`); Manual pending |
| Double-click title to maximize/restore | Implemented in `TitleBar` (`zoom_window`); Manual pending | Same; Manual pending | Same; Manual pending |
| Min / max / close buttons | `TitleBar` draws 46px buttons, `on_click` -> `minimize_window`/`zoom_window`/`remove_window`; Manual pending | Same; Manual pending | Same; Manual pending |
| Snap layouts (Win11 max-button hover flyout) / native caption hit-testing | **Gap**: platform maps `WindowControlArea::{Min,Max,Close}` to `HTMINBUTTON`/`HTMAXBUTTON`/`HTCLOSE` (`windows/events.rs`), but `fluent-app` `TitleBar` never calls `.window_control_area(..)`, so snap layouts and Aero-snap-from-button are not reachable | n/a | n/a |
| Server-side vs client-side decorations | n/a | `window_decorations`/`request_decorations` implemented; Manual pending | Same, compositor-dependent (GNOME has no SSD); Manual pending |
| Window menu (right-click title) | Not wired in `TitleBar` | `show_window_menu` available, not wired in `TitleBar` | Same as X11 |
| System light/dark | `window_appearance` + `WM_SETTINGCHANGE`/`ImmersiveColorSet` -> `appearance_changed`; Code-verified; Manual pending | XDG desktop portal color-scheme (`xdg_desktop_portal.rs`); Code-verified; Manual pending | Same portal path; Code-verified; Manual pending |
| System accent colour | `UISettings` colour lookup exists in `windows/util.rs`; not consumed by `fluent-core` tokens (accent is theme-token driven) | No portal accent read | No portal accent read |

## Findings

1. **Snap layouts gap (Windows):** wiring `window_control_area` on the `TitleBar`
   buttons would route clicks through native `WM_NCLBUTTONUP` handling, which
   already performs min/max/close. The existing `on_click` handlers would then
   double-fire, so the change must replace them on Windows rather than add to
   them. Left unchanged here because it cannot be exercised on this host.
2. **Accent colour:** FluentGUI accent comes from `fluent-core` theme tokens, not
   the OS. Following the OS accent is a feature, not a parity bug; decide
   separately.
3. The `TitleBar` has no window-menu or tiled-edge handling on Linux.

## Manual evidence checklist (to be filled on real hardware)

For each of Windows 11 (DirectX), Linux X11, Linux Wayland (GNOME and KDE):

- [ ] Drag, double-click maximize, min/max/close work; close hover is red.
- [ ] Snap layouts flyout on max-button hover (Windows 11).
- [ ] Resize from edges/corners with the custom title bar.
- [ ] Toggle OS light/dark while running; app follows without restart.
- [ ] Change OS accent; record whether the app should follow.
- [ ] Attach screenshots under `screenshots/` and link them here.
