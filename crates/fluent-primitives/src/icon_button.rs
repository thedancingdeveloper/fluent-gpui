use crate::focus::FluentFocus as _;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, Role, SharedString,
    Window,
};

use crate::button::{ButtonAppearance, ButtonSize};

/// An icon-only variant of `Button` — no visible label.
///
/// Size is square: icon size determines the hit area.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: SharedString,
    label: Option<SharedString>,
    appearance: ButtonAppearance,
    size: ButtonSize,
    disabled: bool,
    tab_index: Option<isize>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            label: None,
            appearance: ButtonAppearance::Subtle,
            size: ButtonSize::Normal,
            disabled: false,
            tab_index: None,
            on_click: None,
        }
    }

    /// The accessible name. An icon-only button has no visible text, so set
    /// this; without it the name falls back to the icon file's stem.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn appearance(mut self, appearance: ButtonAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Explicit position in the Tab order (lower first). Defaults to document order.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = Some(index);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

/// Fallback accessible name: the icon path's file stem (`icons/search.svg` -> `search`).
fn icon_name(icon: &str) -> SharedString {
    let file = icon.rsplit('/').next().unwrap_or(icon);
    file.split('.').next().unwrap_or(file).to_string().into()
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;

        let (bg, bg_hover, bg_active, fg) = match self.appearance {
            ButtonAppearance::Accent => (
                if self.disabled {
                    colors.accent_disabled
                } else {
                    colors.accent
                },
                colors.accent_hover,
                colors.accent_selected,
                if self.disabled {
                    colors.on_accent_disabled
                } else {
                    colors.on_accent
                },
            ),
            _ => (
                colors.subtle,
                colors.subtle_hover,
                colors.subtle_selected,
                if self.disabled {
                    colors.on_subtle_disabled
                } else {
                    colors.on_subtle
                },
            ),
        };

        let (icon_px, hit_px) = match self.size {
            ButtonSize::Normal => (16.0f32, 32.0f32),
            ButtonSize::Compact => (12.0f32, 24.0f32),
        };

        let disabled = self.disabled;
        let tab_index = self.tab_index;
        let focus_ring = colors.stroke_accent;
        let on_click = self.on_click;
        let name = self.label.unwrap_or_else(|| icon_name(&self.icon));

        let debug_id = self.id.to_string();
        let mut el = div()
            .id(self.id)
            .debug_selector(move || debug_id)
            .role(Role::Button)
            .aria_label(name)
            .flex()
            .items_center()
            .justify_center()
            .size(px(hit_px))
            .rounded(px(radii.md))
            .bg(bg);

        if !disabled {
            el = el
                .focusable()
                .tab_stop(true)
                .fluent_focus(tab_index, focus_ring)
                .cursor_pointer()
                .hover(move |s| s.bg(bg_hover))
                .active(move |s| s.bg(bg_active));
        }

        if let Some(handler) = on_click.filter(|_| !disabled) {
            el = el.on_click(move |ev, win, app| handler(ev, win, app));
        }

        el.child(gpui::svg().path(self.icon).size(px(icon_px)).text_color(fg))
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, InputEvent as _, KeyUpEvent, Keystroke, Render, TestAppContext};

    use super::*;

    struct Host {
        disabled: bool,
        clicks: Rc<Cell<usize>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            IconButton::new("icon-button", "icons/checkmark.svg")
                .disabled(self.disabled)
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
        }
    }

    fn open(
        cx: &mut TestAppContext,
        disabled: bool,
    ) -> (Rc<Cell<usize>>, &mut gpui::VisualTestContext) {
        let clicks = Rc::new(Cell::new(0));
        let host_clicks = clicks.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| Host {
            disabled,
            clicks: host_clicks,
        });
        (clicks, cx)
    }

    fn press(cx: &mut gpui::VisualTestContext, key: &'static str) {
        cx.type_keys(key);
        let keystroke = Keystroke::parse(key).expect("valid test keystroke");
        cx.update(|window, app| {
            window.dispatch_event(
                KeyUpEvent {
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

    #[gpui::test]
    fn click_enter_and_space_activate_when_enabled(cx: &mut TestAppContext) {
        let (clicks, cx) = open(cx, false);
        cx.click_selector("icon-button");
        cx.update(|window, app| window.draw(app).clear(app));
        press(cx, "enter");
        press(cx, "space");
        assert_eq!(clicks.get(), 3);
    }

    #[gpui::test]
    fn disabled_icon_button_ignores_clicks(cx: &mut TestAppContext) {
        let (clicks, cx) = open(cx, true);
        cx.click_selector("icon-button");
        assert_eq!(clicks.get(), 0);
    }

    struct A11yHost(Box<dyn Fn() -> gpui::AnyElement>);

    impl Render for A11yHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    fn snapshot(
        cx: &mut TestAppContext,
        build: impl Fn() -> gpui::AnyElement + 'static,
    ) -> Vec<String> {
        let (_, cx) = open_fluent_test_window(cx, move |_, _| A11yHost(Box::new(build)));
        cx.accessibility_snapshot()
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_labelled_icon_button(cx: &mut TestAppContext) {
        let snap = snapshot(cx, || {
            IconButton::new("b", "icons/search.svg")
                .label("Search")
                .into_any_element()
        });
        assert_eq!(
            snap,
            vec!["Window".to_string(), "Button \"Search\"".to_string()]
        );
    }

    #[gpui::test]
    fn accessibility_snapshot_names_unlabelled_icon_button_from_icon(cx: &mut TestAppContext) {
        let snap = snapshot(cx, || {
            IconButton::new("b", "icons/search.svg").into_any_element()
        });
        assert_eq!(
            snap,
            vec!["Window".to_string(), "Button \"search\"".to_string()]
        );
    }
}
