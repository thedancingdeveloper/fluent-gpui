use fluent_app::FluentApp;
use fluent_core::ThemeProvider as _;
use fluent_layout::{
    set_tree_checked, ContextMenuItem, DataCellAlign, DataColumn, DataRow, DataSortDirection,
    DataTable, DocumentViewer, MenuButton, MessageBar, MessageIntent, SelectableList,
    SelectableListItem, Toolbar, ToolbarItem, Tree, TreeItem,
};
use fluent_primitives::{
    Avatar, Badge, Button, ButtonAppearance, ButtonSize, Checkbox, CheckboxState, Chip, Combobox,
    DateTimePicker, DateTimeValue, Divider, Dropdown, Field, FieldValidationState, Icon,
    IconButton, IconSize, Label, LabelSize, ProgressBar, RadioGroup, RadioGroupOrientation,
    Searchbox, Spinner, Switch, TextInput, Textarea, ToggleButton, TokenInput,
    TokenInputSuggestion, TokenInputToken, Tooltip,
};
use gpui::{div, prelude::*, px, App, Context, Entity, IntoElement, Render, SharedString, Window};

struct WidgetGallery {
    check: CheckboxState,
    tree_checks: Vec<TreeItem>,
    switch_on: bool,
    toggle_bold: bool,
    toggle_italic: bool,
    chip_dismissed: bool,
    message_visible: bool,
    radio_value: SharedString,
    toolbar_overflow_open: bool,
    table_sort_column: SharedString,
    table_sort_direction: DataSortDirection,
    text_placeholder: Entity<TextInput>,
    text_filled: Entity<TextInput>,
    text_error: Entity<TextInput>,
    text_disabled: Entity<TextInput>,
    textarea_notes: Entity<Textarea>,
    searchbox: Entity<Searchbox>,
    token_input: Entity<TokenInput>,
    date_time_picker: Entity<DateTimePicker>,
    combobox: Entity<Combobox>,
    dropdown_empty: Entity<Dropdown>,
    dropdown_selected: Entity<Dropdown>,
    dropdown_disabled: Entity<Dropdown>,
    menu_button: Entity<MenuButton>,
}

