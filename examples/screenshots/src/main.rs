// Screenshot capture binary for FluentGUI primitives.
//
// Usage:
//   screenshots <component> [--dark|--light]
//
// Renders a single component in a fullscreen chromeless window, writes
// /tmp/.fluent-screenshot-ready after the first GPU frame, then blocks.
// The companion script (scripts/screenshots.sh) detects the sentinel,
// calls `grim`, and kills this process.
//
// Component names: button icon-button toggle-button checkbox switch label
//   badge chip divider progress-bar spinner skeleton avatar app-dot
//   connection-badge text-input textarea searchbox dropdown combobox
//   radio-group field card section-header states message-bar toolbar tree
//   tab-strip title-bar ribbon settings-nav data-table selectable-list
//   document-viewer token-input date-time-picker toast dialog flyout popover
//   tooltip suggestion-popover context-menu command-palette menu-bar-open
//   submenu-open
//
// Open popups are placed at fixed window coordinates (the window is 1000x600).
// menu-bar-open and submenu-open open the File menu by sending keystrokes
// (Alt+F, then Down Down Right) after the first frame.

use std::fs;

use fluent_app::FluentAssets;
use fluent_app::TitleBar;
use fluent_core::gpui::{
    div, point, prelude::*, px, size, AnyView, App, Bounds, Context, Entity, IntoElement,
    Keystroke, Point, SharedString, Size, TitlebarOptions, Window, WindowBounds, WindowDecorations,
    WindowOptions,
};
use fluent_core::{Theme, ThemeProvider as _};
use fluent_layout::{
    modal::ModalStack, CommandPalette, ContextMenu, ContextMenuItem, DataCellAlign, DataColumn,
    DataRow, DataStatusKind, DataTable, DataTableState, Dialog, DialogAction, DocumentViewer,
    Flyout, MenuBar, MenuItemDef, MessageBar, MessageIntent, PaletteEntry, Popover, SelectableList,
    SelectableListItem, SettingsNav, SettingsNavItem, SettingsNavSection, TabItem, TabStrip, Toast,
    ToastHost, ToastStack, Toolbar, ToolbarItem, Tree, TreeItem,
};
use fluent_primitives::{
    AppDot, Avatar, Badge, Button, ButtonAppearance, ButtonSize, Card, Checkbox, CheckboxState,
    Chip, ConnectionBadge, ConnectionBadgeState, Divider, Dropdown, Field, FieldValidationState,
    Icon, IconButton, IconSize, Label, LabelSize, ProgressBar, RadioGroup, RadioGroupOrientation,
    Searchbox, SectionHeader, Skeleton, SkeletonRow, Spinner, Switch, TextInput, Textarea,
    ToggleButton,
};
use fluent_primitives::{
    DateTimePicker, DateTimeValue, SuggestionItem, SuggestionPopover, TokenInput,
    TokenInputSuggestion, TokenInputToken, Tooltip,
};
use fluent_ribbon::RibbonBar;

const SENTINEL: &str = "/tmp/.fluent-screenshot-ready";

// ── Root view ─────────────────────────────────────────────────────────────

struct Screenshots {
    component: SharedString,
    ready_signaled: bool,
    // Entity-based stateful components (must persist across renders)
    text_placeholder: Entity<TextInput>,
    text_filled: Entity<TextInput>,
    text_error: Entity<TextInput>,
    text_disabled: Entity<TextInput>,
    textarea: Entity<Textarea>,
    searchbox: Entity<Searchbox>,
    dropdown: Entity<Dropdown>,
    combobox: Entity<fluent_primitives::Combobox>,
    /// The stateful view for the layout and overlay components that need one.
    view: Option<AnyView>,
}

impl Screenshots {
    fn new(component: impl Into<SharedString>, cx: &mut App) -> Self {
        let component = component.into();
        let view = stateful_view(&component, cx);
        Self {
            component,
            // Tests render in-process and must not touch the sentinel file.
            ready_signaled: cfg!(test),
            view,
            text_placeholder: cx.new(|_| TextInput::new().placeholder("Placeholder text")),
            text_filled: cx.new(|_| TextInput::new().value("Hello, FluentGUI")),
            text_error: cx.new(|_| TextInput::new().value("bad@input").error(true)),
            text_disabled: cx.new(|_| TextInput::new().value("Read-only").disabled(true)),
            textarea: cx.new(|_| {
                Textarea::new()
                    .placeholder("Notes")
                    .value("Multi-line text input\nsupports several rows.")
                    .rows(4)
            }),
            searchbox: cx.new(|_| Searchbox::new().placeholder("Search connections…")),
            dropdown: cx.new(|_| {
                Dropdown::new("dd-ss")
                    .placeholder("Choose protocol")
                    .option("ssh", "SSH")
                    .option("rdp", "RDP")
                    .option("vnc", "VNC")
                    .option("sftp", "SFTP")
                    .selected("ssh")
            }),
            combobox: cx.new(|_| {
                fluent_primitives::Combobox::new()
                    .placeholder("Protocol")
                    .option("ssh", "SSH")
                    .option("rdp", "RDP")
                    .option("vnc", "VNC")
                    .option("sftp", "SFTP")
            }),
        }
    }
}

