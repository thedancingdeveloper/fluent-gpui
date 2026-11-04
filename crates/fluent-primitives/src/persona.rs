use fluent_core::TextRampExt;
use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, FontWeight, IntoElement, RenderOnce, SharedString, Window};

use crate::{Avatar, PresenceBadge, PresenceBadgeState};

/// Size presets for a persona row.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PersonaSize {
    /// The small option.
    Small,
    #[default]
    /// The medium option.
    Medium,
    /// The large option.
    Large,
}

impl PersonaSize {
    fn avatar_size(self) -> f32 {
        match self {
            Self::Small => 32.0,
            Self::Medium => 40.0,
            Self::Large => 48.0,
        }
    }
}

/// An avatar paired with primary and optional secondary text.
#[derive(IntoElement)]
pub struct Persona {
    avatar: Avatar,
    primary: SharedString,
    secondary: Option<SharedString>,
    presence: Option<PresenceBadgeState>,
    size: PersonaSize,
}

impl Persona {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(avatar: Avatar, primary: impl Into<SharedString>) -> Self {
        Self {
            avatar,
            primary: primary.into(),
            secondary: None,
            presence: None,
            size: PersonaSize::default(),
        }
    }

    /// Sets the secondary configuration for this control.
    pub fn secondary(mut self, secondary: impl Into<SharedString>) -> Self {
        self.secondary = Some(secondary.into());
        self
    }

    /// Sets the presence configuration for this control.
    pub fn presence(mut self, state: PresenceBadgeState) -> Self {
        self.presence = Some(state);
        self
    }

    /// Sets the size configuration for this control.
    pub fn size(mut self, size: PersonaSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for Persona {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let typography = &cx.theme().typography;
        let ramp = match self.size {
            PersonaSize::Small => typography.caption,
            PersonaSize::Medium => typography.body,
            PersonaSize::Large => typography.subtitle,
        };
        let avatar = self.avatar.size(self.size.avatar_size());
        let avatar = match self.presence {
            Some(state) => PresenceBadge::new(avatar, state).into_any_element(),
            None => avatar.into_any_element(),
        };

        div()
            .flex()
            .items_center()
            .gap(px(10.0))
            .child(avatar)
            .child(
                div()
                    .flex()
                    .flex_col()
                    .text_ramp(&ramp)
                    .line_height(px(ramp.line_height))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(self.primary))
                    .when_some(self.secondary, |this, secondary| {
                        this.child(
                            div()
                                .text_color(cx.theme().colors.on_subtle)
                                .child(secondary),
                        )
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn persona_sizes_use_expected_avatar_dimensions() {
        assert_eq!(PersonaSize::Small.avatar_size(), 32.0);
        assert_eq!(PersonaSize::Medium.avatar_size(), 40.0);
        assert_eq!(PersonaSize::Large.avatar_size(), 48.0);
    }
}
