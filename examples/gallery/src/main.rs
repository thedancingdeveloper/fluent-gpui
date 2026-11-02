use fluent_app::{FluentApp, TitleBar};
use fluent_core::ThemeProvider as _;
use fluent_layout::{
    DockPanel, DockPosition, MenuBar, ModalStack, Pane, PaneConstraints, PaneGroup, TabItem,
    TabStrip, Workspace,
};
use fluent_ribbon::RibbonBar;
use gpui::{div, prelude::*, Context, IntoElement, Render, Window};

// ---------------------------------------------------------------------------
// Root gallery view — wires the full shell together
// ---------------------------------------------------------------------------

struct GalleryRoot {
    workspace: gpui::Entity<Workspace>,
}

impl Render for GalleryRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.workspace.clone())
    }
}

// ---------------------------------------------------------------------------
// Placeholder pane content
// ---------------------------------------------------------------------------

struct PlaceholderView {
    label: gpui::SharedString,
}

impl Render for PlaceholderView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        div()
            .size_full()
            .flex()
            .items_center()
            .justify_center()
            .bg(colors.surface)
            .text_color(colors.on_subtle)
            .child(self.label.clone())
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    FluentApp::new("FluentGUI Gallery").dark_theme().run(|cx| {
        // Title bar
        let title_bar = cx.new(|cx| TitleBar::new(cx, "FluentGUI Gallery"));

        // Menu bar
        let menu_bar = cx.new(MenuBar::new);

        // Ribbon
        let ribbon = cx.new(|cx| {
            RibbonBar::new(cx)
                .tab("Home", |t| {
                    t.group("Actions", |g| {
                        g.large_button("New", "icons/add.svg", |_, _, _| {})
                            .large_button("Open", "icons/folder_open.svg", |_, _, _| {})
                            .separator()
                            .stack(|s| {
                                s.button("Cut", "icons/cut.svg", |_, _, _| {})
                                    .button("Copy", "icons/copy.svg", |_, _, _| {})
                                    .button("Paste", "icons/paste.svg", |_, _, _| {})
                            })
                    })
                    .group("View", |g| {
                        g.toggle_button("Dark", "icons/dark_theme.svg", true, |_, _, _, _| {})
                            .toggle_button("Light", "icons/light_theme.svg", false, |_, _, _, _| {})
                    })
                })
                .tab("Edit", |t| {
                    t.group("Text", |g| {
                        g.button("Find", "icons/search.svg", |_, _, _| {}).button(
                            "Replace",
                            "icons/find_replace.svg",
                            |_, _, _| {},
                        )
                    })
                })
                .contextual_tab("Session", true, |t| {
                    t.group("Terminal", |g| {
                        g.large_button("Clear", "icons/clear.svg", |_, _, _| {})
                            .icon_button("icons/copy.svg", "Copy", |_, _, _| {})
                    })
                })
        });

        // Left sidebar
        let sidebar_content = cx.new(|_| PlaceholderView {
            label: "Navigation tree".into(),
        });
        let sidebar = cx.new(|cx| {
            DockPanel::new(cx, DockPosition::Left, "Navigation")
                .size(240.0)
                .content(sidebar_content.into())
        });

        // Main pane with a tab strip
        let tab_strip = cx.new(|cx: &mut gpui::Context<TabStrip>| {
            let mut ts = TabStrip::new(cx);
            ts.add_tab(TabItem::new("dashboard").label("Dashboard").closable(true));
            ts.add_tab(TabItem::new("settings").label("Settings").closable(true));
            ts
        });
        let split_list = cx.new(|_| PlaceholderView {
            label: "Resizable first panel".into(),
        });
        let split_detail = cx.new(|_| PlaceholderView {
            label: "Resizable second panel".into(),
        });
        let pane_content = cx.new(|_| {
            PaneGroup::horizontal()
                .first(split_list.into())
                .second(split_detail.into())
                .ratio(0.36)
                .constraints(
                    PaneConstraints::new()
                        .first_min(220.0)
                        .second_min(320.0)
                        .first_max(Some(520.0)),
                )
        });
        let main_pane = cx.new(|cx| Pane::new(cx).with_tab_strip(tab_strip));
        // set content separately since we need the entity
        main_pane.update(cx, |p, cx| {
            p.set_content(pane_content.into(), cx);
        });

        // Modal host
        ModalStack::init(cx);
        let modal_host = cx.new(fluent_layout::ModalHost::new);

        // Workspace
        let workspace = cx.new(|cx| {
            Workspace::new(cx)
                .title_bar(title_bar.into())
                .menu_bar(menu_bar)
                .with_file_menu("FluentGUI Gallery")
                .ribbon(ribbon.into())
                .left_dock(sidebar)
                .content(main_pane.into())
                .modal_host(modal_host)
        });

        cx.new(|_| GalleryRoot { workspace })
    });
}
