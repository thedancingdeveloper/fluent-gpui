# Local patches

Each local patch to this tree is a commit on the integrating branch, never on `zed-vendor`. List them here so a re-vendor only needs conflicts resolved in these files.

| patch | commit | files |
|---|---|---|
| `assets/fonts` | this commit | `vendor/zed/assets/fonts/` |
| `backdrop-blur` | this commit | `vendor/zed/crates/gpui/src/color.rs`, `vendor/zed/crates/gpui/src/style.rs`, `vendor/zed/crates/gpui/src/styled.rs`, `vendor/zed/crates/gpui_wgpu/src/shaders.wgsl`, `vendor/zed/crates/gpui_wgpu/src/wgpu_renderer.rs`, `vendor/zed/crates/gpui_windows/src/directx_renderer.rs`, `vendor/zed/crates/gpui_windows/src/shaders.hlsl` |
