use std::rc::Rc;

use crate::focus::FluentFocus as _;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, relative, App, ElementId, IntoElement, KeyDownEvent, Orientation,
    RenderOnce, Role, SharedString, Window,
};

const THUMB: f32 = 16.0;
const TRACK: f32 = 4.0;
const LENGTH: f32 = 200.0;

/// Slider axis. Vertical sliders grow upwards.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum SliderOrientation {
    #[default]
    /// The horizontal option.
    Horizontal,
    /// The vertical option.
    Vertical,
}

/// Snap `value` to the `step` grid anchored at `min` and clamp it to `[min, max]`.
pub(crate) fn snap(value: f64, min: f64, max: f64, step: f64) -> f64 {
    if value.is_nan() {
        return min;
    }
    let v = if step > 0.0 {
        min + ((value - min) / step).round() * step
    } else {
        value
    };
    v.clamp(min, max)
}

/// The value a key moves to, or `None` when the key is not a slider key.
/// `page` is the PageUp/PageDown stride.
pub(crate) fn key_target(
    key: &str,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    page: f64,
) -> Option<f64> {
    let next = match key {
        "right" | "up" => value + step,
        "left" | "down" => value - step,
        "pageup" => value + page,
        "pagedown" => value - page,
        "home" => min,
        "end" => max,
        _ => return None,
    };
    Some(snap(next, min, max, step))
}

/// A Fluent 2-styled slider. The caller owns the value.
///
/// Keys: arrows move by `step`, PageUp/PageDown by ten steps, Home/End jump to
/// the ends. Exposes the AccessKit `Slider` role with numeric value, range and
/// value text. Pointer dragging is not supported yet.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Slider {
    id: ElementId,
    value: f64,
    min: f64,
    max: f64,
    step: f64,
    orientation: SliderOrientation,
    disabled: bool,
    label: Option<SharedString>,
    value_text: Option<SharedString>,
    tab_index: Option<isize>,
    length: f32,
    on_change: Option<Rc<dyn Fn(f64, &mut Window, &mut App) + 'static>>,
}

