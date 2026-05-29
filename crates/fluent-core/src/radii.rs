/// Border-radius tokens.
///
/// Values are in logical pixels (DIPs).
///
/// | Token | Value | Usage |
/// |-------|-------|-------|
/// | `sm`  | 2px   | Inputs, toolbar buttons |
/// | `md`  | 4px   | Cards, panels |
/// | `lg`  | 8px   | Dialogs, popovers |
/// | `pill`| 9999px| Badges, chips |
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RadiiTokens {
    pub sm: f32,
    pub md: f32,
    pub lg: f32,
    /// Effectively circular/pill — use for badge and chip shapes.
    pub pill: f32,
}

impl Default for RadiiTokens {
    fn default() -> Self {
        Self {
            sm: 2.0,
            md: 4.0,
            lg: 8.0,
            pill: 9999.0,
        }
    }
}
