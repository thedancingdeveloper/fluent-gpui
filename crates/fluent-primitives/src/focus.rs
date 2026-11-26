use fluent_core::ColorScheme;
use gpui::{
    actions, div, px, App, ElementId, FocusHandle, KeyBinding, KeyDownEvent, ParentElement, Pixels,
    Styled, Window,
};

actions!(fluent_primitives, [FocusNextControl, FocusPreviousControl]);

struct FocusKeybindingsInstalled;

impl gpui::Global for FocusKeybindingsInstalled {}

/// Installs the standard Fluent focus-navigation key bindings (`tab`,
/// `shift-tab`). `FluentApp::run` calls this for you; call it yourself in an
/// app that builds its own gpui `Application`. Safe to call more than once: only
/// the first call binds.
pub fn install_focus_keybindings(cx: &mut App) {
    if cx.has_global::<FocusKeybindingsInstalled>() {
        return;
    }
    cx.set_global(FocusKeybindingsInstalled);
    cx.bind_keys([
        KeyBinding::new("tab", FocusNextControl, None),
        KeyBinding::new("shift-tab", FocusPreviousControl, None),
    ]);
    // With nothing focused no control's `on_action` is on the dispatch path, so
    // the first Tab would be dropped. These app-level handlers only run when no
    // focused element handled the action, and enter the first/last tab stop.
    cx.on_action(|_: &FocusNextControl, cx: &mut App| {
        if let Some(window) = target_window(cx) {
            // The action is dispatched inside the window's own update, so the
            // move is deferred until that update has finished.
            cx.defer(move |cx| {
                window
                    .update(cx, |_, window, cx| window.focus_next(cx))
                    .ok();
            });
        }
    });
    cx.on_action(|_: &FocusPreviousControl, cx: &mut App| {
        if let Some(window) = target_window(cx) {
            // The action is dispatched inside the window's own update, so the
            // move is deferred until that update has finished.
            cx.defer(move |cx| {
                window
                    .update(cx, |_, window, cx| window.focus_prev(cx))
                    .ok();
            });
        }
    });
}

