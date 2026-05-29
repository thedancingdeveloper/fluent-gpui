/// A single typographic ramp step.
///
/// `size` and `line_height` are in logical pixels (DIPs).
/// `weight` follows CSS font-weight conventions (400 = Regular, 600 = SemiBold, 700 = Bold).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TypeRamp {
    pub size: f32,
    pub line_height: f32,
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
    pub caption: TypeRamp,
    pub body: TypeRamp,
    pub subtitle: TypeRamp,
    pub title: TypeRamp,
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
