# Fluent on egui: assessment of the storytold "Craft" apps and options for fluent-gpui

Status: research and design for the operator, 2026-10-14. No code, repository, work item or PR
comes with this document. Session `ses_01M4YA9A18B1ZJDTT9NZQ1BTE5`.

## Decision (operator, 2026-10-14)

After discussing the egui and gpui comparison, the operator chose:

1. **egui is the primary direction.** A new repository (working name `fluent2-egui`) becomes
   the Fluent library going forward. This replaces the staged option C recommended in sections
   0 and 3.3, which are kept below as the original analysis.
2. **The fluent-gpui migration stack (PRs #1–#17, WI-1346) is parked.** The PRs stay open and
   unmerged. fluent-gpui stays on its Oct-2025 gpui base (v2.4, `f601e54`) and gets fixes
   only. The consumers (`email-desktop-fluent`, `rdpapp/migration-tool`) stay pinned and are
   not ported.

Facts that moved the recommendation from "C staged" to "egui primary", found after the first
draft:

- **No estate consumer uses the gpui-only features.** `email-desktop-fluent` (19.6k lines) and
  `rdpapp/migration-tool` (26.4k lines) never call `backdrop_blur`, the video-frame API or
  element scale, and both pin v2.4. They gain nothing from the migration.
- **The migration's port PRs are tiny** (+6, +72, +1, +16, +15, +6 lines). Most of the stack is
  the 208k-line copy of zed's gpui source (#5) and renderer patches with no current user. The
  ongoing cost is that copy, re-copying it on every upstream change, and git-tag-only
  distribution. None of that serves the goal of a library others can `cargo add`.
- For the stated goal, distribution decides it. A library that cannot be published to
  crates.io and tracks Zed's internal churn will not be adopted by other developers. egui has
  crates.io, a slow release cadence, web/wasm, AccessKit, kittest, and a proven record of
  agent-built Office-style UIs (storytold).

Follow-up (same day): the repository was created as **https://github.com/thedancingdeveloper/fluent2-egui**
(MIT OR Apache-2.0; scaffold, conventions, upstream tracking via `upstream.toml` and
`scripts/upstream-check.py`). Its backlog is the Vogt initiative "fluent2-egui: Fluent 2 component
library for egui" (WI-1531 to WI-1589, indexed in that repo's `docs/BACKLOG.md`), written so
small-model workers can take items without design decisions.

What changes in this document as a result: section 5's stage-2 items (tokens inside
fluent-gpui) move into the new repository. Items that would make `fluent-core` consume the
tokens are dropped. Section 4 (repo design) applies as written.

## 0. Summary

- **Licences permit extraction.** Every storytold repo reviewed is `MIT OR Apache-2.0` and ships
  no Microsoft assets: no Fluent icons, no Segoe, no Office artwork. The only restricted material
  is the ArtCraft brand files in `docs/brand/`, which must never be copied. Fluent System Icons
  (MIT), the Fluent UI token sources (MIT) and Selawik (OFL-1.1) can all be redistributed.
  Segoe UI and Segoe Fluent Icons cannot.
- **The brief's "wordcraft" link is wrong.** `PAIR-code/wordcraft` is Google's 2023 TypeScript
  AI co-writing prototype (Apache-2.0, 20 commits, idle since March 2024). It is not part of
  this family. The Word-like app is **`storytold/wordcraft`** (Rust, egui).
- **There is almost no shared code, and copy-paste is not the reason.** About 2–3% of
  normalised UI lines are common between any two of the three apps, and nearly all of those are
  in the credits window, printing glue and desktop-theme detection. Each app has its own
  independently written ribbon, token struct and painter-drawn icon set. The operator's read
  ("no cohesive UI library") is correct. However, storytold has started
  [`craft-libs`](https://github.com/storytold/craft-libs) and its `craft-ui` crate (3.6k lines,
  started 2026-10-09). Its stated design rule is that a universal token struct or theme enum is
  "unnecessary". Its AGENTS.md says "repos don't share code", and its absolute asset rule
  forbids any Microsoft iconography.
- **The name `fluent-egui` is taken.** `laststudio/fluent-egui` (crates.io `fluent-egui` 0.1.1,
  MIT OR Apache-2.0, egui 0.35, ~29k lines, 12 commits, 116 downloads) is a WinUI-3-style egui
  widget library. It has no ribbon, no backstage and no keytips.
