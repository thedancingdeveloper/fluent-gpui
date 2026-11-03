use fluent_core::{Theme, ThemeProvider as _};
use gpui::{div, prelude::*, AnyView, Context, Entity, IntoElement, Render, Window};

use crate::tab_strip::TabStrip;

/// A content area that optionally has a `TabStrip` at the top.
///
/// The content is an `AnyView` — set it via `Pane::set_content`.
#[derive(Default)]
pub struct Pane {
    /// Public API item.
    pub tab_strip: Option<Entity<TabStrip>>,
    /// Public API item.
    pub content: Option<AnyView>,
}

impl Pane {
    /// Public API item.
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self::default()
    }

    /// Public API item.
    pub fn with_tab_strip(mut self, strip: Entity<TabStrip>) -> Self {
        self.tab_strip = Some(strip);
        self
    }

    /// Public API item.
    pub fn set_content(&mut self, content: AnyView, cx: &mut Context<Self>) {
        self.content = Some(content);
        cx.notify();
    }

    /// Public API item.
    pub fn set_tab_strip(&mut self, strip: Entity<TabStrip>, cx: &mut Context<Self>) {
        self.tab_strip = Some(strip);
        cx.notify();
    }
}

impl Render for Pane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();

        let mut pane_div = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .h_full()
            .w_full()
            .bg(colors.surface);

        if let Some(strip) = &self.tab_strip {
            pane_div = pane_div.child(strip.clone());
        }

        let content_div = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .h_full()
            .w_full()
            .overflow_hidden();
        let content_div = if let Some(view) = &self.content {
            content_div.child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .h_full()
                    .w_full()
                    .child(view.clone()),
            )
        } else {
            content_div.bg(colors.surface)
        };

        pane_div.child(content_div)
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Entity, TestAppContext, VisualTestContext};

    use super::*;
    use crate::tab_strip::{TabItem, TabStrip};

    fn pane(cx: &mut TestAppContext) -> (Entity<Pane>, Entity<TabStrip>, &mut VisualTestContext) {
        let mut strip = None;
        let (pane, cx) = open_fluent_test_window(cx, |_, cx| {
            let ts = cx.new(|cx| {
                let mut ts = TabStrip::new(cx);
                for id in ["a", "b", "c"] {
                    ts.add_tab(TabItem::new(id).label(id));
                }
                ts
            });
            strip = Some(ts.clone());
            Pane::new(cx).with_tab_strip(ts)
        });
        (pane, strip.unwrap(), cx)
    }

    #[gpui::test]
    fn clicking_tabs_switches_active_item(cx: &mut TestAppContext) {
        let (_pane, strip, cx) = pane(cx);
        strip.read_with(cx, |t, _| assert_eq!(t.active, 0));
        cx.click_selector("tab-2");
        strip.read_with(cx, |t, _| assert_eq!(t.active, 2));
        cx.click_selector("tab-1");
        strip.read_with(cx, |t, _| assert_eq!(t.active, 1));
    }

    #[gpui::test]
    fn set_tab_strip_and_content_update_pane(cx: &mut TestAppContext) {
        let (pane, _strip, cx) = pane(cx);
        pane.read_with(cx, |p, _| {
            assert!(p.tab_strip.is_some());
            assert!(p.content.is_none());
        });
        let content = cx.new(|_| Pane::default());
        pane.update(cx, |p, cx| p.set_content(content.into(), cx));
        pane.read_with(cx, |p, _| assert!(p.content.is_some()));
    }
}
