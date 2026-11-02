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