impl Render for Screenshots {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.ready_signaled {
            self.ready_signaled = true;
            let keys = menu_keys(&self.component);
            window.on_next_frame(move |window, cx| {
                for key in keys {
                    if let Ok(keystroke) = Keystroke::parse(key) {
                        window.dispatch_keystroke(keystroke, cx);
                    }
                }
                let _ = fs::write(SENTINEL, "1");
            });
        }

        let colors = cx.theme().colors.clone();

        // Clone entities before borrowing self in the match
        let text_placeholder = self.text_placeholder.clone();
        let text_filled = self.text_filled.clone();
        let text_error = self.text_error.clone();
        let text_disabled = self.text_disabled.clone();
        let textarea = self.textarea.clone();
        let searchbox = self.searchbox.clone();
        let dropdown = self.dropdown.clone();
        let combobox = self.combobox.clone();

        let panel_content: fluent_core::gpui::AnyElement = match self.component.as_ref() {
            "button" => ss_buttons().into_any_element(),
            "icon-button" => ss_icon_buttons().into_any_element(),
            "toggle-button" => ss_toggle_buttons().into_any_element(),
            "checkbox" => ss_checkboxes().into_any_element(),
            "switch" => ss_switches().into_any_element(),
            "label" => ss_labels().into_any_element(),
            "badge" => ss_badges().into_any_element(),
            "chip" => ss_chips().into_any_element(),
            "divider" => ss_dividers().into_any_element(),
            "progress-bar" => ss_progress().into_any_element(),
            "spinner" => ss_spinners().into_any_element(),
            "skeleton" => ss_skeletons().into_any_element(),
            "avatar" => ss_avatars().into_any_element(),
            "app-dot" => ss_app_dots().into_any_element(),
            "connection-badge" => ss_connection_badges().into_any_element(),
            "text-input" => {
                ss_text_inputs(text_placeholder, text_filled, text_error, text_disabled)
                    .into_any_element()
            }
            "textarea" => ss_textarea(textarea).into_any_element(),
            "searchbox" => ss_searchbox(searchbox).into_any_element(),
            "dropdown" => ss_dropdown(dropdown).into_any_element(),
            "combobox" => ss_combobox(combobox).into_any_element(),
            "radio-group" => ss_radio_group().into_any_element(),
            "field" => ss_field(text_filled, textarea).into_any_element(),
            "card" => ss_cards().into_any_element(),
            "section-header" => ss_section_headers().into_any_element(),
            "states" => ss_states().into_any_element(),
            "message-bar" => ss_message_bars().into_any_element(),
            "toolbar" => ss_toolbars().into_any_element(),
            "tree" => ss_trees().into_any_element(),
            "data-table" => ss_data_tables().into_any_element(),
            "selectable-list" => ss_selectable_lists().into_any_element(),
            "document-viewer" => ss_document_viewers().into_any_element(),
            "dialog" => ss_dialog().into_any_element(),
            "flyout" => ss_flyout().into_any_element(),
            "popover" => ss_popover().into_any_element(),
            "tooltip" => ss_tooltip().into_any_element(),
            "suggestion-popover" => ss_suggestion_popover().into_any_element(),
            "tab-strip" | "title-bar" | "ribbon" | "settings-nav" | "token-input"
            | "date-time-picker" | "toast" | "context-menu" | "command-palette"
            | "menu-bar-open" | "submenu-open" => match self.view.clone() {
                Some(view) => view.into_any_element(),
                None => div().child(Label::new("Missing view")).into_any_element(),
            },
            other => div()
                .p(px(24.0))
                .child(Label::new(format!("Unknown component: {other}")))
                .into_any_element(),
        };

        div()
            .id("screenshot-root")
            .size_full()
            .bg(colors.surface_dim)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .p(px(40.0))
                    .rounded(px(8.0))
                    .bg(colors.surface)
                    .child(panel_content),
            )
    }
}

// ── Per-component render functions ────────────────────────────────────────

fn row() -> fluent_core::gpui::Div {
    div().flex().flex_row().items_center().gap(px(8.0))
}

fn col() -> fluent_core::gpui::Div {
    div().flex().flex_col().gap(px(12.0))
}

fn group(title: impl Into<SharedString>, content: impl IntoElement) -> impl IntoElement {
    div()
        .flex()
        .flex_col()
        .gap(px(8.0))
        .child(Label::new(title).size(LabelSize::Caption))
        .child(content)
}

