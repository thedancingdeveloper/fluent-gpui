use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, Window};

/// A Fluent 2-styled toggle switch.
///
/// The caller owns the `on` state.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Switch {
    id: ElementId,
    on: bool,
    disabled: bool,
    on_click: Option<Box<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Switch {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            on: false,
            disabled: false,
            on_click: None,
        }
    }

    pub fn on(mut self, on: bool) -> Self {
        self.on = on;
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
        let new_state = !self.on;
        let on_click = self.on_click;

        // Thumb offset: left (off) or right (on)
        let thumb_ml = if self.on { px(20.0) } else { px(2.0) };

        let mut el = div()
            .id(self.id)
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
            el = el.cursor_pointer();
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
