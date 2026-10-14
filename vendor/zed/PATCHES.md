# Local patches

Each local patch to this tree is a commit on the integrating branch, never on `zed-vendor`. List them here so a re-vendor only needs conflicts resolved in these files.

| patch | commit | files |
|---|---|---|
| `assets/fonts` | this commit | `vendor/zed/assets/fonts/` |
| `video-frames` | this commit | `vendor/zed/crates/gpui/src/platform.rs`, `vendor/zed/crates/gpui/src/scene.rs`, `vendor/zed/crates/gpui/src/window.rs`, `vendor/zed/crates/gpui_wgpu/src/wgpu_renderer.rs`, `vendor/zed/crates/gpui_wgpu/src/shaders.wgsl`, `vendor/zed/crates/gpui_wgpu/src/shaders_storage.wgsl`, `vendor/zed/crates/gpui_wgpu/src/shaders_webgl.wgsl` |
