use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, IntoElement, Render, RenderOnce, SharedString, Window};

/// A hover-triggered tooltip anchored to a trigger element.
#[derive(IntoElement)]
pub struct Tooltip {
    text: SharedString,
    trigger: Option<gpui::AnyElement>,
}

impl Tooltip {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            trigger: None,
        }
    }

    pub fn trigger(mut self, trigger: impl IntoElement) -> Self {
        self.trigger = Some(trigger.into_any_element());
        self
    }
}

struct TooltipBubble {
    text: SharedString,
}

impl Render for TooltipBubble {
    fn render(&mut self, _window: &mut Window, cx: &mut gpui::Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;
        let spacing = &theme.spacing;
        let typography = &theme.typography;

        div()
            .px(px(spacing.sm))
            .py(px(spacing.xs))
            .rounded(px(radii.sm))
            .bg(colors.surface_blur_layer)
            .border_1()
            .border_color(colors.stroke_neutral_dim)
            .shadow_md()
            .text_size(px(typography.caption.size))
            .text_color(colors.on_neutral)
            .child(self.text.clone())
    }
}

impl RenderOnce for Tooltip {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let text = self.text.clone();
        let trigger = self
            .trigger
            .unwrap_or_else(|| div().child("").into_any_element());

        div()
            .id("tooltip-trigger")
            .child(trigger)
            .tooltip(move |_window, cx| cx.new(|_| TooltipBubble { text: text.clone() }).into())
    }
}
