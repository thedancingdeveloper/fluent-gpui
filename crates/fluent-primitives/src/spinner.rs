use std::time::Duration;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, Animation, AnimationExt as _, App, ElementId, Hsla, IntoElement,
    RenderOnce, Role, SharedString, Transformation, Window,
};

/// An indeterminate animated activity indicator.
#[derive(IntoElement)]
pub struct Spinner {
    id: ElementId,
    size: f32,
    label: SharedString,
    color: Option<Hsla>,
}

impl Spinner {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new() -> Self {
        Self {
            id: "fluent-spinner-root".into(),
            size: 20.0,
            label: "Loading".into(),
            color: None,
        }
    }

    /// Sets the element id, which is also the id of the spinner's accessibility
    /// node. Give each spinner that shares a parent its own id; two with the same
    /// id collide in the accessibility tree. Defaults to `"fluent-spinner-root"`.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Accessible name announced for the indicator (default "Loading").
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// Overrides the spinner colour (default: the theme accent). Use it on a
    /// filled surface where the accent would blend into the fill.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Sets the size configuration for this control.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Spinner {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = self.color.unwrap_or(cx.theme().colors.accent);

        div()
            .id(self.id.clone())
            .role(Role::ProgressIndicator)
            .aria_label(self.label)
            .size(px(self.size))
            .child(
                svg()
                    .path("icons/spinner.svg")
                    .size(px(self.size))
                    .text_color(color)
                    .with_animation(
                        (self.id, "animation"),
                        Animation::new(Duration::from_millis(900)).repeat(),
                        |svg, delta| {
                            svg.with_transformation(Transformation::rotate(gpui::percentage(delta)))
                        },
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{div, Context, Render, TestAppContext};

    struct TwoHost;

    impl Render for TwoHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .child(Spinner::new().id("spinner-a"))
                .child(Spinner::new().id("spinner-b"))
        }
    }

    /// Two instances in one window used to trip the duplicate accessibility
    /// node id assertion (debug builds); in-process, no real window or GPU.
    #[gpui::test]
    fn two_spinners_with_distinct_ids_render_in_one_window(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, _| TwoHost);
        let snapshot = cx.accessibility_snapshot();
        assert!(!snapshot.is_empty());
    }
}
