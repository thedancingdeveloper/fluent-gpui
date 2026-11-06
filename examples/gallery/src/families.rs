//! Gallery routes organised by Fluent 2 component family.
//!
//! The sidebar lists the families; the content pane shows the active family's
//! page. Every page is a regression matrix: state variants side by side. The
//! theme row on top switches light, dark, high contrast and RTL so each
//! family can be checked in every variant without leaving the page.

use fluent_core::gpui::{
    div, prelude::*, px, AnyElement, AnyView, Context, Entity, FocusHandle, IntoElement, Render,
    ScrollHandle, Window,
};
use fluent_core::{LayoutDirection, Theme, ThemeProvider as _};
use fluent_layout::{
    drag_source, drop_target, DragOperation, DropPolicy, GroupedVirtualList, VirtualList,
    VirtualListGroup,
};
use fluent_primitives::{
    Button, ButtonAppearance, ButtonIconPosition, ButtonShape, ButtonSize, Checkbox, CheckboxState,
    Chip, Divider, Expander, InputCapture, Label, LabelSize, Link, LinkAppearance, RadioGroup,
    Scrollbar, Switch, TerminalView, ToggleButton, VideoSurface,
};

pub struct GroupedListPage;

impl Render for GroupedListPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let groups = vec![
            VirtualListGroup::new("inbox", "Inbox", 8),
            VirtualListGroup::new("archive", "Archive", 6).expanded(false),
            VirtualListGroup::new("large", "Large collection", 1000),
        ];
        GroupedVirtualList::new("gallery-grouped-list", groups, |group, row, _, _| {
            Label::new(format!("Group {group} · item {row}")).into_any_element()
        })
        .row_height(32.0)
        .into_any_element()
    }
}

/// Scrollbar demos: a virtual list, a two-axis scroller and an always-visible bar.
pub struct ScrollbarPage {
    both: ScrollHandle,
    always: ScrollHandle,
}

impl ScrollbarPage {
    pub fn new() -> Self {
        Self {
            both: ScrollHandle::new(),
            always: ScrollHandle::new(),
        }
    }
}

impl Render for ScrollbarPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let frame = move || {
            div()
                .relative()
                .w(px(260.0))
                .h(px(260.0))
                .overflow_hidden()
                .border_1()
                .border_color(colors.stroke_neutral)
        };
        let rows = |n: usize| {
            div().flex().flex_col().children((0..n).map(|i| {
                div()
                    .h(px(32.0))
                    .px_3()
                    .child(Label::new(format!("Row {i}")))
            }))
        };
        div()
            .flex()
            .gap_3()
            .child(
                frame().child(
                    VirtualList::new("gallery-scrollbar-list", 1000, |i, _, _| {
                        Label::new(format!("Virtual row {i}")).into_any_element()
                    })
                    .row_height(32.0)
                    .scrollbar(true),
                ),
            )
            .child(
                frame()
                    .child(
                        div()
                            .id("gallery-scrollbar-both")
                            .size_full()
                            .overflow_scroll()
                            .track_scroll(&self.both)
                            .child(div().w(px(600.0)).child(rows(40))),
                    )
                    .child(Scrollbar::new(
                        "gallery-scrollbar-both-v",
                        self.both.clone(),
                    ))
                    .child(
                        Scrollbar::new("gallery-scrollbar-both-h", self.both.clone()).horizontal(),
                    ),
            )
            .child(
                frame()
                    .child(
                        div()
                            .id("gallery-scrollbar-always")
                            .size_full()
                            .overflow_y_scroll()
                            .track_scroll(&self.always)
                            .child(rows(40)),
                    )
                    .child(
                        Scrollbar::new("gallery-scrollbar-always-v", self.always.clone())
                            .always_visible(),
                    ),
            )
    }
}

pub struct VideoPage {
    pub surface: Entity<VideoSurface>,
}
impl Render for VideoPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Label::new("VideoSurface — synthetic moving test pattern"))
            .child(self.surface.clone())
    }
}
pub struct TerminalPage {
    pub terminal: Entity<TerminalView>,
}
impl Render for TerminalPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Label::new("TerminalView — scripted terminal text"))
            .child(self.terminal.clone())
    }
}
#[derive(Default)]
pub struct EventLog;
impl Render for EventLog {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().p_2().child(
            "Click or press a key to exercise InputCapture (events are synthetic in this page).",
        )
    }
}
pub struct InputPage {
    pub capture: Entity<InputCapture>,
}
impl Render for InputPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Label::new("InputCapture — live key and pointer event log"))
            .child(self.capture.clone())
    }
}
pub struct ExpanderPage;
impl Render for ExpanderPage {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(Label::new(
                "Expander — collapsed, expanded, and disabled states",
            ))
            .child(
                Expander::new("collapsed", "Collapsed")
                    .content(Label::new("Hidden until expanded").into_any_element()),
            )
            .child(
                Expander::new("expanded", "Expanded")
                    .expanded(true)
                    .content(Label::new("Visible content").into_any_element()),
            )
            .child(
                Expander::new("disabled", "Disabled")
                    .disabled(true)
                    .content(Label::new("Unavailable content").into_any_element()),
            )
    }
}

/// Small contract demo: the source and target are intentionally generic so
/// this page exercises the same API an application uses for pane actions.
pub struct DragDropPage {
    focus_handle: FocusHandle,
}

impl DragDropPage {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle().tab_stop(true),
        }
    }
}

struct DragGhost;

impl Render for DragGhost {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        Label::new("Dragging item").into_any_element()
    }
}

impl Render for DragDropPage {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let source = drag_source(
            div().p_3().bg(cx.theme().colors.surface_dim),
            "gallery-drag-source",
            1_u32,
            |_value, _offset, _window, cx| cx.new(|_| DragGhost),
        );
        let target = drop_target(
            div()
                .p_3()
                .border_1()
                .border_color(cx.theme().colors.stroke_neutral_subtle),
            "gallery-drop-target",
            self.focus_handle.clone(),
            cx.theme().colors.focus_ring,
            DropPolicy::new(|value: &u32| *value == 1),
            |_value, operation: DragOperation, _window, _cx| {
                let _ = operation;
                true
            },
            Some(1_u32),
        );
        div()
            .flex()
            .gap_3()
            .child(source.child(Label::new("Drag this item")))
            .child(target.child(Label::new("Drop here (Enter/Space)")))
    }
}

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
                    .h(fluent_core::gpui::px(300.0))
                    .relative()
                    .overflow_hidden()
                    .child(v.clone())
            }))
    }

    fn buttons() -> impl IntoElement {
        let appearances = [
            ("Secondary", ButtonAppearance::Secondary),
            ("Primary", ButtonAppearance::Primary),
            ("Outline", ButtonAppearance::Outline),
            ("Subtle", ButtonAppearance::Subtle),
            ("Transparent", ButtonAppearance::Transparent),
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
                        .label("Small")
                        .size(ButtonSize::Small)
                        .into_any_element(),
                    Button::new("size-m")
                        .label("Medium")
                        .size(ButtonSize::Medium)
                        .into_any_element(),
                    Button::new("size-l")
                        .label("Large")
                        .size(ButtonSize::Large)
                        .into_any_element(),
                    Button::new("icon-after")
                        .label("Icon after")
                        .icon("icons/add.svg")
                        .icon_position(ButtonIconPosition::After)
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
