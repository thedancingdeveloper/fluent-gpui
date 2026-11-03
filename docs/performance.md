# Render-cost budgets

These opt-in Criterion benches build gallery-representative models and count fixture-level
element builds and interaction notifications. They are deliberately outside required CI; run
`cargo bench -p fluent-primitives` and `cargo bench -p fluent-layout` locally.

| Workload | Input | Build budget | `cx.notify` budget |
| --- | --- | ---: | ---: |
| Button grid | 10 x 10 buttons | 100 | 1 |
| TextInput | one edited field | 1 | 1 |
| DataTable | 1,000 rows | 1,000 | 1 |
| TerminalView | 200 x 50 cells | 1 observed grid | 1 |
| VirtualList (WI-3033) | 100,000 rows | 26 viewport/overscan rows | 1 |

`fluent-test-utils::RenderAudit` is opt-in benchmark instrumentation; no library behaviour or
notification path is changed. The VirtualList fixture makes the lazy-rendering property explicit:
the 100k-row input builds only a viewport-sized slice. The initial audit found no redundant
notify/re-render. No follow-up is required unless a future run exceeds a budget or observes more
than one notification for an unchanged interaction.

## Vendored GPUI baseline (WI-2730)

The Linux baseline below was measured from a clean target directory on 2026-11-03 with
`CARGO_BUILD_JOBS=6`, the pinned toolchain from `rust-toolchain.toml`, and no `RUSTC_WRAPPER`.
The command was:

```sh
CARGO_BUILD_JOBS=6 time -p cargo build --manifest-path vendor/zed/crates/gpui/Cargo.toml
```

| Measurement | Result |
| --- | ---: |
| Clean build of `gpui` and `gpui_platform` | 54.71 s |

This was an un-cached build into a fresh `CARGO_TARGET_DIR` on Linux; the `gpui` manifest pulls
in the `gpui_platform` path dependency. Do not compare it with
cached CI builds. `cargo deny check advisories licenses bans sources` passed after applying the
explicit exceptions above. `cargo audit` reported 5 vulnerabilities and 12 allowed warnings:
the vulnerabilities were `crossbeam-epoch` (RUSTSEC-2026-0204), `quick-xml` (RUSTSEC-2026-0194
and RUSTSEC-2026-0195), `anyhow` (RUSTSEC-2026-0190), `event-listener` (RUSTSEC-2026-0221),
`memmap2` (RUSTSEC-2026-0186), and `tokio` (RUSTSEC-2026-0386); the warnings included
unmaintained `paste`, `rustybuzz`, `ttf-parser`, `zbus_names`, `zvariant`, and
`zvariant_derive`, plus yanked `async-executor` and `spin`. The audit job remains informational
until the pinned snapshot is refreshed.
