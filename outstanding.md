# Outstanding Work

Last updated: 2026-11-02

---

## Vendored GPUI Baseline

GPUI is vendored at `vendor/zed` (upstream Zed commit `ba63a022c9a2`). The `gpui` crate is consumed through the vendored closure. Named local patches are listed in `vendor/zed/PATCHES.md` and upstream state in `vendor/zed/UPSTREAM.md`.

Supported platforms are Windows and Linux only. macOS is out of scope.

- Linux renderer: `gpui_wgpu`.
- Windows renderer: `gpui_windows` (DirectX).

---

## VideoFrame Backend Support

Status: **re-applied on both backends, not runtime-validated** (WI-1348 Linux, WI-1434 Windows). The patches are `video-frames` and `video-frames-directx` in `vendor/zed/PATCHES.md`.

The API in the vendored `gpui`:

- `Window::alloc_video_texture(width, height) -> VideoTextureId`
- `Window::upload_video_frame(id, pixels) -> Result<()>`
- `Window::paint_video_frame(bounds, corner_radii, id)`
- `Window::free_video_texture(id)`

The data contract is tightly packed RGBA8 matching the allocation size. Ids are never reused, so a freed id stays stale and is rejected. `upload_video_frame` rejects a zero or over-8192 dimension, and the wgpu renderer skips a frame shorter than `w*h*4`. `fluent-primitives` wraps this as `VideoSurface`.

### Current Support Matrix

| Platform/backend | Status | Notes |
|------------------|--------|-------|
| Linux / wgpu (`gpui_wgpu`) | Basic Wayland software-renderer validation; hardware validation outstanding | Generated video example rendered under headless sway/software GL (WI-1438); alpha, lifecycle/leak, X11, and AMD Vulkan remain unverified |
| Windows / DirectX (`gpui_windows`) | Applied (WI-1434), **unverified on Windows** | Checked with `cargo check`/`clippy` for the Windows GNU target on Linux only; HLSL not compiled, not run |
| GPUI test platform | Unsupported by design | Test platform returns unsupported for video texture allocation |

### Validation Example

`vendor/zed/crates/gpui/examples/video_frame.rs` (patch `video-frames`). It rendered under headless Wayland/software GL in WI-1438; it has not been run on a hardware GPU in this repository's records.

### Remaining VideoFrame Work

1. Run the example on Linux and on Windows with GPU acceleration.
2. Validate invalid frame sizes and stale ids under runtime renderer conditions.
3. Validate alpha/opacity behaviour against normal image/sprite compositing.
4. Validate repeated allocate/upload/free/reallocate cycles for GPU resource leaks.
5. Validate device-loss recreation on Windows after real device reset conditions.

---

## Backdrop Blur Backend Support

Status: **re-applied, runtime validation incomplete (WI-1349)**. `gpui::backdrop_blur(radius, tint)` is present in the vendored `gpui` and the wgpu renderer. WI-1438's headless demo run did not exercise a real blur surface, so overlapping-surface visuals and hardware performance remain unverified.

The earlier design, to be re-applied:

- The renderer captures the current framebuffer into a texture, renders a single-pass cross-pattern blur, and samples that blurred backdrop under the optional tint.
- All blur quads in a batch share one blurred backdrop snapshot.
- This is an approximation, not a multi-pass Gaussian blur.
- `surface_blur_layer` exists in `fluent-core` as a token for tinted blur/glass surfaces.

### Current Support Matrix

| Platform/backend | Status | Notes |
|------------------|--------|-------|
| Linux / wgpu (`gpui_wgpu`) | Applied, not visually validated on hardware | Headless Wayland smoke passed, but the demo used only the blur-layer colour token |
| Windows / DirectX (`gpui_windows`) | Applied fallback, not runtime-validated | Requires a Windows runtime lane |
| Web | Not implemented | Web backend itself is future work |

### Remaining Backdrop Blur Work

1. Validate visual output on Linux with overlapping translucent surfaces.
2. Measure performance cost with multiple blur surfaces in a real app.
3. Validate the Windows fallback on Windows hardware.
4. Document fallback guidance for FluentGUI consumers before advertising blur as cross-platform.

---

## Vendor Maintenance

Outstanding maintenance tasks:

1. Keep the vendored tree and `vendor/zed/PATCHES.md` in step: each local patch is listed with its files.
2. Keep `vendor/zed/UPSTREAM.md` in sync with the script output when the vendored Zed rev changes.
3. Decide whether each local patch should be upstreamed, kept local, or removed after a future GPUI release.
4. Re-run workspace and targeted GPUI checks before publishing FluentGUI crates that rely on fork behaviour.
