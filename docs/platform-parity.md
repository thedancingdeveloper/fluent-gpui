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
| Native title bar and window controls | `TitleBar` can replace native decorations in client/frameless mode; drag uses `start_window_move`; minimize/maximize/close are app controls. | Same Fluent custom-title-bar contract; native decoration policy remains compositor/window-manager controlled. | Same Fluent custom-title-bar contract; compositor controls native decoration policy. | **unverified** natively. The wrapper behavior is covered by source-level/test-platform contracts; run [WI-3055](https://vogt.sprooty.com/work/WI-3055). |
| Snap and caption buttons | Windows snap layouts and caption-button hit testing are OS behaviors; custom chrome must preserve the caption-button affordance and leave controls accessible. | No Windows snap-layout equivalent is promised; tiling/window-manager behavior is external to Fluent. | Tiling and server-side/client-side decoration behavior are compositor-specific; no Windows snap-layout equivalent is promised. | **divergent, unverified**. Follow [WI-3055](https://vogt.sprooty.com/work/WI-3055) for a native interaction pass. |
| System theme and accent | Fluent theme state supplies colors; `TitleBar` uses the Fluent accent colors. Automatic OS theme/accent synchronization is not claimed by the wrapper. | Same Fluent theme contract; desktop-environment synchronization is not claimed. | Same Fluent theme contract; desktop-environment synchronization is not claimed. | **unverified** for native synchronization. Follow [WI-3055](https://vogt.sprooty.com/work/WI-3055). |
| High-contrast detection | The `high-contrast` patch exposes OS state and change notifications, but this repository has no real-Windows run recorded. | Portal-backed `org.freedesktop.appearance` contrast is implemented by the Linux platform layer; desktop coverage is not recorded here. | Portal-backed `org.freedesktop.appearance` contrast is implemented by the Linux platform layer; compositor/portal availability varies. | **unverified** natively. Follow [WI-3056](https://vogt.sprooty.com/work/WI-3056). See [`PATCHES.md`](../vendor/zed/PATCHES.md) for the implementation boundary. |
| IME composition and committed text | GPUI separates composition/committed text from physical key events; native IME coverage is not recorded. | Same contract; toolkit/desktop IME setup is environment-dependent. | Same contract; toolkit/desktop IME setup is environment-dependent. | **unverified** natively. Follow [WI-3057](https://vogt.sprooty.com/work/WI-3057). |
| Physical scancodes and key repeat | Set-1 make code is derived from the Windows message; the key-scan-code patch is not verified on real Windows. | Scancode is derived from X11 event detail (`detail - 8`). | Wayland supplies the evdev key, including repeats. | **divergent, Windows unverified**. Follow [WI-3058](https://vogt.sprooty.com/work/WI-3058); implementation notes are in [`PATCHES.md`](../vendor/zed/PATCHES.md). |
| Clipboard | GPUI platform clipboard API provides text and image read/write; native integration is not manually verified here. | Clipboard ownership and persistence depend on the X11 selection/clipboard owner. | Clipboard access depends on compositor/data-device support and may be asynchronous. | **divergent, unverified**. Follow [WI-3059](https://vogt.sprooty.com/work/WI-3059). |
| File and folder pickers | `fluent-app::pickers` delegates to native GPUI prompts; test-platform filtering and cancellation are tested, native dialogs are not. | Native dialog availability depends on the desktop portal/toolkit environment. | Native dialog availability normally depends on the desktop portal/toolkit environment. | **unverified** natively. Follow [WI-3060](https://vogt.sprooty.com/work/WI-3060); see [picker contracts](consumer-contracts.md#3-file-and-folder-pickers). |
| DPI scaling and device pixels | `WM_DPICHANGED` updates the window scale factor; device-pixel resize sizes are converted to logical pixels before the resize callback. Text and layout continue to use logical pixels, while renderer bounds use device pixels. | The display scale is selected from `GPUI_X11_SCALE_FACTOR`, RandR, `Xft.dpi`, or the default `1.0` (in that order); X11 resize callbacks receive logical size and scale. | The compositor-provided output scale is applied to the surface; configure/resize handling is throttled during interactive resizing and callbacks receive logical size and scale. | **source-audited, unverified manually**. The shared contract is `logical = physical / scale` and `physical = round(logical * scale)`; see the manual evidence and test plan below. Follow [WI-3061](https://vogt.sprooty.com/work/WI-3061) for native runs. |
| X11 vs Wayland window/input differences | Not applicable. | X11 exposes server-side selections and X11 event details; window-manager policy is external. | Wayland delegates more policy to the compositor and uses evdev/data-device protocols; server-side assumptions must not be made. | **divergent** by design and **unverified** as a complete matrix. Follow [WI-3062](https://vogt.sprooty.com/work/WI-3062). |

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
