//! Gallery pages for inputs, status and feedback, and layout components.
//!
//! Each page shows the states a component can be in side by side: default,
//! disabled, error, loading and open. Pages hold no app logic; handlers are
//! no-ops so the page only exercises rendering.

use fluent_core::gpui::{
    div, prelude::*, px, AnyElement, App, ClickEvent, Context, Entity, IntoElement, Render, Window,
};
use fluent_layout::{
    DataStatusKind, MessageBar, MessageIntent, Toolbar, ToolbarItem, Tree, TreeItem,
};
use fluent_primitives::{
    Button, Chip, Combobox, Dropdown, Field, FieldValidationState, Label, LabelSize, ProgressBar,
    Searchbox, Skeleton, SkeletonRow, Spinner, TextInput, Textarea,
};

fn noop(_: &ClickEvent, _: &mut Window, _: &mut App) {}

fn section(title: &'static str, items: Vec<AnyElement>) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(Label::new(title).size(LabelSize::Subtitle))
        .child(
            div()
                .flex()
                .flex_wrap()
                .items_start()
                .gap_3()
                .children(items),
        )
}

/// Text entry controls: default, placeholder, error, disabled and a validated field.
pub struct InputsPage {
    text: Vec<Entity<TextInput>>,
    textarea: Entity<Textarea>,
    textarea_disabled: Entity<Textarea>,
    textarea_error: Entity<Textarea>,
    searchbox: Entity<Searchbox>,
    dropdown: Entity<Dropdown>,
    dropdown_disabled: Entity<Dropdown>,
    dropdown_error: Entity<Dropdown>,
    combobox: Entity<Combobox>,
    combobox_disabled: Entity<Combobox>,
    combobox_error: Entity<Combobox>,
    field_input: Entity<TextInput>,
}

impl InputsPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let dropdown = |id: &'static str, disabled: bool| {
            Dropdown::new(id)
                .placeholder("Protocol")
                .option("ssh", "SSH")
                .option("rdp", "RDP")
                .selected("ssh")
                .disabled(disabled)
        };
        let combobox = |disabled: bool| {
            Combobox::new()
                .placeholder("Protocol")
                .option("ssh", "SSH")
                .option("rdp", "RDP")
                .disabled(disabled)
        };
        Self {
            text: vec![
                cx.new(|_| TextInput::new().placeholder("Placeholder")),
                cx.new(|_| TextInput::new().value("Filled")),
                cx.new(|_| TextInput::new().value("bad@input").error(true)),
                cx.new(|_| TextInput::new().value("Disabled").disabled(true)),
            ],
            textarea: cx.new(|_| Textarea::new().placeholder("Notes").rows(3)),
            textarea_disabled: cx.new(|_| Textarea::new().value("Disabled").disabled(true)),
            textarea_error: cx.new(|_| Textarea::new().value("Too short").rows(3).error(true)),
            searchbox: cx.new(|_| Searchbox::new().placeholder("Search")),
            dropdown: cx.new(|_| dropdown("gal-dd", false)),
            dropdown_disabled: cx.new(|_| dropdown("gal-dd-dis", true)),
            dropdown_error: cx.new(|_| dropdown("gal-dd-err", false).error(true)),
            combobox_error: cx.new(|_| combobox(false).error(true)),
            combobox: cx.new(|_| combobox(false)),
            combobox_disabled: cx.new(|_| combobox(true)),
            field_input: cx.new(|_| TextInput::new().value("").error(true)),
        }
    }
}

impl Render for InputsPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let w = |el: AnyElement| div().w(px(220.0)).child(el).into_any_element();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(section(
                "Text input: placeholder, filled, error, disabled",
                self.text
                    .iter()
                    .map(|t| w(t.clone().into_any_element()))
                    .collect(),
            ))
            .child(section(
                "Textarea (default, disabled, error) and searchbox",
                vec![
                    w(self.textarea.clone().into_any_element()),
                    w(self.textarea_disabled.clone().into_any_element()),
                    w(self.textarea_error.clone().into_any_element()),
                    w(self.searchbox.clone().into_any_element()),
                ],
            ))
            .child(section(
                "Dropdown and combobox: enabled, disabled, error",
                vec![
                    w(self.dropdown.clone().into_any_element()),
                    w(self.dropdown_disabled.clone().into_any_element()),
                    w(self.combobox.clone().into_any_element()),
                    w(self.combobox_disabled.clone().into_any_element()),
                    w(self.dropdown_error.clone().into_any_element()),
                    w(self.combobox_error.clone().into_any_element()),
                ],
            ))
            .child(section(
                "Field with validation error",
                vec![div()
                    .w(px(320.0))
                    .child(
                        Field::new(div().w_full().child(self.field_input.clone()))
                            .label("Host name")
                            .required(true)
                            .validation(FieldValidationState::Error, "Host name is required."),
                    )
                    .into_any_element()],
            ))
    }
}