// ── Button ────────────────────────────────────────────────────────────────
fn ss_buttons() -> impl IntoElement {
    col()
        .child(group(
            "Appearance",
            row()
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
        ))
        .child(group(
            "Size",
            row()
                .child(Button::new("b-default").label("Default"))
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
                ),
        ))
}

// ── IconButton ────────────────────────────────────────────────────────────
fn ss_icon_buttons() -> impl IntoElement {
    col()
        .child(group(
            "Icon sizes",
            row()
                .child(Icon::new("icons/add.svg").size(IconSize::Sm))
                .child(Icon::new("icons/add.svg").size(IconSize::Md))
                .child(Icon::new("icons/add.svg").size(IconSize::Lg))
                .child(Icon::new("icons/add.svg").size(IconSize::Xl)),
        ))
        .child(group(
            "IconButton",
            row()
                .child(IconButton::new("ib-default", "icons/search.svg"))
                .child(
                    IconButton::new("ib-accent", "icons/search.svg")
                        .appearance(ButtonAppearance::Accent),
                )
                .child(
                    IconButton::new("ib-subtle", "icons/settings.svg")
                        .appearance(ButtonAppearance::Subtle),
                )
                .child(IconButton::new("ib-disabled", "icons/search.svg").disabled(true)),
        ))
}

// ── ToggleButton ──────────────────────────────────────────────────────────
fn ss_toggle_buttons() -> impl IntoElement {
    group(
        "States",
        row()
            .child(
                ToggleButton::new("tb-off")
                    .label("Unselected")
                    .selected(false),
            )
            .child(ToggleButton::new("tb-on").label("Selected").selected(true))
            .child(
                ToggleButton::new("tb-disabled")
                    .label("Disabled")
                    .disabled(true),
            ),
    )
}

// ── Checkbox ──────────────────────────────────────────────────────────────
fn ss_checkboxes() -> impl IntoElement {
    group(
        "States",
        row()
            .child(
                row()
                    .child(Checkbox::new("cb-off").state(CheckboxState::Unchecked))
                    .child(Label::new("Unchecked").size(LabelSize::Caption)),
            )
            .child(
                row()
                    .child(Checkbox::new("cb-on").state(CheckboxState::Checked))
                    .child(Label::new("Checked").size(LabelSize::Caption)),
            )
            .child(
                row()
                    .child(Checkbox::new("cb-indet").state(CheckboxState::Indeterminate))
                    .child(Label::new("Indeterminate").size(LabelSize::Caption)),
            )
            .child(
                row()
                    .child(
                        Checkbox::new("cb-disabled")
                            .state(CheckboxState::Unchecked)
                            .disabled(true),
                    )
                    .child(Label::new("Disabled").size(LabelSize::Caption)),
            ),
    )
}

// ── Switch ────────────────────────────────────────────────────────────────
fn ss_switches() -> impl IntoElement {
    group(
        "States",
        row()
            .child(
                row()
                    .child(Switch::new("sw-off").on(false))
                    .child(Label::new("Off").size(LabelSize::Caption)),
            )
            .child(
                row()
                    .child(Switch::new("sw-on").on(true))
                    .child(Label::new("On").size(LabelSize::Caption)),
            )
            .child(
                row()
                    .child(Switch::new("sw-disabled").on(false).disabled(true))
                    .child(Label::new("Disabled").size(LabelSize::Caption)),
            ),
    )
}

// ── Label ─────────────────────────────────────────────────────────────────
fn ss_labels() -> impl IntoElement {
    col()
        .child(group(
            "Sizes",
            col()
                .child(Label::new("Caption — 12pt").size(LabelSize::Caption))
                .child(Label::new("Body — 14pt").size(LabelSize::Body))
                .child(Label::new("Subtitle — 16pt semibold").size(LabelSize::Subtitle))
                .child(Label::new("Title — 20pt semibold").size(LabelSize::Title))
                .child(Label::new("Display — 28pt semibold").size(LabelSize::Display)),
        ))
        .child(group("Eyebrow", Label::eyebrow("SECTION LABEL")))
}

// ── Badge ─────────────────────────────────────────────────────────────────
fn ss_badges() -> impl IntoElement {
    group(
        "Variants",
        row()
            .child(Badge::count(3))
            .child(Badge::count(42))
            .child(Badge::dot()),
    )
}

// ── Chip ──────────────────────────────────────────────────────────────────
fn ss_chips() -> impl IntoElement {
    group(
        "Chip",
        row()
            .child(Chip::new("chip-1", "SSH"))
            .child(Chip::new("chip-2", "RDP"))
            .child(Chip::new("chip-3", "VNC")),
    )
}

