use gpui::{hsla, px, BoxShadow};

/// Fluent 2 elevation shadow tokens.
///
/// Each token contains the key and ambient shadow layers, in that order. All
/// offsets, blur radii, and spread radii use logical pixels (DIPs).
#[derive(Clone, Debug, PartialEq)]
pub struct ShadowTokens {
    /// Public API item.
    pub shadow2: [BoxShadow; 2],
    /// Public API item.
    pub shadow4: [BoxShadow; 2],
    /// Public API item.
    pub shadow8: [BoxShadow; 2],
    /// Public API item.
    pub shadow16: [BoxShadow; 2],
    /// Public API item.
    pub shadow28: [BoxShadow; 2],
    /// Public API item.
    pub shadow64: [BoxShadow; 2],
}

impl Default for ShadowTokens {
    fn default() -> Self {
        Self {
            shadow2: shadow_layers(1.0, 2.0, 0.0, 2.0),
            shadow4: shadow_layers(2.0, 4.0, 0.0, 2.0),
            shadow8: shadow_layers(4.0, 8.0, 0.0, 2.0),
            shadow16: shadow_layers(8.0, 16.0, 0.0, 2.0),
            shadow28: shadow_layers(14.0, 28.0, 0.0, 8.0),
            shadow64: shadow_layers(32.0, 64.0, 0.0, 8.0),
        }
    }
}

fn shadow_layers(
    key_offset_y: f32,
    key_blur_radius: f32,
    ambient_offset_y: f32,
    ambient_blur_radius: f32,
) -> [BoxShadow; 2] {
    [
        BoxShadow::new(px(0.0), px(key_offset_y), hsla(0.0, 0.0, 0.0, 0.14))
            .blur_radius(px(key_blur_radius)),
        BoxShadow::new(px(0.0), px(ambient_offset_y), hsla(0.0, 0.0, 0.0, 0.12))
            .blur_radius(px(ambient_blur_radius)),
    ]
}