- **Recommendation: option C, staged.** Keep fluent-gpui and let the pending PR stack (#1–#17)
  proceed unchanged. After that stack merges, extract a renderer-neutral **`fluent2-tokens`**
  crate (Fluent 2 tokens, type ramp, metrics, icon catalogue) inside this workspace. Start an
  egui renderer only once a consumer needs it (web/wasm, or an app that can't take the gpui
  vendoring). Do not pursue option D for Fluent work: storytold's rules exclude it.

## 1. What is in the repositories

Revisions reviewed (cloned into a scratch directory, never into this project):

| repo | revision | first commit | commits | Rust lines | stage |
|---|---|---|---:|---:|---|
| storytold/wordcraft | `cf8aea688620` | 2026-10-05 | 607 | 228k | alpha (own ROADMAP) |
| storytold/gridcraft | `d9d09f0baa91` | 2026-10-05 | 1,014 | 134k | alpha |
| storytold/deckcraft | `ea904591a59d` | 2026-10-05 | 846 | 202k | alpha |
| storytold/craft-libs | `457a07880969` | 2026-10-09 | 106 | 20k | 0.1.0 |
| laststudio/fluent-egui (prior art) | `ec6e3a3f6cf6` | 2026-07-19 | 12 | 42k incl. 13k demo | 0.1.1 |
| PAIR-code/wordcraft (not in the family) | — | 2023-05-04 | 20 | 0 (TypeScript) | idle |

The storytold organisation has 14+ "Craft" apps (PhotoCraft, VectorCraft, DesignCraft, PdfCraft,
FilmCraft, EffectCraft, LightCraft, SoundCraft, CADCraft, FigCraft and others). Every one of them
was pushed in the last day, and `craft-libs` audits twelve of them.

### 1.1 Architecture (identical across the three apps)

- Cargo workspace, edition 2024, Rust ≥ 1.90, `unsafe_code = "forbid"`, no-panic clippy denies.
- Strict layering enforced by `cargo xtask layers`: format/model/engine crates at the bottom,
  `crates/ui-egui` above them (L6), `apps/<app>`, `-cli` and `-web` (wasm) on top. **Nothing
  below the UI crate depends on egui.** The UI crate is described as "swappable".
- "Everything is a command." The UI calls `app.run("home.bold", params)`. The same command ids are
  reachable from a JSON-lines control port, the CLI and an MCP server.
- **egui / eframe 0.36** (latest is 0.36.2, released 2026-09-08). `egui_kittest` 0.36 is used
  for UI tests with wgpu snapshot rendering.
- Title bar: on macOS, `with_fullsize_content_view(true)` with the traffic lights inset. On
  Windows and Linux the OS decorations stay, with an in-app title row and Quick Access Toolbar
  (QAT) below them. None of the apps does client-side decorations.

### 1.2 UI crate sizes and Fluent-styled pieces

| piece | wordcraft | gridcraft | deckcraft |
|---|---|---|---|
| `ui-egui` total lines | 51.0k | 35.7k | 30.8k |
| ribbon | `ribbon.rs` 2,965 + `widgets.rs` 1,063 | `ribbon.rs` 2,516 (includes title bar/QAT and shortcuts) + `widgets.rs` 364 | `ribbon.rs` 3,619 (includes title bar) + `widgets.rs` 569 |
| tokens | `theme.rs` 836: `Tokens` with 32 colour fields, `Appearance` System/Light/Dark, `InterfaceSize` 50–300 %, `Animations` | `theme.rs` 937: 38 fields, grid-specific (headers, selection, info bars) | `theme.rs` 268: 24 fields, `Brightness` Light/Dark |
| icons | `icons.rs` 1,472: painter-drawn on a 20×20 grid, two colours | `icons.rs` 756: painter-drawn on a 16×16 grid (32×32 large) | `icons.rs` 2,471: painter-drawn on a 24×24 grid. Lucide (ISC/MIT) is used for *document* icons only |
| keytips (Alt badges) | `keytips.rs` 1,190 | `keytips.rs` 227 + generated table 373 | — |
| backstage / File | `backstage.rs` 860 | `backstage.rs` 1,088 | `backstage.rs` 1,382 |
| dialogs | `dialogs*.rs` ≈ 11k across 9 files | `dialogs.rs` 2,741 | `dialogs.rs` 3,583 |
| panes / task panes | `panes.rs` 807 | `panes.rs` 288 | `panes.rs` 1,574 |
| grid / data view | tables inside the document canvas | `grid.rs` 3,630 (virtualised sheet, headers, freeze panes, fill handle) | slide sorter, thumbnails |
| other chrome | mini toolbar, status bar, scroll, a11y (`a11y.rs` 710) | formula bar, sheet tabs, outline, status bar | QAT, status bar, toast, notes |
| high contrast | none (1 mention) | none | none |
| accent | `#3B5BDB` (own blue) | `#107C41` (Excel brand green) | `#D95A1A` (orange) |
| fonts | Inter, Source Sans/Serif 3/4, JetBrains Mono (OFL, bundled) + craft-fonts | OS UI font at runtime (Segoe UI on Windows, never bundled) + craft-fonts | Inter (craft-fonts) + egui defaults |

How Fluent-like they are: they reproduce the *Office* layout (ribbon, groups, galleries, split
buttons, contextual tabs, backstage, keytips, QAT). Their neutral greys land on Fluent 2 values:
all three use `#242424` for text, and two use `#616161` for dim text. Those match Fluent's
`colorNeutralForeground1` and `colorNeutralForeground3`. **None uses Fluent token names, Fluent
icons, Fluent type ramps or Fluent corner, spacing or elevation scales.** Each app owns its own
look on purpose: the clean-room rules forbid copying vendor artwork or themes.

### 1.3 Shared code: duplication, forks and shared crates

Measured with a normalised-line and 4-line-shingle comparison across the three `ui-egui/src`
trees (lines of 25+ characters, comments and braces removed):

| pair | identical lines | as a share of the smaller crate |
|---|---:|---:|
| wordcraft ∩ gridcraft | 423 | 3.3 % |
| wordcraft ∩ deckcraft | 406 | 3.2 % |
| gridcraft ∩ deckcraft | 357 | 2.8 % |
| all three | 240 | — |

Shingle matches between files are limited to `credits.rs` (a near-identical About/Contributors
window), `printing.rs` (glue around `craft-ui::printing`), `desktop_theme.rs` (Linux portal
dark-mode query) and a small part of `control.rs`. **The ribbons, themes, icons, backstages and
dialogs are independent implementations, not forks of each other.** They differ in signatures,
too. Wordcraft's `widgets::big(ui, app, icon: &str, …)` dispatches by command id through `&mut
WordApp`. Gridcraft's `big_button(ui, icon: Icon, label, tip, dropdown)` returns a `Response`.
Deckcraft's `big_button_ex(…) -> (Response, Option<Response>)` returns the split-button halves.

Shared crates:

- `craft-libs/crates/ui` (`craft-ui` 0.1.0, git-pinned, not on crates.io): SVG icon catalogue,
  `IconButton`, accessible tabs and ribbon-tab interaction, toolbar fitting, split geometry,
  resize handles, a transactional docking model and surface, print settings, and a
  five-state `WidgetPalette`. **88 tests pass** (`cargo test -p craft-ui` on this host, Rust
  1.99). Its docs (`ui-components-audit.md`, `theme-infrastructure.md`) are careful and worth
  reading.
- Adoption at HEAD: wordcraft and gridcraft pin `craft-ui` at `d27646a5`, but **use only
  `craft_ui::printing`**. Deckcraft does not depend on it. The audit records ribbon-tab trials
  for all three, but they are not in these apps' `main`.
- Stated boundary (`theme-infrastructure.md`): "A universal token structure, theme enum,
  application identifier or global context key is unnecessary." Each app keeps its own palette,
  and shared components take small style structs. A Fluent design system is the opposite design
  choice.

### 1.4 Quality, tests and maintenance

- **Activity is extreme and agent-driven.** The repos are 9–10 days old with 600–1,000 commits
  each, and 372–757 commits in the last three days. Commit trailers show `Claude Opus 5.5` on most
  commits, with Cursor, Sonnet and Codex also present. A small core of humans (Brandon Thomas,
  Valentin Klein, Dmitrii Khlapov, Nikita Rakitin and others) runs autonomous agent sessions
  against a local `plan/` directory.
- Tests: wordcraft has 2,523 `#[test]` (505 in the UI crate), gridcraft 1,611 (677), deckcraft
  1,645 (203), craft-libs 291. There are offscreen kittest renders, command fuzzers and soak
  tests.
- CI: gridcraft and deckcraft run `ci.yml` (fmt, clippy `-D warnings`, tests, asset and layer
  checks, wasm). Wordcraft has no general CI workflow, only printing, Windows ARM64, FreeBSD,
  packaging and release workflows.
- Code style is dense and pragmatic, with long functions taking `&mut App`. The UI code is not
  written as a reusable library: ribbon groups call engine commands directly. This is fine for
  apps, but anything extracted would need its API redesigned, not just moved.
- Risk: churn. Line numbers and APIs change daily, so anything copied today is a snapshot to
  attribute, not something to track.

### 1.5 Licences and third-party assets (decides whether extraction is possible)

| item | licence | effect |
|---|---|---|
| wordcraft / gridcraft / deckcraft / craft-libs code | **MIT OR Apache-2.0** (`LICENSE-MIT`, `LICENSE-APACHE`, `NOTICE`), © 2026 ArtCraft Team and the *X* contributors | Extraction allowed. Keep the copyright line and NOTICE, and record provenance per file. |
| ArtCraft name, wordmark, logos (`docs/brand/`) | **Not open source** (`LICENSE-brand.txt`): use only inside that app; no forks, no other products | Never copy; never use the ArtCraft name. |
| app icons, DMG backgrounds | MIT OR Apache-2.0, original | Not useful to us. |
| UI icons | original painter code under the repo licence | Allowed, but not Fluent-styled and different per app (20, 16 and 24 px grids). |
| Lucide paths (deckcraft `engine/src/icon_data.rs`) | ISC; Feather-derived icons MIT | Document-insert icons, not UI. |
| fonts | Inter, Source Sans/Serif, JetBrains Mono, Carlito, Caladea, Liberation and others (OFL-1.1); Ubuntu Font Licence through egui defaults; craft-fonts repo | No Segoe or other Microsoft font is bundled anywhere. Gridcraft only *names* Segoe UI when loading the OS font at runtime. |
| Microsoft trademarks | Excel/Word/PowerPoint names used as feature-parity references ("clean-room … Microsoft Excel-style") | Nominative use. Gridcraft's default accent is Excel's brand green; we should not copy that. |
| laststudio/fluent-egui | MIT OR Apache-2.0. Bundles **Fluent System Icons** TTF (MIT, `FLUENT_SYSTEM_ICONS_LICENSE.txt`), Noto Sans SC (OFL) and WinUI-Gallery-style sample PNGs (`Cards.*.png`, `cliff.png` and others) with no per-file attribution. Uses "Segoe Fluent Icons" only if installed. | Code is reusable. **Check the PNGs' provenance before reusing any asset.** |
| PAIR-code/wordcraft | Apache-2.0 | Irrelevant (TypeScript, different product). |

Fluent sources for our own library:

| source | licence | use |
|---|---|---|
| `microsoft/fluentui` (`@fluentui/tokens`) | MIT. The repo note says Fluent UI React's *fonts and icons* fall under aka.ms/fluentui-assets-license | Generate token values from the MIT token package at a pinned commit. Do not pull the CDN fonts or MDL2 icons. |
| `microsoft/fluentui-system-icons` | **MIT** | The icon set. Ship a subset (SVG or a subset TTF) with the MIT notice. |
| `microsoft/Selawik` | **OFL-1.1** | Open, metric-compatible fallback for Segoe UI off Windows. |
| `microsoft/WinUI-Gallery` | MIT | Reference for control behaviour. Copy no images without checking each one. |
| Segoe UI, Segoe UI Variable, Segoe Fluent Icons | Microsoft proprietary; Windows-only use | **Never bundle.** Load from the OS on Windows at runtime only. |
| "Microsoft", "Office", "Fluent" marks | trademarks | Add a "not affiliated with or endorsed by Microsoft" line. No Office app logos and no product brand colours as defaults. Our crate names describe compatibility ("fluent2-…") without implying endorsement. |

**Conclusion:** licences allow extraction, with attribution. The obstacle is engineering, not
law: there is little reusable, Fluent-shaped code to extract.

## 2. Is a single cohesive egui Fluent library feasible?

Yes, technically. egui 0.36 has what is needed. But a cohesive library would be **designed new
from Fluent 2 specs**, using the storytold apps as behaviour references. It would not be
assembled from their code. Reusable pieces worth adapting, with attribution: the keytip state
machine (wordcraft `keytips.rs`), ribbon tab fitting and collapse (`ribbon::fit_tabs`), the
Linux portal theme query (`desktop_theme.rs`), backstage page structure, and craft-ui's toolbar
fitting, split and resize geometry and docking model (MIT OR Apache-2.0; already library-shaped
and tested).

### 2.1 Component inventory and API shape

API principles: plain egui idiom. Builders implement `egui::Widget` and return `Response`.
Containers take `impl FnOnce(&mut Ui)`. Multi-part widgets return a struct of responses (split
button main and arrow). Callers supply stable `Id`s. The library never dispatches commands.

| tier | components |
|---|---|
| foundation | `Theme` (install into `egui::Style`), tokens access (`Theme::get(ctx)`), `Icon` (Fluent System Icons, regular and filled, 16/20/24), typography helpers (`Text::body1()`, `title3()` …), focus ring, elevation (shadow 2–64), motion (durations and curves mapped to `ctx.animate_*`) |
| controls | Button (primary, secondary, outline, subtle, transparent), ToggleButton, SplitButton, MenuButton, CompoundButton, Link, Checkbox, Radio, Switch, Slider, SpinButton, Input, SearchBox, Textarea, Combobox, Dropdown, ProgressBar, Spinner, Badge, Avatar, Divider, Tooltip, Label |
| surfaces | Card, Dialog (modal, focus trap, Escape/default buttons, nested popups), Popover/Flyout, Menu/ContextMenu, MessageBar, Toast, TabList, Accordion, Drawer |
| Office chrome (what laststudio lacks) | TitleBar (custom decorations on Windows and Linux, inset on macOS, caption buttons, drag and resize), QuickAccessToolbar, Ribbon (tabs, groups, large/small/split buttons, galleries, contextual tabs, collapse/pin, adaptive group scaling and overflow), KeyTips, Backstage/File view, CommandBar, StatusBar, MiniToolbar |
| data and layout | DataGrid (virtualised, sortable, resizable columns, selection; over `egui_extras::TableBuilder`), TreeView, List, NavigationView/sidebar, SplitView, docking (adopt `egui_dock`/`egui_tiles`, or depend on craft-ui docking) |

### 2.2 Theming and design tokens (Fluent 2 mapping)

- A `fluent2-tokens` crate with no egui dependency. Its data is generated at build time from
  `@fluentui/tokens` at a pinned commit and checked in as Rust consts:
  - global tokens: the neutral ramp, the brand ramp (16 steps), shared colours;
  - alias tokens per theme: light, dark and high contrast (`colorNeutralBackground1..6`,
    `colorNeutralForeground1..4`, `colorBrandBackground*`, `colorNeutralStroke*`,
    `colorStatus*`);
  - the type ramp (`fontSizeBase100..600`, `lineHeight*`, weights);
  - spacing (`spacingHorizontalXXS..XXXL`), radii (`borderRadiusSmall..Circular`), stroke
    widths, shadows (2/4/8/16/28/64 with ambient and key layers), and durations and curves.
- Accent: build a 16-step brand ramp from one seed colour, as Fluent's theme designer does. On
  Windows, read the OS accent and high-contrast state (optional `windows` feature). On Linux,
  read the XDG portal (`org.freedesktop.appearance`: color-scheme, accent-color, contrast).
- High contrast: Fluent has no fixed HC palette for desktop. Map to OS system colours (Windows
  `GetSysColor`/UISettings) with a built-in black-and-white fallback. **No storytold app
  implements HC today**, so this is new work.
- egui mapping: `Theme::install(ctx)` writes `Visuals` (five widget states from alias tokens,
  selection, window and popup radius and shadow, and text colours) and `Spacing` (from spacing
  tokens). It stores the full token set in `ctx.data` for custom-painted widgets.

### 2.3 Accessibility

- eframe enables AccessKit through its `accesskit` feature on Windows (UIA), macOS (NSAccessibility)
  and Linux (AT-SPI via zbus). Standard egui widgets publish roles and labels through
  `Response::widget_info`. Custom-painted widgets (every ribbon control) must do the same, and
  use `ctx.accesskit_node_builder` for roles egui's `WidgetType` lacks (tab list, toolbar, menu
  button, grid cells).