impl WidgetGallery {
    fn new(cx: &mut App) -> Self {
        Self {
            check: CheckboxState::Unchecked,
            tree_checks: vec![
                TreeItem::new("docs", "Documents")
                    .checkable(true)
                    .child(
                        TreeItem::new("report", "Report.pdf")
                            .checkable(true)
                            .checked(true),
                    )
                    .child(TreeItem::new("notes", "Notes.md").checkable(true)),
                TreeItem::new("media", "Media")
                    .checkable(true)
                    .child(TreeItem::new("photo", "Photo.png").checkable(true)),
            ],
            switch_on: false,
            toggle_bold: false,
            toggle_italic: false,
            chip_dismissed: false,
            message_visible: true,
            radio_value: "comfortable".into(),
            toolbar_overflow_open: false,
            table_sort_column: "name".into(),
            table_sort_direction: DataSortDirection::Ascending,
            text_placeholder: cx.new(|_| TextInput::new().placeholder("Type something...")),
            text_filled: cx.new(|_| TextInput::new().value("Hello, FluentGUI")),
            text_error: cx.new(|_| TextInput::new().value("bad input").error(true)),
            text_disabled: cx.new(|_| TextInput::new().value("Read only").disabled(true)),
            textarea_notes: cx.new(|_| {
                Textarea::new()
                    .placeholder("Notes")
                    .value("Multiline content\nuses the same focus indicator.")
                    .rows(3)
            }),
            searchbox: cx.new(|_| Searchbox::new().placeholder("Search connections")),
            token_input: cx.new(|_| {
                TokenInput::new()
                    .placeholder("Add recipients")
                    .tokens([TokenInputToken::new("alex@example.com", "Alex")])
                    .suggestions([
                        TokenInputSuggestion::new("bianca@example.com", "Bianca")
                            .detail("bianca@example.com"),
                        TokenInputSuggestion::new("chris@example.com", "Chris")
                            .detail("chris@example.com"),
                        TokenInputSuggestion::new("drew@example.com", "Drew")
                            .detail("drew@example.com"),
                    ])
            }),
            date_time_picker: cx.new(|_| {
                DateTimePicker::new()
                    .placeholder("Pick a local time")
                    .value(DateTimeValue::new(2026, 5, 25, 9, 30).unwrap())
            }),
            combobox: cx.new(|_| {
                Combobox::new()
                    .placeholder("Protocol")
                    .option("ssh", "SSH")
                    .option("rdp", "RDP")
                    .option("vnc", "VNC")
                    .option("sftp", "SFTP")
            }),
            dropdown_empty: cx.new(|_| {
                Dropdown::new("dd-empty")
                    .placeholder("Choose protocol")
                    .option("ssh", "SSH")
                    .option("rdp", "RDP")
                    .option("vnc", "VNC")
            }),
            dropdown_selected: cx.new(|_| {
                Dropdown::new("dd-selected")
                    .selected("ssh")
                    .option("ssh", "SSH")
                    .option("rdp", "RDP")
                    .option("vnc", "VNC")
            }),
            dropdown_disabled: cx.new(|_| {
                Dropdown::new("dd-disabled")
                    .placeholder("Unavailable")
                    .disabled(true)
                    .option("ssh", "SSH")
            }),
            menu_button: cx.new(|_| {
                MenuButton::new("mb-actions", "Actions")
                    .item(ContextMenuItem::action_with_icon(
                        "New item",
                        "icons/add.svg",
                        |_, _, _| {},
                    ))
                    .item(ContextMenuItem::action("Rename", |_, _, _| {}))
                    .item(ContextMenuItem::checkbox("Pinned", true, |_, _, _, _| {}))
                    .separator()
                    .item(ContextMenuItem::submenu(
                        "Export",
                        vec![
                            ContextMenuItem::action("As JSON", |_, _, _| {}),
                            ContextMenuItem::action("As CSV", |_, _, _| {}),
                        ],
                    ))
            }),
        }
    }
}

impl Render for WidgetGallery {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let entity = cx.entity();

        let check_entity = entity.clone();
        let tree_check_entity = entity.clone();
        let switch_entity = entity.clone();
        let bold_entity = entity.clone();
        let italic_entity = entity.clone();
        let chip_entity = entity.clone();
        let radio_entity = entity.clone();
        let message_entity = entity.clone();
        let toolbar_entity = entity.clone();
        let table_entity = entity.clone();

