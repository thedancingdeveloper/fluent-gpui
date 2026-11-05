use fluent_core::ThemeProvider as _;
use gpui::{
    anchored, deferred, prelude::*, px, Anchor, AnyElement, Bounds, IntoElement, Pixels,
    RenderOnce, Window,
};

/// Standard flyout placements relative to the invoking element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlyoutPlacement {
    /// Left edges aligned, just below the trigger.
    BelowStart,
    /// Top edges aligned, just right of the trigger.
    RightStart,
}

/// A flyout anchored to the window-space bounds of its trigger.
///
/// The flyout is positioned in window coordinates, so the trigger need not be
/// its parent, and it snaps back inside the window. Read the trigger's bounds
/// from a prepaint canvas (as `Dropdown` does) and pass them in. Pointer-positioned
/// context menus should use `ContextMenu`.
#[derive(IntoElement)]
pub struct Flyout {
    child: AnyElement,
    trigger: Bounds<Pixels>,
    placement: FlyoutPlacement,
    priority: usize,
}

impl Flyout {
    /// A flyout showing `child` next to `trigger`, the trigger's bounds in window space.
    pub fn new(trigger: Bounds<Pixels>, child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
            trigger,
            placement: FlyoutPlacement::BelowStart,
            priority: fluent_core::layer::MENU,
        }
    }

    /// Where the flyout sits relative to the trigger.
    pub fn placement(mut self, placement: FlyoutPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Paint priority; defaults to `fluent_core::layer::MENU`. A flyout opened
    /// from another popup passes that popup's priority plus one.
    pub fn priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }
}

impl RenderOnce for Flyout {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let position = match self.placement {
            FlyoutPlacement::BelowStart => self.trigger.bottom_left(),
            FlyoutPlacement::RightStart => self.trigger.top_right(),
        };

        let child = fluent_core::popup_motion_surface(
            "flyout-motion",
            false,
            cx.theme().motion(),
            self.child,
        );
        deferred(
            anchored()
                .anchor(Anchor::TopLeft)
                .position(position)
                .snap_to_window_with_margin(px(8.))
                .child(child),
        )
        .with_priority(self.priority)
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{div, point, px, size, TestAppContext};

    use super::*;

    const TRIGGER: Bounds<Pixels> = Bounds {
        origin: gpui::Point {
            x: px(100.),
            y: px(100.),
        },
        size: gpui::Size {
            width: px(40.),
            height: px(40.),
        },
    };

    /// The entrance animation starts a couple of pixels off; allow for it.
    fn near(a: Pixels, b: Pixels) -> bool {
        (f32::from(a) - f32::from(b)).abs() <= 4.0
    }

    /// The trigger is deliberately not the flyout's parent.
    struct Host(FlyoutPlacement, Bounds<Pixels>);

    impl gpui::Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            div().size_full().child(
                Flyout::new(self.1, div().size(px(30.)).debug_selector(|| "fly".into()))
                    .placement(self.0),
            )
        }
    }

    #[gpui::test]
    fn below_start_sits_under_the_trigger(cx: &mut TestAppContext) {
        let (_, cx) =
            open_fluent_test_window(cx, |_, _| Host(FlyoutPlacement::BelowStart, TRIGGER));
        cx.run_until_parked();
        let f = cx.selector_bounds("fly").unwrap();
        assert!(near(f.origin.x, TRIGGER.origin.x));
        assert!(near(f.origin.y, TRIGGER.bottom_left().y));
    }

    #[gpui::test]
    fn right_start_sits_beside_the_trigger(cx: &mut TestAppContext) {
        let (_, cx) =
            open_fluent_test_window(cx, |_, _| Host(FlyoutPlacement::RightStart, TRIGGER));
        cx.run_until_parked();
        let f = cx.selector_bounds("fly").unwrap();
        assert!(near(f.origin.x, TRIGGER.top_right().x));
        assert!(near(f.origin.y, TRIGGER.origin.y));
    }

    #[gpui::test]
    fn snaps_back_inside_the_window(cx: &mut TestAppContext) {
        let far = Bounds::new(point(px(5000.), px(5000.)), size(px(40.), px(40.)));
        let (_, cx) = open_fluent_test_window(cx, |_, _| Host(FlyoutPlacement::BelowStart, far));
        cx.run_until_parked();
        let f = cx.selector_bounds("fly").unwrap();
        let window = cx.update(|window, _| window.viewport_size());
        assert!(f.origin.x + f.size.width <= window.width);
        assert!(f.origin.y + f.size.height <= window.height);
    }
}
