//! [`TerminalView`]: a monospace cell-grid element that paints a caller-owned
//! [`TerminalGrid`], with a cursor, mouse selection, copy and a scrollback viewport. Keyboard and
//! pointer input arrive through an [`InputCapture`] surface.
//!
//! # Model
//!
//! The caller owns the grid as an `Entity<TerminalGrid>` and updates it (and calls
//! `cx.notify()` on it) as its emulator produces output. The view observes it and repaints. The
//! view does no VT parsing.
//!
//! # Size
//!
//! The view fits as many cells as its bounds hold. When that changes it resizes the grid and
//! emits [`TerminalViewEvent::Resize`] so the caller can tell its pty or remote side.
//!
//! # Input
//!
//! Click the view (or focus it and press Enter) to grab input; Escape or Control+Alt releases,
//! as for [`InputCapture`]. While grabbed:
//!
//! - every key press and release is re-emitted as [`TerminalViewEvent::Key`], except
//!   Control+Shift+C, which copies the selection to the clipboard. A key press clears the
//!   selection and scrolls back to the live screen.
//! - a primary-button drag selects text; Shift+click extends the selection.
//! - the wheel scrolls the viewport through scrollback.
//!
//! # Rendering
//!
//! Each row is shaped with gpui's text system in a monospace font and cached by its stable line
//! id and change counter, so only rows that changed are shaped again. A character the font lacks
//! is drawn from a fallback font and may not line up with the cell grid.
//!
//! # Accessibility
//!
//! The element has the `Terminal` role, the label set with [`TerminalView::label`], and the
//! visible text as its value (see [`TerminalView::accessible_text`]).

use std::{cell::RefCell, rc::Rc, time::Duration};

use fluent_core::ThemeProvider as _;
use gpui::{
    canvas, div, fill, font, point, prelude::*, px, size, App, Bounds, ClipboardItem, Context,
    Entity, EventEmitter, FocusHandle, Focusable, FontStyle, FontWeight, Hsla, IntoElement,
    Modifiers, MouseButton, Pixels, Point, Render, Role, ScrollDelta, ShapedLine, SharedString,
    Size, Stateful, Subscription, TextAlign, TextRun, UnderlineStyle, WeakEntity, Window,
};

use crate::{
    input_capture::{InputCapture, InputCaptureEvent},
    terminal_grid::{CursorShape, GridPoint, TerminalCell, TerminalGrid, TerminalSelection},
};

/// Lines scrolled per wheel notch.
const WHEEL_LINES: f32 = 3.0;
/// Cursor blink half-period.
const BLINK_INTERVAL: Duration = Duration::from_millis(530);
/// Line height as a multiple of the font size.
const LINE_HEIGHT_FACTOR: f32 = 1.25;

/// The default monospace family for the platform. The font is the system's; none is bundled.
#[cfg(target_os = "windows")]
pub const DEFAULT_TERMINAL_FONT: &str = "Consolas";
/// The default monospace family for the platform. The font is the system's; none is bundled.
#[cfg(not(target_os = "windows"))]
pub const DEFAULT_TERMINAL_FONT: &str = "DejaVu Sans Mono";

/// Something the terminal view reports to its owner.
#[derive(Clone, Debug, PartialEq)]
pub enum TerminalViewEvent {
    /// A key press or release while input is grabbed; see [`InputCaptureEvent::Key`].
    Key {
        /// gpui's logical key name.
        key: String,
        /// Best-effort W3C `KeyboardEvent.code`.
        code: Option<String>,
        /// Press (`true`) or release.
        pressed: bool,
        /// Auto-repeat of a held key.
        is_held: bool,
        /// Modifiers held.
        modifiers: Modifiers,
    },
    /// The grid was resized to fit the view.
    Resize {
        /// New column count.
        cols: usize,
        /// New row count.
        rows: usize,
    },
    /// Input was grabbed (`true`) or released.
    GrabChanged(bool),
    /// The selection was copied to the clipboard.
    Copied(String),
}

