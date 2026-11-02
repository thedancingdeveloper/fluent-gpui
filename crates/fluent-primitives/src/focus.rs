use gpui::{actions, App, KeyBinding, KeyDownEvent, Window};

actions!(fluent_primitives, [FocusNextControl, FocusPreviousControl]);

pub fn install_focus_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusNextControl, None),
        KeyBinding::new("shift-tab", FocusPreviousControl, None),
    ]);
}

pub(crate) fn focus_next(_: &FocusNextControl, window: &mut Window, cx: &mut App) {
    window.focus_next(cx);
}

pub(crate) fn focus_previous(_: &FocusPreviousControl, window: &mut Window, cx: &mut App) {
    window.focus_prev(cx);
}

pub(crate) fn handle_tab_navigation(ev: &KeyDownEvent, window: &mut Window, cx: &mut App) -> bool {
    if ev.keystroke.key != "tab" {
        return false;
    }

    if ev.keystroke.modifiers.shift {
        window.focus_prev(cx);
    } else {
        window.focus_next(cx);
    }
    true
}

/// Shared focus behaviour for focusable primitives: places the element in the
/// Tab order and draws the Fluent 2 focus stroke for keyboard focus only.
pub(crate) trait FluentFocus: Sized {
    fn fluent_focus(self, tab_index: Option<isize>, ring: gpui::Hsla) -> Self;
}

impl FluentFocus for gpui::Stateful<gpui::Div> {
    fn fluent_focus(self, tab_index: Option<isize>, ring: gpui::Hsla) -> Self {
        use gpui::{point, px, BoxShadow, InteractiveElement as _, Styled as _};
        let el = match tab_index {
            Some(index) => self.tab_index(index),
            None => self,
        };
        el.on_action(focus_next)
            .on_action(focus_previous)
            .focus_visible(move |s| {
                s.shadow(vec![BoxShadow {
                    color: ring,
                    offset: point(px(0.0), px(0.0)),
                    blur_radius: px(0.0),
                    spread_radius: px(2.0),
                    inset: false,
                }])
            })
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{
        div, Context, InputEvent as _, IntoElement, KeyUpEvent, Keystroke, ParentElement as _,
        Render, TestAppContext, Window,
    };

    use crate::{Button, Checkbox, Switch};

    type Log = Rc<RefCell<Vec<&'static str>>>;

    /// A form whose DOM order (switch, checkbox, button) differs from its
    /// Tab order (button, switch, checkbox).
    struct Form(Log);

    impl Render for Form {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (a, b, c) = (self.0.clone(), self.0.clone(), self.0.clone());
            div()
                .child(
                    Switch::new("switch")
                        .tab_index(2)
                        .on_click(move |_, _, _, _| a.borrow_mut().push("switch")),
                )
                .child(
                    Checkbox::new("checkbox")
                        .tab_index(3)
                        .on_click(move |_, _, _, _| b.borrow_mut().push("checkbox")),
                )
                .child(
                    Button::new("button")
                        .tab_index(1)
                        .on_click(move |_, _, _| c.borrow_mut().push("button")),
                )
        }
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
    fn tab_and_shift_tab_follow_tab_index_through_a_form(cx: &mut TestAppContext) {
        let log: Log = Rc::default();
        let (_, cx) = open_fluent_test_window(cx, {
            let log = log.clone();
            move |_, _| Form(log)
        });
        cx.update(|_, app| super::install_focus_keybindings(app));
        cx.update(|window, app| window.draw(app).clear(app));

        // Nothing is focused yet, so the first stop is entered by the window.
        cx.update(|window, app| window.focus_next(app));
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
        press(cx, "enter");
        press(cx, "tab");
        press(cx, "space");
        press(cx, "tab");
        press(cx, "enter");
        assert_eq!(*log.borrow(), ["button", "switch", "checkbox"]);

        log.borrow_mut().clear();
        press(cx, "shift-tab");
        press(cx, "enter");
        press(cx, "shift-tab");
        press(cx, "enter");
        assert_eq!(*log.borrow(), ["switch", "button"]);
    }
}