/// The window an app-level Tab belongs to: the platform-active one, else the
/// most recently opened (platforms that do not report activation).
fn target_window(cx: &App) -> Option<gpui::AnyWindowHandle> {
    cx.active_window().or_else(|| cx.windows().last().copied())
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

/// The `FocusHandle` of a stateless (`RenderOnce`) control, kept across frames
/// under `id` and configured as a Tab stop at `tab_index` (`0` when `None`).
/// Pair it with `track_focus` and [`FluentFocus::fluent_tab_stop`]; a disabled
/// control must not call it, so it never enters the Tab order.
pub(crate) fn control_focus_handle(
    id: &ElementId,
    tab_index: Option<isize>,
    window: &mut Window,
    cx: &mut App,
) -> FocusHandle {
    let state = window.use_keyed_state((id.clone(), "fluent-focus"), cx, |_, cx| cx.focus_handle());
    let handle = state.read(cx).clone();
    handle.tab_index(tab_index.unwrap_or(0)).tab_stop(true)
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

/// Keyboard-focus behaviour shared by the button family (`Button`,
/// `IconButton`, `ToggleButton`): Tab stop, Space/Enter activation, two-tone
/// ring for keyboard focus only, and disabled controls skipped.
#[cfg(test)]
mod button_family_tests {
    use std::{cell::Cell, rc::Rc};

    use fluent_core::ThemeProvider as _;
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{
        div, Context, InputEvent as _, IntoElement, KeyUpEvent, Keystroke, ParentElement as _,
        Render, TestAppContext, VisualTestContext, Window,
    };

    use crate::{Button, IconButton, ToggleButton};

    #[derive(Clone, Copy, Debug)]
    enum Kind {
        Button,
        Icon,
        Toggle,
    }

    /// `[first (maybe disabled)] [second]`; only `second` is always enabled.
    struct Host {
        kind: Kind,
        first_disabled: bool,
        first: Rc<Cell<usize>>,
        second: Rc<Cell<usize>>,
    }

    impl Host {
        fn control(
            &self,
            id: &'static str,
            disabled: bool,
            hits: Rc<Cell<usize>>,
        ) -> gpui::AnyElement {
            match self.kind {
                Kind::Button => Button::new(id)
                    .label("Go")
                    .disabled(disabled)
                    .on_click(move |_, _, _| hits.set(hits.get() + 1))
                    .into_any_element(),
                Kind::Icon => IconButton::new(id, "icons/checkmark.svg")
                    .disabled(disabled)
                    .on_click(move |_, _, _| hits.set(hits.get() + 1))
                    .into_any_element(),
                Kind::Toggle => ToggleButton::new(id)
                    .label("Go")
                    .disabled(disabled)
                    .on_click(move |_, _, _, _| hits.set(hits.get() + 1))
                    .into_any_element(),
            }
        }
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(self.control("first", self.first_disabled, self.first.clone()))
                .child(self.control("second", false, self.second.clone()))
        }
    }

    fn open(
        cx: &mut TestAppContext,
        kind: Kind,
        first_disabled: bool,
    ) -> (Rc<Cell<usize>>, Rc<Cell<usize>>, &mut VisualTestContext) {
        let (first, second) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
        let host = (first.clone(), second.clone());
        let (_, cx) = open_fluent_test_window(cx, move |_, _| Host {
            kind,
            first_disabled,
            first: host.0,
            second: host.1,
        });
        cx.update(|_, app| super::install_focus_keybindings(app));
        cx.update(|window, app| window.draw(app).clear(app));
        (first, second, cx)
    }

    fn press(cx: &mut VisualTestContext, key: &'static str) {
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

    /// Enter the first Tab stop the way the window does before any key press.
    fn focus_first(cx: &mut VisualTestContext) {
        cx.update(|window, app| window.focus_next(app));
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
    }

    fn outer_ring_quads(cx: &mut VisualTestContext) -> usize {
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

    const KINDS: [Kind; 3] = [Kind::Button, Kind::Icon, Kind::Toggle];

    #[gpui::test]
    fn tab_reaches_each_button_in_order(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (first, second, cx) = open(cx, kind, false);
            focus_first(cx);
            press(cx, "space");
            assert_eq!((first.get(), second.get()), (1, 0), "{kind:?}: first stop");
            press(cx, "tab");
            press(cx, "enter");
            assert_eq!((first.get(), second.get()), (1, 1), "{kind:?}: second stop");
        }
    }

    #[gpui::test]
    fn space_and_enter_activate_the_focused_button(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (first, _, cx) = open(cx, kind, false);
            focus_first(cx);
            press(cx, "enter");
            press(cx, "space");
            assert_eq!(first.get(), 2, "{kind:?}");
        }
    }

    #[gpui::test]
    fn ring_shows_for_keyboard_focus_only(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (_, _, cx) = open(cx, kind, false);
            assert_eq!(outer_ring_quads(cx), 0, "{kind:?}: nothing focused");
            focus_first(cx);
            press(cx, "tab");
            assert_eq!(outer_ring_quads(cx), 1, "{kind:?}: keyboard focus");
            cx.click_selector("first");
            cx.update(|window, app| window.draw(app).clear(app));
            assert!(cx.update(|window, app| window.focused(app).is_some()));
            assert_eq!(
                outer_ring_quads(cx),
                0,
                "{kind:?}: mouse focus draws no ring"
            );
        }
    }

    #[gpui::test]
    fn disabled_button_is_skipped_and_never_activates(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (first, second, cx) = open(cx, kind, true);
            focus_first(cx);
            press(cx, "enter");
            press(cx, "space");
            assert_eq!(first.get(), 0, "{kind:?}: disabled must not activate");
            assert_eq!(
                second.get(),
                2,
                "{kind:?}: focus skipped to the enabled button"
            );
        }
    }
}

/// Keyboard-focus behaviour shared by the selection controls (`Checkbox`,
/// `Switch`, `RadioGroup`, `Dropdown`, dismissable `Chip`, clickable `Card`):
/// Tab stop, Space/Enter activation, two-tone ring for keyboard focus only,
/// and disabled controls skipped.
#[cfg(test)]
mod selection_family_tests {
    use std::{cell::Cell, cell::RefCell, rc::Rc};

