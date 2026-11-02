use fluent_app::{FluentApp, TitleBar};
use fluent_core::{LayoutDirection, Theme, ThemeProvider as _};
use fluent_layout::{
    DataCell, DataColumn, DataRow, DataStatusKind, DataTable, DataTableState, DockPanel,
    DockPosition, MenuBar, MessageBar, MessageIntent, ModalStack, Pane, SelectableList,
    SelectableListItem, TabItem, TabStrip, Tree, TreeItem, Workspace,
};
use fluent_primitives::{
    Avatar, Button, ButtonAppearance, Checkbox, CounterBadge, CounterBadgeAppearance,
    DateTimeLocale, DateTimePicker, DateTimeValue, InputCapture, InputCaptureEvent, Link,
    LinkAppearance, Persona, PersonaSize, PresenceBadgeState, ProgressBar, Slider, SpinButton,
    Spinner, Switch, TerminalCell, TerminalGrid, TerminalView, TerminalViewEvent, ToggleButton,
    VideoScaling, VideoSurface, VideoSurfaceEvent,
};
use fluent_ribbon::RibbonBar;

mod families;
use families::{DemoViews, FamilyHost, FamilyNav};
use gpui::{div, prelude::*, rgb, Context, Entity, IntoElement, Render, SharedString, Window};

// ---------------------------------------------------------------------------
// Root gallery view — wires the full shell together
// ---------------------------------------------------------------------------

struct GalleryRoot {
    workspace: gpui::Entity<Workspace>,
}

/// Data-heavy screens in every non-ready state: an rtbit torrent list and an
/// rdpapp connection tree.
struct DataStatesDemo;

impl DataStatesDemo {
    fn torrents(state: DataTableState, full: bool) -> DataTable {
        let mut table = DataTable::new()
            .columns([
                DataColumn::new("name", "Name"),
                DataColumn::new("progress", "Progress").width(90.0),
                DataColumn::new("down", "Down").width(90.0),
            ])
            .state(state);
        if full {
            table = table.rows([
                DataRow::new(
                    "t1",
                    [
                        DataCell::new("ubuntu-24.04.iso"),
                        DataCell::new("62%"),
                        DataCell::new("4.1 MB/s"),
                    ],
                ),
                DataRow::new(
                    "t2",
                    [
                        DataCell::new("debian-12.img"),
                        DataCell::new("100%"),
                        DataCell::new("0 B/s"),
                    ],
                ),
            ]);
        }
        table
    }
}

/// A virtualised SelectableList over 100,000 rows; every third row wraps a
/// second line, so row heights vary. Only the rows near the viewport are built.
struct BigListDemo {
    selected: usize,
}

/// A 100,000-row DataTable story demonstrating the same lazy row construction
/// used by production tables.
struct BigTableDemo;

impl Render for BigTableDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().p_2().child(
            DataTable::new()
                .columns([
                    DataColumn::new("name", "Name").width(220.0),
                    DataColumn::new("value", "Value").width(100.0),
                ])
                .rows((0..100_000).map(|ix| {
                    DataRow::new(
                        format!("table-row-{ix}"),
                        [format!("Row {ix}"), ix.to_string()],
                    )
                }))
                .max_body_height(360.0),
        )
    }
}

impl Render for BigListDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_2()
            .child(div().text_size(gpui::px(12.0)).child(format!(
                "100,000 rows (variable height), row {} selected: click, or use Up/Down/PageUp/PageDown/Home/End",
                self.selected + 1
            )))
            .child(
                SelectableList::new("gallery-big-list")
                    .rows(100_000, |ix| {
                        let item = SelectableListItem::new(format!("row-{ix}"), format!("Row {ix}"))
                            .meta(format!("#{ix}"));
                        if ix % 3 == 0 {
                            item.subtitle("A second line that only every third row has")
                        } else {
                            item
                        }
                    })
                    .variable_row_heights(true)
                    .row_height(44.0)
                    .selected_index(self.selected)
                    .max_body_height(360.0)
                    .on_select(move |id, _, cx| {
                        if let Some(ix) = id.strip_prefix("row-").and_then(|n| n.parse().ok()) {
                            view.update(cx, |demo, cx| {
                                demo.selected = ix;
                                cx.notify();
                            });
                        }
                    }),
            )
    }
}

