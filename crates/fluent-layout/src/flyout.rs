use fluent_core::ThemeProvider as _;
use gpui::{deferred, div, prelude::*, px, AnyElement, IntoElement, RenderOnce, Window};

/// Standard flyout placements relative to the invoking element.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FlyoutPlacement {
    BelowStart,
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
    pub fn new(child: impl IntoElement) -> Self {
        Self {
            child: child.into_any_element(),
            placement: FlyoutPlacement::BelowStart,
            trigger_width: 0.0,
            trigger_height: 0.0,
            priority: 1,
        }
    }

    pub fn placement(mut self, placement: FlyoutPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn trigger_size(mut self, width: f32, height: f32) -> Self {
        self.trigger_width = width;
        self.trigger_height = height;
        self
    }

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