/// Cell size in pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CellMetrics {
    /// Width of one cell.
    pub cell_width: Pixels,
    /// Height of one row.
    pub line_height: Pixels,
}

impl CellMetrics {
    /// How many whole cells fit in `bounds` (at least 1 x 1).
    pub fn grid_size(&self, bounds: Size<Pixels>) -> (usize, usize) {
        (
            fit(bounds.width, self.cell_width),
            fit(bounds.height, self.line_height),
        )
    }

    /// The (viewport row, column) under an element-local position, clamped to the grid.
    pub fn cell_at(&self, position: Point<Pixels>, cols: usize, rows: usize) -> (usize, usize) {
        let col = cell_index(position.x, self.cell_width, cols);
        let row = cell_index(position.y, self.line_height, rows);
        (row, col)
    }
}

fn fit(length: Pixels, cell: Pixels) -> usize {
    if cell <= px(0.0) {
        return 1;
    }
    let count = (length / cell).floor();
    if count.is_finite() && count >= 1.0 {
        count as usize
    } else {
        1
    }
}

fn cell_index(offset: Pixels, cell: Pixels, count: usize) -> usize {
    if cell <= px(0.0) || count == 0 {
        return 0;
    }
    let index = (offset / cell).floor();
    if index.is_finite() && index > 0.0 {
        (index as usize).min(count - 1)
    } else {
        0
    }
}

/// Colours the grid paints with when a cell sets none.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Palette {
    fg: Hsla,
    bg: Hsla,
    cursor: Hsla,
    selection: Hsla,
}

/// A shaped row, valid while its line id, version and the style inputs match.
struct CachedRow {
    line: u64,
    version: u64,
    shaped: ShapedLine,
}

/// State shared between the view and its paint closure.
#[derive(Default)]
struct PaintCache {
    /// Inputs every cached row depends on; a change clears the cache.
    key: Option<(SharedString, Pixels, Palette)>,
    rows: Vec<Option<CachedRow>>,
    metrics: Option<CellMetrics>,
}

impl PaintCache {
    fn row(&mut self, slot: usize) -> &mut Option<CachedRow> {
        if self.rows.len() <= slot {
            self.rows.resize_with(slot + 1, || None);
        }
        &mut self.rows[slot]
    }
}

/// The monospace cell-grid view. See the [module docs](self).
pub struct TerminalView {
    grid: Entity<TerminalGrid>,
    capture: Entity<InputCapture>,
    font_family: SharedString,
    font_size: Pixels,
    label: SharedString,
    /// Lines scrolled back into scrollback; 0 is the live screen.
    scroll_offset: usize,
    selection: Option<TerminalSelection>,
    selecting: bool,
    /// Swallow the release of the Control+Shift+C that copied.
    swallow_copy_release: bool,
    blink_on: bool,
    blink_started: bool,
    cache: Rc<RefCell<PaintCache>>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<TerminalViewEvent> for TerminalView {}

impl Focusable for TerminalView {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.capture.read(cx).focus_handle(cx)
    }
}

impl TerminalView {
    /// A view of `grid`.
    pub fn new(grid: Entity<TerminalGrid>, cx: &mut Context<Self>) -> Self {
        let capture = cx.new(InputCapture::new);
        let observe = cx.observe(&grid, |_, _, cx| cx.notify());
        let input = cx.subscribe(&capture, |this, _, event: &InputCaptureEvent, cx| {
            this.on_input(event, cx);
        });
        Self {
            grid,
            capture,
            font_family: DEFAULT_TERMINAL_FONT.into(),
            font_size: px(14.0),
            label: "Terminal".into(),
            scroll_offset: 0,
            selection: None,
            selecting: false,
            swallow_copy_release: false,
            blink_on: true,
            blink_started: false,
            cache: Rc::default(),
            _subscriptions: vec![observe, input],
        }
    }

    /// Use a different monospace font family.
    pub fn font_family(mut self, family: impl Into<SharedString>) -> Self {
        self.font_family = family.into();
        self
    }

    /// Use a different font size (default 14 px).
    pub fn font_size(mut self, size: Pixels) -> Self {
        self.font_size = size;
        self
    }

