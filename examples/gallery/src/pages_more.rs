//! Gallery pages for overlays, menus and commands, and data display.
//!
//! Overlays that need an anchor (flyout, popover, suggestions, context menu)
//! open at the pointer when their button is clicked, because the gallery has
//! no prepaint bounds to hand them. Handlers are otherwise no-ops: the pages
//! exercise rendering, not application logic.

use fluent_core::gpui::{
    div, prelude::*, px, size, AnyElement, App, Bounds, ClickEvent, Context, Entity, IntoElement,
    Pixels, Point, Render, Window,
};
use fluent_core::ThemeProvider as _;
use fluent_layout::{
    CommandPalette, ContextMenu, ContextMenuItem, DataCellAlign, DataColumn, DataRow, DataTable,
    DataTableState, Dialog, DialogAction, DocumentViewer, Flyout, MenuButton, MessageIntent,
    PaletteEntry, Popover, SelectableList, SelectableListItem, Toast, ToastHost, ToastStack,
};
use fluent_primitives::{
    Button, DateTimePicker, DateTimeValue, Label, LabelSize, SuggestionItem, SuggestionPopover,
    TokenInput, TokenInputSuggestion, TokenInputToken, Tooltip,
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

/// The anchored popup the overlays page currently shows, if any.
enum Popup {
    Flyout(Bounds<Pixels>),
    Popover(Point<Pixels>),
    Suggestions(Point<Pixels>),
}

/// Toast, dialog, flyout, popover, suggestion popover and tooltip.
pub struct OverlaysPage {
    toast_host: Entity<ToastHost>,
    popup: Option<Popup>,
}

impl OverlaysPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        ToastStack::init(cx);
        ToastStack::push(
            Toast::new("gal-toast-open", "Connection restored.")
                .title("Toast")
                .intent(MessageIntent::Success)
                .persistent(),
            cx,
        );
        Self {
            toast_host: cx.new(ToastHost::new),
            popup: None,
        }
    }

    /// Whether a flyout is showing.
    #[cfg(test)]
    pub fn flyout_open(&self) -> bool {
        matches!(self.popup, Some(Popup::Flyout(_)))
    }
}

impl Render for OverlaysPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let surface = {
            let colors = colors.clone();
            move || {
                div()
                    .p_3()
                    .bg(colors.surface)
                    .border_1()
                    .border_color(colors.stroke_neutral)
            }
        };
        let push = |id: &'static str, intent: MessageIntent, title: &'static str| {
            Button::new(format!("gal-push-{id}"))
                .label(format!("Push {title}"))
                .on_click(move |_, _, cx| {
                    // One toast at a time: ToastHost gives every MessageBar the
                    // same element id, so two at once collide in the a11y tree.
                    ToastStack::clear(cx);
                    ToastStack::push(
                        Toast::new(id, "Something happened.")
                            .title(title)
                            .intent(intent),
                        cx,
                    )
                })
                .into_any_element()
        };
        let opener = |id: &'static str,
                      label: &'static str,
                      f: fn(Point<Pixels>) -> Popup,
                      cx: &mut Context<Self>| {
            Button::new(id)
                .label(label)
                .on_click(cx.listener(move |this, ev: &ClickEvent, _, cx| {
                    this.popup = Some(f(ev.position()));
                    cx.notify();
                }))
                .into_any_element()
        };
        let popup: Option<AnyElement> = match &self.popup {
            Some(Popup::Flyout(bounds)) => Some(
                Flyout::new(*bounds, surface().child(Label::new("Flyout content")))
                    .into_any_element(),
            ),
            Some(Popup::Popover(at)) => Some(
                Popover::new(*at, surface().child(Label::new("Popover content")))
                    .into_any_element(),
            ),
            Some(Popup::Suggestions(at)) => Some(
                SuggestionPopover::new(*at)
                    .items([
                        SuggestionItem::new("alpha", "Alpha").detail("First"),
                        SuggestionItem::new("beta", "Beta").detail("Second"),
                        SuggestionItem::new("gamma", "Gamma (disabled)").disabled(true),
                    ])
                    .highlighted(0)
                    .into_any_element(),
            ),
            None => None,
        };
        let close = cx.listener(|this, _: &ClickEvent, _, cx| {
            this.popup = None;
            cx.notify();
        });

        div()
            .relative()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.toast_host.clone())
            .child(section(
                "Toast: one is open; pushing replaces it (one at a time)",
                vec![
                    push("gal-t-info", MessageIntent::Info, "Info"),
                    push("gal-t-ok", MessageIntent::Success, "Success"),
                    push("gal-t-warn", MessageIntent::Warning, "Warning"),
                    push("gal-t-err", MessageIntent::Error, "Error"),
                ],
            ))
            .child(section(
                "Dialog: title, body, secondary and primary action",
                vec![div()
                    .w(px(420.0))
                    .child(
                        Dialog::new("Delete connection?")
                            .body(Label::new("This removes the saved connection."))
                            .action(DialogAction::new("Cancel", noop))
                            .action(DialogAction::new("Delete", noop).primary(true)),
                    )
                    .into_any_element()],
            ))
            .child(section(
                "Anchored popups: open at the click, one at a time",
                vec![
                    opener(
                        "gal-open-flyout",
                        "Open flyout",
                        |at| Popup::Flyout(Bounds::new(at, size(px(1.0), px(1.0)))),
                        cx,
                    ),
                    opener("gal-open-popover", "Open popover", Popup::Popover, cx),
                    opener(
                        "gal-open-suggest",
                        "Open suggestions",
                        Popup::Suggestions,
                        cx,
                    ),
                    Button::new("gal-close-popup")
                        .label("Close")
                        .on_click(close)
                        .into_any_element(),
                ],
            ))
            .child(section(
                "Tooltip: hover or focus the button",
                vec![Tooltip::new("Saves your work to disk")
                    .trigger(Button::new("gal-tt-save").label("Save"))
                    .into_any_element()],
            ))
            .children(popup)
    }
}

