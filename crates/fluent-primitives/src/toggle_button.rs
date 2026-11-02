use crate::focus::FluentFocus as _;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window,
};

use crate::button::ButtonSize;

/// A button that toggles between selected and unselected states.
///
/// The caller owns the `selected` state and passes it in on each render.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct ToggleButton {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<SharedString>,
    selected: bool,
    disabled: bool,
    tab_index: Option<isize>,
    size: ButtonSize,
    on_click: Option<Box<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl ToggleButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            selected: false,
            disabled: false,
            tab_index: None,
            size: ButtonSize::Normal,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
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

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Callback receives the *new* desired state (the toggle target).
    pub fn on_click(
        mut self,
        handler: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for ToggleButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;
        let spacing = &theme.spacing;
        let typography = &theme.typography;

        let (bg, bg_hover, fg) = if self.selected {
            (
                colors.neutral_selected,
                colors.neutral_hover,
                colors.on_neutral_selected,
            )
        } else {
            (colors.subtle, colors.subtle_hover, colors.on_subtle)
        };

        let fg = if self.disabled {
            colors.on_neutral_disabled
        } else {
            fg
        };

        let radius = px(radii.md);
        let (px_val, py_val, font_size) = match self.size {
            ButtonSize::Normal => (px(spacing.md), px(spacing.sm), px(typography.body.size)),
            ButtonSize::Compact => (px(spacing.sm), px(spacing.xs), px(typography.caption.size)),
        };

        let disabled = self.disabled;
        let tab_index = self.tab_index;
        let focus_ring = colors.stroke_accent;
        let selected = self.selected;
        let on_click = self.on_click;
        let icon = self.icon;
        let label = self.label;

        let debug_id = self.id.to_string();
        let mut el = div()
            .id(self.id)
            .debug_selector(move || debug_id)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.sm))
            .px(px_val)
            .py(py_val)
            .rounded(radius)
            .bg(bg)
            .text_size(font_size)
            .text_color(fg);

        if !disabled {
            el = el
                .focusable()
                .tab_stop(true)
                .fluent_focus(tab_index, focus_ring)
                .cursor_pointer()
                .hover(move |s| s.bg(bg_hover));
        }

        if let Some(handler) = on_click.filter(|_| !disabled) {
            let new_state = !selected;
            el = el.on_click(move |ev, win, app| handler(new_state, ev, win, app));
        }

        if let Some(icon_path) = icon {
            let icon_size = px(16.0);
            el = el.child(gpui::svg().path(icon_path).size(icon_size).text_color(fg));
        }

        if let Some(text) = label {
            el = el.child(text);
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Entity, InputEvent as _, KeyUpEvent, Keystroke, Render, TestAppContext};

    use super::*;

    struct Host {
        selected: bool,
        disabled: bool,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.entity();
            ToggleButton::new("toggle-button")
                .label("Pin")
                .selected(self.selected)
                .disabled(self.disabled)
                .on_click(move |selected, _, _, cx| {
                    entity.update(cx, |host, cx| {
                        host.selected = selected;
                        cx.notify();
                    });
                })
        }
    }

    fn open(
        cx: &mut TestAppContext,
        disabled: bool,
    ) -> (Entity<Host>, &mut gpui::VisualTestContext) {
        open_fluent_test_window(cx, move |_, _| Host {
            selected: false,
            disabled,
        })
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
    fn click_enter_and_space_flip_selected_state(cx: &mut TestAppContext) {
        let (host, cx) = open(cx, false);
        cx.click_selector("toggle-button");
        assert!(host.read_with(cx, |host, _| host.selected));
        cx.update(|window, app| window.draw(app).clear(app));
        press(cx, "enter");
        assert!(!host.read_with(cx, |host, _| host.selected));
        press(cx, "space");
        assert!(host.read_with(cx, |host, _| host.selected));
    }

    #[gpui::test]
    fn disabled_toggle_button_does_not_flip(cx: &mut TestAppContext) {
        let (host, cx) = open(cx, true);
        cx.click_selector("toggle-button");
        assert!(!host.read_with(cx, |host, _| host.selected));
    }
}
