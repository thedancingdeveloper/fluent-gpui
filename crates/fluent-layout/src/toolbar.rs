use fluent_core::TextRampExt;
use std::sync::Arc;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, App, ClickEvent, ElementId, FocusHandle, IntoElement, KeyDownEvent,
    RenderOnce, Role, SharedString, Toggled, Window,
};

type ToolbarClick = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type KeyNav = Arc<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
type ToolbarToggleClick = Arc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>;

/// Roving-focus wiring for one visible toolbar item.
struct Roving {
    handle: FocusHandle,
    tab_stop: bool,
    on_key: KeyNav,
}

#[derive(Clone)]
/// Public API item.
pub enum ToolbarItem {
    /// Public API item.
    Action {
        /// Public API item.
        id: SharedString,
        /// Public API item.
        label: SharedString,
        /// Public API item.
        icon: Option<SharedString>,
        /// Public API item.
        disabled: bool,
        /// Public API item.
        on_click: ToolbarClick,
    },
    /// Public API item.
    Toggle {
        /// Public API item.
        id: SharedString,
        /// Public API item.
        label: SharedString,
        /// Public API item.
        icon: Option<SharedString>,
        /// Public API item.
        selected: bool,
        /// Public API item.
        disabled: bool,
        /// Public API item.
        on_click: ToolbarToggleClick,
    },
    /// Public API item.
    Separator,
}