- `egui_kittest` queries the AccessKit tree. Tests find widgets by role and label, so a11y gets
  tested on every interaction test at no extra cost. PdfCraft's and craft-ui's accessible button
  metadata are good references.
- Keyboard: Fluent focus rings, roving tab index inside the ribbon, KeyTips (Alt/F10), and
  shortcuts reported in tooltips. Respect the OS reduced-motion setting (wordcraft's
  `Animations::System` pattern).

### 2.4 Layout, DPI and text rendering

- DPI: egui works in points times `pixels_per_point`, which follows the OS scale per monitor.
  Wordcraft's `InterfaceSize` (50–300 % over native) is a good pattern to copy as a setting.
- Text: epaint 0.36 shapes with **harfrust**, parses fonts with **skrifa** and rasterises with
  **vello_cpu**, so complex scripts work. Rendering is grayscale AA with no ClearType or
  subpixel rendering, so text will look slightly softer than WinUI on Windows. Fonts: Segoe UI
  Variable or Segoe UI from the OS on Windows, Selawik or Inter elsewhere, plus CJK and RTL
  fallback chains (gridcraft's `theme.rs` font discovery is a thorough reference, ~600 lines).
- Layout: Fluent 4 px grid metrics come from tokens. Ribbon adaptive scaling (large → small →
  icon-only → overflow) needs a measure pass. In immediate mode that means caching the previous
  frame's group widths, as both wordcraft and deckcraft do. That is the hardest generic piece.

### 2.5 Examples and gallery

A gallery app (`examples/gallery`) with one page per component, each showing light, dark and HC,
accent picker, density and RTL. The same binary builds for wasm and deploys to GitHub Pages:
this is egui's main advantage over gpui. Plus three sample "apps" (document editor, data grid,
settings) that use the full chrome, rendered to PNGs with kittest for README and visual
regression.

