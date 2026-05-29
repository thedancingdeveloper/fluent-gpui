mod async_resource;
mod colors;
mod components;
mod motion;
mod radii;
mod spacing;
mod theme;
mod typography;

pub use async_resource::{AsyncResource, AsyncResourceState, ResourceRequest};
pub use colors::{gradient_from_hue, hue_for, tint, with_alpha, ColorScheme};
pub use components::ComponentTokens;
pub use motion::{popup_motion_surface, MotionTokens};
pub use radii::RadiiTokens;
pub use spacing::SpacingTokens;
pub use theme::{Brightness, Theme, ThemeProvider};
pub use typography::{TypeRamp, TypographyTokens};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spacing_tokens_values() {
        let s = SpacingTokens::default();
        assert_eq!(s.xs, 2.0);
        assert_eq!(s.sm, 4.0);
        assert_eq!(s.md, 8.0);
        assert_eq!(s.lg, 12.0);
        assert_eq!(s.xl, 16.0);
        assert_eq!(s.xxl, 24.0);
    }

    #[test]
    fn radii_tokens_values() {
        let r = RadiiTokens::default();
        assert_eq!(r.sm, 2.0);
        assert_eq!(r.md, 4.0);
        assert_eq!(r.lg, 8.0);
        assert_eq!(r.pill, 9999.0);
    }

    #[test]
    fn typography_ramp_defaults() {
        let t = TypographyTokens::default();
        assert_eq!(t.caption.size, 11.0);
        assert_eq!(t.body.size, 13.0);
        assert_eq!(t.body.line_height, 20.0);
        assert_eq!(t.body.weight, 400);
        assert_eq!(t.subtitle.size, 15.0);
        assert_eq!(t.subtitle.weight, 600);
        assert_eq!(t.title.size, 20.0);
        assert_eq!(t.display.size, 28.0);
        assert_eq!(t.display.weight, 700);
    }

    #[test]
    fn component_geometry_defaults() {
        let c = ComponentTokens::default();
        assert_eq!(c.menu_bar_height, 28.0);
        assert_eq!(c.menu_item_height, 28.0);
        assert_eq!(c.dropdown_height, 32.0);
        assert_eq!(c.dropdown_option_height, 28.0);
        assert_eq!(c.text_input_height, 32.0);
        assert_eq!(c.ribbon_tab_height, 28.0);
    }

    #[test]
    fn motion_tokens_are_subtle() {
        let m = MotionTokens::default();
        assert_eq!(m.popup_enter_duration_ms, 160);
        assert_eq!(m.popup_exit_duration_ms, 160);
        assert_eq!(m.popup_offset, 2.0);
        assert_eq!(m.popup_scale_delta, 0.02);
    }

    #[test]
    fn dark_theme_surface_is_near_black() {
        let theme = Theme::dark();
        assert!(
            theme.colors.surface.l < 0.15,
            "dark surface lightness = {}, expected < 0.15",
            theme.colors.surface.l
        );
    }

    #[test]
    fn light_theme_surface_is_near_white() {
        let theme = Theme::light();
        assert!(
            theme.colors.surface.l > 0.9,
            "light surface lightness = {}, expected > 0.9",
            theme.colors.surface.l
        );
    }

    #[test]
    fn dark_accent_has_blue_hue() {
        let theme = Theme::dark();
        let h = theme.colors.accent.h;
        // Fluent 2 brand blue #115EA3 is ~207° → 0.575 in 0-1 range
        assert!(
            (h - 0.575).abs() < 0.02,
            "dark accent hue = {h:.3}, expected ~0.575"
        );
    }

    #[test]
    fn light_accent_has_blue_hue() {
        let theme = Theme::light();
        let h = theme.colors.accent.h;
        // Fluent 2 brand blue #0078D4 is ~206° → 0.572 in 0-1 range
        assert!(
            (h - 0.572).abs() < 0.02,
            "light accent hue = {h:.3}, expected ~0.572"
        );
    }

    #[test]
    fn theme_brightness_flag() {
        assert!(Theme::dark().is_dark());
        assert!(!Theme::light().is_dark());
    }

    #[test]
    fn tint_scales_alpha_clamped_to_unit_interval() {
        use gpui::hsla;
        let opaque_red = hsla(0.0, 1.0, 0.5, 1.0);
        assert_eq!(tint(opaque_red, 0.18).a, 0.18);
        assert_eq!(tint(opaque_red, -0.5).a, 0.0);
        assert_eq!(tint(opaque_red, 1.5).a, 1.0);
        // Hue, saturation, lightness preserved.
        let result = tint(opaque_red, 0.5);
        assert_eq!(result.h, 0.0);
        assert_eq!(result.s, 1.0);
        assert_eq!(result.l, 0.5);
    }

    #[test]
    fn with_alpha_replaces_rather_than_scales() {
        use gpui::hsla;
        let half_red = hsla(0.0, 1.0, 0.5, 0.5);
        assert_eq!(with_alpha(half_red, 0.18).a, 0.18);
        // tint would have multiplied (0.5 * 0.18 = 0.09); with_alpha sets it.
        assert_ne!(with_alpha(half_red, 0.18).a, tint(half_red, 0.18).a);
    }

    #[test]
    fn hue_for_is_deterministic_and_in_range() {
        let a = hue_for("WhatsApp");
        let b = hue_for("WhatsApp");
        assert_eq!(a, b);
        assert!((0.0..360.0).contains(&a));
        // Different inputs almost always produce different hues.
        assert_ne!(hue_for("WhatsApp"), hue_for("Messages"));
    }

    #[test]
    fn gradient_from_hue_returns_a_linear_gradient_background() {
        // Smoke test: constructor returns a Background — we can't inspect the
        // private tag from outside the crate but at least confirm it builds.
        let _bg = gradient_from_hue(207.0);
    }

    #[test]
    fn light_and_dark_accent_differ() {
        let dark = Theme::dark();
        let light = Theme::light();
        // Dark uses #115EA3, light uses #0078D4 — different lightness
        assert!(
            (dark.colors.accent.l - light.colors.accent.l).abs() > 0.05,
            "dark accent l={} light accent l={}",
            dark.colors.accent.l,
            light.colors.accent.l
        );
    }

    #[test]
    fn semantic_status_colors_are_distinct() {
        let theme = Theme::light();
        assert_ne!(theme.colors.status_success, theme.colors.status_error);
        assert_ne!(theme.colors.status_warning, theme.colors.status_error);
        assert_ne!(theme.colors.status_info, theme.colors.status_success);
    }
}
