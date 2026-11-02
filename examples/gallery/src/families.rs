//! Gallery routes organised by Fluent 2 component family.
//!
//! The sidebar lists the families; the content pane shows the active family's
//! page. Every page is a regression matrix: state variants side by side. The
//! theme row on top switches light, dark, high contrast and RTL so each
//! family can be checked in every variant without leaving the page.

use fluent_core::{LayoutDirection, Theme, ThemeProvider as _};
use fluent_primitives::{
    Button, ButtonAppearance, ButtonShape, ButtonSize, Checkbox, CheckboxState, Chip, Divider,
    Label, LabelSize, Link, LinkAppearance, RadioGroup, Switch, ToggleButton,
};
use gpui::{div, prelude::*, AnyElement, AnyView, Context, Entity, IntoElement, Render, Window};

/// A Fluent 2 component family, one gallery route each.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Family {
    ButtonsAndLinks,
    Selection,
    Text,
    StatusAndFeedback,
    DataAndLists,
    DateAndInput,
    Media,
    ThemeAndDirection,
}

impl Family {
    pub const ALL: [Family; 8] = [
        Family::ButtonsAndLinks,
        Family::Selection,
        Family::Text,
        Family::StatusAndFeedback,
        Family::DataAndLists,
        Family::DateAndInput,
        Family::Media,
        Family::ThemeAndDirection,
    ];

    pub fn title(self) -> &'static str {
        match self {
            Family::ButtonsAndLinks => "Buttons and links",
            Family::Selection => "Selection controls",
            Family::Text => "Text and labels",
            Family::StatusAndFeedback => "Status and feedback",
            Family::DataAndLists => "Data and lists",
            Family::DateAndInput => "Date and input",
            Family::Media => "Terminal and video",
            Family::ThemeAndDirection => "Theme and direction",
        }
    }
}

/// Pre-built views for the pages that wrap existing demos.
pub struct DemoViews {
    pub status: Vec<AnyView>,
    pub data: Vec<AnyView>,
    pub date_input: Vec<AnyView>,
    pub media: Vec<AnyView>,
    pub theme: Vec<AnyView>,
}

pub struct FamilyHost {
    active: Family,
    demos: DemoViews,
}

impl FamilyHost {
    pub fn new(demos: DemoViews) -> Self {
        Self {
            active: Family::ButtonsAndLinks,
            demos,
        }
    }

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
                    .items_center()
                    .gap_2()
                    .children(items),
            )
            .child(Divider::horizontal())
    }

    fn views(views: &[AnyView]) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .children(views.iter().map(|v| {
                div()
                    .h(gpui::px(300.0))
                    .relative()
                    .overflow_hidden()
                    .child(v.clone())
            }))
    }

    fn buttons() -> impl IntoElement {
        let appearances = [
            ("Neutral", ButtonAppearance::Neutral),
            ("Accent", ButtonAppearance::Accent),
            ("Danger", ButtonAppearance::Danger),
            ("Subtle", ButtonAppearance::Subtle),
            ("Hyperlink", ButtonAppearance::Hyperlink),
        ];
        let enabled = appearances
            .iter()
            .map(|(n, a)| {
                Button::new(format!("b-{n}"))
                    .label(*n)
                    .appearance(*a)
                    .into_any_element()
            })
            .collect();
        let disabled = appearances
            .iter()
            .map(|(n, a)| {
                Button::new(format!("bd-{n}"))
                    .label(*n)
                    .appearance(*a)
                    .disabled(true)
                    .into_any_element()
            })
            .collect();
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(Self::section("Button appearances", enabled))
            .child(Self::section("Button appearances, disabled", disabled))
            .child(Self::section(
                "Button shapes and sizes",
                vec![
                    Button::new("shape-r").label("Rounded").into_any_element(),
                    Button::new("shape-s")
                        .label("Square")
                        .shape(ButtonShape::Square)
                        .into_any_element(),
                    Button::new("shape-c")
                        .label("Circular")
                        .shape(ButtonShape::Circular)
                        .into_any_element(),
                    Button::new("size-c")
                        .label("Compact")
                        .size(ButtonSize::Compact)
                        .into_any_element(),
                ],
            ))
            .child(Self::section(
                "Toggle button",
                vec![
                    ToggleButton::new("t-off").label("Off").into_any_element(),
                    ToggleButton::new("t-on")
                        .label("On")
                        .selected(true)
                        .into_any_element(),
                    ToggleButton::new("t-dis")
                        .label("Disabled")
                        .disabled(true)
                        .into_any_element(),
                ],
            ))
            .child(Self::section(
                "Link",
                vec![
                    Link::new("l-def", "Default").into_any_element(),
                    Link::new("l-sub", "Subtle")
                        .appearance(LinkAppearance::Subtle)
                        .into_any_element(),
                    Link::new("l-dis", "Disabled")
                        .disabled(true)
                        .into_any_element(),
                ],
            ))
    }

    fn selection() -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(Self::section(
                "Checkbox: unchecked, checked, indeterminate, disabled",
                vec![
                    Checkbox::new("c-un").label("Unchecked").into_any_element(),
                    Checkbox::new("c-ch")
                        .label("Checked")
                        .checked(true)
                        .into_any_element(),
                    Checkbox::new("c-in")
                        .label("Mixed")
                        .state(CheckboxState::Indeterminate)
                        .into_any_element(),
                    Checkbox::new("c-dis")
                        .label("Disabled")
                        .disabled(true)
                        .into_any_element(),
                    Checkbox::new("c-dis-on")
                        .label("Disabled, checked")
                        .checked(true)
                        .disabled(true)
                        .into_any_element(),
                ],
            ))
            .child(Self::section(
                "Switch: off, on, disabled",
                vec![
                    Switch::new("s-off").label("Off").into_any_element(),
                    Switch::new("s-on").label("On").on(true).into_any_element(),
                    Switch::new("s-dis")
                        .label("Disabled")
                        .disabled(true)
                        .into_any_element(),
                ],
            ))
            .child(Self::section(
                "Radio group: enabled and disabled",
                vec![
                    RadioGroup::new("r-en")
                        .label("Enabled")
                        .option("a", "Alpha")
                        .option("b", "Beta")
                        .selected("a")
                        .into_any_element(),
                    RadioGroup::new("r-dis")
                        .label("Disabled")
                        .option("a", "Alpha")
                        .option("b", "Beta")
                        .selected("b")
                        .disabled(true)
                        .into_any_element(),
                ],
            ))
            .child(Self::section(
                "Chip",
                vec![Chip::new("chip", "Filter: open").into_any_element()],
            ))
    }

    fn text() -> impl IntoElement {
        let sizes = [
            ("Caption", LabelSize::Caption),
            ("Body", LabelSize::Body),
            ("Subtitle", LabelSize::Subtitle),
            ("Title", LabelSize::Title),
            ("Display", LabelSize::Display),
        ];
        div()
            .flex()
            .flex_col()
            .gap_3()
            .child(Self::section(
                "Label sizes",
                sizes
                    .iter()
                    .map(|(n, s)| Label::new(*n).size(*s).into_any_element())
                    .collect(),
            ))
            .child(Self::section(
                "Label variants",
                vec![
                    Label::eyebrow("Eyebrow").into_any_element(),
                    Label::new("A long label that is truncated to its container")
                        .truncate()
                        .into_any_element(),
                ],
            ))
    }
}

