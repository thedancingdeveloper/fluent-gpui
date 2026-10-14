# Local patches

Each local patch to this tree is a commit on the integrating branch, never on `zed-vendor`. List them here so a re-vendor only needs conflicts resolved in these files.

| patch | commit | files |
|---|---|---|
| `assets/fonts` | this commit | `vendor/zed/assets/fonts/` |
| element visual scale | this commit | `vendor/zed/crates/gpui/src/style.rs`, `styled.rs`, `window.rs`, `elements/div.rs`, `examples/scale.rs`, `Cargo.toml` |
