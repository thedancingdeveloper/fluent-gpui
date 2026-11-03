use fluent_core::{LayoutDirection, Theme, ThemeProvider as _};
use gpui::{
    div, ease_out_quint, prelude::*, px, Animation, AnimationExt as _, AnyView, Context,
    FocusHandle, IntoElement, KeyDownEvent, MouseButton, Render, WeakFocusHandle, Window,
};

/// Whether a drawer participates in layout or floats above it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerMode {
    /// The drawer occupies its width in the surrounding row.
    Inline,
    /// The drawer floats above content and supplies a dismissing backdrop.
    #[default]
    Overlay,
}

/// The logical edge from which a drawer opens.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DrawerPosition {
    /// The reading-order start edge (left in LTR, right in RTL).
    #[default]
    Start,
    /// The reading-order end edge (right in LTR, left in RTL).
    End,
}

/// Fluent drawer width presets.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum DrawerSize {
    /// Public API item.
    Small,
    #[default]
    /// Public API item.
    Medium,
    /// Public API item.
    Large,
    /// Public API item.
    Custom(f32),
}

impl DrawerSize {
    fn pixels(self) -> f32 {
        match self {
            Self::Small => 256.0,
            Self::Medium => 320.0,
            Self::Large => 400.0,
            Self::Custom(width) => width.max(0.0),
        }
    }
}

/// A side sheet that can either occupy layout space or appear above content.
///
/// Overlay drawers follow the same interaction contract as other nested
/// overlays: Escape and backdrop clicks are consumed, focus stays inside the
/// drawer, and the opener is restored only after the drawer closes. Inline
/// drawers do not alter focus or install a backdrop.
pub struct Drawer {
    content: AnyView,
    mode: DrawerMode,
    position: DrawerPosition,
    size: DrawerSize,
    open: bool,
    dismissible: bool,
    focus_handle: FocusHandle,
    prior_focus: Option<WeakFocusHandle>,
    had_overlay: bool,
}

impl Drawer {
    /// Public API item.
    pub fn new(content: AnyView, cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        cx.observe_global::<LayoutDirection>(|_, cx| cx.notify())
            .detach();
        Self {
            content,
            mode: DrawerMode::Overlay,
            position: DrawerPosition::Start,
            size: DrawerSize::Medium,
            open: false,
            dismissible: true,
            focus_handle: cx.focus_handle(),
            prior_focus: None,
            had_overlay: false,
        }
    }

    /// Public API item.
    pub fn mode(mut self, mode: DrawerMode) -> Self {
        self.mode = mode;
        self
    }

    /// Public API item.
    pub fn position(mut self, position: DrawerPosition) -> Self {
        self.position = position;
        self
    }

    /// Public API item.
    pub fn size(mut self, size: DrawerSize) -> Self {
        self.size = size;
        self
    }

    /// Public API item.
    pub fn dismissible(mut self, dismissible: bool) -> Self {
        self.dismissible = dismissible;
        self
    }

    /// Public API item.
    pub fn initially_open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Public API item.
    pub fn is_open(&self) -> bool {
        self.open
    }

    /// Public API item.
    pub fn set_open(&mut self, open: bool, cx: &mut Context<Self>) {
        if self.open != open {
            self.open = open;
            cx.notify();
        }
    }

    /// Public API item.
    pub fn toggle(&mut self, cx: &mut Context<Self>) {
        self.set_open(!self.open, cx);
    }

    /// Public API item.
    pub fn set_mode(&mut self, mode: DrawerMode, cx: &mut Context<Self>) {
        if self.mode != mode {
            self.mode = mode;
            cx.notify();
        }
    }

    /// Public API item.
    pub fn set_position(&mut self, position: DrawerPosition, cx: &mut Context<Self>) {
        if self.position != position {
            self.position = position;
            cx.notify();
        }
    }

    fn close_if_dismissible(&mut self, cx: &mut Context<Self>) {
        if self.dismissible {
            self.set_open(false, cx);
        }
    }
}

impl Render for Drawer {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let overlay_open = self.open && self.mode == DrawerMode::Overlay;
        if !overlay_open && self.had_overlay {
            match self.prior_focus.take().and_then(|focus| focus.upgrade()) {
                Some(prior) => window.focus(&prior, cx),
                None => window.blur(cx),
            }
            self.had_overlay = false;
        }

        if !self.open {
            return div().into_any_element();
        }

        let colors = cx.theme().colors.clone();
        let spacing = cx.theme().spacing;
        let duration = cx.theme().motion().normal_duration();
        let width = self.size.pixels();
        let physical_end = self.position == DrawerPosition::End;
        let on_right = physical_end != LayoutDirection::current(cx).is_rtl();
        let surface = div()
            .id("drawer-surface")
            .debug_selector(|| "drawer-surface".into())
            .track_focus(&self.focus_handle)
            .h_full()
            .w(px(width))
            .p(px(spacing.lg))
            .overflow_hidden()
            .bg(colors.surface)
            .border_1()
            .border_color(colors.stroke_neutral)
            .child(self.content.clone());

