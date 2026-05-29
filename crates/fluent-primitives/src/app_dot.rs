use fluent_core::{gradient_from_hue, hue_for, ThemeProvider as _};
use gpui::{div, prelude::*, px, App, FontWeight, IntoElement, RenderOnce, SharedString, Window};

/// A rounded-square coloured swatch with a single capital letter. Used to
/// represent an app or contact in a notification or list row.
///
/// Pass the originating string (app package, contact name) as `seed` and a
/// deterministic gradient is chosen for it. Override with `hue(deg)` if a
/// brand colour is known.
#[derive(IntoElement)]
pub struct AppDot {
    seed: SharedString,
    letter: SharedString,
    hue: Option<f32>,
    size: f32,
}

impl AppDot {
    pub fn new(seed: impl Into<SharedString>) -> Self {
        let seed: SharedString = seed.into();
        let letter: SharedString = seed
            .chars()
            .next()
            .map(|c| c.to_uppercase().to_string())
            .unwrap_or_else(|| "•".to_owned())
            .into();
        Self {
            seed,
            letter,
            hue: None,
            size: 28.0,
        }
    }

    pub fn letter(mut self, letter: impl Into<SharedString>) -> Self {
        self.letter = letter.into();
        self
    }

    pub fn hue(mut self, deg: f32) -> Self {
        self.hue = Some(deg);
        self
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }
}

#[cfg(test)]
impl AppDot {
    pub(crate) fn letter_for(&self) -> &str {
        &self.letter
    }
}

impl RenderOnce for AppDot {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let typography = &cx.theme().typography;
        let hue = self.hue.unwrap_or_else(|| hue_for(&self.seed));
        let text_size = if self.size <= 22.0 {
            10.0
        } else if self.size <= 32.0 {
            12.0
        } else {
            typography.body.size
        };
        div()
            .size(px(self.size))
            .rounded(px(6.0))
            .bg(gradient_from_hue(hue))
            .flex()
            .items_center()
            .justify_center()
            .text_color(gpui::white())
            .font_weight(FontWeight::BOLD)
            .text_size(px(text_size))
            .child(self.letter)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn derives_first_uppercase_letter_from_seed() {
        assert_eq!(AppDot::new("whatsapp").letter_for(), "W");
        assert_eq!(AppDot::new("Messages").letter_for(), "M");
        assert_eq!(AppDot::new("").letter_for(), "•");
    }

    #[test]
    fn letter_override_takes_precedence() {
        assert_eq!(AppDot::new("whatsapp").letter("✓").letter_for(), "✓");
    }
}