    /// The accessible name (default "Terminal").
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = label.into();
        self
    }

    /// The grid this view paints.
    pub fn grid(&self) -> &Entity<TerminalGrid> {
        &self.grid
    }

    /// The input surface, for grabbing or releasing input from code.
    pub fn capture(&self) -> &Entity<InputCapture> {
        &self.capture
    }

    /// The cell size of the last paint, if painted.
    pub fn cell_metrics(&self) -> Option<CellMetrics> {
        self.cache.borrow().metrics
    }

    /// Lines scrolled back into scrollback (0 is the live screen).
    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    /// Scroll the viewport `offset` lines back (clamped to the scrollback).
    pub fn set_scroll_offset(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = offset.min(self.grid.read(cx).scrollback_len());
        if offset != self.scroll_offset {
            self.scroll_offset = offset;
            cx.notify();
        }
    }

    /// The current selection.
    pub fn selection(&self) -> Option<TerminalSelection> {
        self.selection
    }

    /// Replace the selection.
    pub fn set_selection(&mut self, selection: Option<TerminalSelection>, cx: &mut Context<Self>) {
        self.selection = selection;
        cx.notify();
    }

    /// The selected text, if anything is selected.
    pub fn selected_text(&self, cx: &App) -> Option<String> {
        let selection = self.selection?;
        Some(self.grid.read(cx).selection_text(&selection))
    }

    /// Copy the selection to the clipboard and emit [`TerminalViewEvent::Copied`]. Returns the
    /// copied text; `None` when nothing is selected.
    pub fn copy_selection(&mut self, cx: &mut Context<Self>) -> Option<String> {
        let text = self.selected_text(cx)?;
        cx.write_to_clipboard(ClipboardItem::new_string(text.clone()));
        cx.emit(TerminalViewEvent::Copied(text.clone()));
        Some(text)
    }

    /// The text alternative: the visible lines, one per row.
    pub fn accessible_text(&self, cx: &App) -> String {
        self.grid.read(cx).viewport_text(self.scroll_offset)
    }

    fn point_at(&self, position: Point<Pixels>, cx: &App) -> Option<GridPoint> {
        let metrics = self.cell_metrics()?;
        let grid = self.grid.read(cx);
        let (slot, col) = metrics.cell_at(position, grid.cols(), grid.rows());
        Some(GridPoint {
            line: grid.line_id_in_viewport(self.scroll_offset, slot),
            col,
        })
    }

    fn on_input(&mut self, event: &InputCaptureEvent, cx: &mut Context<Self>) {
        match event {
            InputCaptureEvent::Key {
                key,
                code,
                pressed,
                is_held,
                modifiers,
                ..
            } => {
                let copy_chord = key == "c" && modifiers.control && modifiers.shift;
                if *pressed && copy_chord {
                    self.copy_selection(cx);
                    self.swallow_copy_release = true;
                    return;
                }
                if !*pressed && key == "c" && std::mem::take(&mut self.swallow_copy_release) {
                    return;
                }
                if *pressed && !is_modifier(key) {
                    self.selection = None;
                    self.selecting = false;
                    self.scroll_offset = 0;
                    cx.notify();
                }
                cx.emit(TerminalViewEvent::Key {
                    key: key.clone(),
                    code: code.clone(),
                    pressed: *pressed,
                    is_held: *is_held,
                    modifiers: *modifiers,
                });
            }
            InputCaptureEvent::PointerButton {
                button: MouseButton::Left,
                pressed,
                position,
                modifiers,
            } => {
                if !*pressed {
                    self.selecting = false;
                    if self.selection.is_some_and(|s| s.anchor == s.head) {
                        self.selection = None;
                        cx.notify();
                    }
                    return;
                }
                let Some(point) = self.point_at(*position, cx) else {
                    return;
                };
                self.selection = match self.selection {
                    Some(selection) if modifiers.shift => Some(TerminalSelection {
                        head: point,
                        ..selection
                    }),
                    _ => Some(TerminalSelection::at(point)),
                };
                self.selecting = true;
                cx.notify();
            }
            InputCaptureEvent::PointerMove {
                position,
                pressed_button: Some(MouseButton::Left),
                ..
            } if self.selecting => {
                if let (Some(point), Some(selection)) =
                    (self.point_at(*position, cx), self.selection.as_mut())
                {
                    if selection.head != point {
                        selection.head = point;
                        cx.notify();
                    }
                }
            }
            InputCaptureEvent::Wheel { delta, .. } => {
                let lines = match delta {
                    ScrollDelta::Lines(lines) => lines.y * WHEEL_LINES,
                    ScrollDelta::Pixels(pixels) => match self.cell_metrics() {
                        Some(metrics) if metrics.line_height > px(0.0) => {
                            pixels.y / metrics.line_height
                        }
                        _ => 0.0,
                    },
                };
                let lines = lines.round();
                if !lines.is_finite() || lines == 0.0 {
                    return;
                }
                let target = if lines > 0.0 {
                    self.scroll_offset.saturating_add(lines as usize)
                } else {
                    self.scroll_offset.saturating_sub((-lines) as usize)
                };
                self.set_scroll_offset(target, cx);
            }
            InputCaptureEvent::GrabChanged(grabbed) => {
                self.selecting = false;
                cx.emit(TerminalViewEvent::GrabChanged(*grabbed));
                cx.notify();
            }
            _ => {}
        }
    }

    /// Toggle the blink phase while the cursor blinks; runs for the view's lifetime.
    fn start_blink(&mut self, cx: &mut Context<Self>) {
        if self.blink_started {
            return;
        }
        self.blink_started = true;
        cx.spawn(move |view: WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
            let mut cx = cx.clone();
            async move {
                loop {
                    cx.background_executor().timer(BLINK_INTERVAL).await;
                    let alive = view.update(&mut cx, |view, cx| {
                        if view.grid.read(cx).cursor().blink {
                            view.blink_on = !view.blink_on;
                            cx.notify();
                        } else {
                            view.blink_on = true;
                        }
                    });
                    if alive.is_err() {
                        break;
                    }
                }
            }
        })
        .detach();
    }

    /// Fit the grid to `size` cells, emitting `Resize`, after the current frame.
    fn fit_grid(weak: WeakEntity<Self>, cols: usize, rows: usize, cx: &mut App) {
        cx.defer(move |cx| {
            weak.update(cx, |view, cx| {
                let grid = view.grid.read(cx);
                if (grid.cols(), grid.rows()) == (cols, rows) {
                    return;
                }
                view.grid.update(cx, |grid, cx| {
                    grid.resize(cols, rows);
                    cx.notify();
                });
                view.scroll_offset = view.scroll_offset.min(view.grid.read(cx).scrollback_len());
                cx.emit(TerminalViewEvent::Resize { cols, rows });
                cx.notify();
            })
            .ok();
        });
    }
}