### 2.6 Versioning against egui

egui ships a minor release every 2–4 months (0.33 Oct 2025, 0.34 Mar 2026, 0.35 Jun 2026,
0.36 Aug 2026) with breaking changes in each. Policy:

- The UI crate is `0.x` and supports exactly one egui minor. Bump our minor with every egui
  minor, within about two weeks of the egui release. Keep an egui-to-crate compatibility table
  in the README.
- `fluent2-tokens` has no egui dependency, so it follows semver on its own schedule and can
  reach 1.0.
- CI runs against the pinned egui minor, plus a non-blocking job against egui `main`, so breaks
  show up early.

### 2.7 Relationship with the storytold apps

- **They will not depend on a Fluent library.** Their absolute rule bans "any vendor's
  iconography … themes" (Fluent System Icons are Microsoft iconography, even though MIT). Their
  docs reject universal tokens, and their AGENTS.md says repos don't share code beyond
  craft-libs.
- What can flow upstream: generic, unbranded fixes to `craft-ui`. Examples: a ribbon split
  button (their audit lists it as "remains proposed"), keytip layout, a numeric field. These go
  as PRs to craft-libs under its licence and acceptance rules (two real consumers, render and
  interaction tests). That is goodwill and shared maintenance, not a home for this project.
- What flows to us: attributed adaptations under MIT OR Apache-2.0 (section 4.4).

