# Outstanding Work

Last updated: 2026-05-28

---

## GPUI Fork Baseline

The repository vendors GPUI at `crates/gpui`. The local fork is currently versioned as `0.2.5` and consumed through the workspace dependency:

```toml
gpui = { path = "crates/gpui", version = "0.2" }
```

The fork contains FluentGUI-required renderer and platform work that has not been fully validated across all target platforms.

---

## VideoFrame Backend Support

The GPUI fork exposes a low-level streaming-frame path:

- `Window::alloc_video_texture(width, height) -> Result<VideoTextureId>`
- `Window::paint_video_frame(id, bounds, rgba8_data) -> Result<()>`
- `Window::free_video_texture(id)`

The data contract is tightly packed RGBA8 matching the allocation size. Texture ids are generation-checked so stale/freed ids are rejected or skipped instead of aliasing newly allocated textures.

### Current Support Matrix

| Platform/backend | Status | Notes |
|------------------|--------|-------|
| Linux / Blade | Implemented, pending real hardware validation | Allocation, upload, scene batching, shader, draw, free, and stale-id handling exist |
| Windows / DirectX | Implemented, pending Windows GPU validation | DirectX texture allocation, `UpdateSubresource` upload, shader, pipeline, draw, and device-loss recreation paths exist |
| macOS / Metal | Not implemented | `alloc_video_texture` returns an unsupported error and `VideoFrame` batches are skipped |
| macOS / Blade feature path | Not validated | Code is closer to the Blade implementation but is not confirmed as a supported product path |
| GPUI test platform | Unsupported by design | Test platform returns unsupported for video texture allocation |

### Validation Example

`crates/gpui/examples/video_frame.rs` allocates a 256x144 texture, uploads changing RGBA frames, paints to a 512x288 canvas, and frees the texture on view release.

Run:

```sh
cargo run -p gpui --example video_frame
```

### Local Validation Already Performed

Earlier local validation in this Linux environment covered:

```sh
cargo fmt --check
cargo check -p gpui
cargo check -p gpui --target x86_64-pc-windows-gnu
cargo test -p gpui video_
cargo test -p gpui --target x86_64-pc-windows-gnu video_ --no-run
cargo check --workspace
cargo test --workspace
```

These checks do not replace runtime validation on Windows/DirectX or macOS/Metal.

### Remaining VideoFrame Work

1. Run `cargo run -p gpui --example video_frame` on Linux/Blade target hardware.
2. Run the same example on Windows with GPU acceleration and the DirectX backend.
3. Implement macOS/Metal texture allocation, upload, shader, pipeline, draw, and free paths, or document Metal as unsupported for the first release that exposes video surfaces.
4. Validate invalid frame sizes and stale ids under runtime renderer conditions.
5. Validate alpha/opacity behavior against normal image/sprite compositing.
6. Validate repeated allocate/upload/free/reallocate cycles for GPU resource leaks.
7. Validate device-loss recreation on Windows after real device reset conditions.

---

## Backdrop Blur Backend Support

The GPUI fork adds `gpui::backdrop_blur(radius, tint)` as a `Background` variant.

Current implementation notes:

- The Blade/WGSL path captures the current framebuffer into a texture, renders a single-pass cross-pattern blur, and samples that blurred backdrop under the optional tint.
- All blur quads in a batch share one blurred backdrop snapshot.
- This is an approximation, not a multi-pass Gaussian blur.
- `surface_blur_layer` exists in `fluent-core` as a token for tinted blur/glass surfaces.

### Current Support Matrix

| Platform/backend | Status | Notes |
|------------------|--------|-------|
| Linux / Blade | Implemented, pending visual/performance validation | Main implementation path |
| Windows / DirectX | Not documented as implemented | Needs renderer audit before claiming support |
| macOS / Metal | Not documented as implemented | Needs renderer audit before claiming support |
| Web | Not implemented | Web backend itself is future work |

### Remaining Backdrop Blur Work

1. Validate visual output on Linux/Blade with overlapping translucent surfaces.
2. Measure performance cost with multiple blur surfaces in a real app.
3. Decide whether Windows/DirectX and macOS/Metal should implement equivalent blur or fall back to tinted solid surfaces.
4. Document fallback guidance for FluentGUI consumers before advertising blur as cross-platform.

---

## GPUI Fork Maintenance

Outstanding maintenance tasks:

1. Keep `crates/gpui/Cargo.toml` and `crates/gpui/Cargo.toml.orig` aligned when the vendored fork version changes.
2. Keep root `Cargo.toml` dependency range compatible with the fork version.
3. Track local GPUI patches in commit messages and docs so downstream apps know why the fork exists.
4. Decide whether each local patch should be upstreamed, kept local, or removed after a future GPUI release.
5. Re-run workspace and targeted GPUI checks before publishing FluentGUI crates that rely on fork behavior.

---

## Downstream Application Parity

Remaining rdpapp parity is tracked downstream, but these items can still drive reusable framework work if repeated patterns emerge:

- full SFTP browser;
- richer credential CRUD;
- offline/deactivation licensing flows;
- pop-out windows;
- import/export UI polish;
- broader RDP/VNC controls.

Do not implement these directly in FluentGUI unless they clearly reduce to generic primitives, tokens, or shell behavior.