        if self.mode == DrawerMode::Inline {
            return div()
                .id("drawer-inline")
                .debug_selector(|| "drawer-inline".into())
                .h_full()
                .w(px(width))
                .flex_none()
                .when(on_right, |drawer| drawer.ml_auto())
                .when(!on_right, |drawer| drawer.mr_auto())
                .child(surface)
                .into_any_element();
        }

        if !self.had_overlay {
            self.prior_focus = window.focused(cx).map(|focus| focus.downgrade());
            window.focus(&self.focus_handle, cx);
            self.had_overlay = true;
        }
        if !self.focus_handle.contains_focused(window, cx) {
            window.focus(&self.focus_handle, cx);
        }

        let backdrop = {
            let mut color = colors.surface_dim;
            color.a = 0.6;
            color
        };
        let motion_id = ("drawer-motion", width.to_bits());
        let animated_surface = surface.with_animation(
            motion_id,
            Animation::new(duration).with_easing(ease_out_quint()),
            move |surface, delta| {
                let offset = width * (1.0 - delta);
                if on_right {
                    surface.mr(px(-offset))
                } else {
                    surface.ml(px(-offset))
                }
            },
        );

        div()
            .id("drawer-overlay")
            .debug_selector(|| "drawer-overlay".into())
            .absolute()
            .inset_0()
            .size_full()
            .flex()
            .when(on_right, |overlay| overlay.justify_end())
            .bg(backdrop)
            .on_key_down(cx.listener(|this, event: &KeyDownEvent, window, cx| {
                match event.keystroke.key.as_str() {
                    "escape" => {
                        cx.stop_propagation();
                        this.close_if_dismissible(cx);
                    }
                    "tab" => {
                        cx.stop_propagation();
                        if let Some(focus) = window.focused(cx) {
                            window.focus(&focus, cx);
                        }
                    }
                    _ => {}
                }
            }))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, _, cx| {
                    cx.stop_propagation();
                    this.close_if_dismissible(cx);
                }),
            )
            .child(
                div()
                    .h_full()
                    .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                    .child(animated_surface),
            )
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{point, px, Modifiers, TestAppContext};

    use super::*;

    struct Content;

    struct Root(gpui::Entity<Drawer>);

    impl Render for Root {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().relative().size_full().child(self.0.clone())
        }
    }

    impl Render for Content {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().child("Drawer content")
        }
    }

    fn drawer(
        cx: &mut TestAppContext,
        mode: DrawerMode,
        position: DrawerPosition,
    ) -> (gpui::Entity<Drawer>, &mut gpui::VisualTestContext) {
        let mut handle = None;
        let (_, cx) = open_fluent_test_window(cx, |_, cx| {
            let content = cx.new(|_| Content);
            let entity = cx.new(|cx| {
                Drawer::new(content.into(), cx)
                    .mode(mode)
                    .position(position)
                    .initially_open(true)
            });
            handle = Some(entity.clone());
            Root(entity)
        });
        (handle.expect("drawer entity is created"), cx)
    }

    #[gpui::test]
    fn overlay_backdrop_and_escape_close(cx: &mut TestAppContext) {
        let (drawer, cx) = drawer(cx, DrawerMode::Overlay, DrawerPosition::Start);
        cx.run_until_parked();
        assert!(drawer.read_with(cx, |drawer, _| drawer.is_open()));
        cx.click_at(
            point(px(700.), px(300.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        assert!(!drawer.read_with(cx, |drawer, _| drawer.is_open()));

        drawer.update(cx, |drawer, cx| drawer.set_open(true, cx));
        cx.run_until_parked();
        cx.type_keys("escape");
        assert!(!drawer.read_with(cx, |drawer, _| drawer.is_open()));
    }

    #[gpui::test]
    fn inline_drawer_does_not_install_overlay(cx: &mut TestAppContext) {
        let (drawer, cx) = drawer(cx, DrawerMode::Inline, DrawerPosition::End);
        cx.run_until_parked();
        assert!(drawer.read_with(cx, |drawer, _| drawer.is_open()));
        assert!(cx.debug_bounds("drawer-inline").is_some());
        assert!(cx.debug_bounds("drawer-overlay").is_none());
    }

    #[test]
    fn size_presets_are_stable() {
        assert_eq!(DrawerSize::Small.pixels(), 256.0);
        assert_eq!(DrawerSize::Medium.pixels(), 320.0);
        assert_eq!(DrawerSize::Large.pixels(), 400.0);
        assert_eq!(DrawerSize::Custom(-1.0).pixels(), 0.0);
    }
}