// ── Divider ───────────────────────────────────────────────────────────────
fn ss_dividers() -> impl IntoElement {
    col()
        .w(px(400.0))
        .child(group("Horizontal", Divider::horizontal()))
        .child(group(
            "Vertical",
            div()
                .h(px(40.0))
                .flex()
                .flex_row()
                .gap(px(16.0))
                .items_center()
                .child(Label::new("Left"))
                .child(Divider::vertical())
                .child(Label::new("Right")),
        ))
}

// ── ProgressBar ───────────────────────────────────────────────────────────
fn ss_progress() -> impl IntoElement {
    col()
        .w(px(360.0))
        .child(group("25%", ProgressBar::new(0.25).id("ss-pg-25")))
        .child(group("60%", ProgressBar::new(0.60).id("ss-pg-60")))
        .child(group("100%", ProgressBar::new(1.0).id("ss-pg-100")))
}

// ── Spinner ───────────────────────────────────────────────────────────────
fn ss_spinners() -> impl IntoElement {
    group(
        "Sizes",
        row()
            .child(Spinner::new().size(16.0).id("ss-sp-16"))
            .child(Spinner::new().size(24.0).id("ss-sp-24"))
            .child(Spinner::new().size(32.0).id("ss-sp-32"))
            .child(Spinner::new().size(48.0).id("ss-sp-48")),
    )
}

// ── Skeleton ──────────────────────────────────────────────────────────────
fn ss_skeletons() -> impl IntoElement {
    col()
        .w(px(360.0))
        .child(group(
            "Blocks",
            col()
                .child(Skeleton::new("sk-1").w(360.0).h(16.0))
                .child(Skeleton::new("sk-2").w(280.0).h(16.0))
                .child(Skeleton::new("sk-3").w(320.0).h(16.0)),
        ))
        .child(group("Row with avatar", SkeletonRow::new("skr-1")))
}

// ── Avatar ────────────────────────────────────────────────────────────────
fn ss_avatars() -> impl IntoElement {
    group(
        "Initials & sizes",
        row()
            .child(Avatar::initials("AB").size(24.0))
            .child(Avatar::initials("CD").size(32.0))
            .child(Avatar::initials("EF").size(40.0))
            .child(Avatar::initials("GH").size(48.0).hue(210.0))
            .child(Avatar::initials("IJ").size(48.0).hue(30.0)),
    )
}

// ── AppDot ────────────────────────────────────────────────────────────────
fn ss_app_dots() -> impl IntoElement {
    group(
        "Color seeds",
        row()
            .child(AppDot::new("ssh"))
            .child(AppDot::new("rdp"))
            .child(AppDot::new("vnc"))
            .child(AppDot::new("sftp"))
            .child(AppDot::new("telnet")),
    )
}

// ── ConnectionBadge ───────────────────────────────────────────────────────
fn ss_connection_badges() -> impl IntoElement {
    group(
        "States",
        row()
            .child(ConnectionBadge::new(
                "cb-disc",
                ConnectionBadgeState::Disconnected,
            ))
            .child(ConnectionBadge::new(
                "cb-pair",
                ConnectionBadgeState::Pairing,
            ))
            .child(ConnectionBadge::new(
                "cb-conn",
                ConnectionBadgeState::Paired,
            ))
            .child(ConnectionBadge::new("cb-err", ConnectionBadgeState::Error)),
    )
}

// ── TextInput ─────────────────────────────────────────────────────────────
fn ss_text_inputs(
    placeholder: Entity<TextInput>,
    filled: Entity<TextInput>,
    error: Entity<TextInput>,
    disabled: Entity<TextInput>,
) -> impl IntoElement {
    group(
        "States",
        row()
            .child(div().w(px(190.0)).child(placeholder))
            .child(div().w(px(190.0)).child(filled))
            .child(div().w(px(190.0)).child(error))
            .child(div().w(px(190.0)).child(disabled)),
    )
}

// ── Textarea ──────────────────────────────────────────────────────────────
fn ss_textarea(textarea: Entity<Textarea>) -> impl IntoElement {
    group("Textarea", div().w(px(360.0)).child(textarea))
}

// ── Searchbox ─────────────────────────────────────────────────────────────
fn ss_searchbox(searchbox: Entity<Searchbox>) -> impl IntoElement {
    group("Searchbox", div().w(px(280.0)).child(searchbox))
}

// ── Dropdown ──────────────────────────────────────────────────────────────
fn ss_dropdown(dropdown: Entity<Dropdown>) -> impl IntoElement {
    group("Dropdown", div().w(px(220.0)).child(dropdown))
}

// ── Combobox ──────────────────────────────────────────────────────────────
fn ss_combobox(combobox: Entity<fluent_primitives::Combobox>) -> impl IntoElement {
    group("Combobox", div().w(px(220.0)).child(combobox))
}

