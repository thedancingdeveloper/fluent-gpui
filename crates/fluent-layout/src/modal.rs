use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, AnyView, ClickEvent, Context, Entity, Global, IntoElement, MouseButton,
    Render, SharedString, Window,
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
pub struct ModalStack {
    /// Type-erased modal views, bottom to top.
    entries: Vec<(AnyView, ModalSize)>,
}

impl Global for ModalStack {}

impl ModalStack {
    pub fn init(cx: &mut gpui::App) {
        cx.set_global(ModalStack { entries: vec![] });
    }

    /// Push a new modal onto the stack.
    pub fn push(view: AnyView, size: ModalSize, cx: &mut gpui::App) {
        cx.global_mut::<ModalStack>().entries.push((view, size));
    }

    /// Pop the top modal.
    pub fn pop(cx: &mut gpui::App) {
        cx.global_mut::<ModalStack>().entries.pop();
    }

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
#[derive(Default)]
pub struct ModalHost {
    observing: bool,
}

impl ModalHost {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        cx.observe_global::<ModalStack>(|_, cx| cx.notify())
            .detach();
        Self { observing: true }
    }
}

impl Render for ModalHost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            cx.observe_global::<ModalStack>(|_, cx| cx.notify())
                .detach();
            self.observing = true;
        }

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let radii = theme.radii;

        let stack: Vec<(AnyView, ModalSize)> = cx
            .try_global::<ModalStack>()
            .map(|ms| ms.entries.clone())
            .unwrap_or_default();

        if stack.is_empty() {
            return div();
        }

        let (top_view, top_size) = stack.last().unwrap().clone();

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
            .absolute()
            .inset_0()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(backdrop_color)
            .on_mouse_down(MouseButton::Left, |_, _, cx| {
                cx.stop_propagation();
                ModalStack::pop(cx);
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
    pub title: SharedString,
    pub message: SharedString,
    pub confirm_label: SharedString,
    pub cancel_label: SharedString,
    on_confirm: Option<Box<dyn Fn(&mut gpui::App) + 'static>>,
    on_cancel: Option<Box<dyn Fn(&mut gpui::App) + 'static>>,
}

/// A persistent informational modal with a single dismiss action.
pub struct MessageDialog {
    pub title: SharedString,
    pub message: SharedString,
    pub dismiss_label: SharedString,
}

impl MessageDialog {
    pub fn new(title: impl Into<SharedString>, message: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            message: message.into(),
            dismiss_label: "OK".into(),
        }
    }

    pub fn dismiss_label(mut self, label: impl Into<SharedString>) -> Self {
        self.dismiss_label = label.into();
        self
    }

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
                            .text_size(px(typography.subtitle.size))
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
                    .text_size(px(typography.body.size))
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
                            .px(px(spacing.md))
                            .py(px(spacing.sm))
                            .rounded(px(radii.md))
                            .bg(colors.accent)
                            .text_color(colors.on_accent)
                            .text_size(px(typography.body.size))
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

    pub fn confirm_label(mut self, label: impl Into<SharedString>) -> Self {
        self.confirm_label = label.into();
        self
    }

    pub fn cancel_label(mut self, label: impl Into<SharedString>) -> Self {
        self.cancel_label = label.into();
        self
    }

    pub fn on_confirm(mut self, f: impl Fn(&mut gpui::App) + 'static) -> Self {
        self.on_confirm = Some(Box::new(f));
        self
    }

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
                    .text_size(px(typography.subtitle.size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.on_neutral)
                    .child(title),
            )
            .child(
                div()
                    .text_size(px(typography.body.size))
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
                            .text_size(px(typography.body.size))
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
                            .px(px(spacing.md))
                            .py(px(spacing.sm))
                            .rounded(px(radii.md))
                            .bg(colors.accent)
                            .text_size(px(typography.body.size))
                            .text_color(colors.on_accent)
                            .cursor_pointer()
                            .hover(move |s| s.bg(colors.accent_hover))
                            .on_click(confirm_handler)
                            .child(confirm_label),
                    ),
            )
    }
}
