use fluent_app::FluentApp;
use fluent_core::ThemeProvider as _;
use fluent_primitives::{Button, ButtonAppearance, Label, LabelSize};
use gpui::{div, prelude::*, px, ClickEvent, Context, IntoElement, Render, Window};

struct Counter {
    count: u32,
}

impl Render for Counter {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        div()
            .size_full()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(24.0))
            .bg(colors.surface)
            .child(Label::new(format!("Count: {}", self.count)).size(LabelSize::Display))
            .child(
                div()
                    .flex()
                    .flex_row()
                    .gap(px(8.0))
                    .child(
                        Button::new("increment")
                            .label("Increment")
                            .appearance(ButtonAppearance::Accent)
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.count += 1;
                                cx.notify();
                            })),
                    )
                    .child(Button::new("reset").label("Reset").on_click(cx.listener(
                        |this, _: &ClickEvent, _, cx| {
                            this.count = 0;
                            cx.notify();
                        },
                    ))),
            )
    }
}

fn main() {
    FluentApp::new("Hello FluentGUI")
        .window_size(480.0, 320.0)
        .run(|cx| cx.new(|_| Counter { count: 0 }));
}
