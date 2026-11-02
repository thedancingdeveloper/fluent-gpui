use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, IntoElement, RenderOnce, Window};

/// The visual treatment of a [`CounterBadge`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CounterBadgeAppearance {
    #[default]
    Filled,
    Ghost,
}

fn counter_badge_label(count: u32, overflow_count: u32, show_zero: bool) -> Option<String> {
    if count == 0 && !show_zero {
        return None;
    }

    if count > overflow_count {
        Some(format!("{overflow_count}+"))
    } else {
        Some(count.to_string())
    }
}

/// A numeric notification badge with configurable overflow and appearance.
#[derive(IntoElement)]
pub struct CounterBadge {
    count: u32,
    overflow_count: u32,
    show_zero: bool,
    dot: bool,
    appearance: CounterBadgeAppearance,
}

impl CounterBadge {
    pub fn new(count: u32) -> Self {
        Self {
            count,
            overflow_count: 99,
            show_zero: false,
            dot: false,
            appearance: CounterBadgeAppearance::Filled,
        }
    }

    pub fn overflow_count(mut self, count: u32) -> Self {
        self.overflow_count = count.max(1);
        self
    }

    pub fn show_zero(mut self, show: bool) -> Self {
        self.show_zero = show;
        self
    }

    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }

    pub fn appearance(mut self, appearance: CounterBadgeAppearance) -> Self {
        self.appearance = appearance;
        self
    }
}

impl RenderOnce for CounterBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let radii = &cx.theme().radii;
        let typography = &cx.theme().typography;
        let label = counter_badge_label(self.count, self.overflow_count, self.show_zero);
        let ghost = matches!(self.appearance, CounterBadgeAppearance::Ghost);

        if self.dot {
            return div()
                .size(px(8.0))
                .rounded(px(radii.pill))
                .when(!ghost, |this| this.bg(colors.accent))
                .when(ghost, |this| {
                    this.bg(colors.subtle)
                        .border_1()
                        .border_color(colors.accent)
                })
                .into_any_element();
        }

        let Some(label) = label else {
            return div().into_any_element();
        };

        div()
            .min_w(px(18.0))
            .h(px(18.0))
            .px(px(4.0))
            .rounded(px(radii.pill))
            .when(!ghost, |this| this.bg(colors.accent))
            .when(ghost, |this| {
                this.bg(colors.subtle)
                    .border_1()
                    .border_color(colors.accent)
            })
            .flex()
            .items_center()
            .justify_center()
            .text_size(px(typography.caption.size))
            .text_color(if ghost {
                colors.accent
            } else {
                colors.on_accent
            })
            .child(label)
            .into_any_element()
    }
}

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

#[cfg(test)]
mod tests {
    use super::counter_badge_label;

    #[test]
    fn counter_badge_hides_zero_by_default() {
        assert_eq!(counter_badge_label(0, 99, false), None);
    }

    #[test]
    fn counter_badge_can_show_zero() {
        assert_eq!(counter_badge_label(0, 99, true), Some("0".to_string()));
    }

    #[test]
    fn counter_badge_caps_at_overflow_count() {
        assert_eq!(counter_badge_label(99, 99, false), Some("99".to_string()));
        assert_eq!(counter_badge_label(125, 99, false), Some("99+".to_string()));
        assert_eq!(counter_badge_label(98, 99, false), Some("98".to_string()));
    }

    #[test]
    fn counter_badge_supports_custom_overflow_count() {
        assert_eq!(counter_badge_label(10, 9, false), Some("9+".to_string()));
    }
}
