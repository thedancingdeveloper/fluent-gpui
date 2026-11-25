use fluent_core::{LayoutDirection, TextRampExt as _, Theme, ThemeProvider as _, TruncateExt as _};
use gpui::{
    div, prelude::*, px, svg, ClickEvent, Context, FocusHandle, IntoElement, MouseButton,
    MouseDownEvent, Render, Role, SharedString, Window,
};

use crate::window_title::WindowTitle;
use fluent_primitives::{focus_ring, is_keyboard_focused, FluentFocus as _};

/// Pixel threshold below which two left-button presses on the title bar are
/// treated as a double-click that maximizes / restores the window.
const DOUBLE_CLICK_WINDOW_MS: u64 = 350;

/// A custom frameless title bar that replaces the OS-provided one.
///
/// Draggable via `window.start_window_move()`. Renders the app title and
/// window controls (min/max/close) on the right. Enable frameless mode with
/// `TitlebarOptions { appears_transparent: true }` + `WindowDecorations::Client`.
pub struct TitleBar {
    /// Public API item.
    pub title: SharedString,
    /// Public API item.
    pub icon: Option<SharedString>,
    /// Public API item.
    pub show_controls: bool,
    follow_window_title: bool,
    last_press_ms: u64,
    /// Focus handles of the minimize, maximize and close buttons, in that order.
    caption_focus: [FocusHandle; 3],
}

impl TitleBar {
    /// Public API item.
    pub fn new(cx: &mut Context<Self>, title: impl Into<SharedString>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        cx.observe_global::<LayoutDirection>(|_, cx| cx.notify())
            .detach();
        cx.observe_global::<WindowTitle>(|_, cx| cx.notify())
            .detach();
        Self {
            title: title.into(),
            icon: None,
            show_controls: true,
            follow_window_title: false,
            last_press_ms: 0,
            caption_focus: [cx.focus_handle(), cx.focus_handle(), cx.focus_handle()],
        }
    }

    /// Public API item.
    pub fn from_window_title(cx: &mut Context<Self>) -> Self {
        let title = cx
            .try_global::<WindowTitle>()
            .map(|title| title.title().clone())
            .unwrap_or_default();

        Self::new(cx, title).follow_window_title(true)
    }

    /// Public API item.
    pub fn set_title(&mut self, title: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.title = title.into();
        self.follow_window_title = false;
        cx.notify();
    }

    /// Public API item.
    pub fn show_controls(mut self, show: bool) -> Self {
        self.show_controls = show;
        self
    }

    /// Public API item.
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
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let components = theme.components;
        let rtl = LayoutDirection::current(cx).is_rtl();

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
        let close_hover = colors.status_error;
        let ctrl_pressed = colors.subtle_selected;
        let close_pressed = colors.status_error.opacity(0.8);

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

        // Each caption button is its own Tab stop. A tracked handle carries its
        // own tab settings; Enter/Space arrive as gpui's keyboard click.
        let [min_focus, max_focus, close_focus] = self
            .caption_focus
            .clone()
            .map(|handle| handle.tab_index(0).tab_stop(true));
        let ring = |handle: &FocusHandle| is_keyboard_focused(handle, window);
        let (min_ring, max_ring, close_ring) =
            (ring(&min_focus), ring(&max_focus), ring(&close_focus));
        let ring_radius = px(0.0);

        // The drag area is only the title text, NOT the window control buttons.
        // Putting on_mouse_down on the full bar would intercept clicks on min/max/close.
        let icon = self.icon.clone();
        let accent_fg = colors.on_neutral_accent;
        let mut title_area = div()
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .items_center()
            .gap(px(spacing.sm))
            .when(!rtl, |area| area.pl(px(spacing.md)))
            .when(rtl, |area| area.pr(px(spacing.md)))
            .text_ramp(&typography.caption)
            .text_color(fg)
            .on_mouse_down(MouseButton::Left, drag_handler);
        if let Some(path) = icon {
            title_area = title_area.child(
                svg()
                    .path(path)
                    .size(px(components.title_bar_icon_size))
                    .text_color(accent_fg),
            );
        }
        let mut title_area = title_area.child(div().truncate_label().child(title));
        if let Some(status) = status {
            title_area = title_area.child(
                div()
                    .text_ramp(&typography.caption)
                    .text_color(colors.on_subtle)
                    .child(status),
            );
        }

