use fluent_core::{TextRampExt as _, ThemeProvider as _, TruncateExt as _};
use gpui::{
    div, prelude::*, px, svg, App, ClickEvent, ElementId, IntoElement, RenderOnce, Role,
    SharedString, Window,
};

type DismissHandler = Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Public API item.
pub enum MessageIntent {
    #[default]
    /// Public API item.
    Info,
    /// Public API item.
    Success,
    /// Public API item.
    Warning,
    /// Public API item.
    Error,
}

/// Inline status message surface.
#[derive(IntoElement)]
pub struct MessageBar {
    id: ElementId,
    intent: MessageIntent,
    title: Option<SharedString>,
    message: SharedString,
    on_dismiss: Option<DismissHandler>,
    action: Option<(SharedString, DismissHandler)>,
}

impl MessageBar {
    /// Public API item.
    pub fn new(message: impl Into<SharedString>) -> Self {
        Self {
            id: "message-bar".into(),
            intent: MessageIntent::Info,
            title: None,
            message: message.into(),
            on_dismiss: None,
            action: None,
        }
    }

    /// Sets the element id, which is also the id of the bar's accessibility node.
    /// Give each message bar that shares a parent its own id; two with the same
    /// id collide in the accessibility tree. Defaults to `"message-bar"`.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Public API item.
    pub fn intent(mut self, intent: MessageIntent) -> Self {
        self.intent = intent;
        self
    }

    /// Public API item.
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

    /// Public API item.
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

        let mut text = div().flex().flex_col().min_w_0().gap(px(spacing.xs));
        if let Some(title) = self.title {
            text = text.child(
                div()
                    .text_ramp(&typography.body)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(colors.on_neutral)
                    .truncate_label()
                    .child(title),
            );
        }
        text = text.child(
            div()
                .text_ramp(&typography.body)
                .line_height(px(typography.body.line_height))
                .text_color(colors.on_subtle)
                .child(self.message),
        );

        let role = match self.intent {
            MessageIntent::Error | MessageIntent::Warning => Role::Alert,
            _ => Role::Status,
        };
        let mut root = div()
            .id(self.id)
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
                    .text_ramp(&typography.body)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .text_color(accent)
                    .hover(move |s| s.bg(hover))
                    .active(move |s| s.bg(colors.subtle_selected))
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
                    .active({
                        let pressed = colors.subtle_selected;
                        move |s| s.bg(pressed)
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

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{div, Context, Render, TestAppContext};

    struct TwoHost;

    impl Render for TwoHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .flex()
                .flex_col()
                .child(MessageBar::new("One").id("bar-a"))
                .child(MessageBar::new("Two").id("bar-b").on_dismiss(|_, _, _| {}))
        }
    }

    /// Two instances in one window used to trip the duplicate accessibility
    /// node id assertion (debug builds); in-process, no real window or GPU.
    #[gpui::test]
    fn two_message_bars_with_distinct_ids_render_in_one_window(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, _| TwoHost);
        let snapshot = cx.accessibility_snapshot();
        assert!(!snapshot.is_empty());
    }
}
