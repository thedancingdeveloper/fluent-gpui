use std::time::Duration;

use fluent_core::ThemeProvider as _;
use gpui::{
    prelude::*, px, svg, Animation, AnimationExt as _, App, IntoElement, RenderOnce,
    Transformation, Window,
};

/// An indeterminate animated activity indicator.
#[derive(IntoElement)]
pub struct Spinner {
    size: f32,
}

impl Spinner {
    pub fn new() -> Self {
        Self { size: 20.0 }
    }

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
            )
    }
}
