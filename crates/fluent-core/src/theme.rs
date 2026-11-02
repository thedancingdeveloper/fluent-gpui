use gpui::{App, Global};

use crate::{
    ColorScheme, ComponentTokens, MotionTokens, RadiiTokens, SpacingTokens, TypographyTokens,
};

/// Whether the theme is rendered in dark or light mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Brightness {
    Dark,
    Light,
}

/// The reading direction used by the application layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LayoutDirection {
    #[default]
    Ltr,
    Rtl,
}

impl Global for LayoutDirection {}

impl LayoutDirection {
    /// Register left-to-right layout as the default application direction.
    pub fn init(cx: &mut App) {
        cx.set_global(Self::default());
    }

    /// Whether directional visuals should be mirrored.
    pub fn is_rtl(self) -> bool {
        self == Self::Rtl
    }

    /// The active direction. This is the single entry point components use to
    /// read the direction; it falls back to left-to-right when none is registered.
    pub fn current(cx: &App) -> Self {
        cx.layout_direction()
    }

    /// Set the active direction and notify global observers.
    pub fn set(cx: &mut App, direction: Self) {
        cx.set_global(direction);
    }

    /// Switch between left-to-right and right-to-left.
    pub fn toggle(cx: &mut App) {
        let next = if Self::current(cx).is_rtl() {
            Self::Ltr
        } else {
            Self::Rtl
        };
        Self::set(cx, next);
    }

    /// Pick the physical value for a logical start/end pair: `start` for LTR,
    /// `end` for RTL. Use it for icons, alignment and spacing sides. Text is
    /// never mirrored; only layout and directional glyphs are.
    pub fn pick<T>(self, ltr: T, rtl: T) -> T {
        if self.is_rtl() {
            rtl
        } else {
            ltr
        }
    }
}

/// Controls how an application chooses its active theme.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ThemeMode {
    /// Use the operating system's current light or dark preference.
    System,
    /// Always use the dark theme.
    #[default]
    Dark,
    /// Always use the light theme.
    Light,
}