/// Context menu, menu button and command palette.
pub struct MenusPage {
    menu: Option<Entity<ContextMenu>>,
    menu_button: Entity<MenuButton>,
    palette: Entity<CommandPalette>,
}

fn menu_items() -> Vec<ContextMenuItem> {
    vec![
        ContextMenuItem::action_with_icon("New item", "icons/add.svg", |_, _, _| {}),
        ContextMenuItem::action("Rename", |_, _, _| {}).shortcut("F2"),
        ContextMenuItem::checkbox("Pinned", true, |_, _, _, _| {}),
        ContextMenuItem::action("Disabled", |_, _, _| {}).disabled(true),
        ContextMenuItem::separator(),
        ContextMenuItem::submenu(
            "Export",
            vec![
                ContextMenuItem::action("As JSON", |_, _, _| {}),
                ContextMenuItem::action("As CSV", |_, _, _| {}),
            ],
        ),
    ]
}

impl MenusPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        let palette = cx.new(|cx| {
            CommandPalette::new(cx).entries(vec![
                PaletteEntry::new("open", "Open connection")
                    .group("File")
                    .shortcut("Ctrl+O"),
                PaletteEntry::new("theme", "Toggle theme")
                    .group("View")
                    .subtitle("Switch between light and dark"),
                PaletteEntry::new("quit", "Quit")
                    .group("File")
                    .disabled(true),
            ])
        });
        let menu_button = cx.new(|_| {
            menu_items()
                .into_iter()
                .fold(MenuButton::new("gal-menu-button", "Actions"), |b, item| {
                    b.item(item)
                })
        });
        Self {
            menu: None,
            menu_button,
            palette,
        }
    }

    /// Whether the command palette is showing.
    #[cfg(test)]
    pub fn palette_open(&self, cx: &App) -> bool {
        self.palette.read(cx).open
    }
}

impl Render for MenusPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let open_menu = cx.listener(|this, ev: &ClickEvent, _, cx| {
            let at = ev.position();
            this.menu = Some(cx.new(|_| {
                menu_items()
                    .into_iter()
                    .fold(ContextMenu::build(at), |m, item| m.item(item))
            }));
            cx.notify();
        });
        let open_palette =
            cx.listener(|this, _: &ClickEvent, _, cx| this.palette.update(cx, |p, cx| p.show(cx)));
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(section(
                "Menu button and context menu (with a submenu); the menu bar above has nested submenus",
                vec![
                    self.menu_button.clone().into_any_element(),
                    Button::new("gal-open-context")
                        .label("Open context menu")
                        .on_click(open_menu)
                        .into_any_element(),
                ],
            ))
            .child(section(
                "Command palette: entries, group, shortcut, disabled",
                vec![Button::new("gal-open-palette")
                    .label("Open command palette")
                    .on_click(open_palette)
                    .into_any_element()],
            ))
            .children(self.menu.clone())
            .child(self.palette.clone())
    }
}

/// Data table, selectable list, document viewer, token input and date-time picker.
pub struct DataDisplayPage {
    token_input: Entity<TokenInput>,
    token_input_error: Entity<TokenInput>,
    date_time: Entity<DateTimePicker>,
    date_time_error: Entity<DateTimePicker>,
    date_time_disabled: Entity<DateTimePicker>,
}

