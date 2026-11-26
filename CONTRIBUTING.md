# Contributing to fluent-gpui

Thanks for helping. fluent-gpui is a Fluent 2 design system for native Windows
and Linux desktop apps, built on a vendored copy of GPUI. This page covers
setting up, the checks a change must pass, and the pull request checklist.

By taking part you agree to follow the [code of conduct](CODE_OF_CONDUCT.md).
Report security problems as described in [SECURITY.md](SECURITY.md), not in a
public issue. For a bigger change, open an issue first so we can agree on the
approach.

Supported platforms are **Windows and Linux**. macOS, web and wasm are out of
scope, so changes for them will not be accepted.

## Set up

- Install [Rust via rustup](https://rustup.rs). The repository pins its toolchain
  in [`rust-toolchain.toml`](rust-toolchain.toml); rustup installs it, with
  `rustfmt` and `clippy`, the first time you build.
- On Debian or Ubuntu install the GUI libraries:

  ```sh
  sudo apt-get install libxcb1-dev libxkbcommon-dev libxkbcommon-x11-dev libfontconfig-dev libwayland-dev libvulkan-dev
  ```

- Clone the repository and build:

  ```sh
  git clone https://github.com/thedancingdeveloper/fluent-gpui
  cd fluent-gpui
  cargo build --workspace
  ```

## Run the examples

```sh
cargo run -p gallery       # every control, by component family
cargo run -p hello_world   # the smallest app
cargo run -p widgets
cargo run -p form
cargo run -p demo_app
```

The gallery is the visual regression matrix; see [docs/gallery.md](docs/gallery.md).

## Checks

Run these before you open a pull request. CI runs the same checks on Linux and
Windows.

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
python3 scripts/check-layering.py
scripts/check-zed-rev.sh
```

- `cargo test` includes in-process GPUI tests, which run on GPUI's test platform
  and need no display. `fluent-test-utils` has helpers for them, including
  accessibility tree dumps.
- You can limit a run to the crates you touched, for example
  `cargo clippy -p fluent-primitives --all-targets -- -D warnings` and
  `cargo test -p fluent-primitives`. On a small machine, lower the parallelism
  with `CARGO_BUILD_JOBS=2`.
- `cargo deny check licenses bans` checks licences and banned dependencies; CI
  runs it. Install it with `cargo install cargo-deny`.
- Docs: `scripts/check-docs-links.sh` checks that every relative link and image
  in the README, this file and `docs/` points at a file that exists.

## Rules for code

- **Layering.** Only the `fluent-*` crates and the vendored tree (`vendor/zed`)
  may depend on `gpui`. Examples and tests use `fluent-*` too; `fluent-core`
  re-exports what they need as `fluent_core::gpui`. CI enforces this with
  `scripts/check-layering.py` and `deny.toml`.
- **Vendored code.** Do not edit files under `vendor/zed` unless
  [`vendor/zed/PATCHES.md`](vendor/zed/PATCHES.md) lists them for a named patch.
  A new patch adds its module (behind a Cargo feature where possible), an example
  and a test, and its `PATCHES.md` line, in the same change. Do not change the
  upstream Zed revision in a feature change; that is a separate, periodic update.
  See [docs/vendoring.md](docs/vendoring.md).
- **No `unsafe` and no panics** in `fluent-*` library code without discussing it
  first.
- **Accessibility.** Every control has a role, an accessible name and its actions
  in the AccessKit tree, visible focus, a keyboard path for every action, and
  announces status changes.
- **Assets.** No Microsoft fonts or Office artwork. Icons come from the generated
  icon catalogue, and every asset needs an attribution row.
- **Generated files** (tokens, the icon catalogue, the vendored tree) change only
  by re-running their generator.
- **No secrets** in commits: tokens, credentials or environment files.
- **Platforms.** Windows and Linux only: no Metal paths and no macOS-only fixes.

## Screenshots

The images in the README come from a headless harness. It renders each component
and each gallery page in light and dark, and again after a real Tab key press to
show the focus ring.

- The **Screenshots (Xvfb)** workflow
  ([`.github/workflows/screenshots.yml`](.github/workflows/screenshots.yml)) runs
  on demand from the Actions tab ("Run workflow") and in the merge queue, never
  on pull requests. It uploads a `screenshots-xvfb` artifact, kept for 14 days.
- Locally: build, then run the script. You need `xvfb`, `xdotool`, `imagemagick`
  and a Vulkan driver (for example `mesa-vulkan-drivers`).

  ```sh
  cargo build -p screenshots -p gallery
  scripts/screenshots-xvfb.sh --out DIR
  ```

  Options: `--only NAME` for one component, `--dark`, `--light`, `--no-focus`,
  `--no-gallery`. `gallery --list-pages` lists the gallery pages.
- The output has `dark/` and `light/` (1000x600), `focus/dark/` and `focus/light/`,
  `gallery/<page>-<theme>.png` (1280x800, plus `hero-dark.png` and `hero-light.png`)
  and `report.txt`, which says whether each focus ring was visible.
- To update the README images from a run, use
  `scripts/refresh-readme-screenshots.sh <run-id>`; it copies the images into
  `screenshots/` and `docs/images/`.

Software rendering differs from a GPU desktop, so output is not compared pixel
for pixel with the committed images.

## Pull request checklist

- [ ] One focused change, with a clear description of what and why.
- [ ] `cargo fmt`, `clippy` and `cargo test` pass as above.
- [ ] Tests for new behaviour. A bug fix includes a test that fails without the
      fix.
- [ ] An entry in [CHANGELOG.md](CHANGELOG.md) under **Unreleased**: one line in
      plain language that says what changed for a user of the library, starting
      with the area, for example `primitives: Button now ...`. Do not put issue
      numbers or internal references in it.
- [ ] No version bumps; versions change only in release changes.
- [ ] Screenshots, before and after, if the change alters how something looks.
      Check the gallery in light, dark, high contrast and right-to-left.
- [ ] Docs updated if behaviour or public API changed.
- [ ] No unrelated reformatting or generated-file churn.

## Licence of contributions

fluent-gpui is licensed under the [Apache License 2.0](LICENSE). Unless you say
otherwise, a contribution you submit is licensed under the same terms (Apache-2.0
section 5). Do not submit code you do not have the right to share. Do not add
code under a licence that is not compatible with Apache-2.0, such as GPL or AGPL.

## Maintainers

How maintainers land changes, pull upstream Zed and cut releases:
[docs/maintainers/contributing-internal.md](docs/maintainers/contributing-internal.md).
