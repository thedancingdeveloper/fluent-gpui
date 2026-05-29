use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window,
};

use crate::button::{ButtonAppearance, ButtonSize};

/// An icon-only variant of `Button` — no visible label.
///
/// Size is square: icon size determines the hit area.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct IconButton {
    id: ElementId,
    icon: SharedString,
    appearance: ButtonAppearance,
    size: ButtonSize,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl IconButton {
    pub fn new(id: impl Into<ElementId>, icon: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: icon.into(),
            appearance: ButtonAppearance::Subtle,
            size: ButtonSize::Normal,
            disabled: false,
            on_click: None,
        }
    }

    pub fn appearance(mut self, appearance: ButtonAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for IconButton {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;

        let (bg, bg_hover, bg_active, fg) = match self.appearance {
            ButtonAppearance::Accent => (
                if self.disabled {
                    colors.accent_disabled
                } else {
                    colors.accent
                },
                colors.accent_hover,
                colors.accent_selected,
                if self.disabled {
                    colors.on_accent_disabled
                } else {
                    colors.on_accent
                },
            ),
            _ => (
                colors.subtle,
                colors.subtle_hover,
                colors.subtle_selected,
                if self.disabled {
                    colors.on_subtle_disabled
                } else {
                    colors.on_subtle
                },
            ),
        };

        let (icon_px, hit_px) = match self.size {
            ButtonSize::Normal => (16.0f32, 32.0f32),
            ButtonSize::Compact => (12.0f32, 24.0f32),
        };

        let disabled = self.disabled;
        let on_click = self.on_click;

        let mut el = div()
            .id(self.id)
            .flex()
            .items_center()
            .justify_center()
            .size(px(hit_px))
            .rounded(px(radii.md))
            .bg(bg);

        if !disabled {
            el = el
                .cursor_pointer()
                .hover(move |s| s.bg(bg_hover))
                .active(move |s| s.bg(bg_active));
        }

        if let Some(handler) = on_click.filter(|_| !disabled) {
            el = el.on_click(move |ev, win, app| handler(ev, win, app));
        }

        el.child(gpui::svg().path(self.icon).size(px(icon_px)).text_color(fg))
    }
}
