use fluent_core::TextRampExt;
use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, AnyView, ClickEvent, Context, Entity, FocusHandle, Global, IntoElement,
    MouseButton, Render, SharedString, WeakFocusHandle, Window,
};

// ---------------------------------------------------------------------------
// Modal size
// ---------------------------------------------------------------------------

/// How a modal sizes itself.
#[derive(Clone, Copy, Debug)]
pub enum ModalSize {
    /// The modal wraps its content.
    Fit,
    /// Fixed pixel dimensions.
    Fixed(f32, f32),
    /// Fraction of the window (0.0–1.0).
    Fraction(f32, f32),
}

// ---------------------------------------------------------------------------
// Modal stack global
// ---------------------------------------------------------------------------

/// The global modal stack — accessed via `ModalStack::global(cx)`.
///
/// # Nested-overlay contract
///
/// - **Escape** is owned by the topmost modal: it pops that modal only, and the
///   key is consumed so nothing below (or a parent handler) also reacts.
/// - **Outside click** on the backdrop pops the topmost modal only; clicks never
///   reach the layer below.
/// - A modal pushed with `dismissible == false` ignores both Escape and backdrop
///   clicks; only its own actions close it.
/// - **Focus** is trapped inside the host while any modal is open (Tab does not
///   leave). The opener's focus is captured when the first modal opens and
///   restored only when the stack empties; popping an upper modal keeps focus in
///   the host so the modal underneath stays the active layer.
pub struct ModalStack {
    /// Type-erased modal views and their dismissibility, bottom to top.
    entries: Vec<(AnyView, ModalSize, bool)>,
}

impl Global for ModalStack {}

impl ModalStack {
    /// Public API item.
    pub fn init(cx: &mut gpui::App) {
        cx.set_global(ModalStack { entries: vec![] });
    }

    /// Push a new modal onto the stack.
    pub fn push(view: AnyView, size: ModalSize, cx: &mut gpui::App) {
        Self::push_with(view, size, true, cx);
    }

    /// Push a modal, choosing whether Escape and backdrop clicks may dismiss it.
    pub fn push_with(view: AnyView, size: ModalSize, dismissible: bool, cx: &mut gpui::App) {
        cx.global_mut::<ModalStack>()
            .entries
            .push((view, size, dismissible));
    }

    /// Pop the top modal only if it is dismissible (Escape / backdrop policy).
    fn pop_if_dismissible(cx: &mut gpui::App) {
        let stack = cx.global_mut::<ModalStack>();
        if stack.entries.last().is_some_and(|e| e.2) {
            stack.entries.pop();
        }
    }

    /// Pop the top modal.
    pub fn pop(cx: &mut gpui::App) {
        cx.global_mut::<ModalStack>().entries.pop();
    }

    /// Public API item.
    pub fn is_empty(cx: &gpui::App) -> bool {
        cx.global::<ModalStack>().entries.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Modal host — renders the stack as deferred overlays
// ---------------------------------------------------------------------------

/// Renders the active modal stack as a deferred overlay inside `Workspace`.
///
/// Include one `ModalHost` as a child of your `Workspace`; it will render
/// any active modals above all other content.
pub struct ModalHost {
    observing: bool,
    focus_handle: FocusHandle,
    prior_focus: Option<WeakFocusHandle>,
    had_modal: bool,
}

impl ModalHost {
    /// Public API item.
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        cx.observe_global::<ModalStack>(|_, cx| cx.notify())
            .detach();
        Self {
            observing: true,
            focus_handle: cx.focus_handle(),
            prior_focus: None,
            had_modal: false,
        }
    }
}

impl Render for ModalHost {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            cx.observe_global::<ModalStack>(|_, cx| cx.notify())
                .detach();
            self.observing = true;
        }

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let radii = theme.radii;

        let stack: Vec<(AnyView, ModalSize, bool)> = cx
            .try_global::<ModalStack>()
            .map(|ms| ms.entries.clone())
            .unwrap_or_default();

        if stack.is_empty() {
            if self.had_modal {
                match self.prior_focus.take().and_then(|focus| focus.upgrade()) {
                    Some(prior) => window.focus(&prior, cx),
                    None => window.blur(cx),
                }
                self.had_modal = false;
            }
            return div();
        }

        if !self.had_modal {
            self.prior_focus = window.focused(cx).map(|focus| focus.downgrade());
            window.focus(&self.focus_handle, cx);
            self.had_modal = true;
        }

        // Keep focus inside the host when an upper modal closes over a lower one.
        if !self.focus_handle.contains_focused(window, cx) {
            window.focus(&self.focus_handle, cx);
        }

