use std::rc::Rc;

use crate::focus::FluentFocus as _;
use crate::slider::snap;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ElementId, IntoElement, KeyDownEvent, RenderOnce, Role, SharedString,
    Window,
};

type Format = Rc<dyn Fn(f64) -> String>;
type Parse = Rc<dyn Fn(&str) -> Option<f64>>;

/// The value an Up/Down/PageUp/PageDown/Home/End key moves to.
pub(crate) fn key_target(key: &str, value: f64, min: f64, max: f64, step: f64) -> Option<f64> {
    let next = match key {
        "up" => value + step,
        "down" => value - step,
        "pageup" => value + step * 10.0,
        "pagedown" => value - step * 10.0,
        "home" => min,
        "end" => max,
        _ => return None,
    };
    Some(snap(next, min, max, step))
}

/// A Fluent 2-styled numeric spin button. The caller owns the value.
///
/// Shows the value through the `format` hook, with Up/Down step buttons.
/// Up/Down move by `step`, PageUp/PageDown by ten steps, Home/End jump to the
/// bounds; every result is clamped to `[min, max]`. Exposes the AccessKit
/// `SpinButton` role. The control does not host a text caret; a host that
/// accepts typed text calls [`SpinButton::commit_text`] to run the `parse`
/// hook and clamp the result.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct SpinButton {
    id: ElementId,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    disabled: bool,
    label: Option<SharedString>,
    tab_index: Option<isize>,
    format: Format,
    parse: Parse,
    on_change: Option<Rc<dyn Fn(f64, &mut Window, &mut App) + 'static>>,
}

impl SpinButton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: 0.0,
            min: f64::MIN,
            max: f64::MAX,
            step: 1.0,
            disabled: false,
            label: None,
            tab_index: None,
            format: Rc::new(|v| format!("{v}")),
            parse: Rc::new(|s| s.trim().parse().ok()),
            on_change: None,
        }
    }

    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Inclusive bounds. A reversed pair is swapped.
    pub fn range(mut self, min: f64, max: f64) -> Self {
        (self.min, self.max) = if min <= max { (min, max) } else { (max, min) };
        self
    }

    /// Step size; non-positive or non-finite values mean the default of 1.
    pub fn step(mut self, step: f64) -> Self {
        self.step = if step.is_finite() && step > 0.0 {
            step
        } else {
            1.0
        };
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = Some(index);
        self
    }

    /// Turns the value into display text. Defaults to `f64`'s `Display`.
    pub fn format(mut self, f: impl Fn(f64) -> String + 'static) -> Self {
        self.format = Rc::new(f);
        self
    }

    /// Turns typed text into a value; `None` rejects the text.
    pub fn parse(mut self, f: impl Fn(&str) -> Option<f64> + 'static) -> Self {
        self.parse = Rc::new(f);
        self
    }

    /// Called with the new, clamped value.
    pub fn on_change(mut self, handler: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }

    /// Parse `text` with the `parse` hook and clamp to the bounds. Returns
    /// `None` when the hook rejects the text.
    pub fn commit_text(&self, text: &str) -> Option<f64> {
        (self.parse)(text).map(|v| snap(v, self.min, self.max, 0.0))
    }
}

