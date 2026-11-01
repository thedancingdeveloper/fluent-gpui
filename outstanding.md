# Outstanding Work

Last updated: 2026-11-01

---

## Vendored GPUI Baseline

GPUI is vendored at `vendor/zed` (upstream Zed commit `ba63a022c9a2`). The `gpui` crate is consumed through the vendored closure. Named local patches are listed in `vendor/zed/PATCHES.md` and upstream state in `vendor/zed/UPSTREAM.md`.

Supported platforms are Windows and Linux only. macOS is out of scope.

- Linux renderer: `gpui_wgpu`.
- Windows renderer: `gpui_windows` (DirectX).

---

## VideoFrame Backend Support

Status: **not yet re-applied (WI-1348, WI-1434)**. The `alloc_video_texture`, `paint_video_frame` and `free_video_texture` APIs are not present in the vendored `gpui`. The earlier low-level streaming-frame path was written against a renderer that is no longer vendored.

The intended API contract, to be re-applied:

- `Window::alloc_video_texture(width, height) -> Result<VideoTextureId>`
- `Window::paint_video_frame(id, bounds, rgba8_data) -> Result<()>`
- `Window::free_video_texture(id)`

The data contract is tightly packed RGBA8 matching the allocation size. Texture ids are generation-checked so stale or freed ids are rejected or skipped instead of aliasing newly allocated textures.

### Current Support Matrix

| Platform/backend | Status | Notes |
|------------------|--------|-------|
| Linux / wgpu (`gpui_wgpu`) | Not yet re-applied (WI-1348) | No video texture path in the vendored tree |
| Windows / DirectX (`gpui_windows`) | Not yet re-applied (WI-1434) | No video texture path in the vendored tree |
| GPUI test platform | Unsupported by design | Test platform returns unsupported for video texture allocation |

### Validation Example

The former video-frame example no longer exists. It is to be recreated with WI-1348. It should allocate a 256x144 texture, upload changing RGBA frames, paint to a 512x288 canvas, and free the texture on view release.

### Remaining VideoFrame Work

1. Re-apply the `gpui` video-frame API and rewrite the Linux renderer path on `gpui_wgpu` (WI-1348).
2. Re-apply the DirectX video-frame patch onto `gpui_windows` (WI-1434).
3. Run the validation example on Linux and on Windows with GPU acceleration.
4. Validate invalid frame sizes and stale ids under runtime renderer conditions.
5. Validate alpha/opacity behaviour against normal image/sprite compositing.
6. Validate repeated allocate/upload/free/reallocate cycles for GPU resource leaks.
7. Validate device-loss recreation on Windows after real device reset conditions.

---

## Backdrop Blur Backend Support

Status: **not yet re-applied (WI-1349)**. `gpui::backdrop_blur(radius, tint)` as a `Background` variant is not present in the vendored `gpui`. The earlier implementation is not carried into `gpui_wgpu` or `gpui_windows`.

The earlier design, to be re-applied:

- The renderer captures the current framebuffer into a texture, renders a single-pass cross-pattern blur, and samples that blurred backdrop under the optional tint.
- All blur quads in a batch share one blurred backdrop snapshot.
- This is an approximation, not a multi-pass Gaussian blur.
- `surface_blur_layer` exists in `fluent-core` as a token for tinted blur/glass surfaces.

### Current Support Matrix

| Platform/backend | Status | Notes |
|------------------|--------|-------|
| Linux / wgpu (`gpui_wgpu`) | Not yet re-applied (WI-1349) | Main target; the renderer is rewritten on `gpui_wgpu` |
| Windows / DirectX (`gpui_windows`) | Not yet re-applied (WI-1349) | Needs the Windows fallback or equivalent blur |
| Web | Not implemented | Web backend itself is future work |

### Remaining Backdrop Blur Work

1. Re-apply the `Background` API and rewrite the renderer on `gpui_wgpu` (WI-1349).
2. Implement the Windows fallback (WI-1349).
3. Validate visual output on Linux with overlapping translucent surfaces.
4. Measure performance cost with multiple blur surfaces in a real app.
5. Document fallback guidance for FluentGUI consumers before advertising blur as cross-platform.

---

## Vendor Maintenance

Outstanding maintenance tasks:

1. Keep the vendored tree and `vendor/zed/PATCHES.md` in step: each local patch is listed with its files.
2. Keep `vendor/zed/UPSTREAM.md` in sync with the script output when the vendored Zed rev changes.
3. Decide whether each local patch should be upstreamed, kept local, or removed after a future GPUI release.
4. Re-run workspace and targeted GPUI checks before publishing FluentGUI crates that rely on fork behaviour.