impl ThemeMode {
    pub fn from_window_appearance(appearance: gpui::WindowAppearance) -> Self {
        match appearance {
            gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark => Self::Dark,
            gpui::WindowAppearance::Light | gpui::WindowAppearance::VibrantLight => Self::Light,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ThemeMode;
    use gpui::WindowAppearance;

    #[test]
    fn maps_window_appearances_to_theme_modes() {
        assert_eq!(
            ThemeMode::from_window_appearance(WindowAppearance::Dark),
            ThemeMode::Dark
        );
        assert_eq!(
            ThemeMode::from_window_appearance(WindowAppearance::VibrantDark),
            ThemeMode::Dark
        );
        assert_eq!(
            ThemeMode::from_window_appearance(WindowAppearance::Light),
            ThemeMode::Light
        );
        assert_eq!(
            ThemeMode::from_window_appearance(WindowAppearance::VibrantLight),
            ThemeMode::Light
        );
    }
}

/// The top-level theme object stored as a GPUI global.
///
/// Access it from any context via the [`ThemeProvider`] trait extension:
/// ```ignore
/// use fluent_core::ThemeProvider as _;
/// let accent = cx.theme().colors.accent;
/// ```
#[derive(Clone, Debug)]
pub struct Theme {
    pub brightness: Brightness,
    pub colors: ColorScheme,
    pub spacing: SpacingTokens,
    pub radii: RadiiTokens,
    pub typography: TypographyTokens,
    pub components: ComponentTokens,
}

impl Global for Theme {}

impl Theme {
    /// Construct the default dark theme.
    pub fn dark() -> Self {
        Self {
            brightness: Brightness::Dark,
            colors: ColorScheme::dark(),
            spacing: SpacingTokens::default(),
            radii: RadiiTokens::default(),
            typography: TypographyTokens::default(),
            components: ComponentTokens::default(),
        }
    }

    /// Construct the default light theme.
    pub fn light() -> Self {
        Self {
            brightness: Brightness::Light,
            colors: ColorScheme::light(),
            spacing: SpacingTokens::default(),
            radii: RadiiTokens::default(),
            typography: TypographyTokens::default(),
            components: ComponentTokens::default(),
        }
    }

    pub fn is_dark(&self) -> bool {
        self.brightness == Brightness::Dark
    }

    pub fn motion(&self) -> MotionTokens {
        MotionTokens::default()
    }

    /// Register the dark theme as the active global. Call once during app startup.
    pub fn init(cx: &mut App) {
        LayoutDirection::init(cx);
        if cx.high_contrast() {
            cx.set_global(Theme::high_contrast());
        } else {
            cx.set_global(Theme::dark());
        }
    }

    /// Construct the Midnight theme (deep navy darks + violet accent).
    pub fn midnight() -> Self {
        Self {
            brightness: Brightness::Dark,
            colors: ColorScheme::midnight(),
            spacing: SpacingTokens::default(),
            radii: RadiiTokens::default(),
            typography: TypographyTokens::default(),
            components: ComponentTokens::default(),
        }
    }

    /// Construct the Ember theme (warm dark browns + amber accent).
    pub fn ember() -> Self {
        Self {
            brightness: Brightness::Dark,
            colors: ColorScheme::ember(),
            spacing: SpacingTokens::default(),
            radii: RadiiTokens::default(),
            typography: TypographyTokens::default(),
            components: ComponentTokens::default(),
        }
    }

    /// Construct the high-contrast theme (Fluent 2 / Windows system-colour mapping:
    /// Canvas, CanvasText, Highlight, GrayText, ButtonFace). Strokes replace fills.
    pub fn high_contrast() -> Self {
        Self {
            brightness: Brightness::Dark,
            colors: ColorScheme::high_contrast(),
            spacing: SpacingTokens::default(),
            radii: RadiiTokens::default(),
            typography: TypographyTokens::default(),
            components: ComponentTokens::default(),
        }
    }

    /// Switch to dark theme at runtime. All subscribed views re-render automatically.
    pub fn apply_dark(cx: &mut App) {
        cx.set_global(Theme::dark());
    }

    /// Switch to light theme at runtime. All subscribed views re-render automatically.
    pub fn apply_light(cx: &mut App) {
        cx.set_global(Theme::light());
    }

    /// Switch to Midnight theme at runtime.
    pub fn apply_midnight(cx: &mut App) {
        cx.set_global(Theme::midnight());
    }

    /// Switch to Ember theme at runtime.
    pub fn apply_ember(cx: &mut App) {
        cx.set_global(Theme::ember());
    }

    /// Switch to the high-contrast theme at runtime.
    pub fn apply_high_contrast(cx: &mut App) {
        cx.set_global(Theme::high_contrast());
    }

    /// Toggle between dark and light.
    pub fn toggle(cx: &mut App) {
        let is_dark = cx.global::<Theme>().is_dark();
        if is_dark {
            Self::apply_light(cx);
        } else {
            Self::apply_dark(cx);
        }
    }
}

/// Provides access to the active [`Theme`] from any GPUI context.
///
/// Implemented for `App`. Because `Context<T>: Deref<Target = App>`, calling
/// `cx.theme()` on any context type works via Rust's auto-deref — no separate
/// blanket impl is needed (and none is provided to avoid coherence issues).
pub trait ThemeProvider {
    fn theme(&self) -> &Theme;
}

impl ThemeProvider for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

/// Provides access to the active [`LayoutDirection`] from any GPUI context.
///
/// Prefer [`LayoutDirection::current`] in components; it is the documented entry point.
pub trait LayoutDirectionProvider {
    fn layout_direction(&self) -> LayoutDirection;
}

impl LayoutDirectionProvider for App {
    fn layout_direction(&self) -> LayoutDirection {
        self.try_global::<LayoutDirection>()
            .copied()
            .unwrap_or_default()
    }
}
