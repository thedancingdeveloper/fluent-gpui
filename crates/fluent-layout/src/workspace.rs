use fluent_core::{Theme, ThemeProvider as _};
use gpui::{div, prelude::*, AnyView, Context, Entity, IntoElement, Render, SharedString, Window};

use crate::{
    dock::DockPanel,
    menu_bar::{MenuBar, MenuItemDef},
    modal::ModalHost,
};

/// The top-level application workspace shell.
///
/// Assembles: optional title bar, optional menu bar, optional ribbon,
/// left/right/bottom dock panels, central pane area, and modal host overlay.
#[derive(Default)]
pub struct Workspace {
    pub title_bar: Option<AnyView>,
    pub menu_bar: Option<Entity<MenuBar>>,
    pub ribbon: Option<AnyView>,
    pub left_dock: Option<Entity<DockPanel>>,
    pub right_dock: Option<Entity<DockPanel>>,
    pub bottom_dock: Option<Entity<DockPanel>>,
    pub content: Option<AnyView>,
    pub modal_host: Option<Entity<ModalHost>>,
    /// Optional thin status bar rendered at the very bottom of the window (below the body).
    pub status_bar: Option<AnyView>,
    /// When set, a "File" menu is prepended to menu_bar with an Exit item wired to cx.quit().
    app_name_for_file_menu: Option<SharedString>,
}

impl Workspace {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self::default()
    }

    pub fn title_bar(mut self, bar: AnyView) -> Self {
        self.title_bar = Some(bar);
        self
    }

    pub fn menu_bar(mut self, bar: Entity<MenuBar>) -> Self {
        self.menu_bar = Some(bar);
        self
    }

    /// Prepend a built-in "File" menu with an Exit item that calls `cx.quit()`.
    ///
    /// Pass `app_name` to label the menu "File" (ignored for now; reserved for
    /// future "About <app>" or per-platform behaviour).
    pub fn with_file_menu(mut self, _app_name: impl Into<SharedString>) -> Self {
        self.app_name_for_file_menu = Some("File".into());
        self
    }

    pub fn ribbon(mut self, ribbon: AnyView) -> Self {
        self.ribbon = Some(ribbon);
        self
    }

    pub fn left_dock(mut self, dock: Entity<DockPanel>) -> Self {
        self.left_dock = Some(dock);
        self
    }

    pub fn right_dock(mut self, dock: Entity<DockPanel>) -> Self {
        self.right_dock = Some(dock);
        self
    }

    pub fn bottom_dock(mut self, dock: Entity<DockPanel>) -> Self {
        self.bottom_dock = Some(dock);
        self
    }

    pub fn content(mut self, view: AnyView) -> Self {
        self.content = Some(view);
        self
    }

    pub fn modal_host(mut self, host: Entity<ModalHost>) -> Self {
        self.modal_host = Some(host);
        self
    }

    pub fn status_bar(mut self, bar: AnyView) -> Self {
        self.status_bar = Some(bar);
        self
    }

    pub fn set_status_bar(&mut self, bar: AnyView, cx: &mut Context<Self>) {
        self.status_bar = Some(bar);
        cx.notify();
    }

    pub fn set_content(&mut self, view: AnyView, cx: &mut Context<Self>) {
        self.content = Some(view);
        cx.notify();
    }
}

impl Render for Workspace {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();

        // Inject the built-in File menu (Exit) if requested and a menu_bar exists.
        if let Some(ref _label) = self.app_name_for_file_menu {
            if let Some(ref bar) = self.menu_bar {
                let exit_item =
                    MenuItemDef::action_with_shortcut("Exit", "Alt+F4", |_, _, cx| cx.quit());
                bar.update(cx, |bar, _| {
                    bar.ensure_front_menu("File", vec![exit_item]);
                });
            }
        }

        // Full-window flex column: title bar + menu bar + ribbon on top, then the body.
        // .relative() is required so that the ModalHost's absolute overlay is contained here.
        let mut root = div()
            .flex()
            .flex_col()
            .size_full()
            .relative()
            .bg(colors.surface);

        // Title bar (topmost — draggable, window controls)
        if let Some(tb) = &self.title_bar {
            root = root.child(tb.clone());
        }

        // Menu bar (below title bar, above ribbon)
        if let Some(bar) = &self.menu_bar {
            root = root.child(bar.clone());
        }

        // Ribbon
        if let Some(ribbon) = &self.ribbon {
            root = root.child(ribbon.clone());
        }

        // Body: [left dock] [center] [right dock]
        let mut body = div()
            .flex()
            .flex_row()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .h_full()
            .w_full();

        if let Some(dock) = &self.left_dock {
            body = body.child(dock.clone());
        }

        // Center: [content area] stacked with bottom dock
        let mut center = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .h_full()
            .w_full();

        if let Some(content) = &self.content {
            center = center.child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .h_full()
                    .w_full()
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_h_0()
                            .min_w_0()
                            .h_full()
                            .w_full()
                            .child(content.clone()),
                    ),
            );
        }

        if let Some(dock) = &self.bottom_dock {
            center = center.child(dock.clone());
        }

        body = body.child(center);

        if let Some(dock) = &self.right_dock {
            body = body.child(dock.clone());
        }

        root = root.child(body);

        // Status bar (thin strip at the very bottom, above the modal overlay)
        if let Some(bar) = &self.status_bar {
            root = root.child(bar.clone());
        }

        // Modal host paints above siblings because it is the last child of the relative root.
        if let Some(host) = &self.modal_host {
            root = root.child(host.clone());
        }

        root
    }
}
