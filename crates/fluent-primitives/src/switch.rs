use crate::focus::{control_focus_handle, focus_ring, is_keyboard_focused, FluentFocus as _};
use fluent_core::ThemeProvider as _;
use gpui::{
    div, ease_out_quint, prelude::*, px, Animation, AnimationExt as _, App, ClickEvent, ElementId,
    IntoElement, RenderOnce, Role, SharedString, Toggled, Window,
};

/// A Fluent 2-styled toggle switch.
///
/// The caller owns the `on` state.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    on: bool,
    label: Option<SharedString>,
    disabled: bool,
    tab_index: Option<isize>,
    on_click: Option<Box<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Switch {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            on: false,
            label: None,
            disabled: false,
            tab_index: None,
            on_click: None,
        }
    }

    /// The accessible name. The switch draws no text of its own, so set this
    /// to the text of the label shown beside it.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the on configuration for this control.
    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// Explicit position in the Tab order (lower first). Defaults to document order.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = Some(index);
        self
    }

    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Callback receives the *new* desired state.
    pub fn on_click(
        mut self,
        handler: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Switch {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus =
            (!self.disabled).then(|| control_focus_handle(&self.id, self.tab_index, window, cx));
        let theme = cx.theme();
        let colors = &theme.colors;

        let track_bg = if self.on {
            if self.disabled {
                colors.accent_disabled
            } else {
                colors.accent
            }
        } else {
            colors.neutral
        };
        let track_border = if self.on {
            colors.accent
        } else {
            colors.stroke_neutral
        };
        let (track_hover, track_pressed) = if self.on {
            (colors.accent_hover, colors.accent_selected)
        } else {
            (colors.neutral_hover, colors.neutral_selected)
        };
        let thumb_color = if self.on {
            colors.on_accent
        } else {
            colors.on_neutral
        };

        let disabled = self.disabled;
        let tab_index = self.tab_index;
        let new_state = !self.on;
        let on_click = self.on_click;
        let on = self.on;
        let label = self.label;

        // Fluent 2: 40x20 track, 12px knob, 4px inset from the outer edge (3px inside the 1px border)
        let (from_ml, to_ml) = if self.on { (3.0, 23.0) } else { (23.0, 3.0) };
        let motion = theme.motion();
        // Fluent 2: knob grows 12 -> 14px on hover and widens on press; the on-state width stops
        // at 15px so the knob stays inside the 38px inner track.
        let press_w = if on { 15.0 } else { 17.0 };

        let debug_id = self.id.to_string();
        let mut el = div()
            .id(self.id)
            .debug_selector(move || debug_id)
            .role(Role::Switch)
            .aria_toggled(if on { Toggled::True } else { Toggled::False })
            .when_some(label, |d, name| d.aria_label(name))
            .w(px(40.0))
            .h(px(theme.components.toggle_box))
            .group("fluent-switch")
            .rounded(px(theme.radii.pill))
            .bg(track_bg)
            .border_1()
            .border_color(track_border)
            .flex()
            .items_center()
            .relative();

        if !disabled {
            el = el
                .track_focus(focus.as_ref().expect("enabled control has a focus handle"))
                .key_context("Switch")
                .fluent_tab_stop(tab_index.unwrap_or(0))
                .cursor_pointer()
                .hover(move |s| s.bg(track_hover))
                .active(move |s| s.bg(track_pressed));
        }

        if let Some(handler) = on_click.filter(|_| !disabled) {
            el = el.on_click(move |ev, win, app| handler(new_state, ev, win, app));
        }

        let ring = focus
            .as_ref()
            .is_some_and(|h| is_keyboard_focused(h, window));
        let el = el.child(
            div()
                .id("switch-knob")
                .debug_selector(|| "switch-knob".into())
                .absolute()
                .size(px(12.0))
                .when(!disabled, |d| {
                    d.group_hover("fluent-switch", |s| s.size(px(14.0)))
                        .group_active("fluent-switch", move |s| s.w(px(press_w)).h(px(14.0)))
                })
                .ml(px(to_ml))
                .rounded(px(theme.radii.pill))
                .bg(thumb_color)
                .with_animation(
                    ElementId::from(("switch-thumb", on as u64)),
                    Animation::new(motion.popup_enter_duration()).with_easing(ease_out_quint()),
                    move |thumb, delta| thumb.ml(px(from_ml + (to_ml - from_ml) * delta)),
                ),
        );
        focus_ring(el, ring, &theme.colors, px(theme.radii.pill))
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{assert_a11y_node, open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Entity, InputEvent as _, KeyUpEvent, Keystroke, Render, TestAppContext};

    use super::*;

    struct Host {
        on: bool,
        disabled: bool,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.entity();
            Switch::new("switch")
                .on(self.on)
                .disabled(self.disabled)
                .on_click(move |on, _, _, cx| {
                    entity.update(cx, |host, cx| {
                        host.on = on;
                        cx.notify();
                    });
                })
        }
    }

    fn open(
        cx: &mut TestAppContext,
        disabled: bool,
    ) -> (Entity<Host>, &mut gpui::VisualTestContext) {
        open_fluent_test_window(cx, move |_, _| Host {
            on: false,
            disabled,
        })
    }

    fn press(cx: &mut gpui::VisualTestContext, key: &'static str) {
        cx.type_keys(key);
        let keystroke = Keystroke::parse(key).expect("valid test keystroke");
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
    fn click_enter_and_space_flip_switch_state(cx: &mut TestAppContext) {
        let (host, cx) = open(cx, false);
        cx.click_selector("switch");
        assert!(host.read_with(cx, |host, _| host.on));
        cx.update(|window, app| window.draw(app).clear(app));
        press(cx, "enter");
        assert!(!host.read_with(cx, |host, _| host.on));
        press(cx, "space");
        assert!(host.read_with(cx, |host, _| host.on));
    }

    #[gpui::test]
    fn disabled_switch_does_not_flip(cx: &mut TestAppContext) {
        let (host, cx) = open(cx, true);
        cx.click_selector("switch");
        assert!(!host.read_with(cx, |host, _| host.on));
    }

    struct A11yHost(Box<dyn Fn() -> gpui::AnyElement>);

    impl Render for A11yHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    fn snapshot(
        cx: &mut TestAppContext,
        build: impl Fn() -> gpui::AnyElement + 'static,
    ) -> Vec<String> {
        let (_, cx) = open_fluent_test_window(cx, move |_, _| A11yHost(Box::new(build)));
        cx.accessibility_snapshot()
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_switch_state(cx: &mut TestAppContext) {
        for on in [false, true] {
            let snap = snapshot(cx, move || {
                Switch::new("s").label("Wi-Fi").on(on).into_any_element()
            });
            assert_eq!(snap.len(), 2, "{snap:?}");
            assert_a11y_node(
                &snap,
                "Switch",
                Some("Wi-Fi"),
                &[if on { "toggled=True" } else { "toggled=False" }],
            );
        }
    }

    /// The fixed `switch-knob` id is nested under each switch's own id, so two
    /// switches in one view do not collide (WI-4756 audit).
    #[gpui::test]
    fn two_switches_in_one_view_do_not_collide(cx: &mut TestAppContext) {
        let snap = snapshot(cx, || {
            gpui::div()
                .child(Switch::new("one").label("One"))
                .child(Switch::new("two").label("Two"))
                .into_any_element()
        });
        assert_eq!(
            snap.iter().filter(|n| n.starts_with("Switch")).count(),
            2,
            "{snap:?}"
        );
    }

    /// The knob's distance from the track's inner edge is the same on both sides, off and on.
    #[gpui::test]
    fn switch_knob_is_inset_matches_off_and_on(cx: &mut TestAppContext) {
        let mut insets = Vec::new();
        for on in [false, true] {
            let (_, cx) = open_fluent_test_window(cx, move |_, _| {
                A11yHost(Box::new(move || {
                    Switch::new("sw").on(on).into_any_element()
                }))
            });
            // Let the slide animation run to its end position.
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(600));
            cx.run_until_parked();
            cx.update(|window, app| window.draw(app).clear(app));
            let track = cx.selector_bounds("sw").expect("track painted");
            let knob = cx.selector_bounds("switch-knob").expect("knob painted");
            let left = f32::from(knob.left() - track.left());
            let right = f32::from(track.right() - knob.right());
            // The knob rests against the left edge when off and the right edge when on.
            insets.push(if on { right } else { left });
            assert_eq!(f32::from(track.size.height), 20.0);
            let token = cx.update(|_, app| app.theme().components.toggle_box);
            assert_eq!(f32::from(track.size.height), token);
        }
        assert_eq!(insets[0], insets[1], "off/on insets differ: {insets:?}");
        assert_eq!(insets[0], 4.0, "Fluent 2 knob inset is 4px");
    }
}
