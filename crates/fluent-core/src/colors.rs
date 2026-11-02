use gpui::{hsla, linear_color_stop, linear_gradient, Background, Hsla};

/// Build a vivid two-stop diagonal gradient from a hue (0-360°) — the canonical
/// avatar / app-dot swatch. Approximates the design spec
/// `oklch(0.55 0.13 H) → oklch(0.42 0.10 H)` in HSL space.
pub fn gradient_from_hue(hue_deg: f32) -> Background {
    let h = (hue_deg.rem_euclid(360.0)) / 360.0;
    let from = hsla(h, 0.55, 0.55, 1.0);
    let to = hsla(h, 0.45, 0.40, 1.0);
    linear_gradient(
        135.0,
        linear_color_stop(from, 0.0),
        linear_color_stop(to, 1.0),
    )
}

/// Deterministic hue (0-360°) for an arbitrary string — used so a contact name
/// or app package consistently maps to the same swatch colour across sessions.
pub fn hue_for(seed: &str) -> f32 {
    let mut h: u32 = 5381;
    for b in seed.bytes() {
        h = h.wrapping_mul(33).wrapping_add(b as u32);
    }
    (h % 360) as f32
}

/// Return `color` with its alpha multiplied by `factor` (clamped to 0..=1).
/// Equivalent to `color-mix(color factor*100%, transparent)` in CSS — useful
/// for accent-tinted backgrounds and overlays.
///
/// ```ignore
/// // 18% accent on a card icon tile:
/// .bg(tint(theme.colors.accent, 0.18))
/// ```
pub fn tint(color: Hsla, factor: f32) -> Hsla {
    Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: (color.a * factor.clamp(0.0, 1.0)).clamp(0.0, 1.0),
    }
}

/// Shorter form of [`tint`] for the common case of overriding the alpha
/// entirely (rather than scaling it). Returns `color` with `a = alpha`.
pub fn with_alpha(color: Hsla, alpha: f32) -> Hsla {
    Hsla {
        h: color.h,
        s: color.s,
        l: color.l,
        a: alpha.clamp(0.0, 1.0),
    }
}

/// All surface, fill, foreground, and stroke colors for the theme.
///
/// Token names follow the Fluent 2 design token vocabulary where possible.
/// See <https://fluent2.microsoft.design/color> for specification.
#[derive(Clone, Debug)]
pub struct NeutralColorStates {
    pub default: Hsla,
    pub hover: Hsla,
    pub pressed: Hsla,
    pub selected: Hsla,
    pub disabled: Hsla,
}

#[derive(Clone, Debug)]
pub struct ColorScheme {
    pub neutral_foreground_1: NeutralColorStates,
    pub neutral_foreground_2: NeutralColorStates,
    pub neutral_foreground_3: NeutralColorStates,
    pub neutral_background_1: NeutralColorStates,
    pub neutral_background_2: NeutralColorStates,
    pub neutral_background_3: NeutralColorStates,
    pub neutral_stroke_1: NeutralColorStates,
    pub neutral_stroke_2: NeutralColorStates,
    pub neutral_stroke_3: NeutralColorStates,

    // --- Fill backgrounds (interactive elements: buttons, inputs, etc.) ---
    /// Neutral fill — default state (Fluent: colorNeutralBackground3)
    pub neutral: Hsla,
    /// Neutral fill — pointer hover
    pub neutral_hover: Hsla,
    /// Neutral fill — disabled
    pub neutral_disabled: Hsla,
    /// Neutral fill — selected/pressed
    pub neutral_selected: Hsla,

    /// Accent fill — default state (brand colour)
    pub accent: Hsla,
    /// Accent fill — pointer hover
    pub accent_hover: Hsla,
    /// Accent fill — disabled
    pub accent_disabled: Hsla,
    /// Accent fill — selected/pressed
    pub accent_selected: Hsla,

    /// Subtle fill — default state (typically transparent)
    pub subtle: Hsla,
    /// Subtle fill — pointer hover
    pub subtle_hover: Hsla,
    /// Subtle fill — disabled
    pub subtle_disabled: Hsla,
    /// Subtle fill — selected/pressed
    pub subtle_selected: Hsla,

    /// Page / window background (Fluent: colorNeutralBackground1)
    pub surface: Hsla,
    /// Dimmed background layer behind surfaces
    pub surface_dim: Hsla,
    /// Frosted / blurred backdrop layer
    pub surface_blur_layer: Hsla,

    // --- Foreground / text colours ---
    /// Primary text on neutral fill
    pub on_neutral: Hsla,
    /// Disabled text on neutral fill
    pub on_neutral_disabled: Hsla,
    /// Text on selected neutral fill
    pub on_neutral_selected: Hsla,
    /// Brand-coloured text on neutral surface (links, icons)
    pub on_neutral_accent: Hsla,

