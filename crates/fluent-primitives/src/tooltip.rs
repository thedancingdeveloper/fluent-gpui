use fluent_core::TextRampExt;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ElementId, IntoElement, Render, RenderOnce, SharedString, Window,
};

/// A hover-triggered tooltip anchored to a trigger element.
#[derive(IntoElement)]
pub struct Tooltip {
    id: ElementId,
    text: SharedString,
    trigger: Option<gpui::AnyElement>,
}

impl Tooltip {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            id: "tooltip-trigger".into(),
            text: text.into(),
            trigger: None,
        }
    }

    /// Sets the element id of the trigger wrapper, which keys its hover and open
    /// state. Give each tooltip that shares a parent its own id; two with the
    /// same id share that state. Defaults to `"tooltip-trigger"`.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Sets the trigger configuration for this control.
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

        let selector = format!("tooltip-bubble-{}", self.text);
        div()
            .debug_selector(move || selector)
            .px(px(spacing.md))
            .py(px(spacing.sm))
            .rounded(px(radii.md))
            .bg(colors.surface_blur_layer)
            .border_1()
            .border_color(colors.stroke_neutral_dim)
            .shadow_md()
            .text_ramp(&typography.caption)
            .text_size(px(12.0))
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
            .id(self.id)
            .child(trigger)
            .tooltip(move |_window, cx| cx.new(|_| TooltipBubble { text: text.clone() }).into())
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Modifiers, TestAppContext, VisualTestContext};

    use super::*;

    struct Two;

    impl Render for Two {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .gap_4()
                .child(
                    Tooltip::new("A").id("tip-a").trigger(
                        div()
                            .debug_selector(|| "trig-a".into())
                            .w(px(60.0))
                            .h(px(20.0))
                            .child("a"),
                    ),
                )
                .child(
                    Tooltip::new("B").id("tip-b").trigger(
                        div()
                            .debug_selector(|| "trig-b".into())
                            .w(px(60.0))
                            .h(px(20.0))
                            .child("b"),
                    ),
                )
        }
    }

    fn hover(cx: &mut VisualTestContext, selector: &'static str) {
        let at = cx
            .selector_bounds(selector)
            .expect("trigger painted")
            .center();
        cx.simulate_mouse_move(at, None, Modifiers::default());
        cx.executor().advance_clock(Duration::from_secs(2));
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
    }

    /// Two tooltips in one view share the default `tooltip-trigger` id, which keys
    /// each trigger's hover/open state; hovering one must open only its own
    /// bubble. In-process, no real window or GPU.
    #[gpui::test]
    fn two_tooltips_open_independently(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, _| Two);
        hover(cx, "trig-a");
        assert!(cx.selector_bounds("tooltip-bubble-A").is_some(), "A open");
        assert!(cx.selector_bounds("tooltip-bubble-B").is_none(), "B closed");
        hover(cx, "trig-b");
        assert!(cx.selector_bounds("tooltip-bubble-B").is_some(), "B open");
        assert!(cx.selector_bounds("tooltip-bubble-A").is_none(), "A closed");
    }
}
