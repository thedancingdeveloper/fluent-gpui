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
    /// Creates a new instance for use in a Fluent interface.
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

    /// Sets the letter configuration for this control.
    pub fn letter(mut self, letter: impl Into<SharedString>) -> Self {
        self.letter = letter.into();
        self
    }

    /// Sets the hue configuration for this control.
    pub fn hue(mut self, deg: f32) -> Self {
        self.hue = Some(deg);
        self
    }

    /// Sets the size configuration for this control.
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

/// Corner radius and letter size for a dot of `size` px: small dots use the
/// `md` radius and caption text, mid dots `lg` and 12px, large dots `lg` and body text.
pub(crate) fn dot_metrics(
    size: f32,
    radii: &fluent_core::RadiiTokens,
    caption: f32,
    body: f32,
) -> (f32, f32) {
    if size <= 22.0 {
        (radii.md, caption)
    } else if size <= 32.0 {
        (radii.lg, 12.0)
    } else {
        (radii.lg, body)
    }
}

impl RenderOnce for AppDot {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let typography = &cx.theme().typography;
        let hue = self.hue.unwrap_or_else(|| hue_for(&self.seed));
        let (radius, text_size) = dot_metrics(
            self.size,
            &cx.theme().radii,
            typography.caption.size,
            typography.body.size,
        );
        div()
            .size(px(self.size))
            .rounded(px(radius))
            .bg(gradient_from_hue(hue))
            .flex()
            .items_center()
            .justify_center()
            .text_color(cx.theme().colors.on_accent)
            .font_weight(FontWeight::BOLD)
            .text_size(px(text_size))
            .line_height(px(text_size.round()))
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
    fn metrics_are_whole_pixels_on_the_scale() {
        let r = fluent_core::RadiiTokens::default();
        assert_eq!(dot_metrics(20.0, &r, 11.0, 13.0), (4.0, 11.0));
        assert_eq!(dot_metrics(28.0, &r, 11.0, 13.0), (8.0, 12.0));
        assert_eq!(dot_metrics(40.0, &r, 11.0, 13.0), (8.0, 13.0));
    }

    #[test]
    fn letter_override_takes_precedence() {
        assert_eq!(AppDot::new("whatsapp").letter("✓").letter_for(), "✓");
    }
}