// ── RadioGroup ────────────────────────────────────────────────────────────
fn ss_radio_group() -> impl IntoElement {
    col()
        .child(group(
            "Vertical",
            RadioGroup::new("rg-vert")
                .option("compact", "Compact")
                .option("comfortable", "Comfortable")
                .option("spacious", "Spacious")
                .selected("comfortable"),
        ))
        .child(group(
            "Horizontal",
            RadioGroup::new("rg-horiz")
                .orientation(RadioGroupOrientation::Horizontal)
                .option("light", "Light")
                .option("dark", "Dark")
                .option("system", "System")
                .selected("dark"),
        ))
}

// ── Field ─────────────────────────────────────────────────────────────────
fn ss_field(name: Entity<TextInput>, notes: Entity<Textarea>) -> impl IntoElement {
    col()
        .w(px(360.0))
        .child(
            Field::new(div().w_full().child(name))
                .label("Connection name")
                .required(true)
                .helper_text("A unique label for this connection."),
        )
        .child(
            Field::new(div().w_full().child(notes))
                .label("Notes")
                .validation(FieldValidationState::Success, "Saved."),
        )
}

// ── Card ──────────────────────────────────────────────────────────────────
fn ss_cards() -> impl IntoElement {
    row()
        .items_start()
        .child(
            Card::new()
                .child(Label::new("Default card").size(LabelSize::Body))
                .child(Label::new("Supporting content goes here.").size(LabelSize::Caption)),
        )
        .child(
            Card::new()
                .hoverable()
                .child(Label::new("Hoverable card").size(LabelSize::Body))
                .child(Label::new("Responds to pointer interaction.").size(LabelSize::Caption)),
        )
}

// ── SectionHeader ─────────────────────────────────────────────────────────
fn ss_section_headers() -> impl IntoElement {
    col()
        .w(px(400.0))
        .child(SectionHeader::new("Connections"))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(Label::new("server-01.local").size(LabelSize::Body))
                .child(Label::new("server-02.local").size(LabelSize::Body)),
        )
        .child(SectionHeader::new("Recent"))
        .child(Label::new("home-nas.local").size(LabelSize::Body))
}

// ── Loading / validation states ───────────────────────────────────────────
fn ss_states() -> impl IntoElement {
    col()
        .w(px(360.0))
        .child(group(
            "Indeterminate progress",
            ProgressBar::new(0.0).indeterminate(),
        ))
        .child(group("Spinner", Spinner::new().size(24.0)))
        .child(group(
            "Disabled controls",
            row()
                .child(Button::new("st-btn").label("Disabled").disabled(true))
                .child(Switch::new("st-sw").label("Off").disabled(true)),
        ))
        .child(group(
            "Field validation",
            col()
                .child(
                    Field::new(div().w_full().child(Label::new("value")))
                        .label("Error")
                        .validation(FieldValidationState::Error, "Something is wrong."),
                )
                .child(
                    Field::new(div().w_full().child(Label::new("value")))
                        .label("Warning")
                        .validation(FieldValidationState::Warning, "Check this value."),
                ),
        ))
}

// ── Layout, data and overlay components ──────────────────────────────────

/// Every component name this binary renders, in the order the script captures
/// them. `scripts/screenshots.sh` `ALL_COMPONENTS` lists the same names; a
/// test keeps the two in step.
#[cfg(test)]
const COMPONENTS: &[&str] = &[
    "button",
    "icon-button",
    "toggle-button",
    "checkbox",
    "switch",
    "label",
    "badge",
    "chip",
    "divider",
    "progress-bar",
    "spinner",
    "skeleton",
    "avatar",
    "app-dot",
    "connection-badge",
    "text-input",
    "textarea",
    "searchbox",
    "dropdown",
    "combobox",
    "radio-group",
    "field",
    "card",
    "section-header",
    "states",
    "message-bar",
    "toolbar",
    "tree",
    "data-table",
    "selectable-list",
    "document-viewer",
    "token-input",
    "date-time-picker",
    "tooltip",
    "suggestion-popover",
    "tab-strip",
    "title-bar",
    "ribbon",
    "settings-nav",
    "toast",
    "dialog",
    "flyout",
    "popover",
    "context-menu",
    "command-palette",
    "menu-bar-open",
    "submenu-open",
];

fn file_menu_items() -> Vec<ContextMenuItem> {
    vec![
        MenuItemDef::action_with_shortcut("New", "Ctrl+N", |_, _, _| {}),
        MenuItemDef::action_with_shortcut("Open...", "Ctrl+O", |_, _, _| {}),
        MenuItemDef::submenu(
            "Open Recent",
            vec![
                MenuItemDef::action("Project One", |_, _, _| {}),
                MenuItemDef::action("Project Two", |_, _, _| {}),
            ],
        ),
        MenuItemDef::separator(),
        MenuItemDef::action_with_shortcut("Exit", "Alt+F4", |_, _, _| {}),
    ]
}

