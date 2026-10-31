# Local patches

Each local patch to this tree is a land on the working line (`backlog-1`), never on the `zed-vendor` line. List them here so a re-vendor only needs conflicts resolved in these files.

| patch | commit | files |
|---|---|---|
| `trim-ios` (WI-2728): drop the `target_os = "ios"` arm of `current_platform`, since `gpui_ios` is no longer vendored | WI-2728 land on `backlog-1` | `crates/gpui_platform/src/gpui_platform.rs` |

`gpui_ios` and the `runtime_shaders` feature are removed by `scripts/vendor-zed.py` itself (`EXCLUDED`, `DROPPED_FEATURES`), so they are not patches. A re-vendor keeps that output and then reapplies `trim-ios`.