impl Slider {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            value: 0.0,
            min: 0.0,
            max: 100.0,
            step: 1.0,
            orientation: SliderOrientation::Horizontal,
            disabled: false,
            label: None,
            value_text: None,
            tab_index: None,
            length: LENGTH,
            on_change: None,
        }
    }

    /// Returns the current value.
    pub fn value(mut self, value: f64) -> Self {
        self.value = value;
        self
    }

    /// Inclusive range. A reversed pair is swapped.
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

    /// Sets the orientation configuration for this control.
    pub fn orientation(mut self, orientation: SliderOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Sets the vertical configuration for this control.
    pub fn vertical(self) -> Self {
        self.orientation(SliderOrientation::Vertical)
    }

    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The accessible name.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Human-readable value for assistive technology, e.g. `"40 percent"`.
    pub fn value_text(mut self, text: impl Into<SharedString>) -> Self {
        self.value_text = Some(text.into());
        self
    }

    /// Sets the tab index configuration for this control.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = Some(index);
        self
    }

    /// Track length in logical pixels.
    pub fn length(mut self, length: f32) -> Self {
        self.length = length;
        self
    }

    /// Called with the new value when a key changes it.
    pub fn on_change(mut self, handler: impl Fn(f64, &mut Window, &mut App) + 'static) -> Self {
        self.on_change = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for Slider {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let (min, max, step) = (self.min, self.max, self.step);
        let value = snap(self.value, min, max, step);
        let span = max - min;
        let frac = if span > 0.0 {
            ((value - min) / span) as f32
        } else {
            0.0
        };
        let (fill_color, thumb_color) = if self.disabled {
            (colors.accent_disabled, colors.accent_disabled)
        } else {
            (colors.accent, colors.accent)
        };
        let vertical = self.orientation == SliderOrientation::Vertical;
        let text = self.value_text.unwrap_or_else(|| format!("{value}").into());

        let mut track = div()
            .relative()
            .rounded(px(cx.theme().radii.pill))
            .bg(colors.neutral)
            .border_1()
            .border_color(colors.stroke_neutral);
        let fill = div()
            .absolute()
            .rounded(px(cx.theme().radii.pill))
            .bg(fill_color);
        let thumb = div()
            .absolute()
            .size(px(THUMB))
            .rounded(px(cx.theme().radii.pill))
            .bg(thumb_color)
            .border_2()
            .border_color(colors.on_accent);
        if vertical {
            track = track
                .w(px(TRACK))
                .h(px(self.length))
                .child(fill.bottom_0().left_0().w_full().h(relative(frac)))
                .child(
                    thumb
                        .left(px(-(THUMB - TRACK) / 2.0 - 1.0))
                        .bottom(relative(frac))
                        .mb(px(-THUMB / 2.0)),
                );
        } else {
            track = track
                .h(px(TRACK))
                .w(px(self.length))
                .child(fill.top_0().left_0().h_full().w(relative(frac)))
                .child(
                    thumb
                        .top(px(-(THUMB - TRACK) / 2.0 - 1.0))
                        .left(relative(frac))
                        .ml(px(-THUMB / 2.0)),
                );
        }

        let disabled = self.disabled;
        let page = step * 10.0;
        let on_change = self.on_change;
        let debug_id = self.id.to_string();
        let mut el = div()
            .id(self.id)
            .debug_selector(move || debug_id)
            .role(Role::Slider)
            .aria_orientation(if vertical {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            })
            .aria_min_numeric_value(min)
            .aria_max_numeric_value(max)
            .aria_numeric_value(value)
            .aria_numeric_value_step(step)
            .aria_value(text)
            .when_some(self.label, |d, name| d.aria_label(name))
            .flex()
            .items_center()
            .justify_center()
            .when(vertical, |d| d.w(px(THUMB + 4.0)).py(px(THUMB / 2.0)))
            .when(!vertical, |d| d.h(px(THUMB + 4.0)).px(px(THUMB / 2.0)));

        if !disabled {
            el = el
                .focusable()
                .tab_stop(true)
                .fluent_focus(self.tab_index, colors.focus_ring)
                .cursor_pointer();
            if let Some(handler) = on_change {
                el = el.on_key_down(move |ev: &KeyDownEvent, win, app| {
                    if let Some(next) =
                        key_target(ev.keystroke.key.as_str(), value, min, max, step, page)
                    {
                        if next != value {
                            handler(next, win, app);
                        }
                        app.stop_propagation();
                    }
                });
            }
        }
        el.child(track)
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use fluent_test_utils::{assert_a11y_node, open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Render, TestAppContext};

    use super::*;

    #[test]
    fn snap_clamps_and_aligns_to_step() {
        assert_eq!(snap(7.0, 0.0, 10.0, 5.0), 5.0);
        assert_eq!(snap(8.0, 0.0, 10.0, 5.0), 10.0);
        assert_eq!(snap(-3.0, 0.0, 10.0, 1.0), 0.0);
        assert_eq!(snap(99.0, 0.0, 10.0, 1.0), 10.0);
        assert_eq!(snap(f64::NAN, 2.0, 10.0, 1.0), 2.0);
    }

    #[test]
    fn keys_map_to_values() {
        let k = |key| key_target(key, 50.0, 0.0, 100.0, 5.0, 50.0);
        assert_eq!(k("right"), Some(55.0));
        assert_eq!(k("up"), Some(55.0));
        assert_eq!(k("left"), Some(45.0));
        assert_eq!(k("down"), Some(45.0));
        assert_eq!(k("pageup"), Some(100.0));
        assert_eq!(k("pagedown"), Some(0.0));
        assert_eq!(k("home"), Some(0.0));
        assert_eq!(k("end"), Some(100.0));
        assert_eq!(k("a"), None);
        assert_eq!(
            key_target("right", 100.0, 0.0, 100.0, 5.0, 50.0),
            Some(100.0)
        );
    }

    struct Host {
        vertical: bool,
        disabled: bool,
        value: Rc<Cell<f64>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let cell = self.value.clone();
            let mut s = Slider::new("slider")
                .label("Volume")
                .range(0.0, 10.0)
                .value(self.value.get())
                .disabled(self.disabled)
                .value_text("four")
                .on_change(move |v, _, _| cell.set(v));
            if self.vertical {
                s = s.vertical();
            }
            s
        }
    }

    #[gpui::test]
    fn exposes_slider_role_orientation_and_range(cx: &mut TestAppContext) {
        for vertical in [false, true] {
            let (_, cx) = open_fluent_test_window(cx, move |_, _| Host {
                vertical,
                disabled: false,
                value: Rc::new(Cell::new(4.0)),
            });
            let snap = cx.accessibility_snapshot();
            assert_a11y_node(
                &snap,
                "Slider",
                Some("Volume"),
                &[
                    if vertical {
                        "orientation=Vertical"
                    } else {
                        "orientation=Horizontal"
                    },
                    "numeric_value=4.0",
                    "min_numeric_value=0.0",
                    "max_numeric_value=10.0",
                ],
            );
        }
    }

    #[gpui::test]
    fn arrow_home_end_keys_change_value(cx: &mut TestAppContext) {
        let value = Rc::new(Cell::new(4.0));
        let v = value.clone();
        let (host, cx) = open_fluent_test_window(cx, move |_, _| Host {
            vertical: false,
            disabled: false,
            value: v,
        });
        cx.click_selector("slider");
        for (key, expect) in [
            ("right", 5.0),
            ("end", 10.0),
            ("home", 0.0),
            ("pageup", 10.0),
        ] {
            cx.type_keys(key);
            assert_eq!(value.get(), expect, "{key}");
            host.update(cx, |h, _| h.value.set(value.get()));
            cx.update(|w, a| w.draw(a).clear(a));
        }
    }

    #[gpui::test]
    fn disabled_slider_ignores_keys(cx: &mut TestAppContext) {
        let value = Rc::new(Cell::new(4.0));
        let v = value.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| Host {
            vertical: false,
            disabled: true,
            value: v,
        });
        cx.click_selector("slider");
        cx.type_keys("right");
        assert_eq!(value.get(), 4.0);
    }
}
