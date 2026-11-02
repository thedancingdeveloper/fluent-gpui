use fluent_core::ThemeProvider as _;

type ColorChange = std::rc::Rc<dyn Fn(Color, &mut App)>;
use gpui::{
    div, prelude::*, px, App, Context, ElementId, IntoElement, KeyDownEvent, Render, Role,
    SharedString, Window,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Color {
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}
impl Color {
    pub const WHITE: Self = Self {
        red: 255,
        green: 255,
        blue: 255,
    };
    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
    pub fn parse(s: &str) -> Option<Self> {
        let s = s.trim().strip_prefix('#').unwrap_or(s);
        (s.len() == 6)
            .then(|| u32::from_str_radix(s, 16).ok())
            .flatten()
            .map(|n| Self {
                red: (n >> 16) as u8,
                green: (n >> 8) as u8,
                blue: n as u8,
            })
    }
}
#[derive(Clone)]
pub struct ColorPicker {
    id: ElementId,
    color: Color,
    swatches: Vec<Color>,
    label: SharedString,
    on_change: Option<ColorChange>,
}
impl ColorPicker {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            color: Color::WHITE,
            swatches: vec![
                Color {
                    red: 0,
                    green: 120,
                    blue: 212,
                },
                Color {
                    red: 16,
                    green: 124,
                    blue: 16,
                },
                Color {
                    red: 196,
                    green: 49,
                    blue: 75,
                },
                Color::WHITE,
            ],
            label: "Color".into(),
            on_change: None,
        }
    }
    pub fn value(mut self, c: Color) -> Self {
        self.color = c;
        self
    }
    pub fn swatches(mut self, s: impl IntoIterator<Item = Color>) -> Self {
        self.swatches = s.into_iter().collect();
        self
    }
    pub fn label(mut self, s: impl Into<SharedString>) -> Self {
        self.label = s.into();
        self
    }
    pub fn on_change(mut self, f: impl Fn(Color, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(f));
        self
    }
    pub fn set_hex(&mut self, value: &str) -> bool {
        if let Some(color) = Color::parse(value) {
            self.color = color;
            true
        } else {
            false
        }
    }
    pub fn nudge(&mut self, channel: &str, delta: i16) {
        let adjust = |v: u8| (v as i16).saturating_add(delta).clamp(0, 255) as u8;
        self.color = match channel {
            "red" => Color {
                red: adjust(self.color.red),
                ..self.color
            },
            "green" => Color {
                green: adjust(self.color.green),
                ..self.color
            },
            "blue" => Color {
                blue: adjust(self.color.blue),
                ..self.color
            },
            _ => self.color,
        };
    }

    pub fn color(&self) -> Color {
        self.color
    }
}
impl Render for ColorPicker {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let selected = self.color;
        let handler = self.on_change.clone();
        let mut sw = div().flex().gap_1();
        for (i, color) in self.swatches.iter().copied().enumerate() {
            let h = handler.clone();
            sw = sw.child(
                div()
                    .id(("color-swatch", i as u64))
                    .size(px(28.))
                    .rounded(px(4.))
                    .bg(gpui::rgb(
                        (u32::from(color.red) << 16)
                            | (u32::from(color.green) << 8)
                            | u32::from(color.blue),
                    ))
                    .when(color == selected, |d| {
                        d.border_2().border_color(colors.focus_ring)
                    })
                    .on_click(move |_, _, app| {
                        if let Some(f) = &h {
                            f(color, app)
                        }
                    }),
            );
        }
        div()
            .id(self.id.clone())
            .role(Role::Group)
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .h(px(80.))
                    .w(px(180.))
                    .bg(gpui::rgb(
                        (u32::from(selected.red) << 16)
                            | (u32::from(selected.green) << 8)
                            | u32::from(selected.blue),
                    ))
                    .on_key_down(move |ev: &KeyDownEvent, _, app| {
                        if matches!(ev.keystroke.key.as_str(), "left" | "right" | "up" | "down") {
                            app.stop_propagation()
                        }
                    }),
            )
            .child(sw)
            .child(div().text_size(px(13.)).child(selected.hex()))
    }
}
