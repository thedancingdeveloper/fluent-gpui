use fluent_app::{FluentApp, TitleBar};
use fluent_core::ThemeProvider as _;
use fluent_layout::{
    DockPanel, DockPosition, MenuBar, ModalStack, Pane, PaneConstraints, PaneGroup, TabItem,
    TabStrip, Workspace,
};
use fluent_primitives::{
    InputCapture, InputCaptureEvent, TerminalCell, TerminalGrid, TerminalView, TerminalViewEvent,
};
use fluent_ribbon::RibbonBar;
use gpui::{div, prelude::*, rgb, Context, Entity, IntoElement, Render, SharedString, Window};

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
// InputCapture demo: click (or focus + Enter) to grab, Escape or Ctrl+Alt to release
// ---------------------------------------------------------------------------

struct InputCaptureDemo {
    capture: Entity<InputCapture>,
    last: SharedString,
}

impl InputCaptureDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        let capture = cx.new(InputCapture::new);
        cx.subscribe(&capture, |this, _, event: &InputCaptureEvent, cx| {
            this.last = format!("{event:?}").into();
            cx.notify();
        })
        .detach();
        Self {
            capture,
            last: "Click to grab input; Escape or Ctrl+Alt releases".into(),
        }
    }
}

impl Render for InputCaptureDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        div()
            .size_full()
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .bg(colors.surface)
            .text_color(colors.on_subtle)
            .child(self.last.clone())
            .child(div().flex_1().child(self.capture.clone()))
    }
}

// ---------------------------------------------------------------------------
// TerminalView demo: a local echo "shell" over a caller-owned TerminalGrid (no VT parsing)
// ---------------------------------------------------------------------------

struct TerminalDemo {
    grid: Entity<TerminalGrid>,
    view: Entity<TerminalView>,
}

const PROMPT: &str = "gallery$ ";

impl TerminalDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        let grid = cx.new(|_| {
            let mut grid = TerminalGrid::new(80, 24);
            let bold = TerminalCell {
                bold: true,
                ..TerminalCell::default()
            };
            grid.write_str(0, 0, "TerminalView demo", bold);
            let green = TerminalCell {
                fg: Some(rgb(0x6ccb5f).into()),
                ..TerminalCell::default()
            };
            grid.write_str(
                1,
                0,
                "Click to type. Drag to select, Ctrl+Shift+C copies.",
                green,
            );
            let inverse = TerminalCell {
                inverse: true,
                underline: true,
                ..TerminalCell::default()
            };
            grid.write_str(2, 0, "Escape releases input.", inverse);
            let end = grid.write_str(3, 0, PROMPT, TerminalCell::default());
            grid.move_cursor(3, end);
            grid
        });
        let view = cx.new(|cx| TerminalView::new(grid.clone(), cx).label("Gallery terminal"));
        cx.subscribe(&view, |this, _, event: &TerminalViewEvent, cx| {
            if let TerminalViewEvent::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
            {
                this.on_key(key, modifiers.shift, cx);
            }
        })
        .detach();
        Self { grid, view }
    }

    /// A tiny local line editor: printable keys echo, Enter starts a new prompt.
    fn on_key(&mut self, key: &str, shift: bool, cx: &mut Context<Self>) {
        self.grid.update(cx, |grid, cx| {
            let cursor = grid.cursor();
            match key {
                "enter" => {
                    let mut row = cursor.row + 1;
                    if row >= grid.rows() {
                        grid.scroll_up(1);
                        row = grid.rows() - 1;
                    }
                    let end = grid.write_str(row, 0, PROMPT, TerminalCell::default());
                    grid.move_cursor(row, end);
                }
                "backspace" if cursor.col > PROMPT.len() => {
                    grid.set_cell(cursor.row, cursor.col - 1, TerminalCell::default());
                    grid.move_cursor(cursor.row, cursor.col - 1);
                }
                "space" => {
                    grid.move_cursor(cursor.row, cursor.col + 1);
                }
                _ => {
                    let mut chars = key.chars();
                    if let (Some(ch), None) = (chars.next(), chars.next()) {
                        let ch = if shift { ch.to_ascii_uppercase() } else { ch };
                        grid.set_cell(cursor.row, cursor.col, TerminalCell::new(ch));
                        grid.move_cursor(cursor.row, cursor.col + 1);
                    }
                }
            }
            cx.notify();
        });
    }
}

impl Render for TerminalDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().p_2().child(self.view.clone())
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
        let split_list = cx.new(TerminalDemo::new);
        let split_detail = cx.new(InputCaptureDemo::new);
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
