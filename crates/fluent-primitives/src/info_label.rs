use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, Context, ElementId, IntoElement, Render, Role, SharedString, Window,
};

/// A field label with a keyboard-accessible info button and explanatory popover.
pub struct InfoLabel {
    id: ElementId,
    text: SharedString,
    content: SharedString,
    open: bool,
}

impl InfoLabel {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(
        id: impl Into<ElementId>,
        text: impl Into<SharedString>,
        content: impl Into<SharedString>,
    ) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
            content: content.into(),
            open: false,
        }
    }

    /// Returns the current text.
    pub fn text(&self) -> SharedString {
        self.text.clone()
    }
}

impl Render for InfoLabel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = theme.radii;
        let spacing = theme.spacing;
        let typography = theme.typography;
        let open = self.open;
        let debug_id = self.id.to_string();
        let hover_background = colors.subtle_hover;

        let button = div()
            .id(self.id.clone())
            .debug_selector(move || debug_id)
            .role(Role::Button)
            .aria_label(format!("More information about {}", self.text))
            .aria_expanded(open)
            .focusable()
            .tab_stop(true)
            .cursor_pointer()
            .flex()
            .items_center()
            .justify_center()
            .size(px(18.0))
            .rounded(px(9.0))
            .border_1()
            .border_color(colors.stroke_neutral)
            .text_size(px(typography.caption.size))
            .font_weight(gpui::FontWeight::SEMIBOLD)
            .text_color(colors.on_subtle)
            .hover(move |style| style.bg(hover_background))
            .on_click(cx.listener(|this, _, _, cx| {
                this.open = !this.open;
                cx.notify();
            }))
            .child("i");

        div()
            .relative()
            .flex()
            .items_center()
            .gap(px(spacing.xs))
            .child(self.text.clone())
            .child(button)
            .when(open, |root| {
                root.child(
                    div()
                        .debug_selector(|| "info-label-popover".into())
                        .absolute()
                        .top(px(typography.body.line_height + spacing.xs))
                        .left_0()
                        .w(px(280.0))
                        .px(px(spacing.sm))
                        .py(px(spacing.xs))
                        .rounded(px(radii.sm))
                        .bg(colors.surface_blur_layer)
                        .border_1()
                        .border_color(colors.stroke_neutral_dim)
                        .shadow_md()
                        .text_size(px(typography.caption.size))
                        .line_height(px(typography.caption.line_height))
                        .text_color(colors.on_neutral)
                        .child(self.content.clone()),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{InputEvent as _, KeyUpEvent, Keystroke, TestAppContext};

    use super::*;

    struct Host {
        label: gpui::Entity<InfoLabel>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            self.label.clone()
        }
    }

    fn press_enter(cx: &mut gpui::VisualTestContext) {
        cx.type_keys("enter");
        let keystroke = Keystroke::parse("enter").expect("valid enter keystroke");
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

    #[gpui::test]
    fn click_and_enter_toggle_popover(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, cx| Host {
            label: cx.new(|_| InfoLabel::new("server-info", "Server", "Where requests are sent.")),
        });
        cx.update(|window, app| window.draw(app).clear(app));

        assert!(cx.selector_bounds("info-label-popover").is_none());
        cx.click_selector("server-info");
        assert!(cx.selector_bounds("info-label-popover").is_some());
        cx.click_selector("server-info");
        assert!(cx.selector_bounds("info-label-popover").is_none());

        press_enter(cx);
        assert!(cx.selector_bounds("info-label-popover").is_some());
        press_enter(cx);
        assert!(cx.selector_bounds("info-label-popover").is_none());
    }
}
