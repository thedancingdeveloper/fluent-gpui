use gpui::{px, Styled};

/// A single typographic ramp step.
///
/// `size` and `line_height` are in logical pixels (DIPs).
/// `weight` follows CSS font-weight conventions (400 = Regular, 600 = SemiBold, 700 = Bold).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeRamp {
    /// Public API item.
    pub size: f32,
    /// Public API item.
    pub line_height: f32,
    /// Public API item.
    pub weight: u32,
}

/// Typography scale tokens.
///
/// Matches the Fluent 2 type ramp. The `body` step is the default for UI text.
///
/// | Step       | Size | Line height | Weight |
/// |------------|------|-------------|--------|
/// | `caption`  | 11px | 16px        | 400    |
/// | `body`     | 13px | 20px        | 400    |
/// | `subtitle` | 15px | 22px        | 600    |
/// | `title`    | 20px | 28px        | 600    |
/// | `display`  | 28px | 36px        | 700    |
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypographyTokens {
    /// Public API item.
    pub caption: TypeRamp,
    /// Public API item.
    pub body: TypeRamp,
    /// Public API item.
    pub subtitle: TypeRamp,
    /// Public API item.
    pub title: TypeRamp,
    /// Public API item.
    pub display: TypeRamp,
}

impl Default for TypographyTokens {
    fn default() -> Self {
        Self {
            caption: TypeRamp {
                size: 11.0,
                line_height: 16.0,
                weight: 400,
            },
            body: TypeRamp {
                size: 13.0,
                line_height: 20.0,
                weight: 400,
            },
            subtitle: TypeRamp {
                size: 15.0,
                line_height: 22.0,
                weight: 600,
            },
            title: TypeRamp {
                size: 20.0,
                line_height: 28.0,
                weight: 600,
            },
            display: TypeRamp {
                size: 28.0,
                line_height: 36.0,
                weight: 700,
            },
        }
    }
}

impl TypeRamp {
    /// Font size as a whole-pixel length.
    pub fn font_size_px(&self) -> gpui::Pixels {
        px(self.size.round())
    }

    /// Line height as a whole-pixel length, so line boxes never land on half pixels.
    pub fn line_height_px(&self) -> gpui::Pixels {
        px(self.line_height.round())
    }
}

/// Applies a [`TypeRamp`] step's size and whole-pixel line height together, so a
/// text size can never be set without its line height (gpui's default is 1.618x).
pub trait TextRampExt: Styled + Sized {
    /// Sets `text_size` and `line_height` from `ramp`.
    fn text_ramp(self, ramp: &TypeRamp) -> Self {
        self.text_size(ramp.font_size_px())
            .line_height(ramp.line_height_px())
    }
}

impl<T: Styled> TextRampExt for T {}
