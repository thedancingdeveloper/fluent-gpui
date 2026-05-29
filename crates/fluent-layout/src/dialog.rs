use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, AnyElement, App, ClickEvent, IntoElement, RenderOnce, SharedString,
    Window,
};

use crate::{ModalSize, ModalStack};

/// Fluent dialog content with title/body/actions layout.
#[derive(IntoElement)]
pub struct Dialog {
    title: SharedString,
    body: Option<AnyElement>,
    actions: Vec<DialogAction>,
    dismissible: bool,
}

#[allow(clippy::type_complexity)]
pub struct DialogAction {
    label: SharedString,
    primary: bool,
    on_click: Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
}

impl DialogAction {
    pub fn new(
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            label: label.into(),
            primary: false,
            on_click: Box::new(on_click),
        }
    }

    pub fn primary(mut self, primary: bool) -> Self {
        self.primary = primary;
        self
    }
}

impl Dialog {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            body: None,
            actions: Vec::new(),
            dismissible: true,
        }
    }

    pub fn body(mut self, body: impl IntoElement) -> Self {
        self.body = Some(body.into_any_element());
        self
    }

    pub fn action(mut self, action: DialogAction) -> Self {
        self.actions.push(action);
        self
    }

    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    pub fn show(self, size: ModalSize, cx: &mut App) {
        let entity = cx.new(|_| DialogView { dialog: Some(self) });
        ModalStack::push(entity.into(), size, cx);
    }
}

struct DialogView {
    dialog: Option<Dialog>,
}

impl gpui::Render for DialogView {
    fn render(&mut self, _window: &mut Window, _cx: &mut gpui::Context<Self>) -> impl IntoElement {
        self.dialog.take().unwrap_or_else(|| Dialog::new(""))
    }
}

impl RenderOnce for Dialog {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;

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
                    .px(px(spacing.md))
                    .py(px(spacing.sm))
                    .rounded(px(radii.md))
                    .bg(bg)
                    .border_1()
                    .border_color(colors.stroke_neutral)
                    .text_size(px(typography.body.size))
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
                    .text_size(px(typography.subtitle.size))
                    .line_height(px(typography.subtitle.line_height))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.on_neutral)
                    .child(self.title),
            );

        if self.dismissible {
            header = header.child(
                div()
                    .id("dialog-dismiss")
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

        root.child(actions)
    }
}
