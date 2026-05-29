use fluent_core::{ThemeProvider as _, TypographyTokens};
use gpui::{
    div, prelude::*, px, App, FontWeight, Hsla, IntoElement, RenderOnce, SharedString, Window,
};

/// Which step of the type ramp to use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelSize {
    Caption,
    #[default]
    Body,
    Subtitle,
    Title,
    Display,
}

/// A text label styled with the active theme's type ramp.
///
/// ```ignore
/// Label::new("Hello").size(LabelSize::Subtitle).color(cx.theme().colors.on_accent)
/// ```
#[derive(IntoElement)]
pub struct Label {
    text: SharedString,
    size: LabelSize,
    color: Option<Hsla>,
    truncate: bool,
    eyebrow: bool,
}

impl Label {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            size: LabelSize::default(),
            color: None,
            truncate: false,
            eyebrow: false,
        }
    }

    pub fn size(mut self, size: LabelSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }

    /// Clip text with an ellipsis when it overflows.
    pub fn truncate(mut self) -> Self {
        self.truncate = true;
        self
    }

    /// Section-header / "eyebrow" treatment: uppercase, caption size,
    /// semibold, with extra spacing between characters (faked via a row of
    /// per-character spans because GPUI has no native letter-spacing).
    /// Use for landmarks like "NOW PLAYING · SPOTIFY", "BATTERY", "STORAGE".
    pub fn eyebrow(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            size: LabelSize::Caption,
            color: None,
            truncate: false,
            eyebrow: true,
        }
    }
}

impl RenderOnce for Label {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let default_color = if self.eyebrow {
            theme.colors.on_subtle
        } else {
            theme.colors.on_neutral
        };
        let color = self.color.unwrap_or(default_color);
        let t = &theme.typography;
        let ramp = ramp_for_size(self.size, t);
        let weight = if self.eyebrow {
            FontWeight::SEMIBOLD
        } else {
            FontWeight(ramp.weight as f32)
        };

        if self.eyebrow {
            // Render each character as its own span with right margin so we
            // visually "letter-space" without modifying GPUI's text shaper.
            let mut row = div()
                .flex()
                .flex_row()
                .items_center()
                .text_size(px(ramp.size))
                .line_height(px(ramp.line_height))
                .font_weight(weight)
                .text_color(color);
            let upper: String = self.text.to_uppercase();
            let chars: Vec<char> = upper.chars().collect();
            let last = chars.len().saturating_sub(1);
            for (i, ch) in chars.into_iter().enumerate() {
                let mut span = div().child(ch.to_string());
                // Hair-space-equivalent: 0.06em ≈ caption-size × 0.06.
                if i < last && !ch.is_whitespace() {
                    span = span.mr(px(ramp.size * 0.08));
                }
                row = row.child(span);
            }
            return row.into_any_element();
        }

        let el = div()
            .text_size(px(ramp.size))
            .line_height(px(ramp.line_height))
            .font_weight(weight)
            .text_color(color)
            .child(self.text);

        if self.truncate {
            el.truncate().whitespace_nowrap().into_any_element()
        } else {
            el.into_any_element()
        }
    }
}

fn ramp_for_size(size: LabelSize, t: &TypographyTokens) -> fluent_core::TypeRamp {
    match size {
        LabelSize::Caption => t.caption,
        LabelSize::Body => t.body,
        LabelSize::Subtitle => t.subtitle,
        LabelSize::Title => t.title,
        LabelSize::Display => t.display,
    }
}