fn is_modifier(key: &str) -> bool {
    matches!(key, "shift" | "control" | "alt" | "platform" | "function")
}

/// The element carrying the accessibility semantics: a terminal named by `label` whose value is
/// the visible text.
fn a11y_base(label: SharedString, text: String) -> Stateful<gpui::Div> {
    div()
        .id("fluent-terminal-view")
        .role(Role::Terminal)
        .aria_label(label)
        .aria_value(text)
}

/// Resolved colours of a cell: (foreground, background if not the default).
fn cell_colors(cell: &TerminalCell, palette: &Palette) -> (Hsla, Option<Hsla>) {
    let fg = cell.fg.unwrap_or(palette.fg);
    if cell.inverse {
        (cell.bg.unwrap_or(palette.bg), Some(fg))
    } else {
        (fg, cell.bg)
    }
}

/// Text and runs for one row: consecutive cells with the same style share a run.
fn row_runs(
    cells: &[TerminalCell],
    family: &SharedString,
    palette: &Palette,
) -> (String, Vec<TextRun>) {
    let mut text = String::with_capacity(cells.len());
    let mut runs: Vec<TextRun> = Vec::new();
    let mut last_style = None;
    for cell in cells {
        let ch = cell.display_char();
        text.push(ch);
        let (fg, _) = cell_colors(cell, palette);
        let style = (fg, cell.bold, cell.italic, cell.underline);
        match runs.last_mut() {
            Some(run) if last_style == Some(style) => run.len += ch.len_utf8(),
            _ => {
                let mut run_font = font(family.clone());
                if cell.bold {
                    run_font.weight = FontWeight::BOLD;
                }
                if cell.italic {
                    run_font.style = FontStyle::Italic;
                }
                runs.push(TextRun {
                    len: ch.len_utf8(),
                    font: run_font,
                    color: fg,
                    background_color: None,
                    underline: cell.underline.then_some(UnderlineStyle {
                        color: Some(fg),
                        thickness: px(1.0),
                        wavy: false,
                    }),
                    strikethrough: None,
                });
                last_style = Some(style);
            }
        }
    }
    (text, runs)
}