impl Render for DataStatesDemo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let label = |t: &'static str| div().text_size(gpui::px(12.0)).child(t);
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_3()
            .child(label("rtbit torrents: ready"))
            .child(Self::torrents(DataTableState::Ready, true))
            .child(label("rtbit torrents: loading"))
            .child(Self::torrents(
                DataTableState::Loading("Loading torrents…".into()),
                false,
            ))
            .child(label("rtbit torrents: empty"))
            .child(Self::torrents(
                DataTableState::Empty("No torrents. Add a magnet link to start.".into()),
                false,
            ))
            .child(label("rtbit torrents: no results"))
            .child(Self::torrents(
                DataTableState::NoResults("No torrents match \"ubuntu\".".into()),
                false,
            ))
            .child(label("rtbit torrents: error"))
            .child(Self::torrents(
                DataTableState::Error(
                    "Cannot reach the rtbit session (connection refused).".into(),
                ),
                false,
            ))
            .child(label(
                "rdpapp connections: ready / filter with no match / error",
            ))
            .child(
                Tree::new().item(
                    TreeItem::new("prod", "Production")
                        .expanded(true)
                        .child(TreeItem::new("srv1", "srv-01.corp")),
                ),
            )
            .child(
                Tree::new()
                    .item(
                        TreeItem::new("prod", "Production")
                            .child(TreeItem::new("srv1", "srv-01.corp")),
                    )
                    .filter("zzz"),
            )
            .child(Tree::new().status(DataStatusKind::Error, "Could not load the connection list."))
    }
}

struct CounterBadgeDemo;

struct PersonaDemo;

/// Date/time pickers in en-US and de-DE (day.month.year, 24-hour).
struct DateTimeDemo {
    us: Entity<DateTimePicker>,
    de: Entity<DateTimePicker>,
}

impl DateTimeDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        let when = DateTimeValue::new(2026, 5, 25, 21, 30).unwrap();
        let us = cx.new(|_| {
            DateTimePicker::new()
                .locale(DateTimeLocale::en_us())
                .value(when)
        });
        let de = cx.new(|_| {
            DateTimePicker::new()
                .locale(DateTimeLocale::de_de())
                .value(when)
        });
        Self { us, de }
    }
}

impl Render for DateTimeDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        div()
            .flex()
            .flex_col()
            .gap(gpui::px(theme.spacing.sm))
            .p(gpui::px(theme.spacing.md))
            .child("Date/time: en-US and de-DE (locale and timezone owned by the app)")
            .child(self.us.clone())
            .child(self.de.clone())
    }
}

/// rtbit download progress: determinate, indeterminate, stalled and failed states.
struct DownloadProgressDemo;

impl Render for DownloadProgressDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let row = |name: &'static str, detail: &'static str, bar: ProgressBar| {
            div()
                .flex()
                .flex_col()
                .gap_1()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .child(name)
                        .child(div().text_color(colors.on_subtle).child(detail)),
                )
                .child(bar)
        };
        div()
            .flex()
            .flex_col()
            .gap(gpui::px(theme.spacing.sm))
            .p(gpui::px(theme.spacing.md))
            .child("rtbit downloads (value text is announced; animation honours reduced motion)")
            .child(row(
                "ubuntu-26.04-desktop.iso",
                "412 of 900 MB",
                ProgressBar::new(0.46).value_text("412 of 900 megabytes"),
            ))
            .child(row(
                "Fetching metadata",
                "Waiting for peers",
                ProgressBar::new(0.0).indeterminate(),
            ))
            .child(row(
                "sample-set.tar",
                "Done",
                ProgressBar::new(1.0).success(),
            ))
            .child(row(
                "archive.zip",
                "Stalled",
                ProgressBar::new(0.7).warning(),
            ))
            .child(
                MessageBar::new("Tracker unreachable; the download stopped at 12%.")
                    .intent(MessageIntent::Error)
                    .title("Download failed")
                    .action("Retry", |_, _, _| {}),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Spinner::new().label("Checking files"))
                    .child("Checking files"),
            )
    }
}

struct ThemeMatrixDemo;

/// Shows Arabic and Hebrew sample content under the active layout direction.
/// The rows reorder (logical start/end) but the text itself is never mirrored.
struct RtlDemo;

