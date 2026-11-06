use std::sync::Arc;

use fluent_core::{TextRampExt as _, Theme, ThemeMode, ThemeProvider as _};
use fluent_layout::{modal::ModalStack, ToastStack};
use gpui::{
    div, point, prelude::*, px, size, App, AssetSource, Bounds, Context, Entity, IntoElement,
    Render, SharedString, TitlebarOptions, Window, WindowBounds, WindowDecorations, WindowOptions,
};

use crate::{
    assets::FluentAssets, chrome::wrap_with_chrome, title_bar::TitleBar, window_title::WindowTitle,
};

const DEFAULT_W: f32 = 1280.0;
const DEFAULT_H: f32 = 800.0;

type WindowShouldCloseHandler = Box<dyn Fn(&mut Window, &mut App) -> bool>;

struct ThemeRoot<V: Render + 'static> {
    content: Entity<V>,
    last_window_title: SharedString,
}

impl<V: Render + 'static> ThemeRoot<V> {
    fn new(cx: &mut Context<Self>, content: Entity<V>) -> Self {
        let themed_content = content.clone();
        cx.observe_global::<Theme>(move |_, cx| {
            themed_content.update(cx, |_, cx| cx.notify());
            cx.notify();
        })
        .detach();

        cx.observe_global::<WindowTitle>(|_, cx| cx.notify())
            .detach();

        let last_window_title = cx
            .try_global::<WindowTitle>()
            .map(|title| title.title().clone())
            .unwrap_or_default();

        Self {
            content,
            last_window_title,
        }
    }
}

impl<V: Render + 'static> Render for ThemeRoot<V> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(title) = cx.try_global::<WindowTitle>() {
            let next_title = title.title().clone();
            if self.last_window_title != next_title {
                window.set_window_title(next_title.as_ref());
                self.last_window_title = next_title;
            }
        }

        // Root text style: body ramp with a whole-pixel line height, so text that
        // sets no line height of its own does not inherit gpui's 1.618x default.
        let body = cx.theme().typography.body;
        div()
            .size_full()
            .text_ramp(&body)
            .child(self.content.clone())
    }
}

/// Builder for a FluentGUI application.
///
/// ```ignore
/// FluentApp::new("MyApp")
///     .window_size(1440.0, 900.0)
///     .run(|cx| {
///         cx.new(|_| Workspace::new()...)
///     });
/// ```
pub struct FluentApp {
    title: SharedString,
    window_w: f32,
    window_h: f32,
    window_min_size: Option<(f32, f32)>,
    window_position: Option<(f32, f32)>,
    app_id: Option<String>,
    window_status: Option<SharedString>,
    on_window_should_close: Option<WindowShouldCloseHandler>,
    theme_mode: ThemeMode,
    assets: Option<Arc<dyn AssetSource>>,
}

