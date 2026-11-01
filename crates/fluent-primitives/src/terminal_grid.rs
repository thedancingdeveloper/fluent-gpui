//! [`TerminalGrid`]: the caller-owned cell model a [`TerminalView`](crate::TerminalView) paints.
//!
//! The grid is plain data. It does no VT parsing: the caller (a vte-based emulator, a log
//! viewer, a serial console) writes cells, moves the cursor and scrolls, and the view paints the
//! result. Lines that scroll off the top of the screen go into a bounded scrollback.
//!
//! Coordinates: screen rows count from 0 at the top of the screen (scrollback excluded); columns
//! from 0 at the left. Selection and other positions that must survive scrolling use
//! [`GridPoint`], whose `line` is a stable line id that does not change as lines scroll.

use std::collections::{BTreeSet, VecDeque};

use gpui::Hsla;

/// One character cell.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct TerminalCell {
    /// The character shown. Control characters paint as a space.
    pub ch: char,
    /// Foreground colour; `None` uses the theme's terminal foreground.
    pub fg: Option<Hsla>,
    /// Background colour; `None` uses the theme's terminal background.
    pub bg: Option<Hsla>,
    /// Bold weight.
    pub bold: bool,
    /// Italic style.
    pub italic: bool,
    /// Single underline.
    pub underline: bool,
    /// Swap foreground and background.
    pub inverse: bool,
}

impl Default for TerminalCell {
    fn default() -> Self {
        Self {
            ch: ' ',
            fg: None,
            bg: None,
            bold: false,
            italic: false,
            underline: false,
            inverse: false,
        }
    }
}

impl TerminalCell {
    /// A default-styled cell holding `ch`.
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            ..Self::default()
        }
    }

    /// This cell's style with a different character.
    pub fn with_char(self, ch: char) -> Self {
        Self { ch, ..self }
    }

    /// The character as painted and copied: control characters become a space.
    pub fn display_char(&self) -> char {
        if self.ch.is_control() {
            ' '
        } else {
            self.ch
        }
    }

    fn is_blank(&self) -> bool {
        self.display_char() == ' ' && self.bg.is_none() && !self.inverse && !self.underline
    }
}

/// The cursor's painted shape.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CursorShape {
    /// A filled cell.
    #[default]
    Block,
    /// A bar under the cell.
    Underline,
    /// A thin vertical bar at the cell's left edge.
    Bar,
}

/// The text cursor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalCursor {
    /// Screen row.
    pub row: usize,
    /// Column.
    pub col: usize,
    /// Painted shape.
    pub shape: CursorShape,
    /// Whether the cursor blinks.
    pub blink: bool,
    /// Whether the cursor is shown at all.
    pub visible: bool,
}

impl Default for TerminalCursor {
    fn default() -> Self {
        Self {
            row: 0,
            col: 0,
            shape: CursorShape::Block,
            blink: true,
            visible: true,
        }
    }
}

/// A position that survives scrolling: a stable line id and a column.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GridPoint {
    /// Stable line id (see [`TerminalGrid::line_id_of_screen_row`]).
    pub line: u64,
    /// Column.
    pub col: usize,
}

/// A selection between two grid points, both ends inclusive, in reading order once normalised.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalSelection {
    /// Where the selection started.
    pub anchor: GridPoint,
    /// Where it ends now (follows the pointer).
    pub head: GridPoint,
}

impl TerminalSelection {
    /// A selection of the single cell at `point`.
    pub fn at(point: GridPoint) -> Self {
        Self {
            anchor: point,
            head: point,
        }
    }

    /// `(start, end)` in reading order.
    pub fn ordered(&self) -> (GridPoint, GridPoint) {
        if self.anchor <= self.head {
            (self.anchor, self.head)
        } else {
            (self.head, self.anchor)
        }
    }

    /// The inclusive column range selected on `line`, if any; `cols` is the grid width.
    pub fn columns_on(&self, line: u64, cols: usize) -> Option<(usize, usize)> {
        let (start, end) = self.ordered();
        if cols == 0 || line < start.line || line > end.line {
            return None;
        }
        let first = if line == start.line { start.col } else { 0 };
        let last = if line == end.line { end.col } else { cols - 1 };
        let last = last.min(cols - 1);
        (first <= last).then_some((first, last))
    }
}

