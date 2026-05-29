use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, IntoElement, RenderOnce, Window};

/// Orientation of a `Divider`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DividerOrientation {
    #[default]
    Horizontal,
    Vertical,
}

/// A thin visual separator line.
#[derive(IntoElement)]
pub struct Divider {
    orientation: DividerOrientation,
}

impl Divider {
    pub fn horizontal() -> Self {
        Self {
            orientation: DividerOrientation::Horizontal,
        }
    }

    pub fn vertical() -> Self {
        Self {
            orientation: DividerOrientation::Vertical,
        }
    }
}

impl RenderOnce for Divider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let color = cx.theme().colors.stroke_neutral_subtle;
        match self.orientation {
            DividerOrientation::Horizontal => div().w_full().h(px(1.0)).bg(color),
            DividerOrientation::Vertical => div().h_full().w(px(1.0)).bg(color),
        }
    }
}