        let (top_view, top_size, _) = stack.last().unwrap().clone();

        let backdrop_color = {
            let mut c = colors.surface_dim;
            c.a = 0.6;
            c
        };

        // Modal container dimensions
        let (modal_w, modal_h): (gpui::Length, gpui::Length) = match top_size {
            ModalSize::Fit => (gpui::Length::Auto, gpui::Length::Auto),
            ModalSize::Fixed(w, h) => (px(w).into(), px(h).into()),
            ModalSize::Fraction(fw, fh) => (gpui::relative(fw).into(), gpui::relative(fh).into()),
        };

        let modal_bg = colors.surface;
        let modal_border = colors.stroke_neutral;

        // Absolute overlay filling the workspace's .relative() root. ModalHost is always the last
        // child of Workspace's root div, so natural paint order puts it above all siblings without
        // needing deferred(). Removing deferred() here allows modals to host Dropdown/anchored
        // popups (which use deferred() internally) without triggering GPUI's nested-deferred panic.
        div()
            .track_focus(&self.focus_handle)
            .on_key_down(|event, window, cx| match event.keystroke.key.as_str() {
                "escape" => {
                    cx.stop_propagation();
                    ModalStack::pop_if_dismissible(cx);
                }
                "tab" => {
                    cx.stop_propagation();
                    if let Some(focus) = window.focused(cx) {
                        window.focus(&focus, cx);
                    }
                }
                _ => {}
            })
            .absolute()
            .inset_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(backdrop_color)
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
                ModalStack::pop_if_dismissible(cx);
            })
            .child(
                div()
                    .bg(modal_bg)
                    .border_1()
                    .border_color(modal_border)
                    .rounded(px(radii.lg))
                    .w(modal_w)
                    .h(modal_h)
                    .on_mouse_down(MouseButton::Left, |_, _, cx| {
                        cx.stop_propagation();
                    })
                    .child(top_view),
            )
    }
}

// ---------------------------------------------------------------------------
// Confirm modal
// ---------------------------------------------------------------------------

/// A generic yes/no confirmation modal.
#[allow(clippy::type_complexity)]
pub struct ConfirmModal {
    /// Public API item.
    pub title: SharedString,
    /// Public API item.
    pub message: SharedString,
    /// Public API item.
    pub confirm_label: SharedString,
    /// Public API item.
    pub cancel_label: SharedString,
    on_confirm: Option<Box<dyn Fn(&mut gpui::App) + 'static>>,
    on_cancel: Option<Box<dyn Fn(&mut gpui::App) + 'static>>,
}

/// A persistent informational modal with a single dismiss action.
pub struct MessageDialog {
    /// Public API item.
    pub title: SharedString,
    /// Public API item.
    pub message: SharedString,
    /// Public API item.
    pub dismiss_label: SharedString,
}

impl MessageDialog {
    /// Public API item.
    pub fn new(title: impl Into<SharedString>, message: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            dismiss_label: "OK".into(),
        }
    }

    /// Public API item.
    pub fn dismiss_label(mut self, label: impl Into<SharedString>) -> Self {
        self.dismiss_label = label.into();
        self
    }

    /// Public API item.
    pub fn show(self, size: ModalSize, cx: &mut gpui::App) -> Entity<Self> {
        let entity = cx.new(|_| self);
        ModalStack::push(entity.clone().into(), size, cx);
        entity
    }
}

impl Render for MessageDialog {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;

        let title = self.title.clone();
        let message = self.message.clone();
        let dismiss_label = self.dismiss_label.clone();

        div()
            .size_full()
            .bg(colors.surface)
            .flex()
            .flex_col()
            .overflow_hidden()
            .child(
                div()
                    .flex_none()
                    .px(px(spacing.lg))
                    .py(px(spacing.md))
                    .border_b_1()
                    .border_color(colors.stroke_neutral_subtle)
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_between()
                    .gap(px(spacing.md))
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .text_ramp(&typography.subtitle)
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(colors.on_neutral)
                            .child(title),
                    )
                    .child(
                        div()
                            .id("message-dialog-dismiss")
                            .flex_none()
                            .size(px(28.0))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(radii.sm))
                            .cursor_pointer()
                            .hover({
                                let hover = colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_click(|_, _, cx| ModalStack::pop(cx))
                            .child(
                                gpui::svg()
                                    .path("icons/dismiss.svg")
                                    .size(px(12.0))
                                    .text_color(colors.on_subtle),
                            ),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_hidden()
                    .p(px(spacing.lg))
                    .text_ramp(&typography.body)
                    .line_height(px(typography.body.line_height))
                    .text_color(colors.on_subtle)
                    .child(message),
            )
            .child(
                div()
                    .flex_none()
                    .px(px(spacing.lg))
                    .py(px(spacing.md))
                    .border_t_1()
                    .border_color(colors.stroke_neutral_subtle)
                    .flex()
                    .justify_end()
                    .child(
                        div()
                            .id("message-dialog-ok")
                            .debug_selector(|| "message-dialog-ok".into())
                            .px(px(spacing.md))
                            .py(px(spacing.sm))
                            .rounded(px(radii.md))
                            .bg(colors.accent)
                            .text_color(colors.on_accent)
                            .text_ramp(&typography.body)
                            .cursor_pointer()
                            .hover({
                                let hover = colors.accent_hover;
                                move |s| s.bg(hover)
                            })
                            .on_click(|_, _, cx| ModalStack::pop(cx))
                            .child(dismiss_label),
                    ),
            )
    }
}

