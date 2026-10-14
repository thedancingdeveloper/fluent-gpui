use fluent_core::Theme;

pub use gpui::{Context, Entity, TestAppContext, VisualTestContext, Window};

/// Build the GPUI application the way the ported crates do: the native platform
/// comes from `gpui_platform`, not `gpui::Application::new`, which the vendored
/// gpui no longer provides.
pub fn application() -> gpui::Application {
    gpui_platform::application()
}

pub fn init_fluent_test(cx: &mut TestAppContext) {
    cx.update(Theme::init);
}

pub fn open_fluent_test_window<V: gpui::Render>(
    cx: &mut TestAppContext,
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
) -> (Entity<V>, &mut VisualTestContext) {
    init_fluent_test(cx);
    cx.add_window_view(build)
}