    use fluent_core::ThemeProvider as _;
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{
        div, AppContext as _, Context, Entity, InputEvent as _, IntoElement, KeyUpEvent, Keystroke,
        ParentElement as _, Render, SharedString, TestAppContext, VisualTestContext, Window,
    };

    use crate::radio_group::RadioOption;
    use crate::{Card, Checkbox, Chip, Dropdown, RadioGroup, Switch};

    #[derive(Clone, Copy, Debug)]
    enum Kind {
        Checkbox,
        Switch,
        Chip,
        Card,
        Dropdown,
    }

    const KINDS: [Kind; 5] = [
        Kind::Checkbox,
        Kind::Switch,
        Kind::Chip,
        Kind::Card,
        Kind::Dropdown,
    ];

    /// `[first (maybe disabled)] [second]`; only `second` is always enabled.
    struct Host {
        kind: Kind,
        first_disabled: bool,
        first: Rc<Cell<usize>>,
        second: Rc<Cell<usize>>,
        dropdowns: Vec<Entity<Dropdown>>,
    }

    fn control(
        kind: Kind,
        id: &'static str,
        disabled: bool,
        hits: Rc<Cell<usize>>,
        dropdown: &Entity<Dropdown>,
    ) -> gpui::AnyElement {
        match kind {
            Kind::Checkbox => Checkbox::new(id)
                .disabled(disabled)
                .on_click(move |_, _, _, _| hits.set(hits.get() + 1))
                .into_any_element(),
            Kind::Switch => Switch::new(id)
                .disabled(disabled)
                .on_click(move |_, _, _, _| hits.set(hits.get() + 1))
                .into_any_element(),
            // A chip's only control is its dismiss button; a disabled chip
            // simply has none.
            Kind::Chip => {
                let chip = Chip::new(id, "Tag");
                if disabled {
                    chip.into_any_element()
                } else {
                    chip.on_dismiss(move |_, _, _| hits.set(hits.get() + 1))
                        .into_any_element()
                }
            }
            // A card without a click handler is not a control.
            Kind::Card => {
                let card = Card::new().id(id).child(div().child("Card"));
                if disabled {
                    card.into_any_element()
                } else {
                    card.on_click(move |_, _, _| hits.set(hits.get() + 1))
                        .into_any_element()
                }
            }
            Kind::Dropdown => dropdown.clone().into_any_element(),
        }
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(control(
                    self.kind,
                    "first",
                    self.first_disabled,
                    self.first.clone(),
                    &self.dropdowns[0],
                ))
                .child(control(
                    self.kind,
                    "second",
                    false,
                    self.second.clone(),
                    &self.dropdowns[1],
                ))
        }
    }

    fn make_dropdown(
        cx: &mut Context<Host>,
        id: &'static str,
        disabled: bool,
        hits: Rc<Cell<usize>>,
    ) -> Entity<Dropdown> {
        cx.new(|_| {
            Dropdown::new(id)
                .option("one", "One")
                .disabled(disabled)
                .on_select(move |_, _| hits.set(hits.get() + 1))
        })
    }

    fn open(
        cx: &mut TestAppContext,
        kind: Kind,
        first_disabled: bool,
    ) -> (Rc<Cell<usize>>, Rc<Cell<usize>>, &mut VisualTestContext) {
        let (first, second) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
        let (f, s) = (first.clone(), second.clone());
        let (_, cx) = open_fluent_test_window(cx, move |_, cx| Host {
            kind,
            first_disabled,
            dropdowns: vec![
                make_dropdown(cx, "dd-first", first_disabled, f.clone()),
                make_dropdown(cx, "dd-second", false, s.clone()),
            ],
            first: f,
            second: s,
        });
        cx.update(|_, app| super::install_focus_keybindings(app));
        cx.update(|window, app| window.draw(app).clear(app));
        (first, second, cx)
    }

    fn press(cx: &mut VisualTestContext, key: &'static str) {
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

    fn focus_first(cx: &mut VisualTestContext) {
        cx.update(|window, app| window.focus_next(app));
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
    }

    fn outer_ring_quads(cx: &mut VisualTestContext) -> usize {
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

    /// Activate the focused control. A dropdown needs a second press: the
    /// first opens its list, the second picks the highlighted option.
    fn activate(cx: &mut VisualTestContext, kind: Kind, key: &'static str) {
        press(cx, key);
        if matches!(kind, Kind::Dropdown) {
            press(cx, key);
        }
    }

    #[gpui::test]
    fn tab_reaches_each_control_in_order(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (first, second, cx) = open(cx, kind, false);
            focus_first(cx);
            activate(cx, kind, "space");
            assert_eq!((first.get(), second.get()), (1, 0), "{kind:?}: first stop");
            press(cx, "tab");
            activate(cx, kind, "space");
            assert_eq!((first.get(), second.get()), (1, 1), "{kind:?}: second stop");
        }
    }

    #[gpui::test]
    fn space_toggles_checkbox_and_switch(cx: &mut TestAppContext) {
        for kind in [Kind::Checkbox, Kind::Switch] {
            let (first, _, cx) = open(cx, kind, false);
            focus_first(cx);
            press(cx, "space");
            press(cx, "space");
            assert_eq!(first.get(), 2, "{kind:?}");
        }
    }

    #[gpui::test]
    fn ring_shows_for_keyboard_focus_only(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (_, _, cx) = open(cx, kind, false);
            assert_eq!(outer_ring_quads(cx), 0, "{kind:?}: nothing focused");
            focus_first(cx);
            press(cx, "tab");
            assert_eq!(outer_ring_quads(cx), 1, "{kind:?}: keyboard focus");
        }
    }

    #[gpui::test]
    fn disabled_control_is_skipped_and_never_activates(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (first, second, cx) = open(cx, kind, true);
            focus_first(cx);
            activate(cx, kind, "space");
            assert_eq!(first.get(), 0, "{kind:?}: disabled must not activate");
            assert_eq!(
                second.get(),
                1,
                "{kind:?}: focus skipped to the enabled one"
            );
        }
    }

    type RadioLog = Rc<RefCell<Vec<String>>>;

    /// `[RadioGroup a b(disabled) c] [Checkbox]`.
    struct RadioHost {
        selected: SharedString,
        log: Rc<RefCell<Vec<String>>>,
        after: Rc<Cell<usize>>,
    }

    impl Render for RadioHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.entity();
            let after = self.after.clone();
            let log = self.log.clone();
            div()
                .child(
                    RadioGroup::new("radios")
                        .options([
                            RadioOption::new("a", "A"),
                            RadioOption::new("b", "B").disabled(true),
                            RadioOption::new("c", "C"),
                        ])
                        .selected(self.selected.clone())
                        .on_select(move |value, _, _, cx| {
                            log.borrow_mut().push(value.to_string());
                            entity.update(cx, |host, cx| {
                                host.selected = value;
                                cx.notify();
                            });
                        }),
                )
                .child(
                    Checkbox::new("after").on_click(move |_, _, _, _| after.set(after.get() + 1)),
                )
        }
    }

    fn open_radios(cx: &mut TestAppContext) -> (RadioLog, Rc<Cell<usize>>, &mut VisualTestContext) {
        let (log, after) = (Rc::new(RefCell::new(Vec::new())), Rc::new(Cell::new(0)));
        let (l, a) = (log.clone(), after.clone());
        let (_, cx) = open_fluent_test_window(cx, move |_, _| RadioHost {
            selected: "a".into(),
            log: l,
            after: a,
        });
        cx.update(|_, app| super::install_focus_keybindings(app));
        cx.update(|window, app| window.draw(app).clear(app));
        (log, after, cx)
    }

    #[gpui::test]
    fn radio_arrows_move_selection_and_tab_leaves_the_group(cx: &mut TestAppContext) {
        let (log, after, cx) = open_radios(cx);
        focus_first(cx);
        // Arrows move within the group, skipping the disabled option.
        press(cx, "down");
        assert_eq!(*log.borrow(), ["c"]);
        press(cx, "up");
        assert_eq!(*log.borrow(), ["c", "a"]);
        // Space selects the focused option exactly once.
        press(cx, "space");
        assert_eq!(*log.borrow(), ["c", "a", "a"]);
        // One Tab stop per group: a single Tab leaves it for the checkbox.
        press(cx, "tab");
        press(cx, "space");
        assert_eq!(after.get(), 1);
        // And Shift-Tab re-enters the group at the selected option.
        press(cx, "shift-tab");
        press(cx, "down");
        assert_eq!(*log.borrow(), ["c", "a", "a", "c"]);
    }

    #[gpui::test]
    fn radio_ring_shows_for_keyboard_focus_only(cx: &mut TestAppContext) {
        let (_, _, cx) = open_radios(cx);
        assert_eq!(outer_ring_quads(cx), 0);
        focus_first(cx);
        press(cx, "down");
        assert_eq!(outer_ring_quads(cx), 1);
        cx.click_selector("radio-option-0");
        cx.update(|window, app| window.draw(app).clear(app));
        assert_eq!(outer_ring_quads(cx), 0, "mouse focus draws no ring");
    }

    #[gpui::test]
    fn mouse_focus_draws_no_ring_on_selection_controls(cx: &mut TestAppContext) {
        for (kind, selector) in [(Kind::Checkbox, "first"), (Kind::Switch, "first")] {
            let (_, _, cx) = open(cx, kind, false);
            focus_first(cx);
            press(cx, "tab");
            assert_eq!(outer_ring_quads(cx), 1, "{kind:?}: keyboard focus");
            cx.click_selector(selector);
            cx.update(|window, app| window.draw(app).clear(app));
            assert_eq!(
                outer_ring_quads(cx),
                0,
                "{kind:?}: mouse focus draws no ring"
            );
        }
    }
}

