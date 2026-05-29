use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, svg, ClickEvent, Context, IntoElement, MouseButton, MouseDownEvent,
    Render, SharedString, Window,
};

use crate::window_title::WindowTitle;

/// Pixel threshold below which two left-button presses on the title bar are
/// treated as a double-click that maximizes / restores the window.
const DOUBLE_CLICK_WINDOW_MS: u64 = 350;

/// A custom frameless title bar that replaces the OS-provided one.
///
/// Draggable via `window.start_window_move()`. Renders the app title and
/// window controls (min/max/close) on the right. Enable frameless mode with
/// `TitlebarOptions { appears_transparent: true }` + `WindowDecorations::Client`.
pub struct TitleBar {
    pub title: SharedString,
    pub icon: Option<SharedString>,
    pub show_controls: bool,
    follow_window_title: bool,
    last_press_ms: u64,
}

impl TitleBar {
    pub fn new(cx: &mut Context<Self>, title: impl Into<SharedString>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        cx.observe_global::<WindowTitle>(|_, cx| cx.notify())
            .detach();
        Self {
            title: title.into(),
            icon: None,
            show_controls: true,
            follow_window_title: false,
            last_press_ms: 0,
        }
    }

    pub fn from_window_title(cx: &mut Context<Self>) -> Self {
        let title = cx
            .try_global::<WindowTitle>()
            .map(|title| title.title().clone())
            .unwrap_or_default();

        Self::new(cx, title).follow_window_title(true)
    }

    pub fn set_title(&mut self, title: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.title = title.into();
        self.follow_window_title = false;
        cx.notify();
    }

    pub fn show_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    pub fn follow_window_title(mut self, follow: bool) -> Self {
        self.follow_window_title = follow;
        self
    }

    /// Set a leading 16×16 SVG icon (typically the app logo) shown left of
    /// the title. Renders in `on_neutral_accent`.
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }
}

impl Render for TitleBar {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;

        let (title, status) = if self.follow_window_title {
            cx.try_global::<WindowTitle>()
                .map(|window_title| (window_title.title().clone(), window_title.status().cloned()))
                .unwrap_or_else(|| (self.title.clone(), None))
        } else {
            (self.title.clone(), None)
        };
        let show_controls = self.show_controls;

        let bar_bg = colors.surface_dim;
        let fg = colors.on_neutral;
        let ctrl_hover = colors.subtle_hover;
        let close_hover: gpui::Hsla = gpui::rgb(0xC42B1C).into();

        // Drag: left-button press starts window move; consecutive presses
        // within DOUBLE_CLICK_WINDOW_MS toggle maximize / restore instead.
        let drag_handler = cx.listener(
            |this: &mut TitleBar, _: &MouseDownEvent, window: &mut Window, _| {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|d| d.as_millis() as u64)
                    .unwrap_or(0);
                if now.saturating_sub(this.last_press_ms) < DOUBLE_CLICK_WINDOW_MS {
                    this.last_press_ms = 0;
                    window.zoom_window();
                    return;
                }
                this.last_press_ms = now;
                window.start_window_move();
            },
        );

        let min_handler =
            cx.listener(|_: &mut TitleBar, _: &ClickEvent, window: &mut Window, _| {
                window.minimize_window();
            });
        let max_handler =
            cx.listener(|_: &mut TitleBar, _: &ClickEvent, window: &mut Window, _| {
                window.zoom_window();
            });
        let close_handler =
            cx.listener(|_: &mut TitleBar, _: &ClickEvent, window: &mut Window, _| {
                window.remove_window();
            });

        // The drag area is only the title text, NOT the window control buttons.
        // Putting on_mouse_down on the full bar would intercept clicks on min/max/close.
        let icon = self.icon.clone();
        let accent_fg = colors.on_neutral_accent;
        let mut title_area = div()
            .flex_1()
            .h_full()
            .flex()
            .items_center()
            .gap(px(spacing.sm))
            .pl(px(spacing.md))
            .text_size(px(typography.caption.size))
            .text_color(fg)
            .on_mouse_down(MouseButton::Left, drag_handler);
        if let Some(path) = icon {
            title_area = title_area.child(svg().path(path).size(px(16.0)).text_color(accent_fg));
        }
        let mut title_area = title_area.child(title);
        if let Some(status) = status {
            title_area = title_area.child(
                div()
                    .text_size(px(typography.caption.size))
                    .text_color(colors.on_subtle)
                    .child(status),
            );
        }

        let bar = div()
            .flex()
            .flex_row()
            .h(px(36.0))
            .bg(bar_bg)
            .child(title_area);

        if !show_controls {
            return bar;
        }

        // Controls container fills full bar height so hover fills the entire button box.
        bar.child(
            div()
                .flex()
                .flex_row()
                .h_full()
                .child(
                    div()
                        .id("titlebar-min")
                        .w(px(46.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(ctrl_hover))
                        .on_click(min_handler)
                        .child(
                            svg()
                                .path("icons/minimize.svg")
                                .size(px(10.0))
                                .text_color(fg),
                        ),
                )
                .child(
                    div()
                        .id("titlebar-max")
                        .w(px(46.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(ctrl_hover))
                        .on_click(max_handler)
                        .child(
                            svg()
                                .path("icons/maximize.svg")
                                .size(px(10.0))
                                .text_color(fg),
                        ),
                )
                .child(
                    div()
                        .id("titlebar-close")
                        .w(px(46.0))
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(close_hover))
                        .on_click(close_handler)
                        .child(
                            svg()
                                .path("icons/dismiss.svg")
                                .size(px(10.0))
                                .text_color(fg),
                        ),
                ),
        )
    }
}
