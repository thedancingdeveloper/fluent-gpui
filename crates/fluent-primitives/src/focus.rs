use fluent_core::ColorScheme;
use gpui::{
    actions, div, px, App, FocusHandle, KeyBinding, KeyDownEvent, ParentElement, Pixels, Styled,
    Window,
};

actions!(fluent_primitives, [FocusNextControl, FocusPreviousControl]);

/// Installs the standard Fluent focus-navigation key bindings.
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

/// Whether `handle` has focus *and* the last input was the keyboard, i.e. the
/// focus indicator should be drawn. Mouse focus must not show a ring.
///
/// The vendored gpui tracks the input modality itself: a key-down switches the
/// window to keyboard modality (Tab navigation included) and a mouse-down or
/// mouse-move switches it back, so no separate global flag is needed.
pub fn is_keyboard_focused(handle: &FocusHandle, window: &Window) -> bool {
    handle.is_focused(window) && window.last_input_was_keyboard()
}

/// Draws the Fluent 2 focus indicator on `el` when `keyboard_focused`: a 2px
/// `stroke_focus_outer` ring just outside the control (rounded `radius + 2`)
/// and a 1px `stroke_focus_inner` ring just inside its edge. The ring is an
/// absolutely positioned, non-interactive child, so `el` gets `relative()` and
/// any `overflow_hidden` ancestor needs 2px of headroom around `el`.
///
/// Compute `keyboard_focused` with [`is_keyboard_focused`].
pub fn focus_ring<E>(el: E, keyboard_focused: bool, colors: &ColorScheme, radius: Pixels) -> E
where
    E: Styled + ParentElement,
{
    if !keyboard_focused {
        return el;
    }
    el.relative().child(
        div()
            .absolute()
            .top(px(-2.0))
            .right(px(-2.0))
            .bottom(px(-2.0))
            .left(px(-2.0))
            .rounded(radius + px(2.0))
            .border_2()
            .border_color(colors.stroke_focus_outer)
            .child(
                div()
                    .size_full()
                    .rounded(radius)
                    .border_1()
                    .border_color(colors.stroke_focus_inner),
            ),
    )
}

/// Shared focus behaviour for focusable primitives: places the element in the
/// Tab order and draws the Fluent 2 focus stroke for keyboard focus only.
/// Shared focus behaviour for focusable Fluent controls.
pub trait FluentFocus: Sized {
    /// Adds the element to the Tab order and draws `ring` when keyboard-focused.
    fn fluent_focus(self, tab_index: Option<isize>, ring: gpui::Hsla) -> Self;

    /// Adds the element to the Tab order and handles Tab / Shift-Tab, without
    /// drawing anything. Pair it with [`focus_ring`] and a tracked
    /// `FocusHandle` to draw the two-tone Fluent 2 ring.
    fn fluent_tab_stop(self, tab_index: isize) -> Self;
}

impl FluentFocus for gpui::Stateful<gpui::Div> {
    fn fluent_tab_stop(self, tab_index: isize) -> Self {
        use gpui::InteractiveElement as _;
        self.tab_index(tab_index)
            .on_action(focus_next)
            .on_action(focus_previous)
    }

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

    struct Ringed(bool);

    impl Render for Ringed {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            use fluent_core::ThemeProvider as _;
            use gpui::{px, Styled as _};
            let colors = cx.theme().colors.clone();
            super::focus_ring(div().size(px(40.0)), self.0, &colors, px(4.0))
        }
    }

    fn ring_quads(cx: &mut gpui::VisualTestContext) -> (usize, usize) {
        cx.update(|window, app| window.draw(app).clear(app));
        cx.update(|window, app| {
            use fluent_core::ThemeProvider as _;
            let colors = app.theme().colors.clone();
            let quads = window.painted_quads();
            // Identical quads from repeated draws count once.
            let count = |c| {
                let mut b: Vec<String> = quads
                    .iter()
                    .filter(|q| q.border_color == c)
                    .map(|q| format!("{:?}", q.bounds))
                    .collect();
                b.sort();
                b.dedup();
                b.len()
            };
            (
                count(colors.stroke_focus_outer),
                count(colors.stroke_focus_inner),
            )
        })
    }

    #[gpui::test]
    fn focus_ring_draws_outer_and_inner_only_when_keyboard_focused(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, _| Ringed(false));
        assert_eq!(ring_quads(cx), (0, 0));
        let (_, cx) = open_fluent_test_window(cx, |_, _| Ringed(true));
        assert_eq!(ring_quads(cx), (1, 1));
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
