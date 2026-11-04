use crate::focus::FluentFocus as _;
use fluent_core::TextRampExt;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, Role, SharedString,
    Window,
};

/// Visual style of a [`Link`].
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LinkAppearance {
    #[default]
    /// The default option.
    Default,
    /// The subtle option.
    Subtle,
}

/// A Fluent 2-styled text link for inline or standalone use.
///
/// Use [`Link::on_click`] for application navigation or [`Link::open_url`] to
/// open an external URL with the platform handler.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Link {
    id: ElementId,
    label: SharedString,
    appearance: LinkAppearance,
    disabled: bool,
    tab_index: Option<isize>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Link {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            appearance: LinkAppearance::default(),
            disabled: false,
            tab_index: None,
            on_click: None,
        }
    }

    /// Sets the appearance configuration for this control.
    pub fn appearance(mut self, appearance: LinkAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Explicit position in the Tab order (lower first). Defaults to document order.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = Some(index);
        self
    }

    /// Registers a handler for click events.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Returns the open url value.
    pub fn open_url(self, url: impl Into<SharedString>) -> Self {
        let url = url.into();
        self.on_click(move |_, _, cx| cx.open_url(&url))
    }
}

impl RenderOnce for Link {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let typography = &theme.typography;
        let foreground = if self.disabled {
            colors.on_subtle_disabled
        } else {
            match self.appearance {
                LinkAppearance::Default => colors.on_neutral_accent,
                LinkAppearance::Subtle => colors.on_subtle,
            }
        };

        let disabled = self.disabled;
        let tab_index = self.tab_index;
        let focus_ring = colors.focus_ring;
        let on_click = self.on_click;
        let label = self.label;
        let debug_id = self.id.to_string();

        let mut element = div()
            .id(self.id)
            .debug_selector(move || debug_id)
            .role(Role::Link)
            .aria_label(label.clone())
            .flex()
            .text_ramp(&typography.body)
            .line_height(px(typography.body.line_height))
            .text_color(foreground)
            .child(label);

        if !disabled {
            element = element
                .focusable()
                .tab_stop(true)
                .fluent_focus(tab_index, focus_ring)
                .cursor_pointer()
                .hover(|style| style.underline());
        }

        if let Some(handler) = on_click.filter(|_| !disabled) {
            element = element.on_click(move |event, window, app| handler(event, window, app));
        }

        element
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, InputEvent as _, KeyUpEvent, Keystroke, Render, TestAppContext};

    use super::*;

    struct Host {
        disabled: bool,
        clicks: Rc<Cell<usize>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            Link::new("link", "Learn more")
                .disabled(self.disabled)
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
        }
    }

    fn open(
        cx: &mut TestAppContext,
        disabled: bool,
    ) -> (Rc<Cell<usize>>, &mut gpui::VisualTestContext) {
        let clicks = Rc::new(Cell::new(0));
        let host_clicks = clicks.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| Host {
            disabled,
            clicks: host_clicks,
        });
        (clicks, cx)
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
    fn click_and_enter_activate_when_enabled(cx: &mut TestAppContext) {
        let (clicks, cx) = open(cx, false);
        cx.click_selector("link");
        cx.update(|window, app| window.draw(app).clear(app));
        press(cx, "enter");
        assert_eq!(clicks.get(), 2);
    }

    #[gpui::test]
    fn disabled_link_ignores_clicks(cx: &mut TestAppContext) {
        let (clicks, cx) = open(cx, true);
        cx.click_selector("link");
        assert_eq!(clicks.get(), 0);
    }

    struct A11yHost(Box<dyn Fn() -> gpui::AnyElement>);

    impl Render for A11yHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_link(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, _| {
            A11yHost(Box::new(|| {
                Link::new("docs", "Documentation").into_any_element()
            }))
        });
        assert_eq!(
            cx.accessibility_snapshot(),
            vec!["Window".to_string(), "Link \"Documentation\"".to_string()]
        );
    }

    #[gpui::test]
    fn open_url_uses_platform_handler(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, _| {
            A11yHost(Box::new(|| {
                Link::new("website", "Website")
                    .open_url("https://example.com")
                    .into_any_element()
            }))
        });
        cx.click_selector("website");
        assert_eq!(cx.opened_url().as_deref(), Some("https://example.com"));
    }
}
