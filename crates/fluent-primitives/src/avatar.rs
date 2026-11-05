use fluent_core::{gradient_from_hue, hue_for, ThemeProvider as _};
use gpui::{
    div, img, prelude::*, px, App, FontWeight, IntoElement, ObjectFit, RenderOnce, SharedString,
    Window,
};

/// A circular avatar: initials fallback or image. Initials avatars use a
/// deterministic hue derived from the initials string so the same person
/// always renders with the same swatch — override with `hue(deg)` when a
/// brand colour is known.
#[derive(IntoElement)]
pub struct Avatar {
    initials: Option<SharedString>,
    image_path: Option<SharedString>,
    hue: Option<f32>,
    size: f32,
}

impl Avatar {
    /// Returns the initials value.
    pub fn initials(initials: impl Into<SharedString>) -> Self {
        Self {
            initials: Some(initials.into()),
            image_path: None,
            hue: None,
            size: 32.0,
        }
    }

    /// Returns the image value.
    pub fn image(path: impl Into<SharedString>) -> Self {
        Self {
            initials: None,
            image_path: Some(path.into()),
            hue: None,
            size: 32.0,
        }
    }

    /// Sets the hue configuration for this control.
    pub fn hue(mut self, deg: f32) -> Self {
        self.hue = Some(deg);
        self
    }

    /// Sets the size configuration for this control.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub(crate) fn size_value(&self) -> f32 {
        self.size
    }
}

impl RenderOnce for Avatar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let typography = &theme.typography;

        let base = div()
            .size(px(self.size))
            .rounded(px(9999.0))
            .flex()
            .items_center()
            .justify_center()
            .overflow_hidden();

        if let Some(initials) = self.initials {
            let hue = self.hue.unwrap_or_else(|| hue_for(&initials));
            let text_size = if self.size <= 28.0 {
                11.0
            } else {
                typography.body.size
            };
            base.bg(gradient_from_hue(hue))
                .text_color(colors.on_accent)
                .text_size(px(text_size))
                .line_height(px(text_size.round()))
                .font_weight(FontWeight::BOLD)
                .child(initials)
        } else if let Some(path) = self.image_path {
            base.child(img(path).size_full().object_fit(ObjectFit::Cover))
        } else {
            base.bg(colors.neutral)
        }
    }
}
