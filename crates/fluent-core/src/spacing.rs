/// Spacing scale tokens.
///
/// Used for margins, padding, and gap values throughout the framework.
/// Values are in logical pixels (DIPs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpacingTokens {
    /// Public API item.
    pub xs: f32, // 2.0
    /// Public API item.
    pub sm: f32, // 4.0
    /// Public API item.
    pub md: f32, // 8.0
    /// Public API item.
    pub lg: f32, // 12.0
    /// Public API item.
    pub xl: f32, // 16.0
    /// Public API item.
    pub xxl: f32, // 24.0
}

impl Default for SpacingTokens {
    fn default() -> Self {
        Self {
            xs: 2.0,
            sm: 4.0,
            md: 8.0,
            lg: 12.0,
            xl: 16.0,
            xxl: 24.0,
        }
    }
}