impl FluentApp {
    /// Public API item.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            window_w: DEFAULT_W,
            window_h: DEFAULT_H,
            window_min_size: None,
            window_position: None,
            app_id: None,
            window_status: None,
            on_window_should_close: None,
            theme_mode: ThemeMode::Dark,
            assets: None,
        }
    }

    /// Public API item.
    pub fn window_size(mut self, w: f32, h: f32) -> Self {
        self.window_w = w;
        self.window_h = h;
        self
    }

    /// Public API item.
    pub fn window_min_size(mut self, w: f32, h: f32) -> Self {
        self.window_min_size = Some((w, h));
        self
    }

    /// Open the window with its top-left corner at `(x, y)` in screen
    /// coordinates (logical pixels), instead of centred.
    ///
    /// The window is clamped to the display it overlaps most so it cannot
    /// open off-screen; if it overlaps no connected display (for example a
    /// saved position from a monitor that is gone) it is centred as usual.
    /// Pair with [`window_bounds`] to save the position on close.
    pub fn window_position(mut self, x: f32, y: f32) -> Self {
        self.window_position = Some((x, y));
        self
    }

    /// Set the desktop application id used by platforms that group windows.
    pub fn app_id(mut self, app_id: impl Into<String>) -> Self {
        self.app_id = Some(app_id.into());
        self
    }

    /// Set the initial generic status associated with the window title.
    pub fn window_status(mut self, status: impl Into<SharedString>) -> Self {
        self.window_status = Some(status.into());
        self
    }

    /// Register a generic close guard for app lifecycle integrations.
    ///
    /// Return `false` to prevent the platform close request.
    pub fn on_window_should_close(
        mut self,
        handler: impl Fn(&mut Window, &mut App) -> bool + 'static,
    ) -> Self {
        self.on_window_should_close = Some(Box::new(handler));
        self
    }

    /// Public API item.
    pub fn dark_theme(mut self) -> Self {
        self.theme_mode = ThemeMode::Dark;
        self
    }

    /// Public API item.
    pub fn light_theme(mut self) -> Self {
        self.theme_mode = ThemeMode::Light;
        self
    }

    /// Follow the operating system's light or dark preference.
    pub fn system_theme(mut self) -> Self {
        self.theme_mode = ThemeMode::System;
        self
    }

    /// Public API item.
    pub fn assets(mut self, assets: impl AssetSource) -> Self {
        self.assets = Some(Arc::new(assets));
        self
    }

    /// Launch the application.
    ///
    /// The `build` closure runs on the main thread with `&mut App` and must
    /// return the root `Entity<V>` to display as the window's content.
    pub fn run<V: Render + 'static>(self, build: impl FnOnce(&mut App) -> Entity<V> + 'static) {
        let title = self.title.clone();
        let w = self.window_w;
        let h = self.window_h;
        let min_size = self.window_min_size;
        let position = self.window_position;
        let app_id = self.app_id;
        let window_status = self.window_status;
        let on_window_should_close = self.on_window_should_close;
        let theme_mode = self.theme_mode;
        let assets = self.assets;

        gpui_platform::application()
            .with_assets(FluentAssets::new(assets))
            .run(move |cx: &mut App| {
                let apply_system_theme = |cx: &mut App, appearance| {
                    match ThemeMode::from_window_appearance(appearance) {
                        ThemeMode::Dark => Theme::apply_dark(cx),
                        ThemeMode::Light => Theme::apply_light(cx),
                        ThemeMode::System => unreachable!(),
                    }
                };
                match theme_mode {
                    ThemeMode::Dark => Theme::apply_dark(cx),
                    ThemeMode::Light => Theme::apply_light(cx),
                    ThemeMode::System => {
                        if cx.high_contrast() {
                            Theme::apply_high_contrast(cx);
                        } else {
                            apply_system_theme(cx, cx.window_appearance());
                        }
                        cx.on_high_contrast_change(move |cx| {
                            if cx.high_contrast() {
                                Theme::apply_high_contrast(cx);
                            } else {
                                apply_system_theme(cx, cx.window_appearance());
                            }
                        })
                        .detach();
                    }
                }
                ModalStack::init(cx);
                ToastStack::init(cx);
                let mut window_title = WindowTitle::new(title.clone());
                if let Some(status) = window_status.clone() {
                    window_title = window_title.with_status(status);
                }
                cx.set_global(window_title);

                let mut bounds = Bounds::centered(None, size(px(w), px(h)), cx);
                if let Some((x, y)) = position {
                    let displays: Vec<_> = cx
                        .displays()
                        .iter()
                        .map(|d| {
                            let b = d.bounds();
                            (
                                f32::from(b.origin.x),
                                f32::from(b.origin.y),
                                f32::from(b.size.width),
                                f32::from(b.size.height),
                            )
                        })
                        .collect();
                    if let Some((x, y)) = clamp_position((x, y, w, h), &displays) {
                        bounds.origin = point(px(x), px(y));
                    }
                }

                cx.open_window(
                    WindowOptions {
                        window_bounds: Some(WindowBounds::Windowed(bounds)),
                        window_min_size: min_size.map(|(w, h)| size(px(w), px(h))),
                        app_id: app_id.clone(),
                        titlebar: Some(TitlebarOptions {
                            title: Some(title.clone()),
                            appears_transparent: true,
                            traffic_light_position: None,
                        }),
                        window_decorations: Some(WindowDecorations::Client),
                        ..Default::default()
                    },
                    move |window, cx: &mut App| {
                        if theme_mode == ThemeMode::System {
                            window
                                .observe_window_appearance(move |window, cx| {
                                    apply_system_theme(cx, window.appearance());
                                })
                                .detach();
                        }
                        if let Some(app_id) = app_id.as_deref() {
                            window.set_app_id(app_id);
                        }
                        if let Some(handler) = on_window_should_close {
                            window
                                .on_window_should_close(cx, move |window, cx| handler(window, cx));
                        }
                        let root = build(cx);
                        let chromed = wrap_with_chrome(root, cx);
                        cx.new(|cx| ThemeRoot::new(cx, chromed))
                    },
                )
                .unwrap();

                cx.activate(true);
            });
    }
}

