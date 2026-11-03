/// Fluent 2 stroke-width tokens.
///
/// Values are in logical pixels (DIPs).
///
/// | Token | Value | Usage |
/// |-------|-------|-------|
/// | `thin` | 1px | Standard borders and outlines |
/// | `thick` | 2px | Emphasized borders and focus indicators |
/// | `thickest` | 3px | Strong emphasis and high-contrast strokes |
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StrokeWidthTokens {
    /// Public API item.
    pub thin: f32,
    /// Public API item.
    pub thick: f32,
    /// Public API item.
    pub thickest: f32,
}

impl Default for StrokeWidthTokens {
    fn default() -> Self {
        Self {
            thin: 1.0,
            thick: 2.0,
            thickest: 3.0,
        }
    }
}