/// Loading, progress and message surfaces.
pub struct StatusPage;

impl Render for StatusPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let bar = |intent, title: &'static str| {
            div()
                .w(px(420.0))
                .child(
                    MessageBar::new("Something happened.")
                        .id(format!("gal-mb-{title}"))
                        .intent(intent)
                        .title(title)
                        .action("Retry", noop)
                        .on_dismiss(noop),
                )
                .into_any_element()
        };
        // MessageBar, Spinner and ProgressBar each take an `id(..)`; repeated
        // instances need distinct ids so their a11y nodes do not collide.
        let progress = |n: usize, bar: ProgressBar| {
            div()
                .w(px(220.0))
                .child(bar.id(format!("gal-pg-{n}")))
                .into_any_element()
        };
        let spin = |n: usize, s: Spinner| s.id(format!("gal-sp-{n}")).into_any_element();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(section(
                "Spinner",
                vec![
                    spin(0, Spinner::new().size(16.0)),
                    spin(1, Spinner::new().size(32.0).label("Loading")),
                ],
            ))
            .child(section(
                "Progress: value, indeterminate, success, warning, error",
                vec![
                    progress(0, ProgressBar::new(0.6)),
                    progress(1, ProgressBar::new(0.0).indeterminate()),
                    progress(2, ProgressBar::new(1.0).success()),
                    progress(3, ProgressBar::new(0.5).warning()),
                    progress(4, ProgressBar::new(0.3).error()),
                ],
            ))
            .child(section(
                "Skeleton",
                vec![
                    Skeleton::new("gal-sk").w(260.0).h(16.0).into_any_element(),
                    div()
                        .w(px(260.0))
                        .child(SkeletonRow::new("gal-skr"))
                        .into_any_element(),
                ],
            ))
            .child(section(
                "Message bar: info, success, warning, error",
                vec![
                    bar(MessageIntent::Info, "Info"),
                    bar(MessageIntent::Success, "Success"),
                    bar(MessageIntent::Warning, "Warning"),
                    bar(MessageIntent::Error, "Error"),
                ],
            ))
            .child(section(
                "Buttons and chips next to a busy indicator",
                vec![
                    Button::new("gal-busy")
                        .label("Saving")
                        .disabled(true)
                        .into_any_element(),
                    Button::new("gal-loading")
                        .label("Saving")
                        .loading(true)
                        .into_any_element(),
                    spin(2, Spinner::new().size(16.0)),
                    Chip::new("gal-chip", "Filter: open").into_any_element(),
                ],
            ))
    }
}

/// Layout components: toolbar (wide, narrow, overflow open) and tree states.
pub struct LayoutPage;

fn toolbar_items() -> Vec<ToolbarItem> {
    vec![
        ToolbarItem::action_with_icon("tb-new", "New", "icons/add.svg", noop),
        ToolbarItem::action_with_icon("tb-copy", "Copy", "icons/copy.svg", noop),
        ToolbarItem::separator(),
        ToolbarItem::toggle("tb-bold", "Bold", true, |_, _, _, _| {}),
        ToolbarItem::toggle("tb-italic", "Italic", false, |_, _, _, _| {}),
        ToolbarItem::action("tb-find", "Find", noop),
        ToolbarItem::action("tb-dis", "Disabled", noop).disabled(true),
    ]
}

impl Render for LayoutPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let toolbar = |id: &'static str, width: f32, open: bool| {
            div()
                .w(px(width))
                .child(
                    Toolbar::new()
                        .id(id)
                        .items(toolbar_items())
                        .available_width(px(width))
                        .overflow_open(open),
                )
                .into_any_element()
        };
        let tree = |id: &'static str, tree: Tree| {
            div()
                .id(id)
                .w(px(240.0))
                .h(px(150.0))
                .child(tree)
                .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(section(
                "Toolbar: wide, narrow (overflow), narrow with overflow open",
                vec![
                    toolbar("tb-wide", 560.0, false),
                    toolbar("tb-narrow", 200.0, false),
                    toolbar("tb-open", 200.0, true),
                ],
            ))
            .child(section(
                "Tree: default, loading child, status",
                vec![
                    tree(
                        "gal-tree",
                        Tree::new().id("gal-tree-v").item(
                            TreeItem::new("root", "Workspace")
                                .expanded(true)
                                .child(TreeItem::new("a", "Alpha"))
                                .child(TreeItem::new("b", "Beta").disabled(true))
                                .child(TreeItem::new("lazy", "Lazy").lazy(true).loading(true)),
                        ),
                    ),
                    tree(
                        "gal-tree-err",
                        Tree::new()
                            .id("gal-tree-e")
                            .status(DataStatusKind::Error, "Could not load items"),
                    ),
                    tree(
                        "gal-tree-load",
                        Tree::new()
                            .id("gal-tree-l")
                            .status(DataStatusKind::Loading, "Loading items"),
                    ),
                ],
            ))
    }
}