## 3. How this relates to fluent-gpui

### 3.1 egui and gpui for this goal

| | egui 0.36 | gpui (vendored zed snapshot) |
|---|---|---|
| model | immediate mode; state is the app's | retained element tree, entities and contexts |
| platforms | Windows, Linux, macOS, **web/wasm**, Android | Windows (DirectX), Linux (wgpu), macOS (Metal); `gpui_web` exists upstream but is young; **macOS out of scope for the operator** |
| distribution | crates.io, semver minor cadence | **not on crates.io** past 0.2.2. The split crates are `publish = false`, so fluent v3 ships as git tags only (decision 1, WI-1346) |
| ecosystem | large (egui_extras, egui_dock, egui_tiles, egui_plot, kittest; 31k stars) | small, mostly zed-internal |
| text | harfrust + vello_cpu, grayscale AA | platform text systems (DirectWrite, Core Text, cosmic-text); sharper on Windows |
| perf and look | repaints on input; very good for tools; animation costs a repaint per frame | GPU-first, smooth animation, real blur/video (our patches) |
| a11y | AccessKit built in | AccessKit present upstream, unproven in our crates |
| cost to us now | new library from scratch | ~19k lines of working fluent crates, 17 PRs mid-migration, 2 estate consumers |

### 3.2 Options