impl Render for RtlDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let rtl = LayoutDirection::current(cx).is_rtl();
        let row = |label: &'static str, text: &'static str| {
            let r = div().flex().items_center().gap_2();
            let r = if rtl { r.flex_row_reverse() } else { r };
            r.child(div().text_color(colors.on_subtle).child(label))
                .child(div().child(text))
        };
        div()
            .size_full()
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .bg(colors.surface)
            .text_color(colors.on_neutral)
            .child("Layout direction (RTL)")
            .child(
                Button::new("rtl-toggle")
                    .label(if rtl {
                        "Switch to LTR"
                    } else {
                        "Switch to RTL"
                    })
                    .on_click(|_, _, cx| LayoutDirection::toggle(cx)),
            )
            .child(row("Arabic", "مرحبا بالعالم"))
            .child(row("Hebrew", "שלום עולם"))
    }
}

impl Render for ThemeMatrixDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        div()
            .size_full()
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .bg(colors.surface)
            .text_color(colors.on_neutral)
            .child("Theme token matrix")
            .child("Neutral, accent, selected, disabled, status, border, and focus states")
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(Button::new("matrix-neutral").label("Neutral"))
                    .child(
                        Button::new("matrix-accent")
                            .label("Accent")
                            .appearance(ButtonAppearance::Accent),
                    )
                    .child(
                        Button::new("matrix-disabled")
                            .label("Disabled")
                            .disabled(true),
                    )
                    .child(
                        ToggleButton::new("matrix-selected")
                            .label("Selected")
                            .selected(true),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        Checkbox::new("matrix-checkbox")
                            .label("Checked")
                            .checked(true),
                    )
                    .child(Switch::new("matrix-switch").label("On").on(true))
                    .child(
                        div()
                            .p_2()
                            .border_1()
                            .border_color(colors.status_error_border)
                            .bg(colors.status_error_bg)
                            .text_color(colors.status_error)
                            .child("Error status with explicit foreground, background, and border"),
                    )
                    .child(
                        div()
                            .p_2()
                            .border_1()
                            .border_color(colors.status_success_border)
                            .bg(colors.status_success_bg)
                            .text_color(colors.status_success)
                            .child("Success status"),
                    )
                    .child(
                        div()
                            .p_2()
                            .border_1()
                            .border_color(colors.status_warning_border)
                            .bg(colors.status_warning_bg)
                            .text_color(colors.status_warning)
                            .child("Warning status"),
                    )
                    .child(
                        div()
                            .p_2()
                            .border_2()
                            .border_color(colors.focus_ring)
                            .text_color(colors.on_neutral)
                            .child("Focus ring"),
                    ),
            )
    }
}

impl Render for PersonaDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_3()
            .flex()
            .flex_col()
            .gap_3()
            .bg(cx.theme().colors.surface)
            .child("Persona")
            .child(
                Persona::new(Avatar::initials("AM"), "Alex Morgan")
                    .secondary("Available")
                    .presence(PresenceBadgeState::Available),
            )
            .child(
                Persona::new(Avatar::initials("JR"), "Jordan Rivera")
                    .secondary("Do not disturb")
                    .presence(PresenceBadgeState::DoNotDisturb)
                    .size(PersonaSize::Large),
            )
    }
}

impl Render for CounterBadgeDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        div()
            .size_full()
            .p_3()
            .flex()
            .flex_col()
            .gap_2()
            .bg(colors.surface)
            .text_color(colors.on_subtle)
            .child("CounterBadge")
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(CounterBadge::new(7))
                    .child(CounterBadge::new(99))
                    .child(CounterBadge::new(0).show_zero(true))
                    .child(CounterBadge::new(0).dot(true))
                    .child(CounterBadge::new(120).appearance(CounterBadgeAppearance::Ghost)),
            )
    }
}

impl Render for GalleryRoot {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div().size_full().child(self.workspace.clone())
    }
}

// ---------------------------------------------------------------------------
// Slider and SpinButton demo: caller-owned values, every state
// ---------------------------------------------------------------------------

struct RangeInputDemo {
    volume: f64,
    vertical: f64,
    threads: f64,
    opacity: f64,
}

