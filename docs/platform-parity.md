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
| High-contrast detection | `SPI_GETHIGHCONTRAST` reads `HCF_HIGHCONTRASTON`; `WM_SETTINGCHANGE` for that action notifies GPUI observers. | The xdg-desktop-portal `org.freedesktop.appearance` `contrast` setting is read at startup and on setting changes; values `>= 2` mean high contrast. | The same portal contract is consumed by the Wayland event loop; the compositor/portal must provide the setting. | **source-audited, unverified manually**. The shared GPUI contract only notifies when the boolean changes. See the [high-contrast evidence](#high-contrast-evidence-wi-3056) and [`PATCHES.md`](../vendor/zed/PATCHES.md). |
| IME composition and committed text | `WM_IME_COMPOSITION` sends committed `GCS_RESULTSTR` text through `replace_text_in_range`, and the active `GCS_COMPSTR` through `replace_and_mark_text_in_range`; the IME context is enabled only while the focused input accepts text. | XIM preedit callbacks map to marked text and XIM commit callbacks map to committed text; XKB compose sequences use the same marked/committed path, while the physical key event remains separate. | `zwp_text_input_v3` preedit maps to marked text and commit strings map to committed text; the text-input state is enabled only for an accepting input and committed before dispatch. | **source-audited; unverified manually**. The shared `TextInput` handler replaces marked text, clears the mark on commit, and preserves UTF-16 ranges. See [IME evidence](#ime-evidence). |
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

## High-contrast evidence (WI-3056)

The source audit on 2026-11-02 confirms the following end-to-end contract:

- **Windows DirectX:** `gpui_windows::system_settings::system_high_contrast`
  calls `SystemParametersInfoW(SPI_GETHIGHCONTRAST)` and checks
  `HCF_HIGHCONTRASTON`. The window message handler watches `WM_SETTINGCHANGE`
  for `SPI_GETHIGHCONTRAST` and invokes the registered callback. GPUI reads
  the setting again when the application asks for `high_contrast()` rather
  than relying on a stale cached value.
- **Linux X11:** the xdg-desktop-portal listener reads
  `org.freedesktop.appearance/contrast` initially and subscribes to its
  setting-change signal. `contrast >= 2` is converted to `true`; the X11
  client updates its state and invokes the callback only when that boolean
  differs from the previous value.
- **Linux Wayland:** the same portal event source is consumed by the Wayland
  client. It applies the same `contrast >= 2` conversion and changed-value
  guard before invoking the callback.
- **Fluent integration:** `FluentApp` applies `Theme::high_contrast()` at
  startup when the system reports high contrast and subscribes to the GPUI
  change notification. Turning the setting off restores the configured light
  or dark system theme.

The source boundary is the `high-contrast` patch listed in
[`vendor/zed/PATCHES.md`](../vendor/zed/PATCHES.md). The relevant files are
`gpui_windows/src/system_settings.rs`, `gpui_windows/src/platform.rs`,
`gpui_windows/src/events.rs`, `gpui_linux/src/linux/xdg_desktop_portal.rs`,
`gpui_linux/src/linux/x11/client.rs`, `gpui_linux/src/linux/wayland/client.rs`,
`gpui/src/platform.rs`, `gpui/src/app.rs`, and
`crates/fluent-app/src/app_builder.rs`.

### Manual test record

| Backend | Host/session evidence | Result | Status |
|---|---|---|---|
| Windows DirectX 11 | No Windows host or desktop session is available in this worker. | Not run; `SPI_GETHIGHCONTRAST` and `WM_SETTINGCHANGE` require a Windows desktop. | **source-audited, unverified manually** |
| Linux X11 | `DISPLAY` is unset in this worker. | Not run; no X11 display or portal signal source is available. | **source-audited, unverified manually** |
| Linux Wayland | `WAYLAND_DISPLAY` and `XDG_SESSION_TYPE` are unset in this worker. | Not run; no Wayland compositor or portal signal source is available. | **source-audited, unverified manually** |

The native follow-up procedure is deliberately recorded here rather than
claiming that a headless test is equivalent to a desktop run: launch the
gallery with the same build, enable high contrast, verify the initial
`high_contrast()` state and Fluent high-contrast tokens, toggle the setting
off and on, and verify exactly one theme update per boolean transition. On
Linux, record the desktop portal's `contrast` value (including a value below
2 and a value of 2 or greater); on Windows, record the Windows contrast
scheme and the `WM_SETTINGCHANGE` transition. Attach the host/session details
and observations to WI-3056 before changing any matrix cell to
**verified-manually**.

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

## IME evidence

The GPUI input contract and the Fluent `TextInput` adapter were source-audited
on 2026-11-02. All three supported native paths deliver composition separately
from physical key events:

- **Windows DirectX:** `WM_IME_COMPOSITION` reads `GCS_COMPSTR` as the active
  composition and `GCS_RESULTSTR` as committed text. The active string is sent
  to `replace_and_mark_text_in_range`; the result string is sent to
  `replace_text_in_range`. The IME context is associated only while the focused
  window has an input handler that accepts text, and an inactive context is
  completed before it is detached.
- **Linux X11:** XIM preedit callbacks send marked text and XIM commit callbacks
  send committed text. The XKB compose path also emits marked text while a
  sequence is in progress and commits the composed character separately. X11
  key events remain distinct from these text callbacks.
- **Linux Wayland:** `zwp_text_input_v3` sends preedit text and commit strings
  independently. GPUI maps these to marked and committed text respectively,
  enables the protocol only for an accepting input handler, updates the IME
  cursor rectangle, and commits the text-input state.

`TextInput` replaces the current marked range during composition, underlines the
marked range, clears it when committed text arrives, and converts the GPUI
UTF-16 ranges at the input boundary. This is the parity contract: composition
may be rendered differently by each native backend, but the consumer receives
the same marked/committed distinction and does not need to process IME text as
physical key presses. The existing test-platform coverage exercises ordinary
editing and UTF-16 offset handling; it does not emulate native IME callbacks.

No native IME run can be claimed from this worker: `DISPLAY`,
`WAYLAND_DISPLAY`, and `XDG_SESSION_TYPE` are unset, and the worker is not a
Windows host. Therefore Windows DirectX, Linux X11, and Linux Wayland remain
**unverified manually**. A native follow-up should use the same `TextInput`
scenario on each backend (Japanese hiragana-to-kanji conversion or an
equivalent composition, cancellation, reconversion, and direct committed text),
record the desktop/IME, whether preedit was visibly marked, the final committed
string, and whether any physical key event was emitted for the committed text.
For Linux also record whether XIM or Wayland text-input-v3 was available; for
Windows record the IME name and whether focus changes commit or cancel the
composition.

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