| option | for | against |
|---|---|---|
| **A. fluent-gpui only** | no new work; the migration stack is already planned and running; estate consumers (`myaiagent/email-rs` `email-desktop-fluent`, `rdpapp/migration-tool`) keep working | no web or wasm; no crates.io after v3; small audience |
| **B. fluent-egui, shelve fluent-gpui** | larger audience, crates.io, wasm | throws away ~19k lines and the in-flight migration; strands two estate apps; video and blur patches have no egui equivalent; laststudio already occupies the basic-WinUI niche |
| **C. both, with a shared tokens/spec crate** | one source of truth for Fluent 2 values, icons and metrics; the tokens crate *can* go to crates.io (no gpui dependency) even while fluent-gpui ships by git tag; an egui renderer becomes cheap to start later | two renderers to keep visually aligned; must be staged or it doubles the work |
| **D. contribute a shared library to storytold** | joins an active community | **blocked by their own rules** (no vendor iconography or themes, no universal tokens, "repos don't share code"); they are not building Fluent; we would not control it |
| E. (found during research) build on laststudio/fluent-egui | 29k lines of WinUI controls already exist, permissive licence | single maintainer, 12 commits, egui 0.35, `ui.winui(…)` extension-trait style, unverified sample assets, no Office chrome; the crate name is theirs |

### 3.3 Recommendation: C, staged

1. **Now:** keep fluent-gpui as the product. **The pending PRs #1–#17 are unaffected.** They
   continue under the WI-1346 decisions (land as PRs, unmerged until the operator says so,
   grok workers, macOS out of scope, v3.0.0 by git tag). Nothing in this assessment argues for
   pausing them. Note that #11 (clippy 1.99) targets `main` and unblocks CI for everything else.
2. **After `gpui-upstream` merges (WI-1441):** split the Fluent *data* out of `fluent-core` into
   `crates/fluent2-tokens` in this workspace, with no gpui dependency, generated from
   `@fluentui/tokens`. Add Fluent System Icons as a catalogue, and have `fluent-core` consume
   both. Doing it before the stack merges would conflict with PR #10 (fluent-core port) and
   everything stacked on it. This step also replaces the hand-drawn SVGs in
   `fluent-app/src/assets.rs` with real Fluent icons. Publish `fluent2-tokens` to crates.io: it
   is the one fluent crate that still can be.
3. **Egui renderer, gated:** start a separate egui repo only when there is a consumer: an estate
   app that wants wasm, or one that can't carry the vendored gpui tree. Its first milestone
   reuses `fluent2-tokens` and goes straight at the Office chrome (TitleBar, Ribbon, KeyTips,
   Backstage), the gap neither laststudio nor storytold fills. Basic controls follow, after a
   time-boxed evaluation of whether depending on, or contributing to, laststudio/fluent-egui
   for them is better than writing our own.
