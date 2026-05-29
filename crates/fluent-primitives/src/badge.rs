use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, IntoElement, RenderOnce, Window};

/// Content of a `Badge`.
#[derive(Clone, Debug)]
pub enum BadgeVariant {
    /// Numeric count (capped display at 99+).
    Count(u32),
    /// Solid status dot — no text.
    Dot,
}

/// A small pill / dot overlay used for notifications and status.
#[derive(IntoElement)]
pub struct Badge {
    variant: BadgeVariant,
}

impl Badge {
    pub fn count(n: u32) -> Self {
        Self {
            variant: BadgeVariant::Count(n),
        }
    }

    pub fn dot() -> Self {
        Self {
            variant: BadgeVariant::Dot,
        }
    }
}

impl RenderOnce for Badge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let radii = &cx.theme().radii;
        let typography = &cx.theme().typography;

        match self.variant {
            BadgeVariant::Dot => div()
                .size(px(8.0))
                .rounded(px(radii.pill))
                .bg(colors.accent),

            BadgeVariant::Count(n) => {
                let label = if n > 99 {
                    "99+".to_string()
                } else {
                    n.to_string()
                };
                div()
                    .min_w(px(18.0))
                    .h(px(18.0))
                    .px(px(4.0))
                    .rounded(px(radii.pill))
                    .bg(colors.accent)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(typography.caption.size))
                    .text_color(colors.on_accent)
                    .child(label)
            }
        }
    }
}
