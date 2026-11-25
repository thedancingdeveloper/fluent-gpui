use crate::focus::{focus_ring, is_keyboard_focused, FluentFocus as _};
use crate::roving;
use fluent_core::ThemeProvider as _;

type ColorChange = std::rc::Rc<dyn Fn(Color, &mut App)>;
use gpui::{
    div, prelude::*, px, App, Context, ElementId, IntoElement, KeyDownEvent, Render, Role,
    SharedString, Window,
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A Fluent primitive for representing color in an interface.
pub struct Color {
    /// The red value.
    pub red: u8,
    /// The green value.
    pub green: u8,
    /// The blue value.
    pub blue: u8,
}
impl Color {
    /// The white constant.
    pub const WHITE: Self = Self {
        red: 255,
        green: 255,
        blue: 255,
    };
    /// Formats the value for display or serialization.
    pub fn hex(self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.red, self.green, self.blue)
    }
    /// Parses an input value when it has a supported format.
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
/// A Fluent primitive for representing color picker in an interface.
pub struct ColorPicker {
    id: ElementId,
    color: Color,
    swatches: Vec<Color>,
    label: SharedString,
    on_change: Option<ColorChange>,
    /// One focus handle per swatch, created on first render.
    swatch_focus: Vec<gpui::FocusHandle>,
}
impl ColorPicker {
    /// Creates a new instance for use in a Fluent interface.
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
            swatch_focus: Vec::new(),
        }
    }
    /// Returns the current value.
    pub fn value(mut self, c: Color) -> Self {
        self.color = c;
        self
    }
    /// Sets the swatches configuration for this control.
    pub fn swatches(mut self, s: impl IntoIterator<Item = Color>) -> Self {
        self.swatches = s.into_iter().collect();
        self
    }
    /// Returns the current label.
    pub fn label(mut self, s: impl Into<SharedString>) -> Self {
        self.label = s.into();
        self
    }
    /// Registers a handler for change events.
    pub fn on_change(mut self, f: impl Fn(Color, &mut App) + 'static) -> Self {
        self.on_change = Some(std::rc::Rc::new(f));
        self
    }
    /// Sets the hex value.
    pub fn set_hex(&mut self, value: &str) -> bool {
        if let Some(color) = Color::parse(value) {
            self.color = color;
            true
        } else {
            false
        }
    }
    /// Adjusts one colour channel by the given amount, clamped to its valid range.
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

    /// Returns the current color.
    pub fn color(&self) -> Color {
        self.color
    }
}
impl Render for ColorPicker {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        while self.swatch_focus.len() < self.swatches.len() {
            self.swatch_focus.push(cx.focus_handle());
        }
        self.swatch_focus.truncate(self.swatches.len());
        let handles = self.swatch_focus.clone();
        let enabled = vec![true; handles.len()];
        // The swatches are one Tab stop (roving): the focused swatch, else the
        // selected one, else the first. Arrows move focus within the group.
        let default = self.swatches.iter().position(|c| *c == self.color);
        let stop = roving::stop_index(window, &handles, &enabled, default);
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let selected = self.color;
        let handler = self.on_change.clone();
        let mut sw = div().flex().gap_1();
        for (i, color) in self.swatches.iter().copied().enumerate() {
            let h = handler.clone();
            let handle = handles[i].clone().tab_index(0).tab_stop(stop == Some(i));
            let ring = is_keyboard_focused(&handle, window);
            let nav_handles = handles.clone();
            let nav_enabled = enabled.clone();
            let swatch = div()
                .id(("color-swatch", i as u64))
                .debug_selector(move || format!("color-swatch-{i}"))
                .size(px(28.))
                .rounded(px(4.))
                .bg(gpui::rgb(
                    (u32::from(color.red) << 16)
                        | (u32::from(color.green) << 8)
                        | u32::from(color.blue),
                ))
                .when(color == selected, |d| {
                    d.border_2().border_color(colors.accent)
                })
                .track_focus(&handle)
                .key_context("ColorPicker")
                .fluent_tab_stop(0)
                .cursor_pointer()
                .on_key_down(move |ev: &KeyDownEvent, win, app| {
                    if let Some(next) = roving::target(ev.keystroke.key.as_str(), i, &nav_enabled) {
                        nav_handles[next].focus(win, app);
                        app.stop_propagation();
                    }
                })
                // Enter/Space arrive as gpui's keyboard click on key-up.
                .on_click(move |_, _, app| {
                    if let Some(f) = &h {
                        f(color, app)
                    }
                });
            sw = sw.child(focus_ring(swatch, ring, &colors, px(4.)));
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