#[derive(Clone, Debug)]
struct Line {
    cells: Vec<TerminalCell>,
    /// Bumped on every change, so painted rows can be cached by (id, version).
    version: u64,
}

impl Line {
    fn blank(cols: usize) -> Self {
        Self {
            cells: vec![TerminalCell::default(); cols],
            version: 0,
        }
    }

    fn is_blank(&self) -> bool {
        self.cells.iter().all(TerminalCell::is_blank)
    }
}

/// Default number of scrollback lines kept.
pub const DEFAULT_SCROLLBACK: usize = 10_000;

/// The cell grid: a screen of `rows` x `cols` cells, a bounded scrollback above it, a cursor,
/// and a record of which screen rows changed. See the [module docs](self).
#[derive(Clone, Debug)]
pub struct TerminalGrid {
    cols: usize,
    rows: usize,
    /// Scrollback then screen: the last `rows` lines are the screen.
    lines: VecDeque<Line>,
    /// Stable id of `lines[0]`.
    first_line_id: u64,
    max_scrollback: usize,
    cursor: TerminalCursor,
    dirty: BTreeSet<usize>,
    next_version: u64,
}

impl TerminalGrid {
    /// A blank grid of `cols` x `rows` (each at least 1) with the default scrollback.
    pub fn new(cols: usize, rows: usize) -> Self {
        let cols = cols.max(1);
        let rows = rows.max(1);
        Self {
            cols,
            rows,
            lines: (0..rows).map(|_| Line::blank(cols)).collect(),
            first_line_id: 0,
            max_scrollback: DEFAULT_SCROLLBACK,
            cursor: TerminalCursor::default(),
            dirty: (0..rows).collect(),
            next_version: 1,
        }
    }

    /// Keep at most `lines` scrollback lines (oldest dropped first).
    pub fn with_max_scrollback(mut self, lines: usize) -> Self {
        self.max_scrollback = lines;
        self.trim_scrollback();
        self
    }

    /// Columns on screen.
    pub fn cols(&self) -> usize {
        self.cols
    }

    /// Rows on screen.
    pub fn rows(&self) -> usize {
        self.rows
    }

    /// Lines currently held in scrollback.
    pub fn scrollback_len(&self) -> usize {
        self.lines.len() - self.rows
    }

    /// The cursor.
    pub fn cursor(&self) -> TerminalCursor {
        self.cursor
    }

    /// Move or restyle the cursor. The position is clamped to the screen. Marks the old and new
    /// cursor rows dirty.
    pub fn set_cursor(&mut self, cursor: TerminalCursor) {
        let old = self.cursor.row;
        self.cursor = cursor;
        self.clamp_cursor();
        self.dirty.insert(old);
        self.dirty.insert(self.cursor.row);
    }

    /// Move the cursor to (`row`, `col`), clamped to the screen, keeping its style.
    pub fn move_cursor(&mut self, row: usize, col: usize) {
        self.set_cursor(TerminalCursor {
            row,
            col,
            ..self.cursor
        });
    }

    fn clamp_cursor(&mut self) {
        self.cursor.row = self.cursor.row.min(self.rows - 1);
        self.cursor.col = self.cursor.col.min(self.cols - 1);
    }

    fn screen_index(&self, row: usize) -> Option<usize> {
        (row < self.rows).then(|| self.scrollback_len() + row)
    }

    fn bump(&mut self, index: usize) {
        let version = self.next_version;
        self.next_version += 1;
        if let Some(line) = self.lines.get_mut(index) {
            line.version = version;
        }
    }

    /// The cell at a screen position.
    pub fn cell(&self, row: usize, col: usize) -> Option<&TerminalCell> {
        self.screen_row(row).and_then(|cells| cells.get(col))
    }

    /// The cells of a screen row.
    pub fn screen_row(&self, row: usize) -> Option<&[TerminalCell]> {
        let index = self.screen_index(row)?;
        self.lines.get(index).map(|line| line.cells.as_slice())
    }

    /// Set the cell at a screen position. Out-of-range positions are ignored.
    pub fn set_cell(&mut self, row: usize, col: usize, cell: TerminalCell) {
        let Some(index) = self.screen_index(row) else {
            return;
        };
        if col >= self.cols {
            return;
        }
        if self.lines[index].cells[col] != cell {
            self.lines[index].cells[col] = cell;
            self.bump(index);
            self.dirty.insert(row);
        }
    }

