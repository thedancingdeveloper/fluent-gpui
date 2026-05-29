use fluent_core::ThemeProvider as _;
use gpui::{
    anchored, deferred, div, prelude::*, px, AnyElement, Corner, IntoElement, Point, RenderOnce,
    Window,
};

/// A lightweight anchored floating surface.
#[derive(IntoElement)]
pub struct Popover {
    open: bool,
    position: Point<gpui::Pixels>,
    anchor: Corner,
    content: AnyElement,
}

impl Popover {
    pub fn new(position: Point<gpui::Pixels>, content: impl IntoElement) -> Self {
        Self {
            open: true,
            position,
            anchor: Corner::TopLeft,
            content: content.into_any_element(),
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn anchor(mut self, anchor: Corner) -> Self {
        self.anchor = anchor;
        self
    }
}

impl RenderOnce for Popover {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        if !self.open {
            return div();
        }

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let radii = theme.radii;
        let motion = theme.motion();

        let surface = div()
            .bg(colors.surface)
            .border_1()
            .border_color(colors.stroke_neutral)
            .rounded(px(radii.md))
            .shadow_md()
            .child(self.content);
        let surface = fluent_core::popup_motion_surface("popover-motion", false, motion, surface);

        div().child(deferred(
            anchored()
                .anchor(self.anchor)
                .position(self.position)
                .snap_to_window()
                .child(surface),
        ))
    }
}
