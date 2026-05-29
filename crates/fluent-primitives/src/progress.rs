use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, IntoElement, RenderOnce, Window};

/// A determinate horizontal progress bar.
///
/// `value` is clamped to `[0.0, 1.0]`.
#[derive(IntoElement)]
pub struct ProgressBar {
    value: f32,
    height: f32,
}

impl ProgressBar {
    pub fn new(value: f32) -> Self {
        Self {
            value: value.clamp(0.0, 1.0),
            height: 4.0,
        }
    }

    pub fn height(mut self, px: f32) -> Self {
        self.height = px;
        self
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;

        div()
            .w_full()
            .h(px(self.height))
            .rounded(px(9999.0))
            .bg(colors.neutral)
            .child(
                div()
                    .h_full()
                    .rounded(px(9999.0))
                    .bg(colors.accent)
                    // Use relative width via percentage — approximate with a flex fill
                    .w(gpui::relative(self.value)),
            )
    }
}
