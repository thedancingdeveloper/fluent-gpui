use crate::focus::FluentFocus as _;
use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, Window};

/// Three-state checkbox value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckboxState {
    #[default]
    Unchecked,
    Checked,
    Indeterminate,
}

/// A Fluent 2-styled checkbox.
///
/// The caller owns the state and passes it in on each render.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    state: CheckboxState,
    disabled: bool,
    tab_index: Option<isize>,
    on_click: Option<Box<dyn Fn(CheckboxState, &ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            state: CheckboxState::Unchecked,
            disabled: false,
            tab_index: None,
            on_click: None,
        }
    }

    pub fn state(mut self, state: CheckboxState) -> Self {
        self.state = state;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.state = if checked {
            CheckboxState::Checked
        } else {
            CheckboxState::Unchecked
        };
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

    /// Callback receives the *new* desired state (the toggle target).
    pub fn on_click(
        mut self,
        handler: impl Fn(CheckboxState, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Checkbox {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;

        let is_checked = matches!(self.state, CheckboxState::Checked);
        let is_indet = matches!(self.state, CheckboxState::Indeterminate);
        let is_active = is_checked || is_indet;

        let box_bg = if is_active {
            if self.disabled {
                colors.accent_disabled
            } else {
                colors.accent
            }
        } else {
            colors.surface
        };
        let box_border = if is_active {
            colors.accent
        } else if self.disabled {
            colors.stroke_neutral_disabled
        } else {
            colors.stroke_neutral
        };
        let mark_color = if self.disabled {
            colors.on_accent_disabled
        } else {
            colors.on_accent
        };

        let disabled = self.disabled;
        let tab_index = self.tab_index;
        let focus_ring = colors.stroke_accent;
        let current = self.state;
        let on_click = self.on_click;

        let next_state = match current {
            CheckboxState::Unchecked | CheckboxState::Indeterminate => CheckboxState::Checked,
            CheckboxState::Checked => CheckboxState::Unchecked,
        };

        let debug_id = self.id.to_string();
        let mut el = div()
            .id(self.id)
            .debug_selector(move || debug_id)
            .size(px(18.0))
            .rounded(px(radii.sm))
            .bg(box_bg)
            .border_1()
            .border_color(box_border)
            .flex()
            .items_center()
            .justify_center();

        if !disabled {
            el = el
                .focusable()
                .tab_stop(true)
                .fluent_focus(tab_index, focus_ring)
                .cursor_pointer()
                .hover(move |s| s.border_color(colors.stroke_accent));
        }

        if let Some(handler) = on_click.filter(|_| !disabled) {
            el = el.on_click(move |ev, win, app| handler(next_state, ev, win, app));
        }

        // Checkmark or dash indicator
        if is_checked {
            el = el.child(
                gpui::svg()
                    .path("icons/checkmark.svg")
                    .size(px(12.0))
                    .text_color(mark_color),
            );
        } else if is_indet {
            el = el.child(
                div()
                    .w(px(10.0))
                    .h(px(2.0))
                    .rounded(px(radii.sm))
                    .bg(mark_color),
            );
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
        state: CheckboxState,
        disabled: bool,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.entity();
            Checkbox::new("checkbox")
                .state(self.state)
                .disabled(self.disabled)
                .on_click(move |state, _, _, cx| {
                    entity.update(cx, |host, cx| {
                        host.state = state;
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
            state: CheckboxState::Unchecked,
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
    fn click_enter_and_space_flip_checked_state(cx: &mut TestAppContext) {
        let (host, cx) = open(cx, false);
        cx.click_selector("checkbox");
        assert_eq!(
            host.read_with(cx, |host, _| host.state),
            CheckboxState::Checked
        );
        cx.update(|window, app| window.draw(app).clear(app));
        press(cx, "enter");
        assert_eq!(
            host.read_with(cx, |host, _| host.state),
            CheckboxState::Unchecked
        );
        press(cx, "space");
        assert_eq!(
            host.read_with(cx, |host, _| host.state),
            CheckboxState::Checked
        );
    }

    #[gpui::test]
    fn disabled_checkbox_does_not_flip(cx: &mut TestAppContext) {
        let (host, cx) = open(cx, true);
        cx.click_selector("checkbox");
        assert_eq!(
            host.read_with(cx, |host, _| host.state),
            CheckboxState::Unchecked
        );
    }
}