    /// Text / icon on accent fill
    pub on_accent: Hsla,
    /// Disabled text on accent fill
    pub on_accent_disabled: Hsla,
    /// Text on selected accent fill
    pub on_accent_selected: Hsla,

    /// Secondary text on subtle fill
    pub on_subtle: Hsla,
    /// Disabled text on subtle fill
    pub on_subtle_disabled: Hsla,
    /// Text on selected subtle fill
    pub on_subtle_selected: Hsla,

    // --- Strokes / borders ---
    /// Default border / outline (Fluent: colorNeutralStroke1)
    pub stroke_neutral: Hsla,
    /// Disabled border
    pub stroke_neutral_disabled: Hsla,
    /// Secondary / dimmer border
    pub stroke_neutral_dim: Hsla,
    /// Very subtle border (dividers, separators)
    pub stroke_neutral_subtle: Hsla,
    /// Brand / accent border
    pub stroke_accent: Hsla,

    // --- Semantic status colours ---
    /// Informational status accent
    pub status_info: Hsla,
    /// Informational status background
    pub status_info_bg: Hsla,
    /// Informational status border
    pub status_info_border: Hsla,
    /// Success status accent
    pub status_success: Hsla,
    /// Success status background
    pub status_success_bg: Hsla,
    /// Success status border
    pub status_success_border: Hsla,
    /// Warning status accent
    pub status_warning: Hsla,
    /// Warning status background
    pub status_warning_bg: Hsla,
    /// Warning status border
    pub status_warning_border: Hsla,
    /// Error / danger status accent
    pub status_error: Hsla,
    /// Error / danger status background
    pub status_error_bg: Hsla,
    /// Error / danger status border
    pub status_error_border: Hsla,

    // --- Ribbon-specific ---
    /// Ribbon bar background
    pub ribbon_bg: Hsla,
    /// Active-tab fill in the ribbon tab strip
    pub ribbon_tab_active_bg: Hsla,
    /// Active-tab bottom indicator / underline accent
    pub ribbon_tab_indicator: Hsla,
    /// Vertical separator between ribbon groups
    pub ribbon_group_separator: Hsla,

    // --- Layout (dock panels, tab strip) ---
    /// Sidebar / dock panel background
    pub panel_bg: Hsla,
    /// Panel edge border
    pub panel_border: Hsla,
    /// Tab bar background (content area)
    pub tab_strip_bg: Hsla,
    /// Active tab in content tab strip
    pub tab_active_bg: Hsla,
    /// Hovered tab in content tab strip
    pub tab_hover_bg: Hsla,
}

/// Convert an RRGGBB hex literal to `Hsla` via `gpui::rgb`.
fn hex(v: u32) -> Hsla {
    gpui::rgb(v).into()
}

fn states(
    default: u32,
    hover: u32,
    pressed: u32,
    selected: u32,
    disabled: u32,
) -> NeutralColorStates {
    color_states(
        hex(default),
        hex(hover),
        hex(pressed),
        hex(selected),
        hex(disabled),
    )
}

fn color_states(
    default: Hsla,
    hover: Hsla,
    pressed: Hsla,
    selected: Hsla,
    disabled: Hsla,
) -> NeutralColorStates {
    NeutralColorStates {
        default,
        hover,
        pressed,
        selected,
        disabled,
    }
}