    /// Write `text` from (`row`, `col`) with `style`'s attributes, one char per cell, clipped at
    /// the right edge (no wrapping). Returns the column after the last cell written.
    pub fn write_str(&mut self, row: usize, col: usize, text: &str, style: TerminalCell) -> usize {
        let mut col = col;
        for ch in text.chars() {
            if col >= self.cols {
                break;
            }
            self.set_cell(row, col, style.with_char(ch));
            col += 1;
        }
        col
    }

    /// Reset a screen row to blank cells.
    pub fn clear_row(&mut self, row: usize) {
        let Some(index) = self.screen_index(row) else {
            return;
        };
        self.lines[index].cells = vec![TerminalCell::default(); self.cols];
        self.bump(index);
        self.dirty.insert(row);
    }

    /// Clear the screen (scrollback is kept).
    pub fn clear_screen(&mut self) {
        for row in 0..self.rows {
            self.clear_row(row);
        }
    }

    /// Scroll the screen up by `n` lines: the top lines move into scrollback and blank lines
    /// appear at the bottom. Every screen row becomes dirty.
    pub fn scroll_up(&mut self, n: usize) {
        for _ in 0..n {
            self.lines.push_back(Line::blank(self.cols));
            let index = self.lines.len() - 1;
            self.bump(index);
        }
        self.trim_scrollback();
        self.dirty.extend(0..self.rows);
    }

    fn trim_scrollback(&mut self) {
        while self.scrollback_len() > self.max_scrollback {
            self.lines.pop_front();
            self.first_line_id += 1;
        }
    }

    /// Resize the screen. Each line is padded or truncated to `cols`. Shrinking the height first
    /// drops blank lines below the cursor, then moves top lines into scrollback; growing it pulls
    /// lines back out of scrollback, then adds blank lines at the bottom. The cursor keeps its
    /// place in the text. All rows become dirty. Sizes below 1 are raised to 1.
    pub fn resize(&mut self, cols: usize, rows: usize) {
        let cols = cols.max(1);
        let rows = rows.max(1);
        if cols == self.cols && rows == self.rows {
            return;
        }
        if cols != self.cols {
            for index in 0..self.lines.len() {
                self.lines[index]
                    .cells
                    .resize(cols, TerminalCell::default());
                self.bump(index);
            }
            self.cols = cols;
        }
        while self.rows > rows {
            let below_cursor = self.cursor.row + 1 < self.rows;
            let last_blank = self.lines.back().is_some_and(Line::is_blank);
            if below_cursor && last_blank {
                self.lines.pop_back();
            } else {
                self.cursor.row = self.cursor.row.saturating_sub(1);
            }
            self.rows -= 1;
        }
        while self.rows < rows {
            if self.scrollback_len() > 0 {
                self.cursor.row += 1;
            } else {
                self.lines.push_back(Line::blank(cols));
                let index = self.lines.len() - 1;
                self.bump(index);
            }
            self.rows += 1;
        }
        self.trim_scrollback();
        self.clamp_cursor();
        self.dirty = (0..self.rows).collect();
    }

    /// Screen rows changed since the last call, in order; clears the record.
    pub fn take_dirty_rows(&mut self) -> Vec<usize> {
        std::mem::take(&mut self.dirty).into_iter().collect()
    }

    /// Whether any screen row changed since the last [`take_dirty_rows`](Self::take_dirty_rows).
    pub fn is_dirty(&self) -> bool {
        !self.dirty.is_empty()
    }

    /// Stable id of a screen row's line.
    pub fn line_id_of_screen_row(&self, row: usize) -> u64 {
        self.first_line_id + (self.scrollback_len() + row.min(self.rows - 1)) as u64
    }

    /// Stable id of the line shown in viewport slot `slot` when the view is scrolled `offset`
    /// lines back into scrollback (`offset` is clamped to the scrollback length).
    pub fn line_id_in_viewport(&self, offset: usize, slot: usize) -> u64 {
        let offset = offset.min(self.scrollback_len());
        self.first_line_id + (self.scrollback_len() - offset + slot) as u64
    }

    fn line_by_id(&self, id: u64) -> Option<&Line> {
        let index = id.checked_sub(self.first_line_id)?;
        self.lines.get(usize::try_from(index).ok()?)
    }