impl DataDisplayPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            token_input: cx.new(|_| {
                TokenInput::new()
                    .placeholder("Add recipients")
                    .tokens([TokenInputToken::new("alex@example.com", "Alex")])
                    .suggestions([
                        TokenInputSuggestion::new("bianca@example.com", "Bianca"),
                        TokenInputSuggestion::new("chris@example.com", "Chris"),
                    ])
            }),
            token_input_error: cx.new(|_| {
                TokenInput::new()
                    .accessible_name("Recipients with error")
                    .token("nobody", "nobody")
                    .error(true)
            }),
            date_time: cx.new(|_| {
                DateTimePicker::new()
                    .placeholder("Pick a local time")
                    .maybe_value(DateTimeValue::new(2026, 5, 25, 9, 30))
            }),
            date_time_error: cx.new(|_| DateTimePicker::new().placeholder("Required").error(true)),
            date_time_disabled: cx.new(|_| {
                DateTimePicker::new()
                    .placeholder("Unavailable")
                    .disabled(true)
            }),
        }
    }
}

fn table(state: Option<DataTableState>) -> DataTable {
    let base = DataTable::new()
        .column(DataColumn::new("name", "Name").width(160.0).sortable(true))
        .column(DataColumn::new("kind", "Kind").width(80.0))
        .column(DataColumn::new("status", "Status").align(DataCellAlign::End));
    match state {
        None => base
            .row(DataRow::new("prod", vec!["Production", "SSH", "Connected"]))
            .row(DataRow::new("stage", vec!["Staging", "RDP", "Idle"]))
            .selected("stage"),
        Some(state) => base.state(state),
    }
}

impl Render for DataDisplayPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Each library widget with a fixed element id sits under its own id
        // scope, as on the status page.
        let framed = |id: &'static str, w: f32, h: f32, el: AnyElement| {
            div().id(id).w(px(w)).h(px(h)).child(el).into_any_element()
        };
        let list =
            |id: &'static str, l: SelectableList| framed(id, 280.0, 150.0, l.into_any_element());
        let items = || {
            vec![
                SelectableListItem::new("a", "CPU threshold")
                    .subtitle("Production")
                    .meta("08:20"),
                SelectableListItem::new("b", "Gateway failover")
                    .subtitle("Network")
                    .badge("P1")
                    .emphasized(true),
                SelectableListItem::new("c", "Maintenance")
                    .subtitle("Platform")
                    .disabled(true),
            ]
        };
        let tbl = |id: &'static str, state: Option<DataTableState>| {
            framed(id, 340.0, 140.0, table(state).into_any_element())
        };
        let w = |el: AnyElement| div().w(px(240.0)).child(el).into_any_element();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(section(
                "Data table: rows with selection, loading, empty, error",
                vec![
                    tbl("gal-dt-ready", None),
                    tbl("gal-dt-load", Some(DataTableState::Loading("Loading rows".into()))),
                    tbl("gal-dt-empty", Some(DataTableState::Empty("No rows".into()))),
                    tbl("gal-dt-err", Some(DataTableState::Error("Could not load rows".into()))),
                ],
            ))
            .child(section(
                "Selectable list: rows, loading, empty, error",
                vec![
                    list(
                        "gal-sl-ready",
                        SelectableList::new("gal-sl-r").items(items()).selected("b"),
                    ),
                    list(
                        "gal-sl-load",
                        SelectableList::new("gal-sl-l").loading("Loading items"),
                    ),
                    list(
                        "gal-sl-empty",
                        SelectableList::new("gal-sl-e").empty_message("No items"),
                    ),
                    list(
                        "gal-sl-err",
                        SelectableList::new("gal-sl-x").error_message("Could not load items"),
                    ),
                ],
            ))
            .child(section(
                "Document viewer: rich text and empty",
                vec![
                    framed(
                        "gal-dv",
                        280.0,
                        160.0,
                        DocumentViewer::html_fragment(
                            "<h2>Status note</h2><p>Renders <strong>safe rich text</strong> and links.</p>",
                        )
                        .max_height(150.0)
                        .into_any_element(),
                    ),
                    framed(
                        "gal-dv-empty",
                        280.0,
                        60.0,
                        DocumentViewer::plain_text("")
                            .empty_message("Nothing to show")
                            .into_any_element(),
                    ),
                ],
            ))
            .child(section(
                "Token input: tokens and suggestions, error; date-time picker: value, error, disabled",
                vec![
                    w(self.token_input.clone().into_any_element()),
                    w(self.token_input_error.clone().into_any_element()),
                    w(self.date_time.clone().into_any_element()),
                    w(self.date_time_error.clone().into_any_element()),
                    w(self.date_time_disabled.clone().into_any_element()),
                ],
            ))
    }
}
