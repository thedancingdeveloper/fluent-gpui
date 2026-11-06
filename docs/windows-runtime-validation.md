# Windows runtime validation (WI-1439, WI-3178)

Host: `desktop-tlk1gff` (Windows, GPU, console session 1). Build: `main` 2b77306,
`cargo build --workspace --all-targets` passed. Each example was launched in the
interactive session through a one-off scheduled task (`/IT /RU`), then the task
was deleted. Nothing persistent was left behind.

| Example | Launches in session 1 | Alive after 10 s | Window title | Pixel check |
|---|---|---|---|---|
| hello_world | yes | yes | Hello FluentGUI | inconclusive |
| demo_app | yes | yes | FluentGUI Demo - Project Tracker | inconclusive |
| gallery | yes | yes | FluentGUI Gallery | inconclusive |
| video_frame (`vendor/zed/crates/gpui`) | yes | yes | (none) | inconclusive |

Over plain SSH the same binaries panic with `cannot open windows while headless`
(expected: no desktop in the SSH session).

## WI-3178 follow-up (2026-11-06)

The follow-up used the existing release binaries on `desktop-tlk1gff` in the
active RDP session (`query user`: `sprooty`, session 1, `Active`). No source or
system-setting changes were made, and no temporary scheduled task was left
behind.

| Check | Result | Evidence or limitation |
|---|---|---|
| `demo_app` minimise, restore, maximise | PASS | Existing interactive-session captures show the window absent, restored, and screen-filling while remaining non-blank. |
| `demo_app` move/resize | PASS | `SetWindowPos(120,120,900,650)` capture shows the moved and resized non-blank window. |
| `gallery` minimise, restore, maximise | PASS | Existing interactive-session captures show all three states and a valid custom title bar. |
| `gallery` move/resize | PASS | `SetWindowPos(120,120,900,650)` capture shows the moved and resized non-blank window. |
| Native title-bar controls after transforms | PASS | Restored and moved captures retain the custom title bar and window controls. |
| Pointer drag and Windows snap gesture | UNVERIFIED | The available automation used Win32 positioning, not a synthetic pointer gesture; no snap result is claimed. |
| Backdrop-blur unsupported-backend fallback | NOT RUN | The repository exposes no Windows environment variable, runtime flag, or test switch that disables backdrop blur. The source-level fallback remains documented below. |
| DirectX device-loss recreation | NOT RUN | The renderer contains recovery code, but the example has no fault-injection hook and no device reset was provoked. |
| Temporary task cleanup | PASS | Any prior follow-up task was removed; `schtasks` showed no `WI3178Chrome` task remaining. |

The host reported `AMD Radeon RX 7800 XT` with driver `32.0.32015.7029` and
status `OK`. The evidence confirms window-state handling only; it does not
substitute for pointer/snap interaction, unsupported-backend blur testing, or
device-loss injection.

## Not verified
- Screenshots: every capture (full screen `CopyFromScreen`, window-rect crop and
  `PrintWindow`) came back uniformly white, even with a correct window rect.
  The RDP console or DirectX flip-model presentation is likely not capturable
  this way, so rendered output is unconfirmed.
- Window chrome (move/resize/zoom/minimise), VideoFrame items 2 and 7, and the
  backdrop-blur fallback need a human at the console or a capture path that
  reads the swap chain (for example DXGI Desktop Duplication).

## Note
`cargo run -p gpui --example video_frame` does not resolve from the workspace
root; the example lives in `vendor/zed/crates/gpui` and builds from there.