    /// The cells of a line by stable id, if still held.
    pub fn line_cells(&self, id: u64) -> Option<&[TerminalCell]> {
        self.line_by_id(id).map(|line| line.cells.as_slice())
    }

    /// The change counter of a line by stable id; it differs after any change to the line.
    pub fn line_version(&self, id: u64) -> Option<u64> {
        self.line_by_id(id).map(|line| line.version)
    }

    /// A line's text with trailing spaces removed.
    pub fn line_text(&self, id: u64) -> Option<String> {
        self.line_cells(id).map(cells_text)
    }

    /// The screen's text, one line per row, trailing spaces trimmed. Used as the accessible text
    /// alternative.
    pub fn screen_text(&self) -> String {
        self.viewport_text(0)
    }

    /// The text of the viewport scrolled `offset` lines back, one line per row.
    pub fn viewport_text(&self, offset: usize) -> String {
        (0..self.rows)
            .filter_map(|slot| self.line_text(self.line_id_in_viewport(offset, slot)))
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// The selected text: lines joined by `\n`, trailing spaces on each line trimmed. Lines that
    /// have left the scrollback are skipped.
    pub fn selection_text(&self, selection: &TerminalSelection) -> String {
        let (start, end) = selection.ordered();
        let mut out = Vec::new();
        for id in start.line..=end.line {
            let Some(cells) = self.line_cells(id) else {
                continue;
            };
            let Some((first, last)) = selection.columns_on(id, cells.len()) else {
                out.push(String::new());
                continue;
            };
            out.push(cells_text(&cells[first..=last]));
        }
        out.join("\n")
    }
}

fn cells_text(cells: &[TerminalCell]) -> String {
    let text: String = cells.iter().map(TerminalCell::display_char).collect();
    text.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grid_with(lines: &[&str], cols: usize) -> TerminalGrid {
        let mut grid = TerminalGrid::new(cols, lines.len());
        for (row, text) in lines.iter().enumerate() {
            grid.write_str(row, 0, text, TerminalCell::default());
        }
        grid
    }

    #[test]
    fn new_grid_is_blank_and_fully_dirty() {
        let mut grid = TerminalGrid::new(10, 3);
        assert_eq!((grid.cols(), grid.rows()), (10, 3));
        assert_eq!(grid.take_dirty_rows(), vec![0, 1, 2]);
        assert!(!grid.is_dirty());
        assert_eq!(grid.screen_text(), "\n\n");
    }

    #[test]
    fn dirty_rows_track_only_changed_rows() {
        let mut grid = TerminalGrid::new(10, 4);
        grid.take_dirty_rows();
        grid.write_str(2, 0, "hi", TerminalCell::default());
        // Writing the same content again does not dirty the row.
        grid.take_dirty_rows();
        grid.write_str(2, 0, "hi", TerminalCell::default());
        assert!(grid.take_dirty_rows().is_empty());
        grid.set_cell(1, 3, TerminalCell::new('x'));
        grid.set_cell(3, 0, TerminalCell::new('y'));
        assert_eq!(grid.take_dirty_rows(), vec![1, 3]);
        // Out-of-range writes are ignored.
        grid.set_cell(9, 0, TerminalCell::new('z'));
        grid.set_cell(0, 99, TerminalCell::new('z'));
        assert!(grid.take_dirty_rows().is_empty());
    }

    #[test]
    fn line_version_changes_with_content() {
        let mut grid = TerminalGrid::new(5, 2);
        let id = grid.line_id_of_screen_row(0);
        let before = grid.line_version(id);
        grid.set_cell(0, 0, TerminalCell::new('a'));
        assert_ne!(grid.line_version(id), before);
        let other = grid.line_id_of_screen_row(1);
        let other_before = grid.line_version(other);
        grid.set_cell(0, 1, TerminalCell::new('b'));
        assert_eq!(grid.line_version(other), other_before);
    }

    #[test]
    fn cursor_moves_mark_rows_dirty_and_clamp() {
        let mut grid = TerminalGrid::new(10, 5);
        grid.take_dirty_rows();
        grid.move_cursor(3, 4);
        assert_eq!((grid.cursor().row, grid.cursor().col), (3, 4));
        assert_eq!(grid.take_dirty_rows(), vec![0, 3]);
        grid.move_cursor(50, 50);
        assert_eq!((grid.cursor().row, grid.cursor().col), (4, 9));
        grid.set_cursor(TerminalCursor {
            shape: CursorShape::Bar,
            blink: false,
            ..grid.cursor()
        });
        assert_eq!(grid.cursor().shape, CursorShape::Bar);
        assert!(!grid.cursor().blink);
    }

    #[test]
    fn scroll_up_moves_lines_to_bounded_scrollback() {
        let mut grid = grid_with(&["one", "two", "three"], 8).with_max_scrollback(2);
        grid.scroll_up(1);
        assert_eq!(grid.scrollback_len(), 1);
        assert_eq!(grid.screen_text(), "two\nthree\n");
        assert_eq!(grid.viewport_text(1), "one\ntwo\nthree");
        grid.scroll_up(2);
        assert_eq!(grid.scrollback_len(), 2, "oldest line dropped");
        assert_eq!(grid.viewport_text(5), "two\nthree\n");
    }

    #[test]
    fn resize_columns_pads_and_truncates() {
        let mut grid = grid_with(&["abcdef", "xy"], 6);
        grid.take_dirty_rows();
        grid.resize(3, 2);
        assert_eq!(grid.screen_text(), "abc\nxy");
        assert_eq!(grid.take_dirty_rows(), vec![0, 1]);
        grid.resize(5, 2);
        assert_eq!(grid.screen_row(0).map(<[_]>::len), Some(5));
        assert_eq!(grid.screen_text(), "abc\nxy");
    }

    #[test]
    fn resize_shrink_drops_blank_lines_below_cursor_first() {
        let mut grid = grid_with(&["prompt$", "", "", ""], 10);
        grid.move_cursor(0, 7);
        grid.resize(10, 2);
        assert_eq!(grid.rows(), 2);
        assert_eq!(grid.scrollback_len(), 0);
        assert_eq!(grid.screen_text(), "prompt$\n");
        assert_eq!((grid.cursor().row, grid.cursor().col), (0, 7));
    }

    #[test]
    fn resize_shrink_then_grow_round_trips_through_scrollback() {
        let mut grid = grid_with(&["a", "b", "c", "d"], 4);
        grid.move_cursor(3, 1);
        grid.resize(4, 2);
        assert_eq!(grid.screen_text(), "c\nd");
        assert_eq!(grid.scrollback_len(), 2);
        assert_eq!(grid.cursor().row, 1, "cursor stays on line d");
        grid.resize(4, 5);
        assert_eq!(grid.screen_text(), "a\nb\nc\nd\n");
        assert_eq!(grid.cursor().row, 3);
        assert_eq!(grid.scrollback_len(), 0);
    }

    #[test]
    fn resize_never_goes_below_one_cell() {
        let mut grid = TerminalGrid::new(4, 4);
        grid.resize(0, 0);
        assert_eq!((grid.cols(), grid.rows()), (1, 1));
        assert_eq!((grid.cursor().row, grid.cursor().col), (0, 0));
    }

    #[test]
    fn selection_text_spans_lines_and_trims() {
        let grid = grid_with(&["hello world", "second line", "third"], 12);
        let id = |row| grid.line_id_of_screen_row(row);
        let single = TerminalSelection {
            anchor: GridPoint {
                line: id(0),
                col: 6,
            },
            head: GridPoint {
                line: id(0),
                col: 10,
            },
        };
        assert_eq!(grid.selection_text(&single), "world");
        // Backwards selection across three lines; trailing blanks trimmed.
        let multi = TerminalSelection {
            anchor: GridPoint {
                line: id(2),
                col: 2,
            },
            head: GridPoint {
                line: id(0),
                col: 6,
            },
        };
        assert_eq!(grid.selection_text(&multi), "world\nsecond line\nthi");
    }

    #[test]
    fn selection_survives_scrolling() {
        let mut grid = grid_with(&["keep me", "x"], 8);
        let line = grid.line_id_of_screen_row(0);
        let selection = TerminalSelection {
            anchor: GridPoint { line, col: 0 },
            head: GridPoint { line, col: 6 },
        };
        grid.scroll_up(1);
        assert_eq!(grid.selection_text(&selection), "keep me");
    }

    #[test]
    fn control_characters_display_as_space() {
        let mut grid = TerminalGrid::new(4, 1);
        grid.write_str(0, 0, "a\u{7}b", TerminalCell::default());
        assert_eq!(grid.screen_text(), "a b");
    }
}
