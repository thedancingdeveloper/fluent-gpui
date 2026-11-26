use fluent_core::TextRampExt;
use fluent_primitives::{
    is_keyboard_focused, ButtonAppearance, ButtonSize, FluentFocus as _, IconButton,
};
use std::sync::Arc;

use fluent_core::{LayoutDirection, Theme, ThemeProvider as _, TruncateExt as _};
use gpui::{
    div, prelude::*, px, svg, ClickEvent, Context, FocusHandle, IntoElement, KeyDownEvent, Render,
    Role, SharedString, Window,
};

/// A single tab in a `TabStrip`.
#[derive(Clone, Debug)]
pub struct TabItem {
    /// Stable identifier for the tab (used to key close/select events).
    pub id: SharedString,
    /// Public API item.
    pub label: SharedString,
    /// Public API item.
    pub icon: Option<SharedString>,
    /// Public API item.
    pub closable: bool,
}

impl TabItem {
    /// Public API item.
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: "".into(),
            icon: None,
            closable: false,
        }
    }

    /// Public API item.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Public API item.
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    /// Public API item.
    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

/// A horizontal strip of closable, selectable tabs (content area tabs, not ribbon tabs).
///
/// ```no_run
/// use fluent_layout::{TabItem, TabStrip};
/// use gpui::{App, AppContext, Entity};
///
/// fn tabs(cx: &mut App) -> Entity<TabStrip> {
///     cx.new(|cx| {
///         let mut strip = TabStrip::new(cx)
///             .on_select(|index, _strip, _window, _cx| println!("selected tab {index}"))
///             .on_close(|index, _strip, _window, _cx| println!("close tab {index}"));
///         strip.add_tab(TabItem::new("a").label("Session A").closable(true));
///         strip.add_tab(TabItem::new("b").label("Session B"));
///         strip
///     })
/// }
/// ```
#[allow(clippy::type_complexity)]
#[derive(Default)]
pub struct TabStrip {
    /// Public API item.
    pub tabs: Vec<TabItem>,
    /// Public API item.
    pub active: usize,
    focus_handles: Vec<FocusHandle>,
    on_select: Option<Arc<dyn Fn(usize, &TabStrip, &mut Window, &mut Context<Self>) + 'static>>,
    on_close: Option<Arc<dyn Fn(usize, &TabStrip, &mut Window, &mut Context<Self>) + 'static>>,
}

