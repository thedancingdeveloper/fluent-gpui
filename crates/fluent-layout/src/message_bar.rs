use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, App, ClickEvent, IntoElement, RenderOnce, Role, SharedString, Window,
};

type DismissHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum MessageIntent {
    #[default]
    Info,
    Success,
    Warning,
    Error,
}

/// Inline status message surface.
#[derive(IntoElement)]
pub struct MessageBar {
    intent: MessageIntent,
    title: Option<SharedString>,
    message: SharedString,
    on_dismiss: Option<DismissHandler>,
    action: Option<(SharedString, DismissHandler)>,
}

impl MessageBar {
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            intent: MessageIntent::Info,
            title: None,
            message: message.into(),
            on_dismiss: None,
            action: None,
        }
    }

    pub fn intent(mut self, intent: MessageIntent) -> Self {
        self.intent = intent;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    /// A recovery affordance such as "Retry", shown after the message.
    pub fn action(
        mut self,
        label: impl Into<SharedString>,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.action = Some((label.into(), Box::new(f)));
        self
    }

    pub fn on_dismiss(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Box::new(f));
        self
    }
}

impl RenderOnce for MessageBar {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let accent = match self.intent {
            MessageIntent::Info => colors.status_info,
            MessageIntent::Success => colors.status_success,
            MessageIntent::Warning => colors.status_warning,
            MessageIntent::Error => colors.status_error,
        };
        let bg = match self.intent {
            MessageIntent::Info => colors.status_info_bg,
            MessageIntent::Success => colors.status_success_bg,
            MessageIntent::Warning => colors.status_warning_bg,
            MessageIntent::Error => colors.status_error_bg,
        };
        let border = match self.intent {
            MessageIntent::Info => colors.status_info_border,
            MessageIntent::Success => colors.status_success_border,
            MessageIntent::Warning => colors.status_warning_border,
            MessageIntent::Error => colors.status_error_border,
        };

        let mut text = div().flex().flex_col().gap(px(spacing.xs));
        if let Some(title) = self.title {
            text = text.child(
                div()
                    .text_size(px(typography.body.size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.on_neutral)
                    .child(title),
            );
        }
        text = text.child(
            div()
                .text_size(px(typography.body.size))
                .line_height(px(typography.body.line_height))
                .text_color(colors.on_subtle)
                .child(self.message),
        );

        let role = match self.intent {
            MessageIntent::Error | MessageIntent::Warning => Role::Alert,
            _ => Role::Status,
        };
        let mut root = div()
            .id("message-bar")
            .role(role)
            .flex()
            .flex_row()
            .items_start()
            .gap(px(spacing.md))
            .p(px(spacing.md))
            .rounded(px(radii.md))
            .bg(bg)
            .border_1()
            .border_color(border)
            .child(div().w(px(3.0)).rounded(px(radii.pill)).bg(accent))
            .child(text.flex_1());

        if let Some((label, on_action)) = self.action {
            let hover = colors.subtle_hover;
            root = root.child(
                div()
                    .id("message-action")
                    .flex_none()
                    .px(px(spacing.sm))
                    .py(px(2.0))
                    .rounded(px(radii.sm))
                    .role(Role::Button)
                    .aria_label(label.clone())
                    .cursor_pointer()
                    .text_size(px(typography.body.size))
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(accent)
                    .hover(move |s| s.bg(hover))
                    .on_click(move |ev, win, app| on_action(ev, win, app))
                    .child(label),
            );
        }

        if let Some(on_dismiss) = self.on_dismiss {
            root = root.child(
                div()
                    .id("message-dismiss")
                    .flex_none()
                    .size(px(20.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(radii.sm))
                    .cursor_pointer()
                    .hover({
                        let hover = colors.subtle_hover;
                        move |s| s.bg(hover)
                    })
                    .on_click(move |ev, win, app| on_dismiss(ev, win, app))
                    .child(
                        svg()
                            .path("icons/dismiss.svg")
                            .size(px(10.0))
                            .text_color(colors.on_subtle),
                    ),
            );
        }

        root
    }
}