impl Render for RangeInputDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let entity = cx.entity();
        let set = |f: fn(&mut RangeInputDemo, f64)| {
            let entity = entity.clone();
            move |v: f64, _: &mut Window, cx: &mut gpui::App| {
                entity.update(cx, |d, cx| {
                    f(d, v);
                    cx.notify();
                })
            }
        };
        div()
            .size_full()
            .p_3()
            .flex()
            .flex_col()
            .gap_3()
            .bg(colors.surface)
            .text_color(colors.on_neutral)
            .child("Slider and SpinButton: click a control, then use the keys")
            .child(format!("Volume {}", self.volume))
            .child(
                Slider::new("gallery-slider")
                    .label("Volume")
                    .value(self.volume)
                    .step(5.0)
                    .on_change(set(|d, v| d.volume = v)),
            )
            .child(
                Slider::new("gallery-slider-disabled")
                    .label("Disabled")
                    .value(30.0)
                    .disabled(true),
            )
            .child(
                Slider::new("gallery-slider-vertical")
                    .label("Vertical")
                    .vertical()
                    .length(120.0)
                    .range(0.0, 1.0)
                    .step(0.1)
                    .value(self.vertical)
                    .on_change(set(|d, v| d.vertical = v)),
            )
            .child(
                SpinButton::new("gallery-spin")
                    .label("Threads")
                    .range(1.0, 16.0)
                    .value(self.threads)
                    .format(|v| format!("{v} threads"))
                    .on_change(set(|d, v| d.threads = v)),
            )
            .child(
                SpinButton::new("gallery-spin-percent")
                    .label("Opacity")
                    .range(0.0, 100.0)
                    .step(10.0)
                    .value(self.opacity)
                    .format(|v| format!("{v}%"))
                    .parse(|t| t.trim_end_matches('%').trim().parse().ok())
                    .on_change(set(|d, v| d.opacity = v)),
            )
            .child(
                SpinButton::new("gallery-spin-disabled")
                    .label("Disabled")
                    .value(4.0)
                    .disabled(true),
            )
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
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap_3()
                    .child(Link::new("gallery-link-default", "Default link").on_click(|_, _, _| {}))
                    .child(
                        Link::new("gallery-link-subtle", "Subtle link")
                            .appearance(LinkAppearance::Subtle)
                            .on_click(|_, _, _| {}),
                    )
                    .child(Link::new("gallery-link-disabled", "Disabled link").disabled(true)),
            )
            .child(
                div()
                    .child("Inline link: read the ")
                    .child(
                        Link::new("gallery-link-inline", "input capture guide")
                            .open_url("https://github.com/sprooty/fluent-gpui"),
                    )
                    .child(" before grabbing input."),
            )
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
// VideoSurface demo: an animated generated RGBA8 frame; click to toggle Fit / Native
// ---------------------------------------------------------------------------

const VIDEO_W: u32 = 320;
const VIDEO_H: u32 = 180;

struct VideoDemo {
    surface: Entity<VideoSurface>,
    tick: u32,
    status: SharedString,
}

impl VideoDemo {
    fn new(cx: &mut Context<Self>) -> Self {
        let surface = cx.new(|cx| {
            let mut surface = VideoSurface::new(cx);
            surface.label("Generated video", cx);
            surface
        });
        cx.subscribe(&surface, |this, _, event: &VideoSurfaceEvent, cx| {
            let VideoSurfaceEvent::Resized {
                device_width,
                device_height,
                ..
            } = event;
            this.status = format!("Surface {device_width}x{device_height} device px").into();
            cx.notify();
        })
        .detach();
        cx.spawn(async move |this, cx| loop {
            cx.background_executor()
                .timer(std::time::Duration::from_millis(33))
                .await;
            if this.update(cx, |this, cx| this.next_frame(cx)).is_err() {
                break;
            }
        })
        .detach();
        Self {
            surface,
            tick: 0,
            status: "".into(),
        }
    }

    /// A moving gradient with a bouncing square, generated on the CPU.
    fn next_frame(&mut self, cx: &mut Context<Self>) {
        self.tick = self.tick.wrapping_add(1);
        let t = self.tick;
        let mut pixels = vec![0u8; (VIDEO_W * VIDEO_H * 4) as usize];
        let sx = (t * 3) % (VIDEO_W - 40);
        let sy = (t * 2) % (VIDEO_H - 40);
        for (i, px) in pixels.as_chunks_mut::<4>().0.iter_mut().enumerate() {
            let (x, y) = (i as u32 % VIDEO_W, i as u32 / VIDEO_W);
            let square = (sx..sx + 40).contains(&x) && (sy..sy + 40).contains(&y);
            px.copy_from_slice(&if square {
                [255, 255, 255, 255]
            } else {
                [(x + t) as u8, (y + t / 2) as u8, 128, 255]
            });
        }
        self.surface.update(cx, |surface, cx| {
            surface.submit_frame(VIDEO_W, VIDEO_H, pixels, cx).ok();
        });
    }
}

