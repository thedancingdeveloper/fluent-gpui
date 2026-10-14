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
//   radio-group field card section-header

use std::fs;

use fluent_app::FluentAssets;
use fluent_core::{Theme, ThemeProvider as _};
use fluent_layout::{modal::ModalStack, ToastStack};
use fluent_primitives::{
    AppDot, Avatar, Badge, Button, ButtonAppearance, ButtonSize, Card, Checkbox, CheckboxState,
    Chip, ConnectionBadge, ConnectionBadgeState, Divider, Dropdown, Field, FieldValidationState,
    Icon, IconButton, IconSize, Label, LabelSize, ProgressBar, RadioGroup, RadioGroupOrientation,
    Searchbox, SectionHeader, Skeleton, SkeletonRow, Spinner, Switch, TextInput, Textarea,
    ToggleButton,
};
use gpui::{
    div, prelude::*, px, App, Bounds, Context, Entity, IntoElement, Point, SharedString, Size,
    TitlebarOptions, Window, WindowBounds, WindowDecorations, WindowOptions,
};

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
}

impl Screenshots {
    fn new(component: impl Into<SharedString>, cx: &mut App) -> Self {
        Self {
            component: component.into(),
            ready_signaled: false,
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
            window.on_next_frame(|_, _| {
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

        let panel_content: gpui::AnyElement = match self.component.as_ref() {
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

fn row() -> gpui::Div {
    div().flex().flex_row().items_center().gap(px(8.0))
}

fn col() -> gpui::Div {
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
        .child(group("25%", ProgressBar::new(0.25)))
        .child(group("60%", ProgressBar::new(0.60)))
        .child(group("100%", ProgressBar::new(1.0)))
}

// ── Spinner ───────────────────────────────────────────────────────────────
fn ss_spinners() -> impl IntoElement {
    group(
        "Sizes",
        row()
            .child(Spinner::new().size(16.0))
            .child(Spinner::new().size(24.0))
            .child(Spinner::new().size(32.0))
            .child(Spinner::new().size(48.0)),
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

// ── Entry point ───────────────────────────────────────────────────────────

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let component = args.get(1).cloned().unwrap_or_else(|| "button".into());
    let dark = !args.iter().any(|a| a == "--light");

    // The vendored gpui removed `Application::new()`; the platform crate
    // constructs the application (see docs/gpui-api-drift.md).
    gpui_platform::application()
        .with_assets(FluentAssets::new(None))
        .run(move |cx: &mut App| {
            if dark {
                Theme::init(cx);
            } else {
                cx.set_global(Theme::light());
            }
            ModalStack::init(cx);
            ToastStack::init(cx);

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
