use fluent_app::FluentApp;
use fluent_core::ThemeProvider as _;
use fluent_primitives::{
    Button, ButtonAppearance, Checkbox, CheckboxState, Divider, Dropdown, Label, LabelSize, Switch,
    TextInput,
};
use gpui::{
    div, prelude::*, px, ClickEvent, Context, Entity, IntoElement, Render, SharedString, Window,
};

struct FormDemo {
    name: Entity<TextInput>,
    host: Entity<TextInput>,
    port: Entity<TextInput>,
    protocol_dropdown: Entity<Dropdown>,
    remember_password: CheckboxState,
    auto_connect: CheckboxState,
    notifications: bool,
    dark_mode: bool,
    result_msg: Option<SharedString>,
}

impl FormDemo {
    fn new(cx: &mut gpui::App) -> Self {
        Self {
            name: cx.new(|_| TextInput::new().placeholder("e.g. My Home Server")),
            host: cx.new(|_| TextInput::new().placeholder("e.g. 192.168.1.100")),
            port: cx.new(|_| TextInput::new().placeholder("e.g. 22")),
            protocol_dropdown: cx.new(|_| {
                Dropdown::new("dd-protocol")
                    .selected("ssh")
                    .option("ssh", "SSH")
                    .option("rdp", "RDP")
                    .option("vnc", "VNC")
            }),
            remember_password: CheckboxState::Unchecked,
            auto_connect: CheckboxState::Unchecked,
            notifications: true,
            dark_mode: true,
            result_msg: None,
        }
    }
}

