# Consumer quick start

This guide builds a small Windows or Linux desktop app with a Fluent theme,
form controls, a ribbon, and a data table. The complete version of this example
is compiled as `demo_app` in `examples/demo_app`.

## 1. Add the crates

`fluent-gpui` is consumed from an exact git tag. Use one tag for every crate;
the placeholder below is replaced with the release tag you adopt.

```toml
[dependencies]
fluent-app = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
fluent-core = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
fluent-layout = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
fluent-primitives = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
fluent-ribbon = { git = "https://github.com/thedancingdeveloper/fluent-gpui", tag = "v3.0.0+zed.<short-sha>" }
```

The first 3.x tag is not yet published in this repository. Until the tag work
lands, a workspace checkout (as used by `demo_app`) can use the same crates as
path dependencies. Apps should depend on `fluent-*`, not directly on `gpui`.

## 2. Create a themed window

`FluentApp` installs the theme, title bar, window chrome, modal and toast hosts,
and bundled assets. `ThemeProvider` exposes the active tokens while rendering.

```rust
use fluent_app::FluentApp;
use fluent_core::ThemeProvider as _;
use fluent_primitives::{Button, ButtonAppearance};
use gpui::{div, prelude::*, ClickEvent, Context, IntoElement, Render, Window};

struct Hello;
impl Render for Hello {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().bg(cx.theme().colors.surface).child(
            Button::new("hello")
                .label("Create project")
                .appearance(ButtonAppearance::Accent)
                .on_click(|_: &ClickEvent, _, _| {}),
        )
    }
}

fn main() {
    FluentApp::new("Project tracker")
        .window_size(960.0, 640.0)
        .run(|cx| cx.new(|_| Hello));
}
```

## 3. Add a form, ribbon, and table

Use `Field` with a control such as `TextInput`, compose commands with
`RibbonBar`, and describe a table with `DataColumn` and `DataRow`:

```rust,no_run
use fluent_layout::{DataCell, DataColumn, DataRow, DataTable};
use fluent_primitives::{Field, TextInput};
use fluent_ribbon::RibbonBar;

let name = TextInput::new("name").placeholder("Project name");
let form_field = Field::new("project-name").label("Project name").control(name);
let table = DataTable::new()
    .columns([DataColumn::new("name", "Name"), DataColumn::new("owner", "Owner")])
    .rows([DataRow::new("p1", [DataCell::new("Design system"), DataCell::new("Alex")])]);
// A RibbonBar is configured in an entity with RibbonBar::new(cx).tab(...).
let _ = (form_field, table);
```

For a runnable composition, see `examples/demo_app/src/main.rs`: it includes a
`SettingsNav` form, a `RibbonBar` with contextual commands, and a `DataTable`
backed by the demo project model. Build it with:

```sh
cargo check -p demo_app
```

## 4. Find the right control

See [the component index](components.md) for exports, states, keyboard
contracts, and gallery routes. The gallery is the visual regression matrix:
run `cargo run -p gallery` and choose a family in its navigation.
