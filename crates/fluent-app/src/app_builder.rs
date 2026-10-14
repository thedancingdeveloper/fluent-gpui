use std::sync::Arc;

use fluent_core::Theme;
use fluent_layout::{modal::ModalStack, ToastStack};
use gpui::{
    prelude::*, px, size, App, AssetSource, Bounds, Context, Entity, IntoElement, Render,
    SharedString, TitlebarOptions, Window, WindowBounds, WindowDecorations, WindowOptions,
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

        self.content.clone()
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
    app_id: Option<String>,
    window_status: Option<SharedString>,
    on_window_should_close: Option<WindowShouldCloseHandler>,
    dark: bool,
    assets: Option<Arc<dyn AssetSource>>,
}

impl FluentApp {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            window_w: DEFAULT_W,
            window_h: DEFAULT_H,
            window_min_size: None,
            app_id: None,
            window_status: None,
            on_window_should_close: None,
            dark: true,
            assets: None,
        }
    }

    pub fn window_size(mut self, w: f32, h: f32) -> Self {
        self.window_w = w;
        self.window_h = h;
        self
    }

    pub fn window_min_size(mut self, w: f32, h: f32) -> Self {
        self.window_min_size = Some((w, h));
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

    pub fn dark_theme(mut self) -> Self {
        self.dark = true;
        self
    }

    pub fn light_theme(mut self) -> Self {
        self.dark = false;
        self
    }

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
        let app_id = self.app_id;
        let window_status = self.window_status;
        let on_window_should_close = self.on_window_should_close;
        let dark = self.dark;
        let assets = self.assets;

        gpui_platform::application()
            .with_assets(FluentAssets::new(assets))
            .run(move |cx: &mut App| {
                if dark {
                    Theme::init(cx);
                } else {
                    cx.set_global(Theme::light());
                }
                ModalStack::init(cx);
                ToastStack::init(cx);
                let mut window_title = WindowTitle::new(title.clone());
                if let Some(status) = window_status.clone() {
                    window_title = window_title.with_status(status);
                }
                cx.set_global(window_title);

                let bounds = Bounds::centered(None, size(px(w), px(h)), cx);

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
    use super::FluentApp;

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