/// Everything the paint closure needs, snapshotted at render.
struct Frame {
    weak: WeakEntity<TerminalView>,
    grid: Entity<TerminalGrid>,
    cache: Rc<RefCell<PaintCache>>,
    family: SharedString,
    font_size: Pixels,
    palette: Palette,
    scroll_offset: usize,
    selection: Option<TerminalSelection>,
    cursor_shown: bool,
    grabbed: bool,
}

impl Frame {
    fn metrics(&self, window: &mut Window) -> CellMetrics {
        let text_system = window.text_system();
        let font_id = text_system.resolve_font(&font(self.family.clone()));
        let cell_width = text_system
            .advance(font_id, self.font_size, 'M')
            .map(|advance| advance.width)
            .ok()
            .filter(|width| *width > px(0.0))
            .unwrap_or(self.font_size * 0.6);
        CellMetrics {
            cell_width,
            line_height: (self.font_size * LINE_HEIGHT_FACTOR).round(),
        }
    }

    fn paint(self, bounds: Bounds<Pixels>, window: &mut Window, cx: &mut App) {
        let metrics = self.metrics(window);
        let (cols, rows) = metrics.grid_size(bounds.size);
        let mut cache = self.cache.borrow_mut();
        cache.metrics = Some(metrics);
        let key = (self.family.clone(), self.font_size, self.palette);
        if cache.key.as_ref() != Some(&key) {
            cache.key = Some(key);
            cache.rows.clear();
        }

        window.paint_quad(fill(bounds, self.palette.bg));
        let grid = self.grid.read(cx);
        if (grid.cols(), grid.rows()) != (cols, rows) {
            TerminalView::fit_grid(self.weak.clone(), cols, rows, cx);
        }
        let grid = self.grid.read(cx);
        let offset = self.scroll_offset.min(grid.scrollback_len());
        let grid_cols = grid.cols();
        let mut shaped_rows = Vec::with_capacity(grid.rows());
        for slot in 0..grid.rows() {
            let line = grid.line_id_in_viewport(offset, slot);
            let (Some(cells), Some(version)) = (grid.line_cells(line), grid.line_version(line))
            else {
                continue;
            };
            let origin = point(
                bounds.left(),
                bounds.top() + metrics.line_height * slot as f32,
            );
            // Backgrounds, merged across runs of equal colour.
            let mut col = 0;
            while col < cells.len() {
                let (_, bg) = cell_colors(&cells[col], &self.palette);
                let start = col;
                while col < cells.len() && cell_colors(&cells[col], &self.palette).1 == bg {
                    col += 1;
                }
                if let Some(bg) = bg {
                    window.paint_quad(fill(
                        Bounds::new(
                            point(origin.x + metrics.cell_width * start as f32, origin.y),
                            size(
                                metrics.cell_width * (col - start) as f32,
                                metrics.line_height,
                            ),
                        ),
                        bg,
                    ));
                }
            }
            if let Some((first, last)) = self.selection.and_then(|s| s.columns_on(line, grid_cols))
            {
                window.paint_quad(fill(
                    Bounds::new(
                        point(origin.x + metrics.cell_width * first as f32, origin.y),
                        size(
                            metrics.cell_width * (last + 1 - first) as f32,
                            metrics.line_height,
                        ),
                    ),
                    self.palette.selection,
                ));
            }
            let cached = cache.row(slot);
            let fresh = cached
                .as_ref()
                .is_some_and(|row| row.line == line && row.version == version);
            if !fresh {
                let (text, runs) = row_runs(cells, &self.family, &self.palette);
                let shaped =
                    window
                        .text_system()
                        .shape_line(text.into(), self.font_size, &runs, None);
                *cached = Some(CachedRow {
                    line,
                    version,
                    shaped,
                });
            }
            if let Some(row) = cache.rows[slot].as_ref() {
                shaped_rows.push((origin, row.shaped.clone()));
            }
        }
        cache.rows.truncate(grid.rows());

        let cursor = grid.cursor();
        let cursor_slot = Some(offset + cursor.row).filter(|slot| *slot < grid.rows());
        drop(cache);
        for (origin, shaped) in shaped_rows {
            shaped
                .paint(
                    origin,
                    metrics.line_height,
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                )
                .ok();
        }

        if !(cursor.visible && self.cursor_shown) {
            return;
        }
        let Some(slot) = cursor_slot else {
            return;
        };
        let cell = Bounds::new(
            point(
                bounds.left() + metrics.cell_width * cursor.col as f32,
                bounds.top() + metrics.line_height * slot as f32,
            ),
            size(metrics.cell_width, metrics.line_height),
        );
        let color = self.palette.cursor;
        let quad = match cursor.shape {
            CursorShape::Block if self.grabbed => fill(cell, color.opacity(0.6)),
            CursorShape::Block => gpui::outline(cell, color, gpui::BorderStyle::Solid),
            CursorShape::Underline => fill(
                Bounds::new(
                    point(cell.left(), cell.bottom() - px(2.0)),
                    size(cell.size.width, px(2.0)),
                ),
                color,
            ),
            CursorShape::Bar => fill(
                Bounds::new(cell.origin, size(px(2.0), cell.size.height)),
                color,
            ),
        };
        window.paint_quad(quad);
    }
}

