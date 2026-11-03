use fluent_core::ThemeProvider as _;
use gpui::{deferred, div, prelude::*, px, AnyElement, IntoElement, RenderOnce, Window};

/// Standard flyout placements relative to the invoking element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlyoutPlacement {
    /// Public API item.
    BelowStart,
    /// Public API item.
    RightStart,
}

/// Lightweight local-positioned flyout wrapper.
///
/// Use this when the invoking element owns the flyout, such as menu-bar
/// dropdowns and cascade menus. Pointer-positioned context menus should use
/// `ContextMenu`.
#[derive(IntoElement)]
pub struct Flyout {
    child: AnyElement,
    placement: FlyoutPlacement,
    trigger_width: f32,
    trigger_height: f32,
    priority: usize,
}

impl Flyout {
    /// Public API item.
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
            placement: FlyoutPlacement::BelowStart,
            trigger_width: 0.0,
            trigger_height: 0.0,
            priority: 1,
        }
    }

    /// Public API item.
    pub fn placement(mut self, placement: FlyoutPlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Public API item.
    pub fn trigger_size(mut self, width: f32, height: f32) -> Self {
        self.trigger_width = width;
        self.trigger_height = height;
        self
    }

    /// Public API item.
    pub fn priority(mut self, priority: usize) -> Self {
        self.priority = priority;
        self
    }
}

impl RenderOnce for Flyout {
    fn render(self, _window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let mut root = div().absolute();
        root = match self.placement {
            FlyoutPlacement::BelowStart => {
                if self.trigger_height > 0.0 {
                    root.left(px(0.0)).top(px(self.trigger_height))
                } else {
                    root.left_0().top_full()
                }
            }
            FlyoutPlacement::RightStart => {
                if self.trigger_width > 0.0 {
                    root.left(px(self.trigger_width)).top(px(0.0))
                } else {
                    root.left_full().top_0()
                }
            }
        };

        let child = fluent_core::popup_motion_surface(
            "flyout-motion",
            false,
            cx.theme().motion(),
            self.child,
        );
        root.child(deferred(child).with_priority(self.priority))
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{px, TestAppContext};

    use super::*;

    /// The entrance animation starts a couple of pixels off; allow for it.
    fn near(a: gpui::Pixels, b: gpui::Pixels) -> bool {
        (f32::from(a) - f32::from(b)).abs() <= 4.0
    }

    struct Host(FlyoutPlacement);

    impl gpui::Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            div().size_full().child(
                div()
                    .relative()
                    .left(px(100.))
                    .top(px(100.))
                    .size(px(40.))
                    .debug_selector(|| "trigger".into())
                    .child(
                        Flyout::new(div().size(px(30.)).debug_selector(|| "fly".into()))
                            .placement(self.0)
                            .trigger_size(40., 40.),
                    ),
            )
        }
    }

    #[gpui::test]
    fn below_start_sits_under_the_trigger(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, _| Host(FlyoutPlacement::BelowStart));
        cx.run_until_parked();
        let t = cx.selector_bounds("trigger").unwrap();
        let f = cx.selector_bounds("fly").unwrap();
        assert!(near(f.origin.x, t.origin.x));
        assert!(near(f.origin.y, t.origin.y + t.size.height));
    }

    #[gpui::test]
    fn right_start_sits_beside_the_trigger(cx: &mut TestAppContext) {
        let (_, cx) = open_fluent_test_window(cx, |_, _| Host(FlyoutPlacement::RightStart));
        cx.run_until_parked();
        let t = cx.selector_bounds("trigger").unwrap();
        let f = cx.selector_bounds("fly").unwrap();
        assert!(near(f.origin.x, t.origin.x + t.size.width));
        assert!(near(f.origin.y, t.origin.y));
    }
}
