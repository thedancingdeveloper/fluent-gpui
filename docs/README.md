# Documentation

## Using fluent-gpui

- [Quick start](quick-start.md): add the crates, open a themed window, add a form, a ribbon and a data table.
- [Component index](components.md): every public component with its states, keyboard contract and gallery page.
- [Design tokens](tokens.md): colours, typography, spacing and motion tokens.
- [Architecture and consumer contract](architecture.md): layering, installing by git tag, versions, feature flags, build requirements, accessibility.
- [Native integration contracts](consumer-contracts.md): what apps can rely on for the video surface, terminal and file pickers.
- [Gallery guide](gallery.md): using the gallery as a visual regression matrix.
- [Visual targets](VISUAL_TARGETS.md): the visual references that shape the components.
- [Performance budgets](performance.md): render-cost budgets and how to run the benchmarks.
- [Vendored upstream](vendoring.md): the vendored GPUI snapshot and its patches.
- [Changelog](../CHANGELOG.md): release notes and the 4.0.0 migration notes.

## Maintaining it

- [Contributor guide](../CONTRIBUTING.md): set up, checks, screenshots and the pull request checklist.
- [Maintainer workflow](maintainers/contributing-internal.md): how changes land, upstream pulls, release rules and review.
- [How changes land (graft)](maintainers/graft.md): the integration line, the mirror and the batch flow.
- [Project status and known gaps](maintainers/status.md): what is validated at runtime and what is outstanding.
- [Linux runtime validation](maintainers/linux-runtime-validation.md): the headless Linux validation record.
- [Windows runtime validation](maintainers/windows-runtime-validation.md): the Windows validation record.
- [Platform parity](maintainers/platform-parity.md): the Windows and Linux behaviour matrix and its evidence.
- [Native chrome and system theme parity](maintainers/NATIVE_CHROME_PARITY.md): title bar, window chrome and system theme behaviour by platform.
- [gpui symbol drift](maintainers/gpui-symbol-drift.md): fluent crates against the vendored gpui.
- [Releasing](maintainers/RELEASING.md): how releases are cut.
- [Third-party licences](maintainers/third-party-licences.md): licences of bundled and vendored third-party material.
