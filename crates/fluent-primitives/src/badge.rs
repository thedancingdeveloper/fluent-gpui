use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, IntoElement, RenderOnce, Window};

use crate::Avatar;

/// Presence state shown on an avatar.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PresenceBadgeState {
    #[default]
    /// The available option.
    Available,
    /// The busy option.
    Busy,
    /// The away option.
    Away,
    /// The offline option.
    Offline,
    /// The do not disturb option.
    DoNotDisturb,
}

/// An avatar with a small status indicator overlaid at its lower-right edge.
#[derive(IntoElement)]
pub struct PresenceBadge {
    avatar: Avatar,
    state: PresenceBadgeState,
}

impl PresenceBadge {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(avatar: Avatar, state: PresenceBadgeState) -> Self {
        Self { avatar, state }
    }
}

impl RenderOnce for PresenceBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let size = self.avatar.size_value();
        let indicator_size = (size * 0.3).clamp(8.0, 12.0);
        let color = match self.state {
            PresenceBadgeState::Available => colors.status_success,
            PresenceBadgeState::Busy | PresenceBadgeState::DoNotDisturb => colors.status_error,
            PresenceBadgeState::Away => colors.status_warning,
            PresenceBadgeState::Offline => colors.neutral,
        };

        div().relative().size(px(size)).child(self.avatar).child(
            div()
                .absolute()
                .right(px(0.0))
                .bottom(px(0.0))
                .size(px(indicator_size))
                .rounded(px(9999.0))
                .bg(color)
                .border_2()
                .border_color(colors.surface),
        )
    }
}

/// The visual treatment of a [`CounterBadge`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CounterBadgeAppearance {
    #[default]
    /// The filled option.
    Filled,
    /// The ghost option.
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
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(count: u32) -> Self {
        Self {
            count,
            overflow_count: 99,
            show_zero: false,
            dot: false,
            appearance: CounterBadgeAppearance::Filled,
        }
    }

    /// Sets the overflow count configuration for this control.
    pub fn overflow_count(mut self, count: u32) -> Self {
        self.overflow_count = count.max(1);
        self
    }

    /// Sets the show zero configuration for this control.
    pub fn show_zero(mut self, show: bool) -> Self {
        self.show_zero = show;
        self
    }

    /// Sets the dot configuration for this control.
    pub fn dot(mut self, dot: bool) -> Self {
        self.dot = dot;
        self
    }

    /// Sets the appearance configuration for this control.
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
    /// Returns the count value.
    pub fn count(n: u32) -> Self {
        Self {
            variant: BadgeVariant::Count(n),
        }
    }

    /// Returns the dot value.
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
