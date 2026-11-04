use fluent_core::TextRampExt;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, AnyElement, App, ClickEvent, IntoElement, RenderOnce, SharedString,
    Window,
};
use std::rc::Rc;

use crate::{ModalSize, ModalStack};

/// Fluent dialog content with title/body/actions layout.
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
        let radii = theme.radii;
        let components = theme.components;

        let mut actions = div().flex().flex_row().justify_end().gap(px(spacing.sm));

        for (idx, action) in self.actions.into_iter().enumerate() {
            let bg = if action.primary {
                colors.accent
            } else {
                colors.neutral
            };
            let hover = if action.primary {
                colors.accent_hover
            } else {
                colors.neutral_hover
            };
            let fg = if action.primary {
                colors.on_accent
            } else {
                colors.on_neutral
            };
            let handler = action.on_click;
            actions = actions.child(
                div()
                    .id(("dialog-action", idx as u64))
                    .debug_selector(|| format!("dialog-action-{idx}"))
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .h(px(components.button_medium_height))
                    .px(px(components.button_medium_padding_x))
                    .rounded(px(radii.md))
                    .bg(bg)
                    .border_1()
                    .border_color(colors.stroke_neutral)
                    .text_ramp(&typography.body)
                    .text_color(fg)
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover))
                    .on_click(move |ev, win, app| handler(ev, win, app))
                    .child(action.label),
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
                div()
                    .id("dialog-dismiss")
                    .debug_selector(|| "dialog-dismiss".into())
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
                        svg()
                            .path("icons/dismiss.svg")
                            .size(px(12.0))
                            .text_color(colors.on_subtle),
                    ),
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