impl ToolbarItem {
    /// Public API item.
    pub fn action(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Action {
            id: id.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    /// Public API item.
    pub fn action_with_icon(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Action {
            id: id.into(),
            label: label.into(),
            icon: Some(icon.into()),
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    /// Public API item.
    pub fn toggle(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Toggle {
            id: id.into(),
            label: label.into(),
            icon: None,
            selected,
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    /// Public API item.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Public API item.
    pub fn disabled(self, disabled: bool) -> Self {
        match self {
            Self::Action {
                id,
                label,
                icon,
                on_click,
                ..
            } => Self::Action {
                id,
                label,
                icon,
                disabled,
                on_click,
            },
            Self::Toggle {
                id,
                label,
                icon,
                selected,
                on_click,
                ..
            } => Self::Toggle {
                id,
                label,
                icon,
                selected,
                disabled,
                on_click,
            },
            Self::Separator => Self::Separator,
        }
    }
}

/// A compact command surface for panes, lists, and editor-like tools.
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    items: Vec<ToolbarItem>,
    max_visible: Option<usize>,
    overflow_open: bool,
    on_overflow_click: Option<ToolbarClick>,
}

impl Toolbar {
    /// Public API item.
    pub fn new() -> Self {
        Self {
            id: "toolbar".into(),
            items: Vec::new(),
            max_visible: None,
            overflow_open: false,
            on_overflow_click: None,
        }
    }

    /// Identifies the toolbar so its focus state survives between frames.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Public API item.
    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.items.push(item);
        self
    }

    /// Public API item.
    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Public API item.
    pub fn max_visible(mut self, max_visible: usize) -> Self {
        self.max_visible = Some(max_visible);
        self
    }

    /// Public API item.
    pub fn overflow_open(mut self, open: bool) -> Self {
        self.overflow_open = open;
        self
    }

    /// Public API item.
    pub fn on_overflow_click(
        mut self,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_overflow_click = Some(Arc::new(f));
        self
    }

    fn with_roving(
        el: gpui::Stateful<gpui::Div>,
        nav: Option<Roving>,
    ) -> gpui::Stateful<gpui::Div> {
        match nav {
            Some(Roving {
                handle,
                tab_stop,
                on_key,
            }) => el
                .track_focus(&handle)
                .tab_stop(tab_stop)
                .on_key_down(move |ev, win, app| on_key(ev, win, app)),
            None => el,
        }
    }

    fn render_item(
        item: ToolbarItem,
        idx: usize,
        nav: Option<Roving>,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let components = theme.components;

        match item {
            ToolbarItem::Separator => div()
                .w(px(1.0))
                .h(px(components.dock_header_height - spacing.sm * 2.0))
                .mx(px(spacing.xs))
                .bg(colors.stroke_neutral_subtle)
                .into_any_element(),
            ToolbarItem::Action {
                id,
                label,
                icon,
                disabled,
                on_click,
            } => {
                let fg = if disabled {
                    colors.on_neutral_disabled
                } else {
                    colors.on_neutral
                };
                let mut el = div()
                    .id(("toolbar-action", idx as u64))
                    .debug_selector(move || format!("toolbar-item-{idx}"))
                    .role(Role::Button)
                    .aria_label(label.clone())
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.xs))
                    .h(px(components.dock_header_height))
                    .px(px(spacing.sm))
                    .rounded(px(radii.sm))
                    .text_ramp(&typography.caption)
                    .text_color(fg)
                    .child(match icon {
                        Some(icon) => svg()
                            .path(icon)
                            .size(px(components.popup_icon_slot))
                            .text_color(fg)
                            .into_any_element(),
                        None => div().size(px(0.0)).into_any_element(),
                    })
                    .child(label);
                if !disabled {
                    let key_click = on_click.clone();
                    el = el
                        .cursor_pointer()
                        .hover({
                            let hover = colors.subtle_hover;
                            move |s| s.bg(hover)
                        })
                        .active({
                            let pressed = colors.subtle_selected;
                            move |s| s.bg(pressed)
                        })
                        .on_key_down(move |ev, win, app| {
                            if matches!(ev.keystroke.key.as_str(), "enter" | "space") {
                                key_click(&ClickEvent::default(), win, app);
                            }
                        })
                        .on_click(move |ev, win, app| on_click(ev, win, app));
                    el = Self::with_roving(el, nav);
                }
                let _ = id;
                el.into_any_element()
            }
            ToolbarItem::Toggle {
                id,
                label,
                icon,
                selected,
                disabled,
                on_click,
            } => {
                let bg = if selected {
                    colors.neutral_selected
                } else {
                    colors.subtle
                };
                let fg = if disabled {
                    colors.on_neutral_disabled
                } else {
                    colors.on_neutral
                };
                let mut el = div()
                    .id(("toolbar-toggle", idx as u64))
                    .debug_selector(move || format!("toolbar-item-{idx}"))
                    .role(Role::Button)
                    .aria_toggled(if selected {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .aria_label(label.clone())
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.xs))
                    .h(px(components.dock_header_height))
                    .px(px(spacing.sm))
                    .rounded(px(radii.sm))
                    .bg(bg)
                    .text_ramp(&typography.caption)
                    .text_color(fg)
                    .child(match icon {
                        Some(icon) => svg()
                            .path(icon)
                            .size(px(components.popup_icon_slot))
                            .text_color(fg)
                            .into_any_element(),
                        None => div().size(px(0.0)).into_any_element(),
                    })
                    .child(label);
                if !disabled {
                    el = el
                        .cursor_pointer()
                        .hover({
                            let hover = colors.neutral_hover;
                            move |s| s.bg(hover)
                        })
                        .active({
                            let pressed = colors.neutral_selected;
                            move |s| s.bg(pressed)
                        })
                        .on_key_down({
                            let key_click = on_click.clone();
                            move |ev, win, app| {
                                if matches!(ev.keystroke.key.as_str(), "enter" | "space") {
                                    key_click(!selected, &ClickEvent::default(), win, app);
                                }
                            }
                        })
                        .on_click(move |ev, win, app| on_click(!selected, ev, win, app));
                    el = Self::with_roving(el, nav);
                }
                let _ = id;
                el.into_any_element()
            }
        }
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Toolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let visible_count = self.max_visible.unwrap_or(self.items.len());
        let visible = visible_count.min(self.items.len());
        let enabled: Vec<bool> = self
            .items
            .iter()
            .take(visible)
            .map(|item| match item {
                ToolbarItem::Action { disabled, .. } | ToolbarItem::Toggle { disabled, .. } => {
                    !disabled
                }
                ToolbarItem::Separator => false,
            })
            .collect();
        let handles = crate::roving::handles(window, cx, &self.id, visible);
        let stop = crate::roving::stop_index(window, &handles, &enabled, None);

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let components = theme.components;
        let radii = theme.radii;
        let typography = theme.typography;

        let mut root = div()
            .id(self.id.clone())
            .role(Role::Toolbar)
            .aria_orientation(gpui::accesskit::Orientation::Horizontal)
            .flex()
            .flex_row()
            .items_center()
            .relative()
            .gap(px(spacing.xs))
            .h(px(components.dock_header_height))
            .px(px(spacing.sm))
            .bg(colors.surface_dim)
            .border_b_1()
            .border_color(colors.stroke_neutral_subtle);

        for (idx, item) in self.items.iter().take(visible_count).cloned().enumerate() {
            let nav = (idx < visible).then(|| {
                let (nav_handles, nav_enabled) = (handles.clone(), enabled.clone());
                Roving {
                    handle: handles[idx].clone(),
                    tab_stop: stop == Some(idx),
                    on_key: Arc::new(move |ev, win, app| {
                        if let Some(next) =
                            crate::roving::target(ev.keystroke.key.as_str(), idx, &nav_enabled)
                        {
                            nav_handles[next].focus(win, app);
                            app.stop_propagation();
                        }
                    }),
                }
            });
            root = root.child(Self::render_item(item, idx, nav, cx));
        }

        let overflow: Vec<ToolbarItem> = self.items.into_iter().skip(visible_count).collect();
        if !overflow.is_empty() {
            let mut overflow_button = div()
                .id("toolbar-overflow")
                .role(Role::Button)
                .aria_label("More")
                .aria_expanded(self.overflow_open)
                .flex()
                .items_center()
                .justify_center()
                .h(px(components.dock_header_height))
                .px(px(spacing.sm))
                .rounded(px(radii.sm))
                .text_ramp(&typography.caption)
                .text_color(colors.on_neutral)
                .cursor_pointer()
                .hover({
                    let hover = colors.subtle_hover;
                    move |s| s.bg(hover)
                })
                .active({
                    let pressed = colors.subtle_selected;
                    move |s| s.bg(pressed)
                })
                .child("More");

            if let Some(handler) = self.on_overflow_click {
                overflow_button =
                    overflow_button.on_click(move |ev, win, app| handler(ev, win, app));
            }

            root = root.child(overflow_button);

            if self.overflow_open {
                let mut menu = div()
                    .absolute()
                    .top(px(components.dock_header_height))
                    .right_0()
                    .min_w(px(180.0))
                    .flex()
                    .flex_col()
                    .bg(colors.surface)
                    .border_1()
                    .border_color(colors.stroke_neutral)
                    .rounded(px(radii.md))
                    .shadow_md()
                    .py(px(spacing.xs));
                for (idx, item) in overflow.into_iter().enumerate() {
                    menu = menu.child(Self::render_item(item, idx + visible_count, None, cx));
                }
                root = root.child(menu);
            }
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Render, TestAppContext};

    use super::*;

    struct Host(Rc<Cell<usize>>);

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let hits = self.0.clone();
            Toolbar::new()
                .item(ToolbarItem::action("a", "A", |_, _, _| {}))
                .item(ToolbarItem::separator())
                .item(ToolbarItem::action("b", "B", |_, _, _| {}).disabled(true))
                .item(ToolbarItem::action("c", "C", move |_, _, _| {
                    hits.set(hits.get() + 1)
                }))
        }
    }

    #[gpui::test]
    fn arrows_skip_disabled_and_home_end_jump(cx: &mut TestAppContext) {
        let hits = Rc::new(Cell::new(0));
        let (_host, cx) = open_fluent_test_window(cx, |_, _| Host(hits.clone()));
        cx.click_selector("toolbar-item-0");
        cx.type_keys("right enter"); // skips the separator and disabled item
        assert_eq!(hits.get(), 1);
        cx.type_keys("home right enter");
        assert_eq!(hits.get(), 2);
        cx.type_keys("end enter");
        assert_eq!(hits.get(), 3);
    }

    struct Toggles {
        action_hits: Rc<Cell<usize>>,
        toggled: Rc<Cell<Option<bool>>>,
        disabled_hits: Rc<Cell<usize>>,
    }

    impl Render for Toggles {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (a, t, d) = (
                self.action_hits.clone(),
                self.toggled.clone(),
                self.disabled_hits.clone(),
            );
            Toolbar::new()
                .item(ToolbarItem::action("a", "A", move |_, _, _| {
                    a.set(a.get() + 1)
                }))
                .item(ToolbarItem::toggle("t", "T", false, move |v, _, _, _| {
                    t.set(Some(v))
                }))
                .item(
                    ToolbarItem::action("d", "D", move |_, _, _| d.set(d.get() + 1)).disabled(true),
                )
        }
    }

