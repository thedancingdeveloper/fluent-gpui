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
        cx.set_global(Theme::dark());
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