        div()
            .id("widget-gallery-scroll")
            .size_full()
            .overflow_y_scroll()
            .bg(colors.surface_dim)
            .child(
                div()
                    .max_w(px(820.0))
                    .mx_auto()
                    .p(px(32.0))
                    .flex()
                    .flex_col()
                    .gap(px(24.0))
                    // ── TYPOGRAPHY ────────────────────────────────────────
                    .child(Label::new("TYPOGRAPHY").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(Label::new("Caption — 12pt").size(LabelSize::Caption))
                            .child(Label::new("Body — 14pt").size(LabelSize::Body))
                            .child(Label::new("Subtitle — 16pt semibold").size(LabelSize::Subtitle))
                            .child(Label::new("Title — 20pt semibold").size(LabelSize::Title))
                            .child(Label::new("Display — 28pt semibold").size(LabelSize::Display)),
                    )
                    .child(Divider::horizontal())
                    // ── BUTTONS ───────────────────────────────────────────
                    .child(Label::new("BUTTONS").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .items_center()
                            .child(Button::new("b-neutral").label("Neutral"))
                            .child(
                                Button::new("b-accent")
                                    .label("Accent")
                                    .appearance(ButtonAppearance::Accent),
                            )
                            .child(
                                Button::new("b-subtle")
                                    .label("Subtle")
                                    .appearance(ButtonAppearance::Subtle),
                            )
                            .child(
                                Button::new("b-link")
                                    .label("Hyperlink")
                                    .appearance(ButtonAppearance::Hyperlink),
                            )
                            .child(Button::new("b-disabled").label("Disabled").disabled(true)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .items_center()
                            .child(
                                Button::new("b-compact")
                                    .label("Compact")
                                    .size(ButtonSize::Compact),
                            )
                            .child(
                                Button::new("b-compact-accent")
                                    .label("Compact Accent")
                                    .appearance(ButtonAppearance::Accent)
                                    .size(ButtonSize::Compact),
                            )
                            .child(
                                Button::new("b-compact-disabled")
                                    .label("Compact Disabled")
                                    .size(ButtonSize::Compact)
                                    .disabled(true),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .items_center()
                            .child(self.menu_button.clone()),
                    )
                    .child(Divider::horizontal())
                    // ── ICON & ICON BUTTON ────────────────────────────────
                    .child(Label::new("ICON & ICON BUTTON").size(LabelSize::Subtitle))
                    .child(
                        // Icon paths are resolved at runtime via GPUI's asset system.
                        // Replace with valid asset paths from your app's asset bundle.
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(12.0))
                            .items_center()
                            .child(Icon::new("icons/add.svg").size(IconSize::Sm))
                            .child(Icon::new("icons/add.svg").size(IconSize::Md))
                            .child(Icon::new("icons/add.svg").size(IconSize::Lg))
                            .child(Icon::new("icons/add.svg").size(IconSize::Xl))
                            .child(IconButton::new("ib-subtle", "icons/search.svg"))
                            .child(
                                IconButton::new("ib-accent", "icons/search.svg")
                                    .appearance(ButtonAppearance::Accent),
                            )
                            .child(
                                IconButton::new("ib-disabled", "icons/search.svg").disabled(true),
                            ),
                    )
                    .child(Divider::horizontal())
                    // ── CONTROLS ─────────────────────────────────────────
                    .child(Label::new("CONTROLS").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(16.0))
                            .items_center()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .items_center()
                                    .child(Checkbox::new("cb-off").state(CheckboxState::Unchecked))
                                    .child(Label::new("Unchecked").size(LabelSize::Caption)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .items_center()
                                    .child(Checkbox::new("cb-on").state(CheckboxState::Checked))
                                    .child(Label::new("Checked").size(LabelSize::Caption)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .items_center()
                                    .child(
                                        Checkbox::new("cb-indet")
                                            .state(CheckboxState::Indeterminate),
                                    )
                                    .child(Label::new("Indeterminate").size(LabelSize::Caption)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .items_center()
                                    .child(
                                        Checkbox::new("cb-interactive").state(self.check).on_click(
                                            move |new_state, _, _, cx| {
                                                check_entity.update(cx, |this, cx| {
                                                    this.check = new_state;
                                                    cx.notify();
                                                });
                                            },
                                        ),
                                    )
                                    .child(Label::new("Interactive").size(LabelSize::Caption)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(16.0))
                            .items_center()
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .items_center()
                                    .child(Switch::new("sw-off").on(false))
                                    .child(Label::new("Off").size(LabelSize::Caption)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .items_center()
                                    .child(Switch::new("sw-on").on(true))
                                    .child(Label::new("On").size(LabelSize::Caption)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_row()
                                    .gap(px(6.0))
                                    .items_center()
                                    .child(
                                        Switch::new("sw-interactive").on(self.switch_on).on_click(
                                            move |new_val, _, _, cx| {
                                                switch_entity.update(cx, |this, cx| {
                                                    this.switch_on = new_val;
                                                    cx.notify();
                                                });
                                            },
                                        ),
                                    )
                                    .child(Label::new("Interactive").size(LabelSize::Caption)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .items_center()
                            .child(
                                ToggleButton::new("tb-bold")
                                    .label("Bold")
                                    .selected(self.toggle_bold)
                                    .on_click(move |new_val, _, _, cx| {
                                        bold_entity.update(cx, |this, cx| {
                                            this.toggle_bold = new_val;
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(
                                ToggleButton::new("tb-italic")
                                    .label("Italic")
                                    .selected(self.toggle_italic)
                                    .on_click(move |new_val, _, _, cx| {
                                        italic_entity.update(cx, |this, cx| {
                                            this.toggle_italic = new_val;
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(
                                ToggleButton::new("tb-disabled")
                                    .label("Disabled")
                                    .disabled(true),
                            ),
                    )
                    .child(Divider::horizontal())
                    // ── TEXT INPUTS ───────────────────────────────────────
                    .child(Label::new("TEXT INPUTS").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .child(div().w(px(190.0)).child(self.text_placeholder.clone()))
                            .child(div().w(px(190.0)).child(self.text_filled.clone()))
                            .child(div().w(px(190.0)).child(self.text_error.clone()))
                            .child(div().w(px(190.0)).child(self.text_disabled.clone())),
                    )
                    .child(Divider::horizontal())
                    // ── FORMS ────────────────────────────────────────────
                    .child(Label::new("FORMS").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(12.0))
                            .child(
                                Field::new(div().w(px(360.0)).child(self.text_filled.clone()))
                                    .label("Connection name")
                                    .required(true)
                                    .helper_text("Required field with helper text."),
                            )
                            .child(
                                Field::new(div().w(px(360.0)).child(self.textarea_notes.clone()))
                                    .label("Notes")
                                    .validation(
                                        FieldValidationState::Success,
                                        "Saved locally for this example.",
                                    ),
                            )
                            .child(
                                Field::new(
                                    RadioGroup::new("density")
                                        .orientation(RadioGroupOrientation::Horizontal)
                                        .selected(self.radio_value.clone())
                                        .option("compact", "Compact")
                                        .option("comfortable", "Comfortable")
                                        .option("spacious", "Spacious")
                                        .on_select(move |value, _, _, cx| {
                                            radio_entity.update(cx, |this, cx| {
                                                this.radio_value = value;
                                                cx.notify();
                                            });
                                        }),
                                )
                                .label("Density"),
                            ),
                    )
                    .child(Divider::horizontal())
                    // ── PICKERS ──────────────────────────────────────────
                    .child(Label::new("PICKERS").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(12.0))
                            .child(
                                Field::new(div().w(px(240.0)).child(self.searchbox.clone()))
                                    .label("Searchbox"),
                            )
                            .child(
                                Field::new(div().w(px(240.0)).child(self.combobox.clone()))
                                    .label("Combobox"),
                            )
                            .child(
                                Field::new(div().w(px(320.0)).child(self.token_input.clone()))
                                    .label("Token input"),
                            )
                            .child(
                                Field::new(div().w(px(240.0)).child(self.date_time_picker.clone()))
                                    .label("Date/time"),
                            ),
                    )
                    .child(Divider::horizontal())
                    // ── DROPDOWN ─────────────────────────────────────────
                    .child(Label::new("DROPDOWN").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .child(div().w(px(190.0)).child(self.dropdown_empty.clone()))
                            .child(div().w(px(190.0)).child(self.dropdown_selected.clone()))
                            .child(div().w(px(190.0)).child(self.dropdown_disabled.clone())),
                    )
                    .child(Divider::horizontal())
                    // ── FEEDBACK ─────────────────────────────────────────
                    .child(Label::new("FEEDBACK").size(LabelSize::Subtitle))
                    .when(self.message_visible, |root| {
                        root.child(
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(8.0))
                                .child(
                                    MessageBar::new("This message bar can be dismissed.")
                                        .title("Informational status")
                                        .intent(MessageIntent::Info)
                                        .on_dismiss(move |_, _, cx| {
                                            message_entity.update(cx, |this, cx| {
                                                this.message_visible = false;
                                                cx.notify();
                                            });
                                        }),
                                )
                                .child(
                                    MessageBar::new("The operation completed successfully.")
                                        .intent(MessageIntent::Success),
                                )
                                .child(
                                    MessageBar::new("Check configuration before continuing.")
                                        .intent(MessageIntent::Warning),
                                )
                                .child(
                                    MessageBar::new("Connection failed.")
                                        .intent(MessageIntent::Error),
                                ),
                        )
                    })
                    .when(!self.message_visible, |root| {
                        root.child(Button::new("reset-message").label("Show message").on_click(
                            move |_, _, cx| {
                                entity.update(cx, |this, cx| {
                                    this.message_visible = true;
                                    cx.notify();
                                });
                            },
                        ))
                    })
                    .child(Divider::horizontal())
                    // ── NAVIGATION & DATA ────────────────────────────────
                    .child(Label::new("NAVIGATION & DATA").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(16.0))
                            .child(
                                div()
                                    .w(px(260.0))
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .child(
                                        Tree::new()
                                            .selected("prod")
                                            .item(
                                                TreeItem::new("connections", "Connections")
                                                    .icon("icons/list_tree.svg")
                                                    .child(
                                                        TreeItem::new("prod", "Production")
                                                            .icon("icons/plug.svg"),
                                                    )
                                                    .child(
                                                        TreeItem::new("stage", "Staging")
                                                            .icon("icons/plug.svg"),
                                                    ),
                                            )
                                            .item(
                                                TreeItem::new("folders", "Folders")
                                                    .icon("icons/folder_open.svg")
                                                    .expanded(false)
                                                    .child(TreeItem::new("archive", "Archive")),
                                            ),
                                    )
                                    .child(
                                        Tree::new()
                                            .items(self.tree_checks.clone())
                                            .on_check(move |id, checked, _, _, cx| {
                                                tree_check_entity.update(cx, |this, cx| {
                                                    set_tree_checked(
                                                        &mut this.tree_checks,
                                                        id.as_ref(),
                                                        checked,
                                                    );
                                                    cx.notify();
                                                });
                                            }),
                                    )
                                    .child(
                                        SelectableList::new("widget-selectable-list")
                                            .selected("incident-2")
                                            .max_body_height(156.0)
                                            .item(
                                                SelectableListItem::new(
                                                    "incident-1",
                                                    "CPU threshold",
                                                )
                                                .subtitle("Production")
                                                .detail("Assigned")
                                                .meta("08:20"),
                                            )
                                            .item(
                                                SelectableListItem::new(
                                                    "incident-2",
                                                    "Gateway failover",
                                                )
                                                .subtitle("Network")
                                                .detail("Investigating")
                                                .meta("09:10")
                                                .badge("P1")
                                                .emphasized(true),
                                            )
                                            .item(
                                                SelectableListItem::new(
                                                    "incident-3",
                                                    "Weekly maintenance",
                                                )
                                                .subtitle("Platform")
                                                .detail("Scheduled")
                                                .meta("Fri"),
                                            ),
                                    ),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .flex()
                                    .flex_col()
                                    .gap(px(8.0))
                                    .child(
                                        Toolbar::new()
                                            .max_visible(3)
                                            .overflow_open(self.toolbar_overflow_open)
                                            .on_overflow_click(move |_, _, cx| {
                                                toolbar_entity.update(cx, |this, cx| {
                                                    this.toolbar_overflow_open =
                                                        !this.toolbar_overflow_open;
                                                    cx.notify();
                                                });
                                            })
                                            .item(ToolbarItem::action_with_icon(
                                                "new",
                                                "New",
                                                "icons/add.svg",
                                                |_, _, _| {},
                                            ))
                                            .item(ToolbarItem::action_with_icon(
                                                "copy",
                                                "Copy",
                                                "icons/copy.svg",
                                                |_, _, _| {},
                                            ))
                                            .item(ToolbarItem::separator())
                                            .item(ToolbarItem::action_with_icon(
                                                "delete",
                                                "Delete",
                                                "icons/delete.svg",
                                                |_, _, _| {},
                                            ))
                                            .item(ToolbarItem::toggle(
                                                "details",
                                                "Details",
                                                true,
                                                |_, _, _, _| {},
                                            )),
                                    )
                                    .child(
                                        DataTable::new()
                                            .sort(
                                                self.table_sort_column.clone(),
                                                self.table_sort_direction,
                                            )
                                            .on_sort(move |column, direction, _, _, cx| {
                                                table_entity.update(cx, |this, cx| {
                                                    this.table_sort_column = column;
                                                    this.table_sort_direction = direction;
                                                    cx.notify();
                                                });
                                            })
                                            .column(
                                                DataColumn::new("name", "Name")
                                                    .width(180.0)
                                                    .sortable(true),
                                            )
                                            .column(
                                                DataColumn::new("kind", "Kind")
                                                    .width(120.0)
                                                    .sortable(true),
                                            )
                                            .column(
                                                DataColumn::new("status", "Status")
                                                    .align(DataCellAlign::End),
                                            )
                                            .row(DataRow::new(
                                                "prod",
                                                vec!["Production", "SSH", "Connected"],
                                            ))
                                            .row(DataRow::new(
                                                "stage",
                                                vec!["Staging", "RDP", "Idle"],
                                            ))
                                            .row(DataRow::new(
                                                "archive",
                                                vec!["Archive", "SFTP", "Offline"],
                                            )),
                                    ),
                            )
                            .child(
                                div().w(px(280.0)).child(
                                    DocumentViewer::html_fragment(
                                        r#"<h2>Status note</h2><p>The document viewer renders <strong>safe rich text</strong>, links, and tables.</p><table><tr><th>Area</th><th>Status</th></tr><tr><td>HTML</td><td>Lossy</td></tr></table>"#,
                                    )
                                    .max_height(220.0),
                                ),
                            ),
                    )
                    .child(Divider::horizontal())
                    // ── STATUS & INDICATORS ───────────────────────────────
                    .child(Label::new("STATUS & INDICATORS").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(12.0))
                            .items_center()
                            .child(Avatar::initials("AB"))
                            .child(Avatar::initials("ZX").size(40.0))
                            .child(Avatar::initials("CD").size(48.0))
                            .child(Badge::dot())
                            .child(Badge::count(3))
                            .child(Badge::count(42))
                            .child(Badge::count(999)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .items_center()
                            .child(Chip::new("chip-static", "Active"))
                            .when(!self.chip_dismissed, |row| {
                                row.child(Chip::new("chip-dismiss", "Tag: Rust").on_dismiss(
                                    move |_, _, cx| {
                                        chip_entity.update(cx, |this, cx| {
                                            this.chip_dismissed = true;
                                            cx.notify();
                                        });
                                    },
                                ))
                            })
                            .when(self.chip_dismissed, |row| {
                                row.child(
                                    Label::new("(chip dismissed — click × to test)")
                                        .size(LabelSize::Caption),
                                )
                            }),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(6.0))
                            .child(ProgressBar::new(0.25))
                            .child(ProgressBar::new(0.6))
                            .child(ProgressBar::new(1.0).height(8.0)),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(12.0))
                            .items_center()
                            .child(Spinner::new())
                            .child(Spinner::new().size(28.0))
                            .child(Spinner::new().size(36.0))
                            .child(Label::new("Animated spinner").size(LabelSize::Caption)),
                    )
                    .child(Divider::horizontal())
                    // ── TOOLTIP ───────────────────────────────────────────
                    .child(Label::new("TOOLTIP").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(16.0))
                            .items_center()
                            .child(
                                Tooltip::new("Saves your work to disk").trigger(
                                    Button::new("tt-save")
                                        .label("Save")
                                        .appearance(ButtonAppearance::Accent),
                                ),
                            )
                            .child(
                                Tooltip::new("Opens the search panel")
                                    .trigger(Button::new("tt-search").label("Search")),
                            ),
                    )
                    .child(Divider::horizontal())
                    // ── DIVIDERS ──────────────────────────────────────────
                    .child(Label::new("DIVIDERS").size(LabelSize::Subtitle))
                    .child(Divider::horizontal())
                    .child(
                        div()
                            .h(px(40.0))
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(12.0))
                            .child(Label::new("Left panel"))
                            .child(Divider::vertical())
                            .child(Label::new("Right panel")),
                    ),
            )
    }
}

fn main() {
    FluentApp::new("FluentGUI — Widget Gallery")
        .window_size(900.0, 720.0)
        .run(|cx| cx.new(|cx| WidgetGallery::new(cx)));
}