    #[gpui::test]
    fn clicking_dispatches_action_and_toggle_but_not_disabled(cx: &mut TestAppContext) {
        let action_hits = Rc::new(Cell::new(0));
        let toggled = Rc::new(Cell::new(None));
        let disabled_hits = Rc::new(Cell::new(0));
        let (_host, cx) = open_fluent_test_window(cx, |_, _| Toggles {
            action_hits: action_hits.clone(),
            toggled: toggled.clone(),
            disabled_hits: disabled_hits.clone(),
        });
        cx.click_selector("toolbar-item-0");
        assert_eq!(action_hits.get(), 1);
        cx.click_selector("toolbar-item-1");
        assert_eq!(toggled.get(), Some(true));
        cx.click_selector("toolbar-item-2");
        assert_eq!(disabled_hits.get(), 0);
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_toolbar_roles(cx: &mut TestAppContext) {
        let (_host, cx) = open_fluent_test_window(cx, |_, _| Toggles {
            action_hits: Rc::new(Cell::new(0)),
            toggled: Rc::new(Cell::new(None)),
            disabled_hits: Rc::new(Cell::new(0)),
        });
        let snap = cx.accessibility_snapshot();
        let joined = snap.join("\n");
        assert!(joined.contains("Toolbar"), "{joined}");
        assert!(joined.contains("Button \"A\""), "{joined}");
        assert!(joined.contains("Button \"T\""), "{joined}");
        assert!(joined.contains("Button \"D\""), "{joined}");
    }
}