impl Render for FamilyHost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let rtl = LayoutDirection::current(cx).is_rtl();
        let page: AnyElement = match self.active {
            Family::ButtonsAndLinks => Self::buttons().into_any_element(),
            Family::Selection => Self::selection().into_any_element(),
            Family::Text => Self::text().into_any_element(),
            Family::StatusAndFeedback => Self::views(&self.demos.status).into_any_element(),
            Family::DataAndLists => Self::views(&self.demos.data).into_any_element(),
            Family::DateAndInput => Self::views(&self.demos.date_input).into_any_element(),
            Family::Media => Self::views(&self.demos.media).into_any_element(),
            Family::ThemeAndDirection => Self::views(&self.demos.theme).into_any_element(),
        };
        let variants = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(Label::new("Variant:"))
            .child(
                Button::new("v-light")
                    .label("Light")
                    .on_click(|_, _, cx| Theme::apply_light(cx)),
            )
            .child(
                Button::new("v-dark")
                    .label("Dark")
                    .on_click(|_, _, cx| Theme::apply_dark(cx)),
            )
            .child(
                Button::new("v-hc")
                    .label("High contrast")
                    .on_click(|_, _, cx| Theme::apply_high_contrast(cx)),
            )
            .child(
                ToggleButton::new("v-rtl")
                    .label("RTL")
                    .selected(rtl)
                    .on_click(|_, _, _, cx| LayoutDirection::toggle(cx)),
            );
        div()
            .id("family-page")
            .size_full()
            .overflow_y_scroll()
            .p_3()
            .flex()
            .flex_col()
            .gap_3()
            .bg(colors.surface)
            .text_color(colors.on_neutral)
            .child(Label::new(self.active.title()).size(LabelSize::Title))
            .child(variants)
            .child(Divider::horizontal())
            .child(page)
    }
}

/// Sidebar navigation: one button per family.
pub struct FamilyNav {
    host: Entity<FamilyHost>,
}

impl FamilyNav {
    pub fn new(host: Entity<FamilyHost>) -> Self {
        Self { host }
    }
}

impl Render for FamilyNav {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let active = self.host.read(cx).active;
        div()
            .size_full()
            .p_2()
            .flex()
            .flex_col()
            .gap_1()
            .bg(colors.surface)
            .children(Family::ALL.iter().map(|&family| {
                let host = self.host.clone();
                let nav = cx.entity();
                ToggleButton::new(format!("nav-{family:?}"))
                    .label(family.title())
                    .selected(family == active)
                    .on_click(move |_, _, _, cx| {
                        host.update(cx, |h, cx| {
                            h.active = family;
                            cx.notify();
                        });
                        nav.update(cx, |_, cx| cx.notify());
                    })
            }))
    }
}
