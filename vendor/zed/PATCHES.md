# Local patches

Each local patch to this tree is a commit on the integrating branch, never on `zed-vendor`. List them here so a re-vendor only needs conflicts resolved in these files.

| patch | commit | files |
|---|---|---|
| `assets/fonts` | 9b658ae | `vendor/zed/assets/fonts/` |

## Feature trims considered (WI-1424)

Reviewed against the snapshot, not applied, because each one is already off
for a Linux or Windows build:

- `screen-capture` is optional on `gpui`, `gpui_linux` and `gpui_windows`,
  and nothing in the fluent crates enables it, so `scap` is not compiled.
- `bench-support` is optional and only required by the two benches in
  `gpui_platform`, which are not part of `cargo check` or `cargo test`.
- `inspector` is optional. The fluent crates only name the
  `InspectorElementId` type, which exists without the feature.
- The wasm crates (`gpui_web`, `scheduler`'s `wasm-threads`) are
  target-gated and are not built for the Linux or Windows targets.

`windows-manifest` stays: `gpui_platform` enables it on Windows, and the
plan calls that out as wanted. macOS features were not touched.