impl Render for TerminalView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.start_blink(cx);
        let colors = &cx.theme().colors;
        let palette = Palette {
            fg: colors.on_subtle,
            bg: colors.surface_dim,
            cursor: colors.accent,
            selection: colors.accent.opacity(0.35),
        };
        let grabbed = self.capture.read(cx).is_grabbed();
        let blink = self.grid.read(cx).cursor().blink;
        let frame = Frame {
            weak: cx.weak_entity(),
            grid: self.grid.clone(),
            cache: self.cache.clone(),
            family: self.font_family.clone(),
            font_size: self.font_size,
            palette,
            scroll_offset: self.scroll_offset,
            selection: self.selection,
            // Blink only while grabbed; an idle terminal shows a steady cursor.
            cursor_shown: self.scroll_offset == 0 && (!grabbed || !blink || self.blink_on),
            grabbed,
        };
        let surface = canvas(
            |_, _, _| (),
            move |bounds, _, window, cx| frame.paint(bounds, window, cx),
        )
        .absolute()
        .size_full();

        a11y_base(self.label.clone(), self.accessible_text(cx))
            .relative()
            .size_full()
            .overflow_hidden()
            .child(surface)
            .child(self.capture.clone())
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, VisualTestContext};
    use gpui::{TestAppContext, TouchPhase};

    use super::*;

    struct Host {
        view: Entity<TerminalView>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .child(div().w(px(400.0)).h(px(200.0)).child(self.view.clone()))
        }
    }

    type Log = Rc<RefCell<Vec<TerminalViewEvent>>>;

    fn setup(
        cx: &mut TestAppContext,
    ) -> (
        Entity<TerminalView>,
        Entity<TerminalGrid>,
        Log,
        &mut VisualTestContext,
    ) {
        cx.executor().allow_parking();
        let log: Log = Rc::default();
        let sink = log.clone();
        let (host, vcx) = open_fluent_test_window(cx, move |_window, cx| {
            let grid = cx.new(|_| TerminalGrid::new(10, 4));
            let view = cx.new(|cx| TerminalView::new(grid, cx));
            cx.subscribe(&view, move |_, _, event: &TerminalViewEvent, _| {
                sink.borrow_mut().push(event.clone());
            })
            .detach();
            Host { view }
        });
        let view = host.read_with(vcx, |host, _| host.view.clone());
        let grid = view.read_with(vcx, |view, _| view.grid.clone());
        vcx.update(|window, cx| window.draw(cx).clear(cx));
        vcx.run_until_parked();
        (view, grid, log, vcx)
    }

    fn metrics(view: &Entity<TerminalView>, vcx: &mut VisualTestContext) -> CellMetrics {
        view.read_with(vcx, |view, _| view.cell_metrics())
            .expect("painted")
    }

    /// Window position of the centre of viewport cell (row, col).
    fn cell_centre(m: CellMetrics, row: usize, col: usize) -> Point<Pixels> {
        point(
            m.cell_width * (col as f32 + 0.5),
            m.line_height * (row as f32 + 0.5),
        )
    }

    fn keys(log: &Log) -> Vec<(String, bool)> {
        log.borrow()
            .iter()
            .filter_map(|e| match e {
                TerminalViewEvent::Key { key, pressed, .. } => Some((key.clone(), *pressed)),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn grid_size_and_cell_mapping() {
        let m = CellMetrics {
            cell_width: px(8.0),
            line_height: px(16.0),
        };
        assert_eq!(m.grid_size(size(px(81.0), px(47.0))), (10, 2));
        assert_eq!(m.grid_size(size(px(3.0), px(3.0))), (1, 1));
        assert_eq!(m.cell_at(point(px(17.0), px(33.0)), 10, 4), (2, 2));
        assert_eq!(m.cell_at(point(px(-5.0), px(999.0)), 10, 4), (3, 0));
        let zero = CellMetrics {
            cell_width: px(0.0),
            line_height: px(0.0),
        };
        assert_eq!(zero.grid_size(size(px(100.0), px(100.0))), (1, 1));
    }

    #[test]
    fn runs_group_equal_styles_and_inverse_swaps_colours() {
        let palette = Palette {
            fg: gpui::white(),
            bg: gpui::black(),
            cursor: gpui::white(),
            selection: gpui::white(),
        };
        let bold = TerminalCell {
            bold: true,
            ..TerminalCell::new('b')
        };
        let cells = [TerminalCell::new('a'), TerminalCell::new('é'), bold, bold];
        let (text, runs) = row_runs(&cells, &"Mono".into(), &palette);
        assert_eq!(text, "aébb");
        assert_eq!(runs.len(), 2);
        assert_eq!(runs[0].len, 3, "byte length, not char count");
        assert_eq!(runs[1].font.weight, FontWeight::BOLD);

        let inverse = TerminalCell {
            inverse: true,
            ..TerminalCell::new('x')
        };
        assert_eq!(
            cell_colors(&inverse, &palette),
            (gpui::black(), Some(gpui::white()))
        );
    }

    #[test]
    fn exposes_terminal_role_label_and_text() {
        let element = a11y_base("Shell".into(), "$ ls\nfile".into());
        assert_eq!(element.a11y_role(), Some(Role::Terminal));
        let mut node = gpui::accesskit::Node::new(Role::Terminal);
        element.write_a11y_info(&mut node);
        assert_eq!(node.label(), Some("Shell"));
        assert_eq!(node.value(), Some("$ ls\nfile"));
    }

    #[gpui::test]
    fn view_fits_grid_to_bounds_and_reports_resize(cx: &mut TestAppContext) {
        let (view, grid, log, vcx) = setup(cx);
        let m = metrics(&view, vcx);
        let expected = m.grid_size(size(px(400.0), px(200.0)));
        let actual = grid.read_with(vcx, |g, _| (g.cols(), g.rows()));
        assert_eq!(actual, expected);
        assert!(log.borrow().contains(&TerminalViewEvent::Resize {
            cols: expected.0,
            rows: expected.1,
        }));
    }

    #[gpui::test]
    fn keys_forward_and_copy_chord_copies_selection(cx: &mut TestAppContext) {
        let (view, grid, log, vcx) = setup(cx);
        grid.update(vcx, |g, cx| {
            g.write_str(0, 0, "hello world", TerminalCell::default());
            cx.notify();
        });
        vcx.simulate_click(point(px(5.0), px(5.0)), Modifiers::default());
        vcx.run_until_parked();
        assert!(log.borrow().contains(&TerminalViewEvent::GrabChanged(true)));

        let line = grid.read_with(vcx, |g, _| g.line_id_of_screen_row(0));
        view.update(vcx, |v, cx| {
            v.set_selection(
                Some(TerminalSelection {
                    anchor: GridPoint { line, col: 0 },
                    head: GridPoint { line, col: 4 },
                }),
                cx,
            )
        });
        vcx.simulate_keystrokes("ctrl-shift-c");
        assert!(log
            .borrow()
            .contains(&TerminalViewEvent::Copied("hello".into())));
        let clip = vcx.update(|_, cx| cx.read_from_clipboard().and_then(|c| c.text()));
        assert_eq!(clip.as_deref(), Some("hello"));
        assert!(
            !keys(&log).iter().any(|(k, _)| k == "c"),
            "the copy chord is not forwarded"
        );

        vcx.simulate_keystrokes("a");
        assert_eq!(keys(&log).last(), Some(&("a".to_string(), true)));
        assert_eq!(view.read_with(vcx, |v, _| v.selection()), None);
    }

    #[gpui::test]
    fn drag_selects_text(cx: &mut TestAppContext) {
        let (view, grid, _log, vcx) = setup(cx);
        grid.update(vcx, |g, cx| {
            g.write_str(1, 0, "select me", TerminalCell::default());
            cx.notify();
        });
        let m = metrics(&view, vcx);
        // The first click grabs and is consumed.
        vcx.simulate_click(point(px(1.0), px(1.0)), Modifiers::default());
        vcx.simulate_mouse_down(
            cell_centre(m, 1, 0),
            MouseButton::Left,
            Modifiers::default(),
        );
        vcx.simulate_mouse_move(
            cell_centre(m, 1, 5),
            Some(MouseButton::Left),
            Modifiers::default(),
        );
        vcx.simulate_mouse_up(
            cell_centre(m, 1, 5),
            MouseButton::Left,
            Modifiers::default(),
        );
        let text = view.read_with(vcx, |v, cx| v.selected_text(cx));
        assert_eq!(text.as_deref(), Some("select"));
    }

    #[gpui::test]
    fn wheel_scrolls_through_scrollback(cx: &mut TestAppContext) {
        let (view, grid, _log, vcx) = setup(cx);
        grid.update(vcx, |g, cx| {
            g.write_str(0, 0, "oldest", TerminalCell::default());
            g.scroll_up(5);
            cx.notify();
        });
        vcx.simulate_click(point(px(1.0), px(1.0)), Modifiers::default());
        vcx.simulate_event(gpui::ScrollEvent {
            position: point(px(10.0), px(10.0)),
            delta: ScrollDelta::Lines(point(0.0, 1.0)),
            modifiers: Modifiers::default(),
            touch_phase: TouchPhase::Moved,
            ..Default::default()
        });
        assert_eq!(view.read_with(vcx, |v, _| v.scroll_offset()), 3);
        vcx.simulate_event(gpui::ScrollEvent {
            position: point(px(10.0), px(10.0)),
            delta: ScrollDelta::Lines(point(0.0, 10.0)),
            modifiers: Modifiers::default(),
            touch_phase: TouchPhase::Moved,
            ..Default::default()
        });
        let max = grid.read_with(vcx, |g, _| g.scrollback_len());
        assert_eq!(view.read_with(vcx, |v, _| v.scroll_offset()), max);
        assert!(view
            .read_with(vcx, |v, cx| v.accessible_text(cx))
            .starts_with("oldest"));
    }
}
