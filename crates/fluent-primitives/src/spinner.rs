use std::time::Duration;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, Animation, AnimationExt as _, App, IntoElement, RenderOnce, Role,
    SharedString, Transformation, Window,
};

/// An indeterminate animated activity indicator.
#[derive(IntoElement)]
pub struct Spinner {
    size: f32,
    label: SharedString,
}

impl Spinner {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new() -> Self {
        Self {
            size: 20.0,
            label: "Loading".into(),
        }
    }

    /// Accessible name announced for the indicator (default "Loading").
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
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
        let colors = &cx.theme().colors;

        div()
            .id("fluent-spinner-root")
            .role(Role::ProgressIndicator)
            .aria_label(self.label)
            .size(px(self.size))
            .child(
                svg()
                    .path("icons/spinner.svg")
                    .size(px(self.size))
                    .text_color(colors.accent)
                    .with_animation(
                        "fluent-spinner",
                        Animation::new(Duration::from_millis(900)).repeat(),
                        |svg, delta| {
                            svg.with_transformation(Transformation::rotate(gpui::percentage(delta)))
                        },
                    ),
            )
    }
}