impl ConfirmModal {
    /// Public API item.
    pub fn new(title: impl Into<SharedString>, message: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            confirm_label: "OK".into(),
            cancel_label: "Cancel".into(),
            on_confirm: None,
            on_cancel: None,
        }
    }

    /// Public API item.
    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.confirm_label = label.into();
        self
    }

    /// Public API item.
    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }

    /// Public API item.
    pub fn on_confirm(mut self, f: impl Fn(&mut gpui::App) + 'static) -> Self {
        self.on_confirm = Some(Box::new(f));
        self
    }

    /// Public API item.
    pub fn on_cancel(mut self, f: impl Fn(&mut gpui::App) + 'static) -> Self {
        self.on_cancel = Some(Box::new(f));
        self
    }

    /// Create the entity and push it onto the modal stack.
    pub fn show(self, cx: &mut gpui::App) -> Entity<Self> {
        let entity = cx.new(|_| self);
        let view: AnyView = entity.clone().into();
        ModalStack::push(view, ModalSize::Fixed(400.0, 180.0), cx);
        entity
    }
}

impl Render for ConfirmModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;

        let title = self.title.clone();
        let message = self.message.clone();
        let confirm_label = self.confirm_label.clone();
        let cancel_label = self.cancel_label.clone();

        let confirm_handler = cx.listener(|modal: &mut ConfirmModal, _: &ClickEvent, _, cx| {
            if let Some(f) = modal.on_confirm.take() {
                f(cx);
            }
            ModalStack::pop(cx);
        });

        let cancel_handler = cx.listener(|modal: &mut ConfirmModal, _: &ClickEvent, _, cx| {
            if let Some(f) = modal.on_cancel.take() {
                f(cx);
            }
            ModalStack::pop(cx);
        });

        div()
            .flex()
            .flex_col()
            .gap(px(spacing.lg))
            .p(px(spacing.xl))
            .child(
                div()
                    .text_ramp(&typography.subtitle)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.on_neutral)
                    .child(title),
            )
            .child(
                div()
                    .text_ramp(&typography.body)
                    .text_color(colors.on_subtle)
                    .child(message),
            )
            .child(
                div()
                    .flex()
                    .flex_row()
                    .justify_end()
                    .gap(px(spacing.sm))
                    .child(
                        // Cancel
                        div()
                            .id("modal-cancel")
                            .px(px(spacing.md))
                            .py(px(spacing.sm))
                            .rounded(px(radii.md))
                            .bg(colors.neutral)
                            .border_1()
                            .border_color(colors.stroke_neutral)
                            .text_ramp(&typography.body)
                            .text_color(colors.on_neutral)
                            .cursor_pointer()
                            .hover(move |s| s.bg(colors.neutral_hover))
                            .on_click(cancel_handler)
                            .child(cancel_label),
                    )
                    .child(
                        // Confirm
                        div()
                            .id("modal-confirm")
                            .debug_selector(|| "modal-confirm".into())
                            .px(px(spacing.md))
                            .py(px(spacing.sm))
                            .rounded(px(radii.md))
                            .bg(colors.accent)
                            .text_ramp(&typography.body)
                            .text_color(colors.on_accent)
                            .cursor_pointer()
                            .hover(move |s| s.bg(colors.accent_hover))
                            .on_click(confirm_handler)
                            .child(confirm_label),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use fluent_test_utils::{
        open_fluent_test_window, FluentAssertExt as _, FluentInteractionExt as _,
    };
    use gpui::{point, px, Modifiers, TestAppContext};

    use super::*;
    use crate::{Dialog, DialogAction};

    /// A relative root holding a clickable button under a `ModalHost`.
    struct Root {
        host: Entity<ModalHost>,
        clicks: Rc<Cell<usize>>,
        opener: FocusHandle,
    }

    impl Render for Root {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            div()
                .relative()
                .size_full()
                .child(
                    div()
                        .id("below")
                        .debug_selector(|| "below".into())
                        .size(px(100.))
                        .track_focus(&self.opener)
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                )
                .child(self.host.clone())
        }
    }

    fn root(cx: &mut TestAppContext) -> (Rc<Cell<usize>>, &mut gpui::VisualTestContext) {
        let clicks = Rc::new(Cell::new(0));
        let c = clicks.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, cx| {
            cx.set_global(ModalStack { entries: vec![] });
            Root {
                host: cx.new(ModalHost::new),
                clicks: c,
                opener: cx.focus_handle(),
            }
        });
        (clicks, cx)
    }

    fn stack_len(cx: &mut gpui::VisualTestContext) -> usize {
        cx.update(|_, cx| cx.global::<ModalStack>().entries.len())
    }

    #[gpui::test]
    fn show_pushes_and_ok_button_pops(cx: &mut TestAppContext) {
        let (_, cx) = root(cx);
        cx.update(|_, cx| {
            MessageDialog::new("T", "body").show(ModalSize::Fixed(400., 180.), cx);
        });
        cx.run_until_parked();
        assert_eq!(stack_len(cx), 1);
        cx.update(|_, cx| assert!(!ModalStack::is_empty(cx)));
        cx.click_selector("message-dialog-ok");
        assert_eq!(stack_len(cx), 0);
    }

    #[gpui::test]
    fn backdrop_click_dismisses_top_modal_only(cx: &mut TestAppContext) {
        let (_, cx) = root(cx);
        cx.update(|_, cx| {
            MessageDialog::new("one", "a").show(ModalSize::Fixed(400., 180.), cx);
            MessageDialog::new("two", "b").show(ModalSize::Fixed(400., 180.), cx);
        });
        cx.run_until_parked();
        assert_eq!(stack_len(cx), 2);
        cx.click_at(point(px(2.), px(2.)), MouseButton::Left, Modifiers::none());
        assert_eq!(stack_len(cx), 1);
    }

    #[gpui::test]
    fn click_inside_modal_does_not_dismiss(cx: &mut TestAppContext) {
        let (_, cx) = root(cx);
        cx.update(|_, cx| {
            MessageDialog::new("T", "body").show(ModalSize::Fixed(400., 180.), cx);
        });
        cx.run_until_parked();
        let b = cx.update(|w, _| w.viewport_size());
        let centre = point(b.width / 2., b.height / 2.);
        cx.click_at(centre, MouseButton::Left, Modifiers::none());
        assert_eq!(stack_len(cx), 1);
    }

    #[gpui::test]
    fn modal_blocks_clicks_to_layer_below(cx: &mut TestAppContext) {
        let (clicks, cx) = root(cx);
        cx.click_selector("below");
        assert_eq!(clicks.get(), 1, "button works with no modal open");

        cx.update(|_, cx| {
            MessageDialog::new("T", "body").show(ModalSize::Fixed(400., 180.), cx);
        });
        cx.run_until_parked();
        // The button lies under the backdrop's top-left corner.
        cx.click_at(
            point(px(50.), px(50.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        assert_eq!(clicks.get(), 1, "click must not reach the layer below");
        assert_eq!(stack_len(cx), 0, "it dismisses the modal instead");
    }

    #[gpui::test]
    fn confirm_runs_callback_once_and_pops(cx: &mut TestAppContext) {
        let (_, cx) = root(cx);
        let confirmed = Rc::new(Cell::new(0));
        let cancelled = Rc::new(Cell::new(0));
        let (a, b) = (confirmed.clone(), cancelled.clone());
        cx.update(|_, cx| {
            ConfirmModal::new("T", "m")
                .on_confirm(move |_| a.set(a.get() + 1))
                .on_cancel(move |_| b.set(b.get() + 1))
                .show(cx);
        });
        cx.run_until_parked();
        cx.click_selector("modal-confirm");
        assert_eq!((confirmed.get(), cancelled.get()), (1, 0));
        assert_eq!(stack_len(cx), 0);
    }

    #[gpui::test]
    fn dialog_close_button_dismisses(cx: &mut TestAppContext) {
        let (_, cx) = root(cx);
        cx.update(|_, cx| Dialog::new("D").show(ModalSize::Fit, cx));
        cx.run_until_parked();
        assert_eq!(stack_len(cx), 1);
        cx.click_selector("dialog-dismiss");
        assert_eq!(stack_len(cx), 0);
    }

    #[gpui::test]
    fn dialog_action_runs_and_does_not_dismiss(cx: &mut TestAppContext) {
        let (_, cx) = root(cx);
        let fired = Rc::new(Cell::new(0));
        let f = fired.clone();
        cx.update(|_, cx| {
            Dialog::new("D")
                .action(DialogAction::new("Go", move |_, _, _| f.set(f.get() + 1)).primary(true))
                .show(ModalSize::Fit, cx);
        });
        cx.run_until_parked();
        cx.click_selector("dialog-action-0");
        assert_eq!(fired.get(), 1);
        assert_eq!(stack_len(cx), 1, "an action does not itself dismiss");
    }

    #[gpui::test]
    fn escape_dismisses_dialog_and_restores_focus(cx: &mut TestAppContext) {
        let clicks = Rc::new(Cell::new(0));
        let c = clicks.clone();
        let (root, cx) = open_fluent_test_window(cx, move |_, cx| {
            cx.set_global(ModalStack { entries: vec![] });
            Root {
                host: cx.new(ModalHost::new),
                clicks: c,
                opener: cx.focus_handle(),
            }
        });
        let opener = root.read_with(cx, |root, _| root.opener.clone());
        cx.update(|window, cx| window.focus(&opener, cx));
        cx.assert_focused(&opener);

        cx.update(|_, cx| Dialog::new("D").show(ModalSize::Fit, cx));
        cx.run_until_parked();
        cx.assert_not_focused(&opener);
        cx.type_keys("escape");

        assert_eq!(stack_len(cx), 0);
        cx.assert_focused(&opener);
    }

    #[gpui::test]
    fn dialog_traps_focus_until_dismissed(cx: &mut TestAppContext) {
        let clicks = Rc::new(Cell::new(0));
        let c = clicks.clone();
        let (root, cx) = open_fluent_test_window(cx, move |_, cx| {
            cx.set_global(ModalStack { entries: vec![] });
            Root {
                host: cx.new(ModalHost::new),
                clicks: c,
                opener: cx.focus_handle(),
            }
        });
        let opener = root.read_with(cx, |root, _| root.opener.clone());
        cx.update(|window, cx| window.focus(&opener, cx));
        cx.update(|_, cx| Dialog::new("D").show(ModalSize::Fit, cx));
        cx.run_until_parked();

        cx.assert_keys_yield("tab shift-tab", |cx| cx.is_focused(&opener), false);
        assert_eq!(stack_len(cx), 1);

        cx.type_keys("escape");
        cx.assert_focused(&opener);
    }

    #[gpui::test]
    fn non_dismissible_dialog_has_no_close_button(cx: &mut TestAppContext) {
        let (_, cx) = root(cx);
        cx.update(|_, cx| Dialog::new("D").dismissible(false).show(ModalSize::Fit, cx));
        cx.run_until_parked();
        assert!(cx.selector_bounds("dialog-dismiss").is_none());
    }

    #[gpui::test]
    fn nested_escape_pops_top_only_and_keeps_focus_trapped(cx: &mut TestAppContext) {
        let clicks = Rc::new(Cell::new(0));
        let c = clicks.clone();
        let (root, cx) = open_fluent_test_window(cx, move |_, cx| {
            cx.set_global(ModalStack { entries: vec![] });
            Root {
                host: cx.new(ModalHost::new),
                clicks: c,
                opener: cx.focus_handle(),
            }
        });
        let opener = root.read_with(cx, |root, _| root.opener.clone());
        cx.update(|window, cx| window.focus(&opener, cx));
        cx.update(|_, cx| {
            Dialog::new("one").show(ModalSize::Fit, cx);
            Dialog::new("two").show(ModalSize::Fit, cx);
        });
        cx.run_until_parked();
        assert_eq!(stack_len(cx), 2);

        cx.type_keys("escape");
        assert_eq!(stack_len(cx), 1, "escape pops only the top modal");
        cx.assert_not_focused(&opener);

        cx.type_keys("escape");
        assert_eq!(stack_len(cx), 0);
        cx.assert_focused(&opener);
    }

    #[gpui::test]
    fn non_dismissible_top_ignores_escape_and_backdrop(cx: &mut TestAppContext) {
        let (_, cx) = root(cx);
        cx.update(|_, cx| {
            Dialog::new("base").show(ModalSize::Fit, cx);
            Dialog::new("locked")
                .dismissible(false)
                .show(ModalSize::Fit, cx);
        });
        cx.run_until_parked();
        cx.type_keys("escape");
        assert_eq!(stack_len(cx), 2);
        cx.click_at(point(px(2.), px(2.)), MouseButton::Left, Modifiers::none());
        assert_eq!(stack_len(cx), 2);
    }
}
