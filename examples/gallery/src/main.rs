use families::{DemoViews, FamilyHost, FamilyNav};
use fluent_app::{FluentApp, TitleBar};
use fluent_core::gpui::{div, prelude::*, Context, IntoElement, Render, Window};
use fluent_layout::{
    DockPanel, DockPosition, MenuBar, ModalStack, Pane, TabItem, TabStrip, Workspace,
};
use fluent_primitives::{InputCapture, TerminalCell, TerminalGrid, TerminalView, VideoSurface};
use fluent_ribbon::RibbonBar;
mod families;

// ---------------------------------------------------------------------------
// Root gallery view — wires the full shell together
// ---------------------------------------------------------------------------

struct GalleryRoot {
    workspace: fluent_core::gpui::Entity<Workspace>,
}

impl Render for GalleryRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.workspace.clone())
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    FluentApp::new("FluentGUI Gallery").dark_theme().run(|cx| {
        let video = cx.new(VideoSurface::new);
        video.update(cx, |surface, cx| {
            let mut pixels = Vec::with_capacity(160 * 90 * 4);
            for y in 0..90u32 {
                for x in 0..160u32 {
                    pixels.extend([((x * 255) / 159) as u8, ((y * 255) / 89) as u8, 180, 255]);
                }
            }
            let _ = surface.submit_frame(160, 90, pixels, cx);
        });
        let video_page = cx.new(|_| families::VideoPage { surface: video });

        let grid = cx.new(|_| TerminalGrid::new(64, 12));
        grid.update(cx, |grid, _| {
            for (row, line) in [
                "Fluent terminal demo",
                "$ cargo run --example gallery",
                "[  OK  ] compositor connected",
                "[  OK  ] synthetic output ready",
                "Type, scroll, and select this terminal surface.",
            ]
            .iter()
            .enumerate()
            {
                grid.write_str(row, 0, line, TerminalCell::new(' '));
            }
        });
        let terminal = cx.new(|cx| TerminalView::new(grid, cx).label("Scripted terminal"));
        let terminal_page = cx.new(|_| families::TerminalPage { terminal });

        let capture_log = cx.new(|_| families::EventLog);
        let capture = cx.new(|cx| InputCapture::new(cx).content(capture_log.clone().into()));
        let input_page = cx.new(|_| families::InputPage { capture });

        let expander_page = cx.new(|_| families::ExpanderPage);
        let drag_drop_page = cx.new(|_| families::DragDropPage);
        let grouped_list_page = cx.new(|_| families::GroupedListPage);
        let family_host = cx.new(|_| {
            FamilyHost::new(DemoViews {
                status: Vec::new(),
                data: vec![drag_drop_page.into(), grouped_list_page.into()],
                date_input: Vec::new(),
                media: vec![video_page.clone().into(), terminal_page.clone().into()],
                theme: vec![input_page.clone().into(), expander_page.clone().into()],
            })
        });
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
        let sidebar = cx.new(|cx| {
            DockPanel::new(cx, DockPosition::Left, "Navigation")
                .size(240.0)
                .content(cx.new(|_| FamilyNav::new(family_host.clone())).into())
        });

        // Main pane with a tab strip
        let tab_strip = cx.new(|cx: &mut fluent_core::gpui::Context<TabStrip>| {
            let mut ts = TabStrip::new(cx);
            ts.add_tab(TabItem::new("dashboard").label("Dashboard").closable(true));
            ts.add_tab(TabItem::new("settings").label("Settings").closable(true));
            ts
        });
        let main_pane = cx.new(|cx| Pane::new(cx).with_tab_strip(tab_strip));
        // set content separately since we need the entity
        main_pane.update(cx, |p, cx| {
            p.set_content(family_host.into(), cx);
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
