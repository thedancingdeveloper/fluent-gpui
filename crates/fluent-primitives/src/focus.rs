use gpui::{actions, App, KeyBinding, KeyDownEvent, Window};

actions!(fluent_primitives, [FocusNextControl, FocusPreviousControl]);

pub fn install_focus_keybindings(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("tab", FocusNextControl, None),
        KeyBinding::new("shift-tab", FocusPreviousControl, None),
    ]);
}

pub(crate) fn focus_next(_: &FocusNextControl, window: &mut Window, _: &mut App) {
    window.focus_next();
}

pub(crate) fn focus_previous(_: &FocusPreviousControl, window: &mut Window, _: &mut App) {
    window.focus_prev();
}

pub(crate) fn handle_tab_navigation(ev: &KeyDownEvent, window: &mut Window) -> bool {
    if ev.keystroke.key != "tab" {
        return false;
    }

    if ev.keystroke.modifiers.shift {
        window.focus_prev();
    } else {
        window.focus_next();
    }
    true
}
