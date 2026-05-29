/// Spacing scale tokens.
///
/// Used for margins, padding, and gap values throughout the framework.
/// Values are in logical pixels (DIPs).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SpacingTokens {
    pub xs: f32,  // 2.0
    pub sm: f32,  // 4.0
    pub md: f32,  // 8.0
    pub lg: f32,  // 12.0
    pub xl: f32,  // 16.0
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