impl Render for VideoDemo {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let scaling = self.surface.read(cx).scaling();
        div()
            .size_full()
            .p_2()
            .flex()
            .flex_col()
            .gap_1()
            .child(format!(
                "{scaling:?} scaling. {} Click to toggle.",
                self.status
            ))
            .child(
                div()
                    .id("video-demo")
                    .flex_1()
                    .on_click(cx.listener(move |this, _, _, cx| {
                        let next = match scaling {
                            VideoScaling::Fit => VideoScaling::Native,
                            VideoScaling::Native => VideoScaling::Fit,
                        };
                        this.surface.update(cx, |s, cx| s.set_scaling(next, cx));
                        cx.notify();
                    }))
                    .child(self.surface.clone()),
            )
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
                        g.toggle_button("Dark", "icons/dark_theme.svg", true, |_, _, _, cx| {
                            Theme::apply_dark(cx)
                        })
                        .toggle_button("Light", "icons/light_theme.svg", false, |_, _, _, cx| {
                            Theme::apply_light(cx)
                        })
                        .toggle_button(
                            "High contrast",
                            "icons/dark_theme.svg",
                            false,
                            |_, _, _, cx| Theme::apply_high_contrast(cx),
                        )
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
                .tab("Connection", |t| {
                    t.group("Session", |g| {
                        g.large_button("Connect", "icons/add.svg", |_, _, _| {})
                            .large_button("Disconnect", "icons/clear.svg", |_, _, _| {})
                            .separator()
                            .toggle_button(
                                "Full screen",
                                "icons/dark_theme.svg",
                                false,
                                |_, _, _, _| {},
                            )
                    })
                    .group("Display", |g| {
                        g.button("Fit to window", "icons/search.svg", |_, _, _| {})
                            .button("Actual size", "icons/search.svg", |_, _, _| {})
                    })
                    .group("Clipboard", |g| {
                        g.stack(|s| {
                            s.button("Send text", "icons/paste.svg", |_, _, _| {})
                                .button("Copy remote", "icons/copy.svg", |_, _, _| {})
                        })
                    })
                    .group("Tools", |g| {
                        g.large_button("Screenshot", "icons/folder_open.svg", |_, _, _| {})
                            .icon_button("icons/search.svg", "Find host", |_, _, _| {})
                    })
                })
                .contextual_tab("Session", true, |t| {
                    t.group("Terminal", |g| {
                        g.large_button("Clear", "icons/clear.svg", |_, _, _| {})
                            .icon_button("icons/copy.svg", "Copy", |_, _, _| {})
                    })
                })
        });

        // Main pane with a tab strip
        let tab_strip = cx.new(|cx: &mut gpui::Context<TabStrip>| {
            let mut ts = TabStrip::new(cx);
            ts.add_tab(TabItem::new("dashboard").label("Dashboard").closable(true));
            ts.add_tab(TabItem::new("settings").label("Settings").closable(true));
            ts
        });
        let badge_demo = cx.new(|_| CounterBadgeDemo);
        let persona_demo = cx.new(|_| PersonaDemo);
        let download_demo = cx.new(|_| DownloadProgressDemo);
        let data_states = cx.new(|_| DataStatesDemo);
        let big_list = cx.new(|_| BigListDemo { selected: 0 });
        let date_demo = cx.new(DateTimeDemo::new);
        let input_demo = cx.new(InputCaptureDemo::new);
        let terminal_demo = cx.new(TerminalDemo::new);
        let video_demo = cx.new(VideoDemo::new);
        let range_demo = cx.new(|_| RangeInputDemo {
            volume: 40.0,
            vertical: 0.5,
            threads: 4.0,
            opacity: 80.0,
        });
        let matrix_demo = cx.new(|_| ThemeMatrixDemo);
        let rtl_demo = cx.new(|cx| {
            cx.observe_global::<LayoutDirection>(|_, cx| cx.notify())
                .detach();
            RtlDemo
        });
        let host = cx.new(|_| {
            FamilyHost::new(DemoViews {
                status: vec![badge_demo.into(), persona_demo.into(), download_demo.into()],
                data: vec![data_states.into(), big_list.into()],
                date_input: vec![date_demo.into(), input_demo.into(), range_demo.into()],
                media: vec![terminal_demo.into(), video_demo.into()],
                theme: vec![matrix_demo.into(), rtl_demo.into()],
            })
        });
        let pane_content = host.clone();

        // Left sidebar
        let sidebar_content = cx.new(|_| FamilyNav::new(host.clone()));
        let sidebar = cx.new(|cx| {
            DockPanel::new(cx, DockPosition::Left, "Navigation")
                .size(240.0)
                .content(sidebar_content.into())
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
