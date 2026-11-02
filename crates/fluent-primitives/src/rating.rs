use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ElementId, IntoElement, KeyDownEvent, RenderOnce, Role, SharedString,
    Window,
};
use std::rc::Rc;

#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Rating {
    id: ElementId,
    value: f64,
    max: u8,
    step: f64,
    readonly: bool,
    disabled: bool,
    label: Option<SharedString>,
    on_change: Option<Rc<dyn Fn(f64, &mut Window, &mut App) + 'static>>,
}
impl Rating {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: 0.0,
            max: 5,
            step: 0.5,
            readonly: false,
            disabled: false,
            label: None,
            on_change: None,
        }
    }
    pub fn value(mut self, v: f64) -> Self {
        self.value = v;
        self
    }
    pub fn max(mut self, v: u8) -> Self {
        self.max = v.max(1);
        self
    }
    pub fn step(mut self, v: f64) -> Self {
        if v == 0.5 || v == 1. {
            self.step = v
        };
        self
    }
    pub fn readonly(mut self, v: bool) -> Self {
        self.readonly = v;
        self
    }
    pub fn interactive(mut self, v: bool) -> Self {
        self.readonly = !v;
        self
    }
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    pub fn label(mut self, v: impl Into<SharedString>) -> Self {
        self.label = Some(v.into());
        self
    }
    pub fn on_change(mut self, f: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(f));
        self
    }
}
impl RenderOnce for Rating {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let c = &t.colors;
        let value = self.value.clamp(0.0, self.max as f64);
        let max = self.max;
        let step = self.step;
        let change = self.on_change.clone();
        let mut root = div()
            .id(self.id)
            .role(Role::Slider)
            .aria_label(self.label.unwrap_or_else(|| "Rating".into()))
            .aria_value(value.to_string())
            .flex()
            .gap(px(2.));
        for i in 1..=max {
            let fill = value - (i - 1) as f64;
            let glyph = if fill >= 1. {
                "★"
            } else if fill >= 0.5 {
                "⯨"
            } else {
                "☆"
            };
            let mut star = div()
                .id(("rating-star", i as u64))
                .text_size(px(24.0))
                .text_color(if fill > 0.0 {
                    c.accent
                } else {
                    c.on_neutral_disabled
                })
                .child(glyph);
            if !self.readonly && !self.disabled {
                if let Some(f) = change.clone() {
                    let nv = (i as f64).min(max as f64);
                    let click_change = f.clone();
                    star = star
                        .cursor_pointer()
                        .on_click(move |_, w, a| click_change(nv, w, a))
                        .on_key_down(move |ev: &KeyDownEvent, w, a| {
                            let k = ev.keystroke.key.as_str();
                            let nv = match k {
                                "right" | "up" => (value + step).min(max as f64),
                                "left" | "down" => (value - step).max(0.),
                                "home" => 0.0,
                                "end" => max as f64,
                                _ => return,
                            };
                            f(nv, w, a);
                        });
                }
            }
            root = root.child(star);
        }
        root
    }
}
