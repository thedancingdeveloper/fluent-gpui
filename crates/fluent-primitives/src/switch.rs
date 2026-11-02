use crate::focus::FluentFocus as _;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, Role, SharedString,
    Toggled, Window,
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

    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
        self
    }

    /// Explicit position in the Tab order (lower first). Defaults to document order.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = Some(index);
        self
    }

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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
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
        let thumb_color = if self.on {
            colors.on_accent
        } else {
            colors.on_neutral
        };

        let disabled = self.disabled;
        let tab_index = self.tab_index;
        let focus_ring = colors.focus_ring;
        let new_state = !self.on;
        let on_click = self.on_click;
        let on = self.on;
        let label = self.label;

        // Thumb offset: left (off) or right (on)
        let thumb_ml = if self.on { px(20.0) } else { px(2.0) };

        let debug_id = self.id.to_string();
        let mut el = div()
            .id(self.id)
            .debug_selector(move || debug_id)
            .role(Role::Switch)
            .aria_toggled(if on { Toggled::True } else { Toggled::False })
            .when_some(label, |d, name| d.aria_label(name))
            .w(px(40.0))
            .h(px(20.0))
            .rounded(px(9999.0))
            .bg(track_bg)
            .border_1()
            .border_color(track_border)
            .flex()
            .items_center()
            .relative();

        if !disabled {
            el = el
                .focusable()
                .tab_stop(true)
                .fluent_focus(tab_index, focus_ring)
                .cursor_pointer();
        }

        if let Some(handler) = on_click.filter(|_| !disabled) {
            el = el.on_click(move |ev, win, app| handler(new_state, ev, win, app));
        }

        el.child(
            div()
                .absolute()
                .size(px(14.0))
                .ml(thumb_ml)
                .rounded(px(9999.0))
                .bg(thumb_color),
        )
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
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
            assert_eq!(
                snap,
                vec![
                    "Window".to_string(),
                    format!(
                        "Switch \"Wi-Fi\" [toggled={}]",
                        if on { "True" } else { "False" }
                    )
                ]
            );
        }
    }
}
