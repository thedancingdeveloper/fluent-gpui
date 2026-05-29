use fluent_core::{Theme, ThemeProvider as _};
use gpui::{div, prelude::*, AnyView, Context, Entity, IntoElement, Render, Window};

use crate::tab_strip::TabStrip;

/// A content area that optionally has a `TabStrip` at the top.
///
/// The content is an `AnyView` — set it via `Pane::set_content`.
#[derive(Default)]
pub struct Pane {
    pub tab_strip: Option<Entity<TabStrip>>,
    pub content: Option<AnyView>,
}

impl Pane {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self::default()
    }

    pub fn with_tab_strip(mut self, strip: Entity<TabStrip>) -> Self {
        self.tab_strip = Some(strip);
        self
    }

    pub fn set_content(&mut self, content: AnyView, cx: &mut Context<Self>) {
        self.content = Some(content);
        cx.notify();
    }

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