        let bar = div()
            .flex()
            .flex_row()
            .h(px(components.title_bar_height))
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
                .child(focus_ring(
                    div()
                        .id("titlebar-min")
                        .debug_selector(|| "titlebar-min".into())
                        .role(Role::Button)
                        .aria_label("Minimize")
                        .track_focus(&min_focus)
                        .key_context("CaptionButton")
                        .fluent_tab_stop(0)
                        .w(px(components.title_bar_button_width))
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(ctrl_hover))
                        .active(move |s| s.bg(ctrl_pressed))
                        .on_click(min_handler)
                        .child(
                            svg()
                                .path("icons/minimize.svg")
                                .size(px(components.title_bar_glyph_size))
                                .text_color(fg),
                        ),
                    min_ring,
                    &colors,
                    ring_radius,
                ))
                .child(focus_ring(
                    div()
                        .id("titlebar-max")
                        .debug_selector(|| "titlebar-max".into())
                        .role(Role::Button)
                        .aria_label("Maximize")
                        .track_focus(&max_focus)
                        .key_context("CaptionButton")
                        .fluent_tab_stop(0)
                        .w(px(components.title_bar_button_width))
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(ctrl_hover))
                        .active(move |s| s.bg(ctrl_pressed))
                        .on_click(max_handler)
                        .child(
                            svg()
                                .path("icons/maximize.svg")
                                .size(px(components.title_bar_glyph_size))
                                .text_color(fg),
                        ),
                    max_ring,
                    &colors,
                    ring_radius,
                ))
                .child(focus_ring(
                    div()
                        .id("titlebar-close")
                        .debug_selector(|| "titlebar-close".into())
                        .role(Role::Button)
                        .aria_label("Close")
                        .track_focus(&close_focus)
                        .key_context("CaptionButton")
                        .fluent_tab_stop(0)
                        .w(px(components.title_bar_button_width))
                        .h_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .hover(move |s| s.bg(close_hover))
                        .active(move |s| s.bg(close_pressed))
                        .on_click(close_handler)
                        .child(
                            svg()
                                .path("icons/dismiss.svg")
                                .size(px(components.title_bar_glyph_size))
                                .text_color(fg),
                        ),
                    close_ring,
                    &colors,
                    ring_radius,
                )),
        )
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::TestAppContext;

    use super::*;

    #[gpui::test]
    fn tab_visits_each_caption_button(cx: &mut TestAppContext) {
        let (_bar, vcx) = open_fluent_test_window(cx, |_, cx| TitleBar::new(cx, "App"));
        vcx.update(|_, app| fluent_primitives::install_focus_keybindings(app));
        let joined = vcx.accessibility_snapshot().join("\n");
        assert!(joined.contains("Button \"Close\""), "{joined}");
        let mut seen = Vec::new();
        for _ in 0..3 {
            vcx.update(|window, app| window.focus_next(app));
            vcx.run_until_parked();
            vcx.update(|window, app| window.draw(app).clear(app));
            let focused = vcx.update(|window, app| window.focused(app));
            seen.push(focused);
        }
        assert!(seen.iter().all(Option::is_some), "{seen:?}");
        assert!(
            seen[0] != seen[1] && seen[1] != seen[2] && seen[0] != seen[2],
            "Tab must visit three distinct caption buttons: {seen:?}"
        );
    }

    fn press(cx: &mut gpui::VisualTestContext, key: &'static str) {
        use gpui::InputEvent as _;
        cx.type_keys(key);
        let keystroke = gpui::Keystroke::parse(key).expect("valid test keystroke");
        cx.update(|window, app| {
            window.dispatch_event(
                gpui::KeyUpEvent {
                    keystroke,
                    scan_code: None,
                }
                .to_platform_input(),
                app,
            );
        });
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
    }

    fn outer_ring_quads(cx: &mut gpui::VisualTestContext) -> usize {
        cx.update(|window, app| window.draw(app).clear(app));
        cx.update(|window, app| {
            let outer = app.theme().colors.stroke_focus_outer;
            let mut rings: Vec<String> = window
                .painted_quads()
                .iter()
                .filter(|q| q.border_color == outer)
                .map(|q| format!("{:?}", q.bounds))
                .collect();
            rings.sort();
            rings.dedup();
            rings.len()
        })
    }

    #[gpui::test]
    fn caption_button_ring_for_keyboard_focus_only(cx: &mut TestAppContext) {
        let (_bar, vcx) = open_fluent_test_window(cx, |_, cx| TitleBar::new(cx, "App"));
        vcx.update(|_, app| fluent_primitives::install_focus_keybindings(app));
        vcx.update(|window, app| window.draw(app).clear(app));
        assert_eq!(outer_ring_quads(vcx), 0, "nothing focused");
        vcx.update(|window, app| window.focus_next(app));
        press(vcx, "tab");
        assert_eq!(outer_ring_quads(vcx), 1, "keyboard focus draws the ring");
        // Mouse-down focuses the button without activating it (no mouse-up).
        let centre = vcx
            .debug_bounds("titlebar-min")
            .expect("min button")
            .center();
        vcx.simulate_mouse_down(centre, gpui::MouseButton::Left, gpui::Modifiers::none());
        vcx.update(|window, app| window.draw(app).clear(app));
        assert!(vcx.update(|window, app| window.focused(app).is_some()));
        assert_eq!(outer_ring_quads(vcx), 0, "mouse focus draws no ring");
    }

    /// The test platform's `minimize` is `unimplemented!()`, so reaching it
    /// proves Enter activated the focused minimize button.
    #[gpui::test]
    #[should_panic(expected = "not implemented")]
    fn enter_on_the_focused_minimize_button_activates_it(cx: &mut TestAppContext) {
        let (_bar, vcx) = open_fluent_test_window(cx, |_, cx| TitleBar::new(cx, "App"));
        vcx.update(|_, app| fluent_primitives::install_focus_keybindings(app));
        vcx.update(|window, app| window.draw(app).clear(app));
        vcx.update(|window, app| window.focus_next(app)); // minimize
        press(vcx, "enter");
    }
}