impl ColorScheme {
    /// Fluent 2 dark colour scheme.
    pub fn dark() -> Self {
        Self {
            neutral_foreground_1: states(0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x5C5C5C),
            neutral_foreground_2: states(0xDEDEDE, 0xFFFFFF, 0xFFFFFF, 0xFFFFFF, 0x5C5C5C),
            neutral_foreground_3: states(0xADADAD, 0xD6D6D6, 0xD6D6D6, 0xD6D6D6, 0x5C5C5C),
            neutral_background_1: states(0x1C1C1C, 0x3D3D3D, 0x1F1F1F, 0x383838, 0x1C1C1C),
            neutral_background_2: states(0x141414, 0x333333, 0x141414, 0x2E2E2E, 0x1C1C1C),
            neutral_background_3: states(0x272727, 0x303030, 0x0A0A0A, 0x2D2D2D, 0x1C1C1C),
            neutral_stroke_1: states(0x666666, 0x757575, 0x6B6B6B, 0x707070, 0x404040),
            neutral_stroke_2: states(0x525252, 0x525252, 0x525252, 0x525252, 0x404040),
            neutral_stroke_3: states(0x383838, 0x383838, 0x383838, 0x383838, 0x404040),
            // --- Fills ---
            neutral: hex(0x272727),
            neutral_hover: hex(0x303030),
            neutral_disabled: hex(0x1C1C1C),
            neutral_selected: hex(0x2D2D2D),

            // Dark mode uses a muted brand blue (#115EA3)
            accent: hex(0x115EA3),
            accent_hover: hex(0x0F548C),
            accent_disabled: hex(0x404040),
            accent_selected: hex(0x0C3B5E),

            subtle: hex(0x1C1C1C),
            subtle_hover: hex(0x252525),
            subtle_disabled: hex(0x1C1C1C),
            subtle_selected: hex(0x222222),

            surface: hex(0x1C1C1C),
            surface_dim: hex(0x141414),
            surface_blur_layer: hex(0x292929),

            // --- Foregrounds ---
            on_neutral: hex(0xFFFFFF),
            on_neutral_disabled: hex(0x5C5C5C),
            on_neutral_selected: hex(0xFFFFFF),
            on_neutral_accent: hex(0x479EF5), // brand-coloured text (#479EF5)

            on_accent: hex(0xFFFFFF),
            on_accent_disabled: hex(0x9E9E9E),
            on_accent_selected: hex(0xFFFFFF),

            on_subtle: hex(0xDEDEDE),
            on_subtle_disabled: hex(0x5C5C5C),
            on_subtle_selected: hex(0xFFFFFF),

            // --- Strokes ---
            stroke_neutral: hex(0x666666),
            stroke_neutral_disabled: hex(0x404040),
            stroke_neutral_dim: hex(0x525252),
            stroke_neutral_subtle: hex(0x383838),
            stroke_accent: hex(0x479EF5),

            // --- Semantic status ---
            status_info: hex(0x479EF5),
            status_info_bg: hex(0x082338),
            status_info_border: hex(0x115EA3),
            status_success: hex(0x54B054),
            status_success_bg: hex(0x052505),
            status_success_border: hex(0x2A7D2E),
            status_warning: hex(0xFCE100),
            status_warning_bg: hex(0x3A2E00),
            status_warning_border: hex(0x817400),
            status_error: hex(0xF85149),
            status_error_bg: hex(0x3B1010),
            status_error_border: hex(0xA4262C),

            // --- Ribbon ---
            ribbon_bg: hex(0x212121),
            ribbon_tab_active_bg: hex(0x2D2D2D),
            ribbon_tab_indicator: hex(0x479EF5),
            ribbon_group_separator: hex(0x3D3D3D),

            // --- Layout ---
            panel_bg: hex(0x1C1C1C),
            panel_border: hex(0x383838),
            tab_strip_bg: hex(0x212121),
            tab_active_bg: hex(0x2D2D2D),
            tab_hover_bg: hex(0x272727),
        }
    }

