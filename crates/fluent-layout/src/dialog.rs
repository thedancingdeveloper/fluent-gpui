use fluent_core::TextRampExt;
use fluent_core::ThemeProvider as _;
use fluent_primitives::{Button, ButtonAppearance, ButtonSize, IconButton};
use gpui::{
    div, prelude::*, px, AnyElement, App, ClickEvent, IntoElement, RenderOnce, SharedString, Window,
};
use std::rc::Rc;

use crate::{ModalSize, ModalStack};

/// Fluent dialog content with title/body/actions layout.
///
/// ```no_run
/// use fluent_layout::{Dialog, DialogAction, ModalSize};
/// use gpui::{div, prelude::*, App};
///
/// fn confirm_delete(cx: &mut App) {
///     Dialog::new("Delete connection?")
///         .body(div().child("This cannot be undone."))
///         .action(DialogAction::new("Cancel", |_event, _window, _cx| {}))
///         .action(DialogAction::new("Delete", |_event, _window, _cx| {}).primary(true))
///         .show(ModalSize::Fixed(420.0, 180.0), cx);
/// }
/// ```
#[derive(IntoElement)]
pub struct Dialog {
    title: SharedString,
    body: Option<AnyElement>,
    actions: Vec<DialogAction>,
    dismissible: bool,
}

#[derive(Clone)]
#[allow(clippy::type_complexity)]
/// Public API item.
pub struct DialogAction {
    label: SharedString,
    primary: bool,
    on_click: Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
}

impl DialogAction {
    /// Public API item.
    pub fn new(
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            primary: false,
            on_click: Rc::new(on_click),
        }
    }

    /// Public API item.
    pub fn primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }
}

impl Dialog {
    /// Public API item.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            body: None,
            actions: Vec::new(),
            dismissible: true,
        }
    }

    /// Public API item.
    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    /// Public API item.
    pub fn action(mut self, action: DialogAction) -> Self {
        self.actions.push(action);
        self
    }

    /// Public API item.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Public API item.
    pub fn show(self, size: ModalSize, cx: &mut App) {
        let dismissible = self.dismissible;
        let entity = cx.new(|_| DialogView { dialog: self });
        ModalStack::push_with(entity.into(), size, dismissible, cx);
    }
}

struct DialogView {
    dialog: Dialog,
}

impl gpui::Render for DialogView {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        Dialog {
            title: self.dialog.title.clone(),
            body: self.dialog.body.take(),
            actions: self.dialog.actions.clone(),
            dismissible: self.dialog.dismissible,
        }
    }
}

impl RenderOnce for Dialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;

        let mut actions = div().flex().flex_row().justify_end().gap(px(spacing.sm));

        for (idx, action) in self.actions.into_iter().enumerate() {
            let handler = action.on_click;
            // The shared `Button` is a Tab stop with the keyboard focus ring and
            // Enter/Space activation; the wrapper keeps the test selector.
            actions = actions.child(
                div()
                    .debug_selector(move || format!("dialog-action-{idx}"))
                    .child(
                        Button::new(("dialog-action", idx as u64))
                            .label(action.label)
                            .appearance(if action.primary {
                                ButtonAppearance::Primary
                            } else {
                                ButtonAppearance::Secondary
                            })
                            .on_click(move |ev, win, app| handler(ev, win, app)),
                    ),
            );
        }

        let mut header = div()
            .flex()
            .flex_row()
            .items_start()
            .gap(px(spacing.md))
            .child(
                div()
                    .flex_1()
                    .text_ramp(&typography.subtitle)
                    .line_height(px(typography.subtitle.line_height))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.on_neutral)
                    .child(self.title),
            );

        if self.dismissible {
            header = header.child(
                IconButton::new("dialog-dismiss", "icons/dismiss.svg")
                    .label("Close")
                    .appearance(ButtonAppearance::Subtle)
                    .size(ButtonSize::Compact)
                    .on_click(|_, _, cx| ModalStack::pop(cx)),
            );
        }

        let mut root = div()
            .flex()
            .flex_col()
            .gap(px(spacing.lg))
            .p(px(spacing.xl))
            .child(header);

        if let Some(body) = self.body {
            root = root.child(body);
        }

        let motion = cx.theme().motion();
        fluent_core::popup_motion_surface("dialog-motion", false, motion, root.child(actions))
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Render, TestAppContext};

    use super::*;
    use crate::roving::test_support::{outer_ring_quads, press, tab};
    use crate::ModalHost;

    struct Root(gpui::Entity<ModalHost>);

    impl Render for Root {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().child(self.0.clone())
        }
    }

    /// Opens a dialog with a close button and two actions.
    fn open(
        cx: &mut TestAppContext,
        dismissible: bool,
    ) -> (Rc<Cell<usize>>, &mut gpui::VisualTestContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, cx| {
            ModalStack::init(cx);
            Root(cx.new(ModalHost::new))
        });
        let fired = Rc::new(Cell::new(0));
        let f = fired.clone();
        cx.update(|_, cx| {
            Dialog::new("D")
                .dismissible(dismissible)
                .action(DialogAction::new("Go", move |_, _, _| f.set(f.get() + 1)).primary(true))
                .action(DialogAction::new("Other", |_, _, _| {}))
                .show(ModalSize::Fit, cx);
        });
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
        (fired, cx)
    }

    fn is_open(cx: &mut gpui::VisualTestContext) -> bool {
        cx.update(|_, cx| !ModalStack::is_empty(cx))
    }

    #[gpui::test]
    fn close_button_is_a_tab_stop_that_enter_activates(cx: &mut TestAppContext) {
        let (_, cx) = open(cx, true);
        // The close button comes first in Tab order.
        tab(cx, 1);
        press(cx, "enter");
        assert!(!is_open(cx), "Enter on the focused close button dismisses");
    }

    #[gpui::test]
    fn actions_are_tab_stops_that_activate_once(cx: &mut TestAppContext) {
        let (fired, cx) = open(cx, true);
        tab(cx, 2); // close button, then the first action
        press(cx, "enter");
        assert_eq!(fired.get(), 1, "Enter activates exactly once");
        press(cx, "space");
        assert_eq!(fired.get(), 2, "Space activates");
        assert!(is_open(cx), "an action does not dismiss");
    }

    #[gpui::test]
    fn mouse_focus_draws_no_ring_on_dialog_controls(cx: &mut TestAppContext) {
        let (_, cx) = open(cx, true);
        cx.click_selector("dialog-action-1");
        cx.update(|window, app| window.draw(app).clear(app));
        assert_eq!(outer_ring_quads(cx), 0);
    }

    #[gpui::test]
    fn undismissible_dialog_has_no_close_stop(cx: &mut TestAppContext) {
        let (fired, cx) = open(cx, false);
        tab(cx, 1); // straight to the first action
        press(cx, "enter");
        assert_eq!(fired.get(), 1);
    }
}