/// Keystrokes that open the menu state a component shows, sent after the first frame.
fn menu_keys(component: &str) -> &'static [&'static str] {
    match component {
        "menu-bar-open" => &["alt-f"],
        "submenu-open" => &["alt-f", "down", "down", "right"],
        _ => &[],
    }
}

/// Builds the entity for components that hold state; others render from plain functions.
fn stateful_view(component: &str, cx: &mut App) -> Option<AnyView> {
    Some(match component {
        "tab-strip" => cx
            .new(|cx| {
                let mut ts = TabStrip::new(cx);
                ts.add_tab(TabItem::new("one").label("Dashboard").closable(true));
                ts.add_tab(TabItem::new("two").label("Settings").closable(true));
                ts
            })
            .into(),
        "title-bar" => cx
            .new(|cx| TitleBar::new(cx, "FluentGUI Screenshots").show_controls(false))
            .into(),
        "ribbon" => cx
            .new(|cx| {
                RibbonBar::new(cx).tab("Home", |t| {
                    t.group("Actions", |g| {
                        g.large_button("New", "icons/add.svg", |_, _, _| {}).button(
                            "Copy",
                            "icons/copy.svg",
                            |_, _, _| {},
                        )
                    })
                })
            })
            .into(),
        "settings-nav" => cx
            .new(|cx| {
                SettingsNav::new(cx)
                    .section(
                        SettingsNavSection::new("General")
                            .item(SettingsNavItem::new("a", "Appearance"))
                            .item(SettingsNavItem::new("b", "Network")),
                    )
                    .section(
                        SettingsNavSection::new("Advanced")
                            .item(SettingsNavItem::new("c", "Logging")),
                    )
            })
            .into(),
        "token-input" => cx
            .new(|_| {
                TokenInput::new()
                    .placeholder("Add recipients")
                    .tokens([TokenInputToken::new("alex@example.com", "Alex")])
                    .suggestions([TokenInputSuggestion::new("bianca@example.com", "Bianca")])
            })
            .into(),
        "date-time-picker" => cx
            .new(|_| DateTimePicker::new().maybe_value(DateTimeValue::new(2026, 5, 25, 9, 30)))
            .into(),
        "toast" => {
            ToastStack::init(cx);
            ToastStack::push(
                Toast::new("t1", "Something happened.")
                    .title("Toast")
                    .intent(MessageIntent::Success)
                    .persistent(),
                cx,
            );
            cx.new(ToastHost::new).into()
        }
        "context-menu" => cx
            .new(|_| {
                file_menu_items()
                    .into_iter()
                    .fold(ContextMenu::build(point(px(380.0), px(120.0))), |m, i| {
                        m.item(i)
                    })
            })
            .into(),
        "command-palette" => cx
            .new(|cx| {
                let mut palette = CommandPalette::new(cx).entries(vec![
                    PaletteEntry::new("open", "Open connection").group("File"),
                    PaletteEntry::new("theme", "Toggle theme").group("View"),
                    PaletteEntry::new("quit", "Quit")
                        .group("File")
                        .disabled(true),
                ]);
                palette.show(cx);
                palette
            })
            .into(),
        "menu-bar-open" | "submenu-open" => cx
            .new(|cx| {
                MenuBar::new(cx)
                    .menu("&File", file_menu_items())
                    .menu("&Edit", vec![MenuItemDef::action("Undo", |_, _, _| {})])
            })
            .into(),
        _ => return None,
    })
}

fn ss_message_bars() -> impl IntoElement {
    let bar = |id: &'static str, intent| {
        div().w(px(420.0)).child(
            MessageBar::new("Something happened.")
                .id(id)
                .intent(intent)
                .title(id)
                .action("Retry", |_, _, _| {})
                .on_dismiss(|_, _, _| {}),
        )
    };
    col()
        .child(bar("info", MessageIntent::Info))
        .child(bar("success", MessageIntent::Success))
        .child(bar("warning", MessageIntent::Warning))
        .child(bar("error", MessageIntent::Error))
}

fn ss_toolbars() -> impl IntoElement {
    let items = || {
        vec![
            ToolbarItem::action_with_icon("tb-new", "New", "icons/add.svg", |_, _, _| {}),
            ToolbarItem::separator(),
            ToolbarItem::toggle("tb-bold", "Bold", true, |_, _, _, _| {}),
            ToolbarItem::action("tb-find", "Find", |_, _, _| {}),
            ToolbarItem::action("tb-dis", "Disabled", |_, _, _| {}).disabled(true),
        ]
    };
    let toolbar = |id: &'static str, width: f32, open: bool| {
        div().w(px(width)).child(
            Toolbar::new()
                .id(id)
                .items(items())
                .available_width(px(width))
                .overflow_open(open),
        )
    };
    col()
        .child(group("Wide", toolbar("tb-wide", 520.0, false)))
        .child(group(
            "Narrow with overflow",
            toolbar("tb-narrow", 200.0, false),
        ))
        .child(group("Overflow open", toolbar("tb-open", 200.0, true)))
}