    /// Fluent 2 light colour scheme.
    pub fn light() -> Self {
        Self {
            neutral_foreground_1: states(0x242424, 0x242424, 0x242424, 0x242424, 0xBDBDBD),
            neutral_foreground_2: states(0x424242, 0x242424, 0x242424, 0x242424, 0xBDBDBD),
            neutral_foreground_3: states(0x616161, 0x424242, 0x424242, 0x424242, 0xBDBDBD),
            neutral_background_1: states(0xFFFFFF, 0xF5F5F5, 0xE0E0E0, 0xEBEBEB, 0xF5F5F5),
            neutral_background_2: states(0xF0F0F0, 0xE8E8E8, 0xDBDBDB, 0xE3E3E3, 0xF5F5F5),
            neutral_background_3: states(0xF0F0F0, 0xE8E8E8, 0xD6D6D6, 0xE3E3E3, 0xF5F5F5),
            neutral_stroke_1: states(0xD1D1D1, 0xC7C7C7, 0xB3B3B3, 0xBDBDBD, 0xE0E0E0),
            neutral_stroke_2: states(0xC7C7C7, 0xE0E0E0, 0xE0E0E0, 0xE0E0E0, 0xE0E0E0),
            neutral_stroke_3: states(0xEBEBEB, 0xF0F0F0, 0xF0F0F0, 0xF0F0F0, 0xE0E0E0),
            // --- Fills ---
            neutral: hex(0xF0F0F0),
            neutral_hover: hex(0xE8E8E8),
            neutral_disabled: hex(0xF5F5F5),
            neutral_selected: hex(0xE3E3E3),

            // Light mode uses full brand blue (#0078D4)
            accent: hex(0x0078D4),
            accent_hover: hex(0x006CBF),
            accent_disabled: hex(0xD1D1D1),
            accent_selected: hex(0x005AA3),

            subtle: hex(0xFFFFFF),
            subtle_hover: hex(0xF5F5F5),
            subtle_disabled: hex(0xFAFAFA),
            subtle_selected: hex(0xEBEBEB),

            surface: hex(0xFFFFFF),
            surface_dim: hex(0xF0F0F0),
            surface_blur_layer: hex(0xFAFAFA),

            // --- Foregrounds ---
            on_neutral: hex(0x242424),
            on_neutral_disabled: hex(0xBDBDBD),
            on_neutral_selected: hex(0x242424),
            on_neutral_accent: hex(0x0078D4),

            on_accent: hex(0xFFFFFF),
            on_accent_disabled: hex(0x737373),
            on_accent_selected: hex(0xFFFFFF),

            on_subtle: hex(0x424242),
            on_subtle_disabled: hex(0xBDBDBD),
            on_subtle_selected: hex(0x242424),

            // --- Strokes ---
            stroke_neutral: hex(0xD1D1D1),
            stroke_neutral_disabled: hex(0xE0E0E0),
            stroke_neutral_dim: hex(0xC7C7C7),
            stroke_neutral_subtle: hex(0xEBEBEB),
            stroke_accent: hex(0x0078D4),

            // --- Semantic status ---
            status_info: hex(0x005A9E),
            status_info_bg: hex(0xE5F3FF),
            status_info_border: hex(0x0078D4),
            status_success: hex(0x107C10),
            status_success_bg: hex(0xEAF6EA),
            status_success_border: hex(0x107C10),
            status_warning: hex(0x8A3700),
            status_warning_bg: hex(0xFFF4CE),
            status_warning_border: hex(0xF7630C),
            status_error: hex(0xC42B1C),
            status_error_bg: hex(0xFDE7E9),
            status_error_border: hex(0xC42B1C),

            // --- Ribbon ---
            ribbon_bg: hex(0xF3F3F3),
            ribbon_tab_active_bg: hex(0xFFFFFF),
            ribbon_tab_indicator: hex(0x0078D4),
            ribbon_group_separator: hex(0xE0E0E0),

            // --- Layout ---
            panel_bg: hex(0xFAFAFA),
            panel_border: hex(0xE5E5E5),
            tab_strip_bg: hex(0xF3F3F3),
            tab_active_bg: hex(0xFFFFFF),
            tab_hover_bg: hex(0xEBEBEB),
        }
    }

    /// Midnight colour scheme — deep navy darks with a vivid violet accent.
    pub fn midnight() -> Self {
        Self {
            neutral_foreground_1: states(0xC9D8F0, 0xE0ECFF, 0xE0ECFF, 0xE0ECFF, 0x3D4D66),
            neutral_foreground_2: states(0xA8BBDA, 0xC9D8F0, 0xC9D8F0, 0xC9D8F0, 0x3D4D66),
            neutral_foreground_3: states(0x7890B5, 0xA8BBDA, 0xA8BBDA, 0xA8BBDA, 0x3D4D66),
            neutral_background_1: states(0x080E1A, 0x10192B, 0x040810, 0x0F1723, 0x0A1020),
            neutral_background_2: states(0x040810, 0x0F1723, 0x020408, 0x0A1020, 0x0A1020),
            neutral_background_3: states(0x0F1723, 0x1A2540, 0x0A1020, 0x142035, 0x0A1020),
            neutral_stroke_1: states(0x1E2D48, 0x294064, 0x192540, 0x223654, 0x141E33),
            neutral_stroke_2: states(0x192540, 0x192540, 0x192540, 0x192540, 0x141E33),
            neutral_stroke_3: states(0x0F1A30, 0x0F1A30, 0x0F1A30, 0x0F1A30, 0x141E33),
            // --- Fills ---
            neutral: hex(0x0F1723),
            neutral_hover: hex(0x1A2540),
            neutral_disabled: hex(0x0A1020),
            neutral_selected: hex(0x142035),

            accent: hex(0x7C3FFF),
            accent_hover: hex(0x6E32EE),
            accent_disabled: hex(0x261A40),
            accent_selected: hex(0x5A28CC),

            subtle: hex(0x080E1A),
            subtle_hover: hex(0x0F1827),
            subtle_disabled: hex(0x080E1A),
            subtle_selected: hex(0x0C1522),

            surface: hex(0x080E1A),
            surface_dim: hex(0x040810),
            surface_blur_layer: hex(0x10192B),

            // --- Foregrounds ---
            on_neutral: hex(0xC9D8F0),
            on_neutral_disabled: hex(0x3D4D66),
            on_neutral_selected: hex(0xE0ECFF),
            on_neutral_accent: hex(0x9D79F8),

            on_accent: hex(0xFFFFFF),
            on_accent_disabled: hex(0x7A6AAA),
            on_accent_selected: hex(0xFFFFFF),

            on_subtle: hex(0xA8BBDA),
            on_subtle_disabled: hex(0x3D4D66),
            on_subtle_selected: hex(0xC9D8F0),

            // --- Strokes ---
            stroke_neutral: hex(0x1E2D48),
            stroke_neutral_disabled: hex(0x141E33),
            stroke_neutral_dim: hex(0x192540),
            stroke_neutral_subtle: hex(0x0F1A30),
            stroke_accent: hex(0x7C3FFF),

            // --- Semantic status ---
            status_info: hex(0x79B8FF),
            status_info_bg: hex(0x040E1E),
            status_info_border: hex(0x1040A0),
            status_success: hex(0x56D364),
            status_success_bg: hex(0x081808),
            status_success_border: hex(0x155528),
            status_warning: hex(0xE3B341),
            status_warning_bg: hex(0x1E1500),
            status_warning_border: hex(0x604C00),
            status_error: hex(0xFF7B72),
            status_error_bg: hex(0x1E0808),
            status_error_border: hex(0x6B1818),

            // --- Ribbon ---
            ribbon_bg: hex(0x060C16),
            ribbon_tab_active_bg: hex(0x0F1827),
            ribbon_tab_indicator: hex(0x7C3FFF),
            ribbon_group_separator: hex(0x141E33),

            // --- Layout ---
            panel_bg: hex(0x080E1A),
            panel_border: hex(0x141E33),
            tab_strip_bg: hex(0x060C16),
            tab_active_bg: hex(0x0F1827),
            tab_hover_bg: hex(0x0C1522),
        }
    }