/// Keyboard-focus behaviour shared by `Link`, `Slider`, `SpinButton`,
/// `Expander` (migrated from `fluent_focus`): Tab stop, keyboard activation,
/// two-tone ring for keyboard focus only, and disabled controls skipped.
#[cfg(test)]
mod leftover_family_tests {
    use std::{cell::Cell, rc::Rc};

    use fluent_core::ThemeProvider as _;
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{
        div, AppContext as _, Context, InputEvent as _, IntoElement, KeyUpEvent, Keystroke,
        ParentElement as _, Render, TestAppContext, VisualTestContext, Window,
    };

    use crate::{Color, ColorPicker, Expander, Link, Slider, SpinButton};

    #[derive(Clone, Copy, Debug)]
    enum Kind {
        Link,
        Slider,
        Spin,
        Expander,
    }

    /// The key that makes each kind fire its callback.
    fn activation_key(kind: Kind) -> &'static str {
        match kind {
            Kind::Link | Kind::Expander => "enter",
            Kind::Slider => "right",
            Kind::Spin => "up",
        }
    }

    /// `[first (maybe disabled)] [second]`; only `second` is always enabled.
    struct Host {
        kind: Kind,
        first_disabled: bool,
        first: Rc<Cell<usize>>,
        second: Rc<Cell<usize>>,
    }

    impl Host {
        fn control(
            &self,
            id: &'static str,
            disabled: bool,
            hits: Rc<Cell<usize>>,
        ) -> gpui::AnyElement {
            match self.kind {
                Kind::Link => Link::new(id, "Go")
                    .disabled(disabled)
                    .on_click(move |_, _, _| hits.set(hits.get() + 1))
                    .into_any_element(),
                Kind::Slider => Slider::new(id)
                    .range(0.0, 100.0)
                    .value(50.0)
                    .disabled(disabled)
                    .on_change(move |_, _, _| hits.set(hits.get() + 1))
                    .into_any_element(),
                Kind::Spin => SpinButton::new(id)
                    .range(0.0, 100.0)
                    .value(50.0)
                    .disabled(disabled)
                    .on_change(move |_, _, _| hits.set(hits.get() + 1))
                    .into_any_element(),
                Kind::Expander => Expander::new(id, "Section")
                    .disabled(disabled)
                    .on_toggle(move |_, _, _, _| hits.set(hits.get() + 1))
                    .into_any_element(),
            }
        }
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(self.control("first", self.first_disabled, self.first.clone()))
                .child(self.control("second", false, self.second.clone()))
        }
    }

    fn open(
        cx: &mut TestAppContext,
        kind: Kind,
        first_disabled: bool,
    ) -> (Rc<Cell<usize>>, Rc<Cell<usize>>, &mut VisualTestContext) {
        let (first, second) = (Rc::new(Cell::new(0)), Rc::new(Cell::new(0)));
        let host = (first.clone(), second.clone());
        let (_, cx) = open_fluent_test_window(cx, move |_, _| Host {
            kind,
            first_disabled,
            first: host.0,
            second: host.1,
        });
        cx.update(|_, app| super::install_focus_keybindings(app));
        cx.update(|window, app| window.draw(app).clear(app));
        (first, second, cx)
    }

    fn press(cx: &mut VisualTestContext, key: &'static str) {
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

    fn focus_first(cx: &mut VisualTestContext) {
        cx.update(|window, app| window.focus_next(app));
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
    }

    fn outer_ring_quads(cx: &mut VisualTestContext) -> usize {
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

    const KINDS: [Kind; 4] = [Kind::Link, Kind::Slider, Kind::Spin, Kind::Expander];

    #[gpui::test]
    fn tab_reaches_each_control_in_order_and_keys_activate_once(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (first, second, cx) = open(cx, kind, false);
            focus_first(cx);
            press(cx, activation_key(kind));
            assert_eq!((first.get(), second.get()), (1, 0), "{kind:?}: first stop");
            press(cx, "tab");
            press(cx, activation_key(kind));
            assert_eq!((first.get(), second.get()), (1, 1), "{kind:?}: second stop");
        }
    }

    #[gpui::test]
    fn ring_shows_for_keyboard_focus_only(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (_, _, cx) = open(cx, kind, false);
            // A slider thumb's white border can share the ring colour in the
            // dark theme, so count the ring relative to the unfocused baseline.
            let base = outer_ring_quads(cx);
            focus_first(cx);
            press(cx, "tab");
            assert_eq!(outer_ring_quads(cx), base + 1, "{kind:?}: keyboard focus");
            cx.click_selector("first");
            cx.update(|window, app| window.draw(app).clear(app));
            assert!(cx.update(|window, app| window.focused(app).is_some()));
            assert_eq!(
                outer_ring_quads(cx),
                base,
                "{kind:?}: mouse focus draws no ring"
            );
        }
    }

    #[gpui::test]
    fn disabled_control_is_skipped_and_never_activates(cx: &mut TestAppContext) {
        for kind in KINDS {
            let (first, second, cx) = open(cx, kind, true);
            focus_first(cx);
            press(cx, activation_key(kind));
            assert_eq!(first.get(), 0, "{kind:?}: disabled must not activate");
            assert_eq!(
                second.get(),
                1,
                "{kind:?}: focus skipped to the enabled one"
            );
        }
    }

    /// A picker followed by a `Link`, so Tab has somewhere to go.
    struct PickerHost(gpui::Entity<ColorPicker>);

    impl Render for PickerHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .child(self.0.clone())
                .child(Link::new("after", "After"))
        }
    }

    /// The default swatches are blue, green, red, white; the value is white.
    fn open_picker(
        cx: &mut TestAppContext,
    ) -> (Rc<std::cell::RefCell<Vec<Color>>>, &mut VisualTestContext) {
        let picked = Rc::new(std::cell::RefCell::new(Vec::new()));
        let log = picked.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, cx| {
            let log = log.clone();
            let picker = cx.new(|_| {
                ColorPicker::new("picker").on_change(move |c, _| log.borrow_mut().push(c))
            });
            PickerHost(picker)
        });
        cx.update(|_, app| super::install_focus_keybindings(app));
        cx.update(|window, app| window.draw(app).clear(app));
        (picked, cx)
    }

    #[gpui::test]
    fn color_swatches_are_one_roving_tab_stop(cx: &mut TestAppContext) {
        let (picked, cx) = open_picker(cx);
        focus_first(cx);
        // The Tab stop lands on the selected swatch (white, the last).
        press(cx, "enter");
        assert_eq!(*picked.borrow(), [Color::WHITE]);
        // Arrows move focus within the group; Space picks the focused swatch.
        press(cx, "left");
        press(cx, "space");
        assert_eq!(picked.borrow().len(), 2, "Space picked exactly once");
        assert_ne!(picked.borrow()[1], Color::WHITE);
        // One Tab leaves the group: there is no second stop to land on.
        press(cx, "tab");
        press(cx, "enter");
        assert_eq!(picked.borrow().len(), 2, "Tab left the swatch group");
    }

    #[gpui::test]
    fn color_swatch_ring_for_keyboard_focus_only(cx: &mut TestAppContext) {
        let (_, cx) = open_picker(cx);
        assert_eq!(outer_ring_quads(cx), 0);
        focus_first(cx);
        press(cx, "left");
        assert_eq!(outer_ring_quads(cx), 1);
        cx.click_selector("color-swatch-0");
        cx.update(|window, app| window.draw(app).clear(app));
        assert_eq!(outer_ring_quads(cx), 0, "mouse focus draws no ring");
    }
}