fn ss_trees() -> impl IntoElement {
    let frame = |id: &'static str, tree: Tree| div().id(id).w(px(240.0)).h(px(150.0)).child(tree);
    row()
        .items_start()
        .child(frame(
            "ss-tree",
            Tree::new().id("ss-tree-v").item(
                TreeItem::new("root", "Workspace")
                    .expanded(true)
                    .child(TreeItem::new("a", "Alpha"))
                    .child(TreeItem::new("b", "Beta").disabled(true))
                    .child(TreeItem::new("lazy", "Lazy").lazy(true).loading(true)),
            ),
        ))
        .child(frame(
            "ss-tree-load",
            Tree::new()
                .id("ss-tree-l")
                .status(DataStatusKind::Loading, "Loading items"),
        ))
        .child(frame(
            "ss-tree-err",
            Tree::new()
                .id("ss-tree-e")
                .status(DataStatusKind::Error, "Could not load items"),
        ))
}

fn ss_data_tables() -> impl IntoElement {
    let table = |state: Option<DataTableState>| {
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
    };
    let frame = |id: &'static str, t: DataTable| div().id(id).w(px(340.0)).h(px(140.0)).child(t);
    col()
        .child(frame("ss-dt-ready", table(None)))
        .child(frame(
            "ss-dt-load",
            table(Some(DataTableState::Loading("Loading rows".into()))),
        ))
        .child(frame(
            "ss-dt-err",
            table(Some(DataTableState::Error("Could not load rows".into()))),
        ))
}

fn ss_selectable_lists() -> impl IntoElement {
    let frame =
        |id: &'static str, l: SelectableList| div().id(id).w(px(280.0)).h(px(150.0)).child(l);
    row()
        .items_start()
        .child(frame(
            "ss-sl-ready",
            SelectableList::new("ss-sl-r")
                .item(SelectableListItem::new("a", "CPU threshold").subtitle("Production"))
                .item(
                    SelectableListItem::new("b", "Gateway failover")
                        .badge("P1")
                        .emphasized(true),
                )
                .item(SelectableListItem::new("c", "Maintenance").disabled(true))
                .selected("b"),
        ))
        .child(frame(
            "ss-sl-load",
            SelectableList::new("ss-sl-l").loading("Loading items"),
        ))
        .child(frame(
            "ss-sl-err",
            SelectableList::new("ss-sl-e").error_message("Could not load items"),
        ))
}

fn ss_document_viewers() -> impl IntoElement {
    col()
        .child(
            div().id("ss-dv").w(px(360.0)).h(px(150.0)).child(
                DocumentViewer::html_fragment(
                    "<h2>Status note</h2><p>Renders <strong>safe rich text</strong> and links.</p>",
                )
                .max_height(140.0),
            ),
        )
        .child(
            div()
                .id("ss-dv-empty")
                .w(px(360.0))
                .child(DocumentViewer::plain_text("").empty_message("Nothing to show")),
        )
}

fn ss_dialog() -> impl IntoElement {
    div().w(px(420.0)).child(
        Dialog::new("Delete connection?")
            .body(Label::new("This removes the saved connection."))
            .action(DialogAction::new("Cancel", |_, _, _| {}))
            .action(DialogAction::new("Delete", |_, _, _| {}).primary(true)),
    )
}

fn popup_surface() -> fluent_core::gpui::Div {
    div().p(px(12.0))
}

fn ss_flyout() -> impl IntoElement {
    let trigger = Bounds::new(point(px(120.0), px(120.0)), size(px(80.0), px(32.0)));
    div()
        .child(Button::new("fly-trigger").label("Trigger"))
        .child(Flyout::new(
            trigger,
            popup_surface().child(Label::new("Flyout content")),
        ))
}

fn ss_popover() -> impl IntoElement {
    div().child(Label::new("Popover below")).child(Popover::new(
        point(px(120.0), px(150.0)),
        popup_surface().child(Label::new("Popover content")),
    ))
}

fn ss_tooltip() -> impl IntoElement {
    // The tooltip shows on hover; the page captures the trigger it belongs to.
    Tooltip::new("Saves your work to disk").trigger(Button::new("tt-save").label("Save"))
}

fn ss_suggestion_popover() -> impl IntoElement {
    div().child(Label::new("Suggestions below")).child(
        SuggestionPopover::new(point(px(120.0), px(150.0)))
            .items([
                SuggestionItem::new("alpha", "Alpha").detail("First"),
                SuggestionItem::new("beta", "Beta").detail("Second"),
                SuggestionItem::new("gamma", "Gamma").disabled(true),
            ])
            .highlighted(0),
    )
}