impl TabStrip {
    /// Public API item.
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        cx.observe_global::<LayoutDirection>(|_, cx| cx.notify())
            .detach();
        Self::default()
    }

    /// Public API item.
    pub fn add_tab(&mut self, tab: TabItem) {
        self.tabs.push(tab);
    }

    /// Public API item.
    pub fn remove_tab(&mut self, idx: usize) {
        if idx < self.tabs.len() {
            self.tabs.remove(idx);
            if idx < self.active {
                self.active = self.active.saturating_sub(1);
            } else if self.active >= self.tabs.len() && self.active > 0 {
                self.active = self.tabs.len().saturating_sub(1);
            }
        }
    }

    /// Public API item.
    pub fn on_select(
        mut self,
        f: impl Fn(usize, &TabStrip, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Public API item.
    pub fn on_close(
        mut self,
        f: impl Fn(usize, &TabStrip, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_close = Some(Arc::new(f));
        self
    }
}

/// Two-tone focus ring drawn inside `el`'s own bounds (2px outer, 1px inner),
/// unlike `focus_ring`, which sits 2px outside. The tabs fill the strip's full
/// height and the strip is `overflow_hidden`, so an outside ring would be
/// clipped; insetting keeps the strip's layout unchanged.
fn inset_focus_ring<E>(el: E, keyboard_focused: bool, colors: &fluent_core::ColorScheme) -> E
where
    E: Styled + ParentElement,
{
    if !keyboard_focused {
        return el;
    }
    el.relative().child(
        div()
            .absolute()
            .inset_0()
            .border_2()
            .border_color(colors.stroke_focus_outer)
            .child(
                div()
                    .size_full()
                    .border_1()
                    .border_color(colors.stroke_focus_inner),
            ),
    )
}

impl Render for TabStrip {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        while self.focus_handles.len() < self.tabs.len() {
            self.focus_handles.push(cx.focus_handle());
        }
        self.focus_handles.truncate(self.tabs.len());
        let enabled = vec![true; self.tabs.len()];
        let stop =
            crate::roving::stop_index(window, &self.focus_handles, &enabled, Some(self.active));

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let components = theme.components;
        let rtl = LayoutDirection::current(cx).is_rtl();

        let active = self.active;
        let n_tabs = self.tabs.len();

        let tab_strip_bg = colors.tab_strip_bg;
        let active_bg = colors.tab_active_bg;
        let hover_bg = colors.tab_hover_bg;
        let on_neutral = colors.on_neutral;
        let border_c = colors.stroke_neutral_subtle;

        let ring_colors = colors.clone();
        let mut strip = div()
            .id("tab-strip")
            .role(Role::TabList)
            .aria_orientation(gpui::accesskit::Orientation::Horizontal)
            .flex()
            .when(rtl, |strip| strip.flex_row_reverse())
            .when(!rtl, |strip| strip.flex_row())
            .w_full()
            .min_w_0()
            .h(px(components.content_tab_strip_height))
            .bg(tab_strip_bg)
            .border_b_1()
            .border_color(border_c)
            .overflow_hidden();

        for (i, tab) in self.tabs.iter().enumerate() {
            let is_active = i == active;
            let tab_label = tab.label.clone();
            let tab_icon = tab.icon.clone();
            let closable = tab.closable;

            let select_handler = cx.listener(move |ts: &mut TabStrip, _: &ClickEvent, win, cx| {
                ts.active = i;
                if let Some(f) = &ts.on_select.clone() {
                    f(i, ts, win, cx);
                }
                cx.notify();
            });

            let close_handler = if closable {
                let close_idx = i;
                Some(
                    cx.listener(move |ts: &mut TabStrip, _: &ClickEvent, win, cx| {
                        cx.stop_propagation();
                        if let Some(f) = &ts.on_close.clone() {
                            f(close_idx, ts, win, cx);
                        }
                        ts.remove_tab(close_idx);
                        cx.notify();
                    }),
                )
            } else {
                None
            };

            let key_handler = cx.listener(move |ts: &mut TabStrip, ev: &KeyDownEvent, win, cx| {
                let enabled = vec![true; ts.tabs.len()];
                let key = match (rtl, ev.keystroke.key.as_str()) {
                    (true, "left") => "right",
                    (true, "right") => "left",
                    (_, key) => key,
                };
                if matches!(key, "enter" | "return" | "space") {
                    ts.active = i;
                    if let Some(f) = &ts.on_select.clone() {
                        f(i, ts, win, cx);
                    }
                    cx.stop_propagation();
                    cx.notify();
                    return;
                }
                if let Some(next) = crate::roving::target(key, i, &enabled) {
                    ts.focus_handles[next].focus(win, cx);
                    ts.active = next;
                    if let Some(f) = &ts.on_select.clone() {
                        f(next, ts, win, cx);
                    }
                    cx.stop_propagation();
                    cx.notify();
                }
            });
            // A tracked handle carries its own tab settings: only the roving
            // stop (the focused or active tab) is in the Tab order.
            let tab_handle = self.focus_handles[i]
                .clone()
                .tab_index(0)
                .tab_stop(stop == Some(i));
            let tab_ring = is_keyboard_focused(&tab_handle, window);

            let tab_bg = if is_active { active_bg } else { tab_strip_bg };

            let mut tab_el = div()
                .id(("tab", (n_tabs as u64 * 100 + i as u64)))
                .debug_selector(move || format!("tab-{i}"))
                .role(Role::Tab)
                .aria_label(tab_label.clone())
                .aria_selected(is_active)
                .aria_position_in_set(i + 1)
                .aria_size_of_set(n_tabs)
                .track_focus(&tab_handle)
                .fluent_tab_stop(0)
                .tab_stop(stop == Some(i))
                .key_context("TabStrip")
                .on_key_down(key_handler)
                .flex()
                .when(rtl, |tab| tab.flex_row_reverse())
                .when(!rtl, |tab| tab.flex_row())
                .items_center()
                .gap(px(spacing.xs))
                .px(px(spacing.md))
                .h_full()
                .max_w(px(components.content_tab_max_width))
                .flex_none()
                .bg(tab_bg)
                .cursor_pointer()
                .text_ramp(&typography.caption)
                .text_color(on_neutral)
                .border_r_1()
                .border_color(border_c)
                .hover(move |s| s.bg(hover_bg))
                .active(move |s| s.bg(colors.neutral_selected))
                .on_click(select_handler);

            if let Some(icon_path) = tab_icon {
                tab_el = tab_el.child(
                    svg()
                        .path(icon_path)
                        .size(px(components.content_tab_icon_size))
                        .text_color(on_neutral),
                );
            }

            tab_el = tab_el.child(div().truncate_label().child(tab_label));

            if let Some(handler) = close_handler {
                let close_btn_id = ("tab-close", (n_tabs as u64 * 100 + i as u64));
                tab_el = tab_el.child(
                    IconButton::new(close_btn_id, "icons/dismiss.svg")
                        .label("Close tab")
                        .appearance(ButtonAppearance::Subtle)
                        .size(ButtonSize::Compact)
                        .on_click(handler),
                );
            }

            strip = strip.child(inset_focus_ring(tab_el, tab_ring, &ring_colors));
        }

        strip
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use crate::roving::test_support::{clipped_ring_quads, outer_ring_quads, press, tab};
    use fluent_primitives::Button;
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::TestAppContext;

    use super::*;

    /// A tab strip followed by a button, to count how many Tab stops the strip has.
    struct Host {
        strip: gpui::Entity<TabStrip>,
        log: Rc<RefCell<Vec<String>>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let log = self.log.clone();
            div()
                .child(self.strip.clone())
                .child(Button::new("after").on_click(move |_, _, _| {
                    log.borrow_mut().push("after".into());
                }))
        }
    }

    fn host(
        cx: &mut TestAppContext,
        log: Rc<RefCell<Vec<String>>>,
    ) -> (gpui::Entity<TabStrip>, &mut gpui::VisualTestContext) {
        let strip_log = log.clone();
        let (host, cx) = open_fluent_test_window(cx, move |_, cx| {
            let strip = cx.new(|cx| {
                let mut ts = TabStrip::new(cx).on_select(move |i, _, _, _| {
                    strip_log.borrow_mut().push(format!("select {i}"));
                });
                for id in ["a", "b", "c"] {
                    ts.add_tab(TabItem::new(id).label(id));
                }
                ts
            });
            Host { strip, log }
        });
        let strip = host.read_with(cx, |h, _| h.strip.clone());
        (strip, cx)
    }

    struct TwoStrips(gpui::Entity<TabStrip>, gpui::Entity<TabStrip>);

    impl Render for TwoStrips {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child(self.0.clone()).child(self.1.clone())
        }
    }

    /// Each `TabStrip` is its own view, so its fixed root id is scoped to that
    /// view: two strips side by side do not collide (WI-4756 audit).
    #[gpui::test]
    fn two_tab_strips_in_one_view_do_not_collide(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, cx| {
            let strip = |cx: &mut Context<TwoStrips>| {
                cx.new(|cx| {
                    let mut ts = TabStrip::new(cx);
                    for id in ["a", "b"] {
                        ts.add_tab(TabItem::new(id).label(id));
                    }
                    ts
                })
            };
            TwoStrips(strip(cx), strip(cx))
        });
        let snapshot = cx.accessibility_snapshot();
        assert_eq!(
            snapshot.iter().filter(|n| n.starts_with("TabList")).count(),
            2,
            "{snapshot:?}"
        );
    }

    #[gpui::test]
    fn tab_reaches_the_strip_once(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (_, cx) = host(cx, log.clone());
        tab(cx, 2);
        // The second Tab left the strip for the next control, not the next tab.
        press(cx, "enter");
        assert_eq!(*log.borrow(), vec!["after".to_string()]);
    }

    #[gpui::test]
    fn arrows_move_the_active_tab_after_tabbing_in(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (ts, cx) = host(cx, log);
        tab(cx, 1);
        let f = ts.read_with(cx, |t, _| t.focus_handles.clone());
        let st: Vec<bool> = cx.update(|w, _| f.iter().map(|h| h.is_focused(w)).collect());
        assert_eq!(
            st,
            [true, false, false],
            "first Tab lands on the active tab"
        );
        cx.type_keys("right");
        ts.read_with(cx, |t, _| assert_eq!(t.active, 1));
        cx.type_keys("right");
        ts.read_with(cx, |t, _| assert_eq!(t.active, 2));
    }

    #[gpui::test]
    fn enter_and_space_select_the_focused_tab(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (_, cx) = host(cx, log.clone());
        cx.click_selector("tab-1");
        log.borrow_mut().clear();
        cx.type_keys("enter");
        cx.type_keys("space");
        assert_eq!(
            *log.borrow(),
            vec!["select 1".to_string(), "select 1".to_string()]
        );
    }

    #[gpui::test]
    fn ring_only_for_keyboard_focus(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (_, cx) = host(cx, log);
        assert_eq!(
            outer_ring_quads(cx),
            0,
            "no ring before anything is focused"
        );
        cx.click_selector("tab-1");
        assert!(cx.update(|window, app| window.focused(app).is_some()));
        assert_eq!(outer_ring_quads(cx), 0, "mouse focus must not draw a ring");
        cx.type_keys("right");
        assert_eq!(outer_ring_quads(cx), 1, "keyboard focus draws one ring");
    }

    #[gpui::test]
    fn ring_is_not_clipped_by_the_strip(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (_, cx) = host(cx, log);
        // Last tab (right edge), then wrap to the first tab (left edge): the
        // strip's `overflow_hidden` clips to its own bounds, so the ring must
        // lie inside them.
        cx.click_selector("tab-1");
        for key in ["right", "right", "left"] {
            cx.type_keys(key);
            assert_eq!(outer_ring_quads(cx), 1, "one ring after {key}");
            let clipped = clipped_ring_quads(cx);
            assert!(clipped.is_empty(), "ring clipped by the strip: {clipped:?}");
        }
    }

    fn strip(cx: &mut TestAppContext) -> (gpui::Entity<TabStrip>, &mut gpui::VisualTestContext) {
        open_fluent_test_window(cx, |_, cx| {
            let mut ts = TabStrip::new(cx);
            for id in ["a", "b", "c"] {
                ts.add_tab(TabItem::new(id).label(id));
            }
            ts
        })
    }

    #[gpui::test]
    fn arrows_select_and_home_end_jump(cx: &mut TestAppContext) {
        let (ts, cx) = strip(cx);
        cx.click_selector("tab-0");
        cx.type_keys("right");
        ts.read_with(cx, |t, _| assert_eq!(t.active, 1));
        cx.type_keys("left left");
        ts.read_with(cx, |t, _| assert_eq!(t.active, 2));
        cx.type_keys("home");
        ts.read_with(cx, |t, _| assert_eq!(t.active, 0));
        cx.type_keys("end");
        ts.read_with(cx, |t, _| assert_eq!(t.active, 2));
    }

    #[gpui::test]
    fn closable_tab_keeps_tab_height(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, cx| {
            let mut ts = TabStrip::new(cx);
            ts.add_tab(TabItem::new("a").label("a").closable(true));
            ts.add_tab(TabItem::new("b").label("b"));
            ts
        });
        let closable = cx.selector_bounds("tab-0").expect("closable tab painted");
        let plain = cx.selector_bounds("tab-1").expect("plain tab painted");
        assert_eq!(closable.size.height, plain.size.height);
        assert!(closable.size.height <= gpui::px(32.0));
    }

    #[test]
    fn remove_tab_before_active_preserves_active_tab_identity() {
        let mut strip = TabStrip::default();
        strip.add_tab(TabItem::new("a").label("A"));
        strip.add_tab(TabItem::new("b").label("B"));
        strip.add_tab(TabItem::new("c").label("C"));
        strip.active = 1;

        strip.remove_tab(0);

        assert_eq!(strip.active, 0);
        assert_eq!(strip.tabs[strip.active].id.as_ref(), "b");
    }

    #[test]
    fn remove_active_last_tab_moves_active_to_previous_tab() {
        let mut strip = TabStrip::default();
        strip.add_tab(TabItem::new("a").label("A"));
        strip.add_tab(TabItem::new("b").label("B"));
        strip.active = 1;

        strip.remove_tab(1);

        assert_eq!(strip.active, 0);
        assert_eq!(strip.tabs[strip.active].id.as_ref(), "a");
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_tablist_and_selected_tab(cx: &mut TestAppContext) {
        let (_, cx) = strip(cx);
        assert_eq!(
            cx.accessibility_snapshot(),
            vec![
                "Window".to_string(),
                "TabList [orientation=Horizontal]".to_string(),
                "Tab \"a\" [selected=true, position_in_set=1, size_of_set=3]".to_string(),
                "Tab \"b\" [selected=false, position_in_set=2, size_of_set=3]".to_string(),
                "Tab \"c\" [selected=false, position_in_set=3, size_of_set=3]".to_string(),
            ]
        );
    }
}