    /// Ember colour scheme — warm dark browns with an amber/orange accent.
    pub fn ember() -> Self {
        Self {
            neutral_foreground_1: states(0xF0E4CC, 0xFFF5E0, 0xFFF5E0, 0xFFF5E0, 0x5A4A30),
            neutral_foreground_2: states(0xD4C09A, 0xF0E4CC, 0xF0E4CC, 0xF0E4CC, 0x5A4A30),
            neutral_foreground_3: states(0xA99068, 0xD4C09A, 0xD4C09A, 0xD4C09A, 0x5A4A30),
            neutral_background_1: states(0x181208, 0x231A0E, 0x100D05, 0x1E1508, 0x181208),
            neutral_background_2: states(0x100D05, 0x1E1508, 0x0A0803, 0x181208, 0x181208),
            neutral_background_3: states(0x1E1508, 0x2A1E0C, 0x181208, 0x251A0A, 0x181208),
            neutral_stroke_1: states(0x4A3820, 0x604828, 0x3E2E18, 0x543F24, 0x2E2214),
            neutral_stroke_2: states(0x3E2E18, 0x3E2E18, 0x3E2E18, 0x3E2E18, 0x2E2214),
            neutral_stroke_3: states(0x281E10, 0x281E10, 0x281E10, 0x281E10, 0x2E2214),
            // --- Fills ---
            neutral: hex(0x1E1508),
            neutral_hover: hex(0x2A1E0C),
            neutral_disabled: hex(0x181208),
            neutral_selected: hex(0x251A0A),

            accent: hex(0xA85520),
            accent_hover: hex(0xCB6B2D),
            accent_disabled: hex(0x3A2414),
            accent_selected: hex(0xA85520),

            subtle: hex(0x181208),
            subtle_hover: hex(0x221910),
            subtle_disabled: hex(0x181208),
            subtle_selected: hex(0x1E160C),

            surface: hex(0x181208),
            surface_dim: hex(0x100D05),
            surface_blur_layer: hex(0x231A0E),

            // --- Foregrounds ---
            on_neutral: hex(0xF0E4CC),
            on_neutral_disabled: hex(0x5A4A30),
            on_neutral_selected: hex(0xFFF5E0),
            on_neutral_accent: hex(0xF5A050),

            on_accent: hex(0xFFF8F0),
            on_accent_disabled: hex(0x9B7A50),
            on_accent_selected: hex(0xFFFFFF),

            on_subtle: hex(0xD4C09A),
            on_subtle_disabled: hex(0x5A4A30),
            on_subtle_selected: hex(0xF0E4CC),

            // --- Strokes ---
            stroke_neutral: hex(0x4A3820),
            stroke_neutral_disabled: hex(0x2E2214),
            stroke_neutral_dim: hex(0x3E2E18),
            stroke_neutral_subtle: hex(0x281E10),
            stroke_accent: hex(0xE07B39),

            // --- Semantic status ---
            status_info: hex(0x79B8FF),
            status_info_bg: hex(0x0A1422),
            status_info_border: hex(0x1A3A80),
            status_success: hex(0x5DAD5D),
            status_success_bg: hex(0x0C1A0C),
            status_success_border: hex(0x244824),
            status_warning: hex(0xF5CC5A),
            status_warning_bg: hex(0x221800),
            status_warning_border: hex(0x706000),
            status_error: hex(0xE05550),
            status_error_bg: hex(0x220A08),
            status_error_border: hex(0x7A2018),

            // --- Ribbon ---
            ribbon_bg: hex(0x120E06),
            ribbon_tab_active_bg: hex(0x1E1508),
            ribbon_tab_indicator: hex(0xE07B39),
            ribbon_group_separator: hex(0x2E2214),

            // --- Layout ---
            panel_bg: hex(0x181208),
            panel_border: hex(0x2E2214),
            tab_strip_bg: hex(0x120E06),
            tab_active_bg: hex(0x1E1508),
            tab_hover_bg: hex(0x1A1408),
        }
    }

