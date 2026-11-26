# Vendored upstream

fluent-gpui builds on [GPUI](https://gpui.rs), the GPU UI framework from the
[Zed](https://github.com/zed-industries/zed) editor. Rather than depend on a
moving upstream, the repository carries a filtered snapshot of Zed `main`.

| Item | What it is |
|---|---|
| `vendor/zed/crates/*` | The Zed crate closure GPUI needs (`gpui`, `gpui_platform`, `gpui_linux`, `gpui_windows`, `sum_tree`, `collections`, `util`, `refineable`, `scheduler`, `http_client`, `macros`, `shared_string`, and others) |
| `vendor/zed/UPSTREAM.md` | The Zed commit, date and crate list of the current snapshot |
| `vendor/zed/PATCHES.md` | Every local patch: name, Cargo feature, upstream files it touches |
| `vendor/zed/README.md` | Notes on the vendored tree |
| `scripts/vendor-zed.py` | The script that produces the snapshot |

Only Windows and Linux crates are vendored; the macOS and web crates are left
out. The vendored crates keep Zed's Apache-2.0 `LICENSE-APACHE` and NOTICE files;
only Apache-2.0-compatible Zed crates are included, with no GPL or AGPL code.

## Patches

The patches add things GPUI does not have, kept small and, where possible, behind
Cargo features:

- `video-frames`, `video-frames-directx`: video texture upload and paint
  (`VideoSurface` in `fluent-primitives` wraps it).
- `backdrop-blur`: `backdrop_blur(radius, tint)`.

`vendor/zed/PATCHES.md` is the authoritative list. Files under `vendor/zed` are not
edited by hand unless that list names them. Contributors: read
[CONTRIBUTING.md](../CONTRIBUTING.md) before changing anything there.

## Upstream pulls

The snapshot is refreshed from upstream roughly monthly, and each refresh makes a
new release tag that records the Zed commit (`+zed.<short-sha>`). Maintainer
procedure: [contributing-internal.md](maintainers/contributing-internal.md).