impl Render for FormDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let spacing = cx.theme().spacing;
        let entity = cx.entity();
        let e1 = entity.clone();
        let e2 = entity.clone();
        let e3 = entity.clone();
        let e4 = entity.clone();

        // ── field helper ──────────────────────────────────────────────────
        // label above input, full width
        let field = |label_text: &'static str, input: Entity<TextInput>| {
            div()
                .flex()
                .flex_col()
                .gap(px(4.0))
                .child(
                    Label::new(label_text)
                        .size(LabelSize::Caption)
                        .color(colors.on_subtle),
                )
                .child(input)
        };

        // ── row helper (label + control, space-between) ────────────────────
        let control_row = |label_text: &'static str, control: gpui::AnyElement| {
            div()
                .flex()
                .flex_row()
                .justify_between()
                .items_center()
                .child(Label::new(label_text))
                .child(control)
        };

        div()
            .size_full()
            .bg(colors.surface_dim)
            .flex()
            .items_center()
            .justify_center()
            .child(
                div()
                    .w(px(440.0))
                    .bg(colors.surface)
                    .rounded(px(8.0))
                    .border_1()
                    .border_color(colors.stroke_neutral_dim)
                    .p(px(28.0))
                    .flex()
                    .flex_col()
                    .gap(px(spacing.md))
                    // ── title ─────────────────────────────────────────────
                    .child(Label::new("Connection Settings").size(LabelSize::Title))
                    .child(Divider::horizontal())
                    // ── basic fields ──────────────────────────────────────
                    .child(field("Name *", self.name.clone()))
                    .child(field("Host / IP *", self.host.clone()))
                    .child(field("Port", self.port.clone()))
                    // ── protocol dropdown ─────────────────────────────────
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(4.0))
                            .child(
                                Label::new("Protocol")
                                    .size(LabelSize::Caption)
                                    .color(colors.on_subtle),
                            )
                            .child(self.protocol_dropdown.clone()),
                    )
                    .child(Divider::horizontal())
                    // ── options ───────────────────────────────────────────
                    .child(Label::new("Options").size(LabelSize::Subtitle))
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .items_center()
                            .child(
                                Checkbox::new("cb-remember")
                                    .state(self.remember_password)
                                    .on_click(move |new_state, _, _, cx| {
                                        e1.update(cx, |this, cx| {
                                            this.remember_password = new_state;
                                            cx.notify();
                                        });
                                    }),
                            )
                            .child(Label::new("Remember password")),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .gap(px(8.0))
                            .items_center()
                            .child(Checkbox::new("cb-auto").state(self.auto_connect).on_click(
                                move |new_state, _, _, cx| {
                                    e2.update(cx, |this, cx| {
                                        this.auto_connect = new_state;
                                        cx.notify();
                                    });
                                },
                            ))
                            .child(Label::new("Auto-connect on launch")),
                    )
                    .child(Divider::horizontal())
                    // ── preferences ───────────────────────────────────────
                    .child(Label::new("Preferences").size(LabelSize::Subtitle))
                    .child(control_row(
                        "Show notifications",
                        Switch::new("sw-notif")
                            .on(self.notifications)
                            .on_click(move |new_val, _, _, cx| {
                                e3.update(cx, |this, cx| {
                                    this.notifications = new_val;
                                    cx.notify();
                                });
                            })
                            .into_any_element(),
                    ))
                    .child(control_row(
                        "Dark mode",
                        Switch::new("sw-dark")
                            .on(self.dark_mode)
                            .on_click(move |new_val, _, _, cx| {
                                e4.update(cx, |this, cx| {
                                    this.dark_mode = new_val;
                                    cx.notify();
                                    if new_val {
                                        fluent_core::Theme::apply_dark(cx);
                                    } else {
                                        fluent_core::Theme::apply_light(cx);
                                    }
                                });
                            })
                            .into_any_element(),
                    ))
                    .child(Divider::horizontal())
                    // ── actions ───────────────────────────────────────────
                    .child(
                        div()
                            .flex()
                            .flex_row()
                            .justify_end()
                            .gap(px(8.0))
                            .child(Button::new("cancel").label("Cancel").on_click(cx.listener(
                                |this, _: &ClickEvent, _, cx| {
                                    this.name.update(cx, |t, cx| {
                                        t.set_value("", cx);
                                        t.set_error(false, cx);
                                    });
                                    this.host.update(cx, |t, cx| {
                                        t.set_value("", cx);
                                        t.set_error(false, cx);
                                    });
                                    this.port.update(cx, |t, cx| {
                                        t.set_value("", cx);
                                    });
                                    this.result_msg = None;
                                    cx.notify();
                                },
                            )))
                            .child(
                                Button::new("save")
                                    .label("Save Connection")
                                    .appearance(ButtonAppearance::Accent)
                                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                        let name = this.name.read(cx).text().to_string();
                                        let host = this.host.read(cx).text().to_string();
                                        let name_empty = name.is_empty();
                                        let host_empty = host.is_empty();
                                        this.name.update(cx, |t, cx| {
                                            t.set_error(name_empty, cx);
                                        });
                                        this.host.update(cx, |t, cx| {
                                            t.set_error(host_empty, cx);
                                        });
                                        if name_empty || host_empty {
                                            this.result_msg = None;
                                        } else {
                                            let protocol = match this
                                                .protocol_dropdown
                                                .read(cx)
                                                .selected_value()
                                            {
                                                Some(value) if value.as_ref() == "rdp" => "RDP",
                                                Some(value) if value.as_ref() == "vnc" => "VNC",
                                                _ => "SSH",
                                            };
                                            this.result_msg = Some(
                                                format!("Saved \"{name}\" at {host} ({protocol})")
                                                    .into(),
                                            );
                                        }
                                        cx.notify();
                                    })),
                            ),
                    )
                    // ── validation / result message ───────────────────────
                    .when_some(self.result_msg.clone(), |panel, msg| {
                        panel.child(
                            Label::new(msg)
                                .size(LabelSize::Caption)
                                .color(colors.accent),
                        )
                    }),
            )
    }
}

fn main() {
    FluentApp::new("Form Demo")
        .window_size(560.0, 680.0)
        .run(|cx| cx.new(|cx| FormDemo::new(cx)));
}
