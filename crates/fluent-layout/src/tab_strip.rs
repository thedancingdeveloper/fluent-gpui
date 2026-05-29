use std::sync::Arc;

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, svg, ClickEvent, Context, IntoElement, Render, SharedString, Window,
};

/// A single tab in a `TabStrip`.
#[derive(Clone, Debug)]
pub struct TabItem {
    /// Stable identifier for the tab (used to key close/select events).
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<SharedString>,
    pub closable: bool,
}

impl TabItem {
    pub fn new(id: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: "".into(),
            icon: None,
            closable: false,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn closable(mut self, closable: bool) -> Self {
        self.closable = closable;
        self
    }
}

/// A horizontal strip of closable, selectable tabs (content area tabs, not ribbon tabs).
#[allow(clippy::type_complexity)]
#[derive(Default)]
pub struct TabStrip {
    pub tabs: Vec<TabItem>,
    pub active: usize,
    on_select: Option<Arc<dyn Fn(usize, &TabStrip, &mut Window, &mut Context<Self>) + 'static>>,
    on_close: Option<Arc<dyn Fn(usize, &TabStrip, &mut Window, &mut Context<Self>) + 'static>>,
}

impl TabStrip {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self::default()
    }

    pub fn add_tab(&mut self, tab: TabItem) {
        self.tabs.push(tab);
    }

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

    pub fn on_select(
        mut self,
        f: impl Fn(usize, &TabStrip, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    pub fn on_close(
        mut self,
        f: impl Fn(usize, &TabStrip, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_close = Some(Arc::new(f));
        self
    }
}

impl Render for TabStrip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let components = theme.components;

        let active = self.active;
        let n_tabs = self.tabs.len();

        let tab_strip_bg = colors.tab_strip_bg;
        let active_bg = colors.tab_active_bg;
        let hover_bg = colors.tab_hover_bg;
        let on_neutral = colors.on_neutral;
        let border_c = colors.stroke_neutral_subtle;

        let mut strip = div()
            .flex()
            .flex_row()
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

            let tab_bg = if is_active { active_bg } else { tab_strip_bg };

            let mut tab_el = div()
                .id(("tab", (n_tabs as u64 * 100 + i as u64)))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing.xs))
                .px(px(spacing.md))
                .h_full()
                .bg(tab_bg)
                .cursor_pointer()
                .text_size(px(typography.caption.size))
                .text_color(on_neutral)
                .border_r_1()
                .border_color(border_c)
                .hover(move |s| s.bg(hover_bg))
                .on_click(select_handler);

            if let Some(icon_path) = tab_icon {
                tab_el = tab_el.child(
                    svg()
                        .path(icon_path)
                        .size(px(components.content_tab_icon_size))
                        .text_color(on_neutral),
                );
            }

            tab_el = tab_el.child(tab_label);

            if let Some(handler) = close_handler {
                let close_btn_id = ("tab-close", (n_tabs as u64 * 100 + i as u64));
                tab_el = tab_el.child(
                    div()
                        .id(close_btn_id)
                        .size(px(components.content_tab_close_size))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(radii.sm))
                        .cursor_pointer()
                        .hover(move |s| s.bg(colors.neutral_hover))
                        .on_click(handler)
                        .child(
                            svg()
                                .path("icons/dismiss.svg")
                                .size(px((components.content_tab_close_size - 4.0).max(8.0)))
                                .text_color(on_neutral),
                        ),
                );
            }

            strip = strip.child(tab_el);
        }

        strip
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
