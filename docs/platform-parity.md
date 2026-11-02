# Windows/Linux platform parity

This matrix is the audit boundary for `fluent-app` consumers. It describes the
contracts exposed by the vendored GPUI platform layer and the Fluent wrappers;
it does not promise identical native UI. The supported targets are Windows
10/11 x64 with DirectX 11, and Linux x64 with X11 or Wayland. See the [consumer
contracts](consumer-contracts.md#4-support-boundaries) for the supported-target
boundary and [CONTRIBUTING.md](../CONTRIBUTING.md) for the repository's
platform policy.

**Evidence labels:** **verified-by-test** means a repository test exercises the
behavior (usually on GPUI's test platform); **verified-manually** means an
operator has exercised the native backend and recorded the result; **source-
audited** means the backend implementation and the shared conversion contract
were inspected, but no native desktop run is claimed; **unverified** means the
source contract is documented but native behavior still needs a backend run. A
row marked **divergent** is an intentional platform difference, not a failure.

## Matrix

| Behavior | Windows DirectX 11 | Linux X11 | Linux Wayland | Evidence and follow-up |
|---|---|---|---|---|
| Window movement and native decorations | `TitleBar` can replace native decorations in client/frameless mode; drag uses `start_window_move`; minimize/maximize/close are app controls. Native frame behavior is a Windows policy. | A move request is handled by the X11 window manager; the app must not assume it can reposition a top-level window directly. Whether the native frame is shown is window-manager policy. | A move request is handled by the compositor through the xdg-shell protocol; the app must not assume it can reposition a top-level surface directly. Server-side/client-side decoration support varies by compositor. | **divergent, unverified** natively. The wrapper behavior is covered by source-level/test-platform contracts; run [WI-3055](https://vogt.sprooty.com/work/WI-3055). |
| Snap and caption buttons | Windows snap layouts and caption-button hit testing are OS behaviors; custom chrome must preserve the caption-button affordance and leave controls accessible. | No Windows snap-layout equivalent is promised; tiling/window-manager behavior is external to Fluent. | Tiling and decoration behavior are compositor-specific; no Windows snap-layout equivalent is promised. | **divergent, unverified**. Follow [WI-3055](https://vogt.sprooty.com/work/WI-3055) for a native interaction pass. |
| System theme and accent | Fluent theme state supplies colors; `TitleBar` uses the Fluent accent colors. Automatic OS theme/accent synchronization is not claimed by the wrapper. | Same Fluent theme contract; desktop-environment synchronization is not claimed. | Same Fluent theme contract; desktop-environment synchronization is not claimed. | **unverified** for native synchronization. Follow [WI-3055](https://vogt.sprooty.com/work/WI-3055). |
| High-contrast detection | The `high-contrast` patch exposes OS state and change notifications, but this repository has no real-Windows run recorded. | Portal-backed `org.freedesktop.appearance` contrast is implemented by the Linux platform layer; desktop coverage is not recorded here. | Portal-backed `org.freedesktop.appearance` contrast is implemented by the Linux platform layer; compositor/portal availability varies. | **unverified** natively. Follow [WI-3056](https://vogt.sprooty.com/work/WI-3056). See [`PATCHES.md`](../vendor/zed/PATCHES.md) for the implementation boundary. |
| IME composition and committed text | GPUI separates composition/committed text from physical key events; native IME coverage is not recorded. | Same contract; toolkit/desktop IME setup is environment-dependent. | Same contract; toolkit/desktop IME setup is environment-dependent. | **unverified** natively. Follow [WI-3057](https://vogt.sprooty.com/work/WI-3057). |
| Physical scancodes and key repeat | Set-1 make code is derived from the Windows message, with `0xE000` marking an extended key. | Scancode is derived from X11 event detail (`detail - 8`). | Wayland supplies the evdev key on the initial event and repeat timer. | **verified-by-test** for the shared patch/InputCapture press-repeat-release contract; **unverified natively** on all three backends. See [scancode evidence](#physical-scancode-evidence) and [`PATCHES.md`](../vendor/zed/PATCHES.md). |
| Clipboard (text and image) | GPUI's DirectX backend reads and writes Unicode text and registered image formats (PNG, GIF, JPEG, SVG). | The X11 selection backend supports UTF-8 text and the GPUI image formats; ownership and persistence depend on the selection owner/clipboard manager. | The data-device backend supports text and image MIME types, with reads transferred through a pipe and cached after a successful offer read. | **source-verified; native run unavailable**. See the [clipboard audit](#clipboard-audit) and follow [WI-3059](https://vogt.sprooty.com/work/WI-3059) for a Windows/X11/Wayland host run. |
| File and folder pickers | `fluent-app::pickers` delegates to native GPUI prompts; test-platform filtering and cancellation are tested, native dialogs are not. | Native dialog availability depends on the desktop portal/toolkit environment. | Native dialog availability normally depends on the desktop portal/toolkit environment. | **unverified** natively. Follow [WI-3060](https://vogt.sprooty.com/work/WI-3060); see [picker contracts](consumer-contracts.md#3-file-and-folder-pickers). |
| DPI scaling and device pixels | `WM_DPICHANGED` updates the window scale factor; device-pixel resize sizes are converted to logical pixels before the resize callback. Text and layout continue to use logical pixels, while renderer bounds use device pixels. | The display scale is selected from `GPUI_X11_SCALE_FACTOR`, RandR, `Xft.dpi`, or the default `1.0` (in that order); X11 resize callbacks receive logical size and scale. | The compositor-provided output scale is applied to the surface; configure/resize handling is throttled during interactive resizing and callbacks receive logical size and scale. | **source-audited, unverified manually**. The shared contract is `logical = physical / scale` and `physical = round(logical * scale)`; see the manual evidence and test plan below. Follow [WI-3061](https://vogt.sprooty.com/work/WI-3061) for native runs. |
| X11 vs Wayland window/input differences | Not applicable. | X11 exposes server-side selections and X11 event details; window-manager policy is external. | Wayland delegates more policy to the compositor and uses evdev/data-device protocols; server-side assumptions must not be made. | **divergent** by design and **unverified** as a complete matrix. Follow [WI-3062](https://vogt.sprooty.com/work/WI-3062). |

## Manual evidence record

A native run must record the session type, desktop/compositor, scale factor,
portal availability, and the result for each row. Use the same build and
scenario in both sessions; do not treat a test-platform result as native proof.
The following checklist is the minimum evidence to attach to a work-item
comment or release note:

1. **Window movement/decorations:** launch the gallery, drag the custom title
   bar, invoke minimize/maximize/close, and repeat with native decorations
   enabled where the desktop allows it. Record whether the move request is
   accepted and which component owns the frame.
2. **Clipboard:** copy and paste text and an image, then close the source
   application and paste again. On X11 also test primary selection; on Wayland
   record whether the data offer resolves asynchronously and which formats are
   offered.
3. **Pickers:** open/save a file and pick a folder, cancel each dialog, and
   test an extension filter. Record whether the portal/toolkit dialog appears,
   the returned path, and cancellation/error behavior.
4. **Input:** exercise key repeat, IME composition, pointer motion, wheel,
   drag, and title-bar buttons. Record physical key/scancode behavior and any
   compositor-specific limitations.
5. **Scaling:** run at 100% and a non-100% scale, move the window between
   outputs if available, and compare logical layout dimensions with device
   pixel resize events.

**Evidence for this documentation land (2026-11-02):** source and test
contracts were audited, but no native evidence is claimed. This worker has no
X11 or Wayland session (`DISPLAY` and `WAYLAND_DISPLAY` are unset), so the
manual scenarios above remain **unverified**. The follow-up items linked in the
matrix are the owners of those desktop runs. This explicit negative record
prevents a headless cargo/test run from being mistaken for manual evidence.

## Audit notes

### DPI audit and manual evidence (WI-3061)

The source audit confirms one shared conversion contract across the three
backends:

- window and text/layout dimensions are represented as logical `Pixels`;
- renderer/window-system dimensions are represented as integer `DevicePixels`;
- conversion from logical to device pixels rounds `logical * scale_factor`, and
  conversion back divides by the same scale factor;
- a backend resize callback carries the logical size and the current scale
  factor, so consumers do not need to perform a second conversion.

Backend-specific observations are also recorded in the matrix: Windows reacts
to `WM_DPICHANGED`, X11 has explicit environment/RandR/Xft fallbacks, and
Wayland applies compositor output scale while handling configure-driven
resizes. These are intentionally not treated as identical native behavior.

Manual evidence collected on 2026-11-02 UTC: this worker environment has no
`DISPLAY`, `WAYLAND_DISPLAY`, or `XDG_SESSION_TYPE`, and it is not a Windows
host. Consequently, no native X11, Wayland, or DirectX interaction run is
claimed. The remaining native evidence needed to close this row is:

1. Windows DirectX: move a window between 100% and 150% monitors, resize it,
   and verify that the reported scale changes, logical content size remains
   stable, device-pixel renderer size changes, and text does not double-scale.
2. Linux X11: repeat at two RandR/Xft scales and with an explicit
   `GPUI_X11_SCALE_FACTOR`, verifying resize callback sizes and rendered text.
3. Linux Wayland: repeat across outputs with different compositor scales,
   including an interactive resize, verifying configure/scale ordering and
   text size.

For each run, record backend, scale settings, logical window size, device
window/render-target size, callback scale factor, and a screenshot of the same
text sample. Do not mark this row **verified-manually** until those observations
are attached to the work item.

- The source audit covered [`title_bar.rs`](../crates/fluent-app/src/title_bar.rs),
  [`pickers.rs`](../crates/fluent-app/src/pickers.rs), and the related
  [`window_title.rs`](../crates/fluent-app/src/window_title.rs). These files
  were read-only inputs to this documentation land.
- The picker tests verify filtering, cancellation/error propagation through the
  test platform, and preferred save extensions. They do not substitute for a
  Windows, X11, or Wayland desktop run.
- The scancode and high-contrast implementation boundaries are recorded in
  [`vendor/zed/PATCHES.md`](../vendor/zed/PATCHES.md). A cross-target cargo
  check is useful build evidence, but it is not a native interaction test.
- No macOS, web, or wasm behavior is included because those targets are outside
  this repository's support contract.

## Physical scancode evidence

WI-3058 audited the `key-scan-code` patch end to end and added a regression test
that injects a physical scancode whose logical key deliberately differs. The
test verifies that the physical code wins, the unchanged scancode is delivered
on press, repeat, and release, and only the repeated press has `is_held: true`.

The backend source contract is:

- Windows extracts the set-1 make code from `WM_KEYDOWN`/`WM_KEYUP` `lParam`
  bits 16–23 and adds `0xE000` when the extended-key bit is set.
- X11 converts the XKB keycode to evdev with `event.detail - 8` for both key
  directions.
- Wayland forwards the compositor's evdev key on both key directions and keeps
  that key on the repeat-timer event.
- IME-committed text, replayed keystrokes, and simulated input may carry
  `scan_code: None`; consumers must retain that fallback.

This session did not have native Windows, X11, and Wayland desktops available,
so it does not label any backend **verified-manually**. A native pass should
capture one ordinary key, one extended key, a held repeat, and its release on
each backend, confirming that the press/repeat/release scancode is stable and
that repeat is reported only on key-down.

## Clipboard audit

The vendored GPUI platform implementations were inspected on 2026-11-02. The
repository's platform test backend verifies clipboard round trips, and the
Windows backend has a native test covering Unicode text, metadata, and NUL
normalization. The Linux backends expose text and image MIME formats: X11 uses
selection requests and a clipboard-manager handover, while Wayland reads the
data offer through a bounded pipe and caches successful reads.

The asynchronous API is a ready task on these native backends and returns
`Ok(None)` when no item is available. The `ClipboardReadError` variants
(`Unavailable`, `Denied`, and `UnsupportedContent`) remain part of the shared
API for permission-gated platforms; no native Windows, X11, or Wayland failure
run was possible in this headless session.

| Target | Text | Image | Async/failure handling | Evidence |
|---|---|---|---|---|
| Windows DirectX 11 | source + backend test | source (PNG/GIF/JPEG/SVG) | synchronous native read wrapped as ready task; no native failure run | source/test verified |
| Linux X11 | source (UTF-8 selection targets) | source (GPUI image MIME targets) | selection timeout/error returns no item; no X server in session | source verified |
| Linux Wayland | source (text MIME offer) | source (image MIME offer) | pipe transfer errors return no item; no compositor in session | source verified |

Native host execution for all three targets remains follow-up work because this
session has no Windows desktop, X11 display, or Wayland compositor (`DISPLAY`
and `WAYLAND_DISPLAY` are unset).