    /// High-contrast colour scheme, mapped from the Windows system colours
    /// (Canvas, CanvasText, Highlight, HighlightText, GrayText, ButtonFace,
    /// Hyperlink). Fills collapse to Canvas and every boundary is a full-contrast
    /// stroke, so nothing relies on shadows or subtle fill differences.
    pub fn high_contrast() -> Self {
        let canvas = hex(0x000000); // Canvas / ButtonFace
        let text = hex(0xFFFFFF); // CanvasText / ButtonText
        let highlight = hex(0x1AEBFF); // Highlight
        let highlight_text = hex(0x000000); // HighlightText
        let gray_text = hex(0x3FF23F); // GrayText
        let hyperlink = hex(0xFFFF00); // Hyperlink
        let hover = hex(0x262626);
        Self {
            neutral_foreground_1: color_states(text, text, text, highlight_text, gray_text),
            neutral_foreground_2: color_states(text, text, text, highlight_text, gray_text),
            neutral_foreground_3: color_states(text, text, text, highlight_text, gray_text),
            neutral_background_1: color_states(canvas, hover, highlight, highlight, canvas),
            neutral_background_2: color_states(canvas, hover, highlight, highlight, canvas),
            neutral_background_3: color_states(canvas, hover, highlight, highlight, canvas),
            neutral_stroke_1: color_states(text, text, highlight, highlight, gray_text),
            neutral_stroke_2: color_states(text, text, highlight, highlight, gray_text),
            neutral_stroke_3: color_states(text, text, highlight, highlight, gray_text),
            neutral: canvas,
            neutral_hover: hover,
            neutral_disabled: canvas,
            neutral_selected: highlight,

            accent: highlight,
            accent_hover: hyperlink,
            accent_disabled: canvas,
            accent_selected: highlight,

            subtle: canvas,
            subtle_hover: hover,
            subtle_disabled: canvas,
            subtle_selected: highlight,

            surface: canvas,
            surface_dim: canvas,
            surface_blur_layer: canvas,

            on_neutral: text,
            on_neutral_disabled: gray_text,
            on_neutral_selected: highlight_text,
            on_neutral_accent: highlight,

            on_accent: highlight_text,
            on_accent_disabled: gray_text,
            on_accent_selected: highlight_text,

            on_subtle: text,
            on_subtle_disabled: gray_text,
            on_subtle_selected: highlight_text,

            stroke_neutral: text,
            stroke_neutral_disabled: gray_text,
            stroke_neutral_dim: text,
            stroke_neutral_subtle: text,
            stroke_accent: highlight,

            status_info: highlight,
            status_info_bg: canvas,
            status_info_border: highlight,
            status_success: gray_text,
            status_success_bg: canvas,
            status_success_border: gray_text,
            status_warning: hyperlink,
            status_warning_bg: canvas,
            status_warning_border: hyperlink,
            status_error: hex(0xFF6B6B),
            status_error_bg: canvas,
            status_error_border: hex(0xFF6B6B),

            ribbon_bg: canvas,
            ribbon_tab_active_bg: canvas,
            ribbon_tab_indicator: highlight,
            ribbon_group_separator: text,

            panel_bg: canvas,
            panel_border: text,
            tab_strip_bg: canvas,
            tab_active_bg: canvas,
            tab_hover_bg: hover,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ColorScheme;
    use gpui::Hsla;

    fn themes() -> [(&'static str, ColorScheme); 5] {
        [
            ("dark", ColorScheme::dark()),
            ("light", ColorScheme::light()),
            ("midnight", ColorScheme::midnight()),
            ("ember", ColorScheme::ember()),
            ("high contrast", ColorScheme::high_contrast()),
        ]
    }

    fn tokens(colors: &ColorScheme) -> [(&'static str, Hsla); 51] {
        [
            ("neutral", colors.neutral),
            ("neutral_hover", colors.neutral_hover),
            ("neutral_disabled", colors.neutral_disabled),
            ("neutral_selected", colors.neutral_selected),
            ("accent", colors.accent),
            ("accent_hover", colors.accent_hover),
            ("accent_disabled", colors.accent_disabled),
            ("accent_selected", colors.accent_selected),
            ("subtle", colors.subtle),
            ("subtle_hover", colors.subtle_hover),
            ("subtle_disabled", colors.subtle_disabled),
            ("subtle_selected", colors.subtle_selected),
            ("surface", colors.surface),
            ("surface_dim", colors.surface_dim),
            ("surface_blur_layer", colors.surface_blur_layer),
            ("on_neutral", colors.on_neutral),
            ("on_neutral_disabled", colors.on_neutral_disabled),
            ("on_neutral_selected", colors.on_neutral_selected),
            ("on_neutral_accent", colors.on_neutral_accent),
            ("on_accent", colors.on_accent),
            ("on_accent_disabled", colors.on_accent_disabled),
            ("on_accent_selected", colors.on_accent_selected),
            ("on_subtle", colors.on_subtle),
            ("on_subtle_disabled", colors.on_subtle_disabled),
            ("on_subtle_selected", colors.on_subtle_selected),
            ("stroke_neutral", colors.stroke_neutral),
            ("stroke_neutral_disabled", colors.stroke_neutral_disabled),
            ("stroke_neutral_dim", colors.stroke_neutral_dim),
            ("stroke_neutral_subtle", colors.stroke_neutral_subtle),
            ("stroke_accent", colors.stroke_accent),
            ("status_info", colors.status_info),
            ("status_info_bg", colors.status_info_bg),
            ("status_info_border", colors.status_info_border),
            ("status_success", colors.status_success),
            ("status_success_bg", colors.status_success_bg),
            ("status_success_border", colors.status_success_border),
            ("status_warning", colors.status_warning),
            ("status_warning_bg", colors.status_warning_bg),
            ("status_warning_border", colors.status_warning_border),
            ("status_error", colors.status_error),
            ("status_error_bg", colors.status_error_bg),
            ("status_error_border", colors.status_error_border),
            ("ribbon_bg", colors.ribbon_bg),
            ("ribbon_tab_active_bg", colors.ribbon_tab_active_bg),
            ("ribbon_tab_indicator", colors.ribbon_tab_indicator),
            ("ribbon_group_separator", colors.ribbon_group_separator),
            ("panel_bg", colors.panel_bg),
            ("panel_border", colors.panel_border),
            ("tab_strip_bg", colors.tab_strip_bg),
            ("tab_active_bg", colors.tab_active_bg),
            ("tab_hover_bg", colors.tab_hover_bg),
        ]
    }

    fn relative_luminance(color: Hsla) -> f32 {
        let rgb = color.to_rgb();
        [rgb.r, rgb.g, rgb.b]
            .map(|channel| {
                if channel <= 0.04045 {
                    channel / 12.92
                } else {
                    ((channel + 0.055) / 1.055).powf(2.4)
                }
            })
            .iter()
            .zip([0.2126, 0.7152, 0.0722])
            .map(|(channel, weight)| channel * weight)
            .sum()
    }

    fn contrast(foreground: Hsla, background: Hsla) -> f32 {
        let foreground = relative_luminance(foreground);
        let background = relative_luminance(background);
        (foreground.max(background) + 0.05) / (foreground.min(background) + 0.05)
    }

    #[test]
    fn every_theme_sets_every_color_token() {
        for (theme, colors) in themes() {
            for (name, token) in tokens(&colors) {
                assert!(
                    token.h.is_finite()
                        && token.s.is_finite()
                        && token.l.is_finite()
                        && token.a.is_finite(),
                    "{theme} theme token {name} is not initialized"
                );
                assert_eq!(token.a, 1.0, "{theme} theme token {name} is not opaque");
            }
        }
    }

    #[test]
    fn neutral_aliases_cover_every_state() {
        for (theme, colors) in themes() {
            let groups = [
                ("foreground 1", &colors.neutral_foreground_1),
                ("foreground 2", &colors.neutral_foreground_2),
                ("foreground 3", &colors.neutral_foreground_3),
                ("background 1", &colors.neutral_background_1),
                ("background 2", &colors.neutral_background_2),
                ("background 3", &colors.neutral_background_3),
                ("stroke 1", &colors.neutral_stroke_1),
                ("stroke 2", &colors.neutral_stroke_2),
                ("stroke 3", &colors.neutral_stroke_3),
            ];
            for (group, states) in groups {
                for (state, color) in [
                    ("default", states.default),
                    ("hover", states.hover),
                    ("pressed", states.pressed),
                    ("selected", states.selected),
                    ("disabled", states.disabled),
                ] {
                    assert!(
                        color.h.is_finite()
                            && color.s.is_finite()
                            && color.l.is_finite()
                            && color.a.is_finite(),
                        "{theme} {group} {state} is not initialized"
                    );
                }
            }
        }
    }

    #[test]
    fn neutral_alias_defaults_preserve_existing_names() {
        for colors in themes().map(|(_, colors)| colors) {
            assert_eq!(colors.neutral_foreground_1.default, colors.on_neutral);
            assert_eq!(
                colors.neutral_foreground_1.disabled,
                colors.on_neutral_disabled
            );
            assert_eq!(
                colors.neutral_foreground_1.selected,
                colors.on_neutral_selected
            );
            assert_eq!(colors.neutral_foreground_2.default, colors.on_subtle);
            assert_eq!(
                colors.neutral_foreground_2.disabled,
                colors.on_subtle_disabled
            );
            assert_eq!(
                colors.neutral_foreground_2.selected,
                colors.on_subtle_selected
            );
            assert_eq!(colors.neutral_background_1.default, colors.surface);
            assert_eq!(colors.neutral_background_2.default, colors.surface_dim);
            assert_eq!(colors.neutral_background_3.default, colors.neutral);
            assert_eq!(colors.neutral_background_3.hover, colors.neutral_hover);
            assert_eq!(
                colors.neutral_background_3.disabled,
                colors.neutral_disabled
            );
            assert_eq!(
                colors.neutral_background_3.selected,
                colors.neutral_selected
            );
            assert_eq!(colors.neutral_stroke_1.default, colors.stroke_neutral);
            assert_eq!(
                colors.neutral_stroke_1.disabled,
                colors.stroke_neutral_disabled
            );
            assert_eq!(colors.neutral_stroke_2.default, colors.stroke_neutral_dim);
            assert_eq!(
                colors.neutral_stroke_3.default,
                colors.stroke_neutral_subtle
            );
        }
    }

    #[test]
    fn required_foreground_pairs_meet_wcag_contrast() {
        for (theme, colors) in themes() {
            let pairs = [
                ("text on surface", colors.on_neutral, colors.surface),
                (
                    "brand text on surface",
                    colors.on_neutral_accent,
                    colors.surface,
                ),
                ("text on brand", colors.on_accent, colors.accent),
                ("secondary text on subtle", colors.on_subtle, colors.subtle),
                ("info status", colors.status_info, colors.status_info_bg),
                (
                    "success status",
                    colors.status_success,
                    colors.status_success_bg,
                ),
                (
                    "warning status",
                    colors.status_warning,
                    colors.status_warning_bg,
                ),
                ("error status", colors.status_error, colors.status_error_bg),
            ];
            for (name, foreground, background) in pairs {
                assert!(
                    contrast(foreground, background) >= 4.5,
                    "{theme} {name} contrast is {:.2}:1, expected at least 4.5:1",
                    contrast(foreground, background)
                );
            }
        }
    }

    // Disabled foregrounds are intentionally muted to communicate disabled state;
    // they are excluded from the normal-text WCAG requirement above.

    #[test]
    fn high_contrast_pairs_meet_wcag_aaa() {
        let c = ColorScheme::high_contrast();
        let pairs = [
            ("text on surface", c.on_neutral, c.surface),
            ("text on neutral", c.on_neutral, c.neutral),
            ("text on hover", c.on_neutral, c.neutral_hover),
            (
                "text on selected",
                c.on_neutral_selected,
                c.neutral_selected,
            ),
            ("text on accent", c.on_accent, c.accent),
            ("text on accent hover", c.on_accent, c.accent_hover),
            ("disabled text on surface", c.on_neutral_disabled, c.surface),
            ("stroke on surface", c.stroke_neutral, c.surface),
            ("accent stroke on surface", c.stroke_accent, c.surface),
            ("panel border on panel", c.panel_border, c.panel_bg),
        ];
        for (name, fg, bg) in pairs {
            assert!(
                contrast(fg, bg) >= 7.0,
                "high contrast {name} is {:.2}:1, expected at least 7:1",
                contrast(fg, bg)
            );
        }
    }
}