4. **storytold:** no repo or library there. Optionally send small unbranded `craft-ui` PRs
   (ribbon split button, keytips) when we build them anyway.

Why not B: fluent-gpui works, has consumers, and its migration is planned and mostly delegated.
egui's advantages (wasm, crates.io, ecosystem) matter only once someone needs them, and staging
via a tokens crate keeps that door open at low cost.

## 4. If the egui repo goes ahead (design, for when the gate opens)

### 4.1 Name and location

- `fluent-egui` is taken (crate and repo by laststudio). Use **`fluent2-egui`** for the repo and
  crate prefix (crates.io names `fluent2-egui`, `fluent2-tokens`, `egui-fluent2` were all free on
  2026-10-14).
- Location: **`thedancingdeveloper/fluent2-egui`**. fluent-gpui was transferred from
  AusAgentSmith-org to thedancingdeveloper on 2026-10-13 (the old URL redirects). Estate pushes
  to AusAgentSmith-org failed with 401/403 then, and keeping both libraries under one owner
  keeps CI, tokens and issues together. Choose AusAgentSmith-org only if the operator wants
  org-level ownership for outside contributors.
- Licence: **MIT OR Apache-2.0**, matching egui, storytold and laststudio, so code can flow in
  every direction. fluent-gpui is Apache-2.0 only. The operator can relicense their own code
  into the new repo; confirm no third-party contributions exist in fluent-gpui first.

### 4.2 Crate layout

```
fluent2-egui/
  crates/
    fluent2-tokens/     # (or a dependency on the crate from fluent-gpui) tokens, ramps, type, metrics; no egui
    fluent2-icons/      # Fluent System Icons subset, generated; MIT notice; feature-gated sizes/styles
    fluent2-egui/       # theme install, foundation, controls, surfaces
    fluent2-egui-office/# TitleBar, QAT, Ribbon, KeyTips, Backstage, CommandBar, StatusBar
    fluent2-egui-data/  # DataGrid, TreeView, List, NavigationView, docking adapter
    fluent2-egui-platform/ # optional: Windows accent/HC (windows crate), Linux portal, custom decorations
  examples/gallery/     # native + wasm (trunk), deployed to Pages
  examples/{editor,grid,settings}/
  xtask/                # token/icon generation, asset audit, wasm check, snapshot update
  NOTICE  ATTRIBUTION.md  LICENSE-MIT  LICENSE-APACHE
```

### 4.3 MVP and milestones

| milestone | contents | exit criteria |
|---|---|---|
| M0 Foundations | tokens and icons generation, `Theme::install`, light/dark/brand ramp, fonts (OS Segoe or Selawik), focus ring, gallery skeleton with a wasm build, CI (fmt, clippy, kittest, wasm, asset audit) | gallery renders light and dark on Windows, Linux and web |
| M1 Office chrome | TitleBar (custom decorations on Windows and Linux), QAT, Ribbon (tabs, groups, large/small/split/toggle buttons, gallery, contextual tabs, collapse, adaptive scaling), KeyTips, Backstage, StatusBar | sample editor app driven by kittest with AccessKit queries; snapshots at 100 % and 200 % |
| M2 Core controls | buttons, inputs, selection controls, combobox and dropdown, menus, tooltip, dialog, message bar, toast, tab list | per-control a11y and keyboard tests; HC theme |
| M3 Data and layout | DataGrid, TreeView, NavigationView, split view, docking adapter | 100k-row grid scrolls at 60 fps in release |
| M4 Release 0.1 | docs, compatibility table, crates.io publish, Pages gallery | published; one estate consumer adopted |

### 4.4 Attribution and licence approach for extracted code

- Only `MIT OR Apache-2.0` sources (storytold, craft-libs, laststudio) and MIT/OFL Fluent assets.
- Each adapted file gets a header: "Adapted from storytold/wordcraft
  `crates/ui-egui/src/keytips.rs` @ `cf8aea688620`. Copyright (c) 2026 ArtCraft Team and the
  WordCraft contributors. MIT OR Apache-2.0." The NOTICE file gets a "Source provenance"
  section in the same form craft-libs uses.
- `ATTRIBUTION.md` has one row per asset (author, source, licence), enforced by
  `cargo xtask assets`, as storytold does.
- Never copy `docs/brand/` or the ArtCraft name, nor laststudio's sample images, unless each is
  traced to a licence. Never bundle Segoe or any Microsoft font. Never use Office product
  logos or brand colours as defaults; the default accent is the Fluent brand ramp.
- README carries a "not affiliated with or endorsed by Microsoft" disclaimer.

## 5. Backlog sketch (not raised in Vogt)