// ── Entry point ───────────────────────────────────────────────────────────

/// Global services every harness window needs: the modal and toast stacks and
/// the Tab / Shift-Tab bindings (without them a real Tab key does nothing, WI-4840).
fn init_services(cx: &mut App) {
    ModalStack::init(cx);
    ToastStack::init(cx);
    fluent_primitives::install_focus_keybindings(cx);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let component = args.get(1).cloned().unwrap_or_else(|| "button".into());
    let dark = !args.iter().any(|a| a == "--light");

    fluent_core::gpui_platform::application()
        .with_assets(FluentAssets::new(None))
        .run(move |cx: &mut App| {
            if dark {
                Theme::init(cx);
            } else {
                cx.set_global(Theme::light());
            }
            init_services(cx);

            cx.open_window(
                WindowOptions {
                    // Fullscreen crashes the AMD Vulkan driver on this GNOME compositor.
                    // Use a fixed window size that fits all components but is smaller
                    // than any monitor so the desktop remains visible at the corners
                    // (required for the autocrop pass in the companion script).
                    window_bounds: Some(WindowBounds::Windowed(Bounds {
                        origin: Point::new(px(0.), px(0.)),
                        size: Size {
                            width: px(1000.),
                            height: px(600.),
                        },
                    })),
                    titlebar: Some(TitlebarOptions {
                        title: Some("FluentGUI-Screenshots".into()),
                        appears_transparent: true,
                        traffic_light_position: None,
                    }),
                    window_decorations: Some(WindowDecorations::Client),
                    focus: true,
                    show: true,
                    ..Default::default()
                },
                move |_window, cx: &mut App| cx.new(|cx| Screenshots::new(component.clone(), cx)),
            )
            .unwrap();

            cx.activate(true);
        });
}

#[cfg(test)]
mod tests {
    //! In-process tests: no window system and no GPU, so they check that every
    //! component renders and that the lists stay in step, not how it looks.

    use super::*;
    use fluent_core::gpui::TestAppContext;
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};

    /// The names in `scripts/screenshots.sh` `ALL_COMPONENTS`.
    fn script_components() -> Vec<String> {
        let script = include_str!("../../../scripts/screenshots.sh");
        let start = script
            .find("ALL_COMPONENTS=(")
            .map(|i| i + "ALL_COMPONENTS=(".len())
            .expect("script has an ALL_COMPONENTS array");
        let end = start + script[start..].find(')').expect("array is closed");
        script[start..end]
            .split_whitespace()
            .map(str::to_owned)
            .collect()
    }

    fn snapshot_of(component: &str) -> Vec<String> {
        let mut app = TestAppContext::single();
        let name = component.to_owned();
        let (_view, cx) = open_fluent_test_window(&mut app, move |_, cx| {
            ModalStack::init(cx);
            ToastStack::init(cx);
            Screenshots::new(name.clone(), cx)
        });
        cx.accessibility_snapshot()
    }

    #[test]
    fn components_match_the_screenshot_script() {
        let mut script = script_components();
        let mut known: Vec<String> = COMPONENTS.iter().map(|c| (*c).to_owned()).collect();
        script.sort();
        known.sort();
        assert_eq!(
            script, known,
            "scripts/screenshots.sh ALL_COMPONENTS and COMPONENTS differ"
        );
    }

    #[test]
    fn every_component_renders() {
        for component in COMPONENTS {
            let snapshot = snapshot_of(component);
            assert!(
                !snapshot.iter().any(|n| n.contains("Unknown component")),
                "{component} has no render arm"
            );
            assert!(!snapshot.is_empty(), "{component} rendered no nodes");
        }
    }

    #[test]
    fn menu_components_open_their_menus_from_keystrokes() {
        for (component, expect) in [
            ("menu-bar-open", "Open Recent"),
            ("submenu-open", "Project One"),
        ] {
            let mut app = TestAppContext::single();
            let name = component.to_owned();
            let (_view, cx) =
                open_fluent_test_window(&mut app, move |_, cx| Screenshots::new(name.clone(), cx));
            cx.type_keys(&menu_keys(component).join(" "));
            let snapshot = cx.accessibility_snapshot();
            assert!(
                snapshot.iter().any(|n| n.contains(expect)),
                "{component}: no {expect:?} in {snapshot:?}"
            );
        }
    }

    /// WI-4840: the harness setup binds Tab, so a real `tab` focuses a control.
    #[test]
    fn harness_setup_binds_tab() {
        let mut app = TestAppContext::single();
        let (_view, cx) = open_fluent_test_window(&mut app, move |_, cx| {
            init_services(cx);
            Screenshots::new("button".to_owned(), cx)
        });
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
        cx.type_keys("tab");
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
        assert!(
            cx.update(|window, app| window.focused(app)).is_some(),
            "Tab must focus the first control in the harness"
        );
    }
}