/// WI-4840: with Tab bound, the text-like controls (which also handle Tab in
/// their own key-down) must move focus exactly one stop per press.
#[cfg(test)]
mod single_move_tests {
    use std::{cell::RefCell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{
        div, AnyView, AppContext as _, Context, Entity, Focusable, IntoElement, ParentElement as _,
        Render, TestAppContext, VisualTestContext, Window,
    };

    use crate::{Button, Combobox, DateTimePicker, Searchbox, TextInput, Textarea, TokenInput};

    type Log = Rc<RefCell<Vec<&'static str>>>;

    /// `[before] [middle control] [after]`.
    struct Host(Log, AnyView);

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (a, b) = (self.0.clone(), self.0.clone());
            div()
                .child(
                    Button::new("before")
                        .label("Before")
                        .on_click(move |_, _, _| {
                            a.borrow_mut().push("before");
                        }),
                )
                .child(self.1.clone())
                .child(
                    Button::new("after")
                        .label("After")
                        .on_click(move |_, _, _| {
                            b.borrow_mut().push("after");
                        }),
                )
        }
    }

    fn check<T: Render + Focusable>(
        cx: &mut TestAppContext,
        name: &str,
        make: impl FnOnce(&mut Window, &mut gpui::App) -> Entity<T>,
    ) {
        let log: Log = Rc::default();
        let mut entity = None;
        let (_, cx) = open_fluent_test_window(cx, |window, cx| {
            let e = make(window, cx);
            entity = Some(e.clone());
            Host(log.clone(), e.into())
        });
        cx.update(|_, app| crate::install_focus_keybindings(app));
        settle(cx);
        let entity = entity.unwrap();
        cx.update(|window, app| {
            let handle = entity.read(app).focus_handle(app);
            window.focus(&handle, app);
        });
        settle(cx);
        press(cx, "tab");
        press(cx, "enter");
        assert_eq!(
            *log.borrow(),
            ["after"],
            "{name}: one Tab from the control must land on the next stop"
        );
        log.borrow_mut().clear();
        press(cx, "shift-tab");
        press(cx, "shift-tab");
        press(cx, "enter");
        assert_eq!(
            *log.borrow(),
            ["before"],
            "{name}: Shift-Tab moves back one stop each"
        );
    }

    /// Key-down then key-up, as a real key press (a focused button's click
    /// fires on the key release).
    fn press(cx: &mut VisualTestContext, key: &'static str) {
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
        settle(cx);
    }

    fn settle(cx: &mut VisualTestContext) {
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
    }

    #[gpui::test]
    fn text_input_moves_once(cx: &mut TestAppContext) {
        check(cx, "text-input", |_, app| app.new(|_| TextInput::new()));
    }
    #[gpui::test]
    fn textarea_moves_once(cx: &mut TestAppContext) {
        check(cx, "textarea", |_, app| app.new(|_| Textarea::new()));
    }
    #[gpui::test]
    fn searchbox_moves_once(cx: &mut TestAppContext) {
        check(cx, "searchbox", |_, app| app.new(|_| Searchbox::new()));
    }
    #[gpui::test]
    fn combobox_moves_once(cx: &mut TestAppContext) {
        check(cx, "combobox", |_, app| app.new(|_| Combobox::new()));
    }
    #[gpui::test]
    fn token_input_moves_once(cx: &mut TestAppContext) {
        check(cx, "token-input", |_, app| app.new(|_| TokenInput::new()));
    }
    #[gpui::test]
    fn date_time_picker_moves_once(cx: &mut TestAppContext) {
        check(cx, "date-time-picker", |_, app| {
            app.new(|_| DateTimePicker::new())
        });
    }
}