Sizes: S ≈ one worker session, M ≈ 2–3, L ≈ 4+. "grok" means a grok worker, following the
operator's WI-1346 direction. "opus" marks items where judgement or design dominates.

> After the operator's decision (top of this document), T1–T3 move into the new repository as
> part of M0, and T4 and T5 are dropped. The table is kept as originally drafted.

**Stage 2: tokens in fluent-gpui (after WI-1441):**

| # | item | size | model |
|---|---|---|---|
| T1 | `fluent2-tokens`: generator from `@fluentui/tokens` at a pinned commit; global, alias (light, dark), type, spacing, radius, shadow, motion | M | grok |
| T2 | Brand-ramp generator from a seed colour, plus HC fallback palette | S | grok |
| T3 | `fluent2-icons`: Fluent System Icons subset generator (SVG catalogue, regular and filled 16/20/24), MIT notice | M | grok |
| T4 | `fluent-core` consumes `fluent2-tokens`; swap `fluent-app` hand-drawn SVGs for Fluent icons; screenshot review | M | grok |
| T5 | Publish `fluent2-tokens` and `fluent2-icons` to crates.io | S | grok |

**Stage 3: egui renderer (gated):**

| # | item | size | model |
|---|---|---|---|
| E0 | Time-boxed evaluation of laststudio/fluent-egui: depend, fork or write our own for basic controls | S | opus |
| E1 | Repo scaffold, CI (fmt, clippy, kittest, wasm), xtask asset audit, NOTICE/ATTRIBUTION | S | grok |
| E2 | `Theme::install` and token-to-`Visuals` mapping, fonts, focus ring, motion | M | grok |
| E3 | Platform crate: Windows accent and HC, Linux portal, reduced motion | M | grok |
| E4 | Gallery skeleton with wasm build and Pages deploy | S | grok |
| E5 | TitleBar with custom decorations (drag, resize, caption buttons, snap) on Windows and Linux | M | opus |
| E6 | Ribbon core: tabs, groups, large/small/split/toggle buttons, a11y | L | grok |
| E7 | Ribbon adaptive scaling, overflow, contextual tabs, collapse | M | opus |
| E8 | KeyTips (adapt wordcraft, attributed) | M | grok |
| E9 | Backstage/File view, QAT, StatusBar, CommandBar | M | grok |
| E10 | Buttons, toggles, link, badge, divider, tooltip | M | grok |
| E11 | Inputs: text, search, textarea, spin, slider | M | grok |
| E12 | Selection: checkbox, radio, switch, combobox, dropdown, menus | M | grok |
| E13 | Dialog (modality, focus trap), message bar, toast, popover | M | grok |
| E14 | DataGrid (virtualised, sort, resize, select) | L | grok |
| E15 | TreeView, List, NavigationView, split view, docking adapter | M | grok |
| E16 | HC theme pass and a11y audit across components | M | opus |
| E17 | Sample apps, snapshot suite, docs, 0.1 release | M | grok |

That is about 5 items to stage 2 (2 S, 3 M) and about 18 to stage 3 (3 S, 13 M, 2 L), or 23
items in total. At the operator's usual 2–3 parallel grok workers, stage 2 is roughly one
dependency chain of 4 sequential items. Stage 3 is about four parallel lanes: platform/theme,
Office chrome, controls, data.

## 6. Operator decisions

1. **Strategy.** C staged (recommended) / A only / B / D.
2. **Create the egui repo now?** No: gate it on a consumer, and do the tokens crate in this
   workspace first (recommended) / Yes, now.
3. **Where, if created.** `thedancingdeveloper/fluent2-egui` (recommended) /
   `AusAgentSmith-org/fluent2-egui`.
4. **laststudio/fluent-egui.** Evaluate before writing basic controls (recommended) / Ignore
   it and write our own / Contribute the Office chrome to them instead.
5. **Licence for the new crates.** MIT OR Apache-2.0 (recommended) / Apache-2.0 only, like
   fluent-gpui.

## Appendix: method

- Cloned the repos above with `git clone` into the session scratchpad and treated their content
  as data. Counts come from `git log`, `wc -l` and `grep '#[test]'`.
- Duplication: a script normalised whitespace, dropped comments, braces and lines under 25
  characters, then compared identical-line sets and 4-line shingles per file pair across
  `crates/ui-egui/src`.
- `cargo test -p craft-ui` in craft-libs at `457a07880969`: 88 passed, 0 failed.
- egui release dates and crate-name availability come from the crates.io API on 2026-10-14.
  GitHub licences come from the GitHub API (`microsoft/fluentui` reports NOASSERTION; its
  LICENSE is MIT with the assets note quoted above).
- fluent-gpui state from Vogt WI-1346 and its comments, `docs/gpui-migration-plan.md` (PR #2)
  and `gh pr list`.