impl RenderOnce for SpinButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let (min, max, step) = (self.min, self.max, self.step);
        let value = snap(self.value, min, max, 0.0);
        let text = (self.format)(value);
        let disabled = self.disabled;
        let on_change = self.on_change;
        let fg = if disabled {
            colors.accent_disabled
        } else {
            colors.on_neutral
        };
        let hover = colors.subtle_hover;

        let button = |id: &'static str, glyph: &'static str, delta: f64| {
            let handler = on_change.clone().filter(|_| !disabled);
            div()
                .id((self.id.clone(), id))
                .w(px(20.0))
                .h(px(14.0))
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(8.0))
                .text_color(fg)
                .when(!disabled, |d| {
                    d.cursor_pointer().hover(move |s| s.bg(hover))
                })
                .when_some(handler, |d, h| {
                    d.on_click(move |_, win, app| {
                        let next = snap(value + delta, min, max, 0.0);
                        if next != value {
                            h(next, win, app);
                        }
                    })
                })
                .child(glyph)
        };

        let debug_id = self.id.to_string();
        let key_handler = on_change.clone();
        let mut el = div()
            .id(self.id.clone())
            .debug_selector(move || debug_id)
            .role(Role::SpinButton)
            .aria_min_numeric_value(min)
            .aria_max_numeric_value(max)
            .aria_numeric_value(value)
            .aria_numeric_value_step(step)
            .aria_value(text.clone())
            .when_some(self.label, |d, name| d.aria_label(name))
            .flex()
            .items_center()
            .h(px(32.0))
            .min_w(px(96.0))
            .pl(px(10.0))
            .rounded(px(4.0))
            .bg(colors.neutral)
            .border_1()
            .border_color(colors.stroke_neutral)
            .text_color(fg);

        if !disabled {
            el = el
                .focusable()
                .tab_stop(true)
                .fluent_focus(self.tab_index, colors.focus_ring);
            if let Some(handler) = key_handler {
                el = el.on_key_down(move |ev: &KeyDownEvent, win, app| {
                    if let Some(next) = key_target(ev.keystroke.key.as_str(), value, min, max, step)
                    {
                        if next != value {
                            handler(next, win, app);
                        }
                        app.stop_propagation();
                    }
                });
            }
        }

        // Buttons step from the snapped value by `step` (grid-aligned).
        let (up, down) = (button("up", "▲", step), button("down", "▼", -step));
        el.child(div().flex_1().child(text)).child(
            div()
                .flex()
                .flex_col()
                .justify_center()
                .child(up)
                .child(down),
        )
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use fluent_test_utils::{assert_a11y_node, open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Render, TestAppContext};

    use super::*;

    #[test]
    fn keys_clamp_to_bounds() {
        assert_eq!(key_target("up", 9.5, 0.0, 10.0, 1.0), Some(10.0));
        assert_eq!(key_target("down", 0.0, 0.0, 10.0, 1.0), Some(0.0));
        assert_eq!(key_target("pageup", 4.0, 0.0, 100.0, 2.0), Some(24.0));
        assert_eq!(key_target("home", 5.0, 0.0, 10.0, 1.0), Some(0.0));
        assert_eq!(key_target("x", 5.0, 0.0, 10.0, 1.0), None);
    }

    #[test]
    fn parse_hook_runs_and_result_is_clamped() {
        let s = SpinButton::new("s").range(0.0, 10.0);
        assert_eq!(s.commit_text(" 7 "), Some(7.0));
        assert_eq!(s.commit_text("70"), Some(10.0));
        assert_eq!(s.commit_text("abc"), None);
        let pct = SpinButton::new("p").parse(|t| t.strip_suffix('%')?.parse().ok());
        assert_eq!(pct.commit_text("12%"), Some(12.0));
        assert_eq!(pct.commit_text("12"), None);
    }

    struct Host {
        disabled: bool,
        value: Rc<Cell<f64>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let cell = self.value.clone();
            SpinButton::new("spin")
                .label("Threads")
                .range(0.0, 10.0)
                .value(self.value.get())
                .disabled(self.disabled)
                .format(|v| format!("{v} threads"))
                .on_change(move |v, _, _| cell.set(v))
        }
    }

    #[gpui::test]
    fn exposes_spin_button_role_and_range(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, _| Host {
            disabled: false,
            value: Rc::new(Cell::new(3.0)),
        });
        let snap = cx.accessibility_snapshot();
        assert_a11y_node(
            &snap,
            "SpinButton",
            Some("Threads"),
            &[
                "numeric_value=3.0",
                "min_numeric_value=0.0",
                "max_numeric_value=10.0",
            ],
        );
    }

    #[gpui::test]
    fn up_down_keys_clamp_and_disabled_ignores(cx: &mut TestAppContext) {
        for disabled in [false, true] {
            let value = Rc::new(Cell::new(9.5));
            let v = value.clone();
            let (_, cx) = open_fluent_test_window(cx, move |_, _| Host { disabled, value: v });
            cx.click_selector("spin");
            cx.type_keys("up");
            assert_eq!(value.get(), if disabled { 9.5 } else { 10.0 });
        }
    }
}
