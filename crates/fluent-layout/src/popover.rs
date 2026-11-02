use fluent_core::ThemeProvider as _;
use gpui::{
    anchored, deferred, div, prelude::*, px, Anchor, AnyElement, IntoElement, Point, RenderOnce,
    Window,
};

/// A lightweight anchored floating surface.
#[derive(IntoElement)]
pub struct Popover {
    open: bool,
    position: Point<gpui::Pixels>,
    anchor: Anchor,
    content: AnyElement,
}

impl Popover {
    pub fn new(position: Point<gpui::Pixels>, content: impl IntoElement) -> Self {
        Self {
            open: true,
            position,
            anchor: Anchor::TopLeft,
            content: content.into_any_element(),
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn anchor(mut self, anchor: Anchor) -> Self {
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

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{point, px, TestAppContext};

    use super::*;

    struct Host {
        open: bool,
    }

    impl gpui::Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            div().size_full().child(
                Popover::new(
                    point(px(40.), px(40.)),
                    div().size(px(50.)).debug_selector(|| "pop-content".into()),
                )
                .open(self.open),
            )
        }
    }

    #[gpui::test]
    fn renders_only_while_open(cx: &mut TestAppContext) {
        let (host, cx) = open_fluent_test_window(cx, |_, _| Host { open: true });
        cx.run_until_parked();
        assert!(cx.selector_bounds("pop-content").is_some());
        host.update(cx, |h, cx| {
            h.open = false;
            cx.notify();
        });
        cx.run_until_parked();
        assert!(cx.selector_bounds("pop-content").is_none());
    }
}