/// Current window bounds as `(x, y, width, height)` in screen coordinates
/// (logical pixels).
///
/// Call it from a close handler (see [`FluentApp::on_window_should_close`])
/// to save the geometry, then feed `x`/`y` back to
/// [`FluentApp::window_position`] and `width`/`height` to
/// [`FluentApp::window_size`] on the next launch. When the window is
/// maximised or fullscreen this returns its restore bounds, so the saved
/// geometry is the normal windowed one.
pub fn window_bounds(window: &Window) -> (f32, f32, f32, f32) {
    let b = match window.window_bounds() {
        WindowBounds::Windowed(b) => b,
        WindowBounds::Maximized(b) | WindowBounds::Fullscreen(b) => b,
    };
    (
        f32::from(b.origin.x),
        f32::from(b.origin.y),
        f32::from(b.size.width),
        f32::from(b.size.height),
    )
}

/// Clamp a `(x, y, w, h)` window rectangle into the display it overlaps most.
/// Displays are `(x, y, w, h)`. Returns the adjusted origin, or `None` when
/// the rectangle overlaps no display.
fn clamp_position(
    (x, y, w, h): (f32, f32, f32, f32),
    displays: &[(f32, f32, f32, f32)],
) -> Option<(f32, f32)> {
    let overlap = |d: &(f32, f32, f32, f32)| {
        let ow = (x + w).min(d.0 + d.2) - x.max(d.0);
        let oh = (y + h).min(d.1 + d.3) - y.max(d.1);
        if ow > 0.0 && oh > 0.0 {
            ow * oh
        } else {
            0.0
        }
    };
    let best = displays
        .iter()
        .map(|d| (overlap(d), d))
        .filter(|(a, _)| *a > 0.0)
        .max_by(|a, b| a.0.total_cmp(&b.0))?
        .1;
    let max_x = (best.0 + best.2 - w).max(best.0);
    let max_y = (best.1 + best.3 - h).max(best.1);
    Some((x.clamp(best.0, max_x), y.clamp(best.1, max_y)))
}

/// Create a `TitleBar` entity — include as the first child of your `Workspace`.
pub fn title_bar(title: impl Into<SharedString>, cx: &mut App) -> Entity<TitleBar> {
    let t = title.into();
    cx.new(|cx| TitleBar::new(cx, t))
}

/// Create a `TitleBar` entity whose title/status follow the global
/// [`WindowTitle`] state.
pub fn window_title_bar(cx: &mut App) -> Entity<TitleBar> {
    cx.new(TitleBar::from_window_title)
}

#[cfg(test)]
mod tests {
    use super::{clamp_position, FluentApp};

    const DISPLAY: (f32, f32, f32, f32) = (0.0, 0.0, 1920.0, 1080.0);

    #[test]
    fn builder_stores_window_position() {
        let app = FluentApp::new("App").window_position(40.0, 50.0);

        assert_eq!(app.window_position, Some((40.0, 50.0)));
    }

    #[test]
    fn clamp_keeps_on_screen_position() {
        let r = clamp_position((100.0, 200.0, 800.0, 600.0), &[DISPLAY]);
        assert_eq!(r, Some((100.0, 200.0)));
    }

    #[test]
    fn clamp_pulls_partly_off_screen_window_back() {
        let r = clamp_position((1500.0, -50.0, 800.0, 600.0), &[DISPLAY]);
        assert_eq!(r, Some((1120.0, 0.0)));
    }

    #[test]
    fn clamp_rejects_position_on_missing_display() {
        let r = clamp_position((5000.0, 100.0, 800.0, 600.0), &[DISPLAY]);
        assert_eq!(r, None);
        assert_eq!(clamp_position((0.0, 0.0, 10.0, 10.0), &[]), None);
    }

    #[test]
    fn clamp_picks_display_with_most_overlap() {
        let right = (1920.0, 0.0, 1920.0, 1080.0);
        let r = clamp_position((1800.0, 100.0, 800.0, 600.0), &[DISPLAY, right]);
        assert_eq!(r, Some((1920.0, 100.0)));
    }

    #[test]
    fn builder_stores_window_min_size() {
        let app = FluentApp::new("App").window_min_size(800.0, 600.0);

        assert_eq!(app.window_min_size, Some((800.0, 600.0)));
    }

    #[test]
    fn builder_stores_app_id_and_status() {
        let app = FluentApp::new("App")
            .app_id("org.example.App")
            .window_status("Ready");

        assert_eq!(app.app_id.as_deref(), Some("org.example.App"));
        assert_eq!(
            app.window_status.as_ref().map(|s| s.as_ref()),
            Some("Ready")
        );
    }
}
