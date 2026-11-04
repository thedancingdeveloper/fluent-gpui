use fluent_core::TextRampExt;
use std::sync::Arc;

use crate::{
    data_status::{status_block, DataStatusKind},
    virtual_list::VirtualList,
};
use fluent_core::{ThemeProvider as _, TruncateExt as _};
use gpui::{
    div, prelude::*, px, App, ClickEvent, IntoElement, KeyDownEvent, RenderOnce, Role,
    SharedString, Window,
};

type RowClickHandler = Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>;
type RowSelectHandler = Arc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;
type SortHandler =
    Arc<dyn Fn(SharedString, DataSortDirection, &ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Public API item.
pub enum DataSortDirection {
    /// Public API item.
    Ascending,
    /// Public API item.
    Descending,
}

impl DataSortDirection {
    /// Public API item.
    pub fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Public API item.
pub enum DataCellAlign {
    #[default]
    /// Public API item.
    Start,
    /// Public API item.
    Center,
    /// Public API item.
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Public API item.
pub enum DataSortKind {
    #[default]
    /// Public API item.
    Text,
    /// Public API item.
    Number,
}

#[derive(Clone, Debug)]
/// Public API item.
pub struct DataCell {
    /// Public API item.
    pub text: SharedString,
    /// Public API item.
    pub align: Option<DataCellAlign>,
}

impl DataCell {
    /// Public API item.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            align: None,
        }
    }

    /// Public API item.
    pub fn align(mut self, align: DataCellAlign) -> Self {
        self.align = Some(align);
        self
    }
}

impl From<&'static str> for DataCell {
    fn from(value: &'static str) -> Self {
        Self::new(value)
    }
}

impl From<String> for DataCell {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<SharedString> for DataCell {
    fn from(value: SharedString) -> Self {
        Self::new(value)
    }
}

#[derive(Clone, Debug)]
/// Public API item.
pub enum DataTableState {
    /// Public API item.
    Ready,
    /// Public API item.
    Loading(SharedString),
    /// Public API item.
    Empty(SharedString),
    /// Rows exist but the active filter hides them all.
    NoResults(SharedString),
    /// Public API item.
    Error(SharedString),
}

#[derive(Clone, Debug)]
/// Public API item.
pub struct DataColumn {
    /// Public API item.
    pub id: SharedString,
    /// Public API item.
    pub title: SharedString,
    /// Public API item.
    pub width: Option<f32>,
    /// Public API item.
    pub sortable: bool,
    /// Public API item.
    pub align: DataCellAlign,
    /// Public API item.
    pub sort_kind: DataSortKind,
}

impl DataColumn {
    /// Public API item.
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            width: None,
            sortable: false,
            align: DataCellAlign::Start,
            sort_kind: DataSortKind::Text,
        }
    }

    /// Public API item.
    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width.clamp(48.0, 600.0));
        self
    }

    /// Public API item.
    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    /// Public API item.
    pub fn align(mut self, align: DataCellAlign) -> Self {
        self.align = align;
        self
    }

    /// Public API item.
    pub fn sort_kind(mut self, kind: DataSortKind) -> Self {
        self.sort_kind = kind;
        self
    }
}

#[derive(Clone, Debug)]
/// Public API item.
pub struct DataRow {
    /// Public API item.
    pub id: SharedString,
    /// Public API item.
    pub cells: Vec<DataCell>,
}

impl DataRow {
    /// Public API item.
    pub fn new(
        id: impl Into<SharedString>,
        cells: impl IntoIterator<Item = impl Into<DataCell>>,
    ) -> Self {
        Self {
            id: id.into(),
            cells: cells.into_iter().map(Into::into).collect(),
        }
    }
}

/// A simple Fluent-style data table for dense business-app lists.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct DataTable {
    columns: Vec<DataColumn>,
    rows: Vec<DataRow>,
    state: DataTableState,
    selected_id: Option<SharedString>,
    sort: Option<(SharedString, DataSortDirection)>,
    max_body_height: Option<f32>,
    on_row_click: Option<RowClickHandler>,
    on_row_select: Option<RowSelectHandler>,
    on_row_extend: Option<RowSelectHandler>,
    on_sort: Option<SortHandler>,
}

/// Where a navigation key moves the active row.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RowMove {
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
}

/// The row a move from `current` lands on, clamped to `len` rows (`len > 0`).
/// With no active row, forward moves land on the first row and backward moves on the last.
fn move_target(current: Option<usize>, mv: RowMove, page: usize, len: usize) -> usize {
    let last = len - 1;
    let page = page.max(1);
    match (current, mv) {
        (_, RowMove::Home) => 0,
        (_, RowMove::End) => last,
        (None, RowMove::Down | RowMove::PageDown) => 0,
        (None, RowMove::Up | RowMove::PageUp) => last,
        (Some(c), RowMove::Up) => c.saturating_sub(1),
        (Some(c), RowMove::Down) => (c + 1).min(last),
        (Some(c), RowMove::PageUp) => c.saturating_sub(page),
        (Some(c), RowMove::PageDown) => (c + page).min(last),
    }
}

impl DataTable {
    /// Public API item.
    pub fn new() -> Self {
        Self {
            columns: Vec::new(),
            rows: Vec::new(),
            state: DataTableState::Ready,
            selected_id: None,
            sort: None,
            max_body_height: None,
            on_row_click: None,
            on_row_select: None,
            on_row_extend: None,
            on_sort: None,
        }
    }

    /// Public API item.
    pub fn column(mut self, column: DataColumn) -> Self {
        self.columns.push(column);
        self
    }

    /// Public API item.
    pub fn columns(mut self, columns: impl IntoIterator<Item = DataColumn>) -> Self {
        self.columns = columns.into_iter().collect();
        self
    }

    /// Public API item.
    pub fn row(mut self, row: DataRow) -> Self {
        self.rows.push(row);
        self
    }

    /// Public API item.
    pub fn rows(mut self, rows: impl IntoIterator<Item = DataRow>) -> Self {
        self.rows = rows.into_iter().collect();
        self
    }

    /// Public API item.
    pub fn state(mut self, state: DataTableState) -> Self {
        self.state = state;
        self
    }

    /// Public API item.
    pub fn loading(mut self, message: impl Into<SharedString>) -> Self {
        self.state = DataTableState::Loading(message.into());
        self
    }

    /// Public API item.
    pub fn empty_message(mut self, message: impl Into<SharedString>) -> Self {
        self.state = DataTableState::Empty(message.into());
        self
    }

    /// Public API item.
    pub fn no_results_message(mut self, message: impl Into<SharedString>) -> Self {
        self.state = DataTableState::NoResults(message.into());
        self
    }

    /// Public API item.
    pub fn error_message(mut self, message: impl Into<SharedString>) -> Self {
        self.state = DataTableState::Error(message.into());
        self
    }

    /// Public API item.
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    /// Public API item.
    pub fn sort(
        mut self,
        column_id: impl Into<SharedString>,
        direction: DataSortDirection,
    ) -> Self {
        self.sort = Some((column_id.into(), direction));
        self
    }

    /// Public API item.
    pub fn max_body_height(mut self, height: f32) -> Self {
        self.max_body_height = Some(height);
        self
    }

    /// Public API item.
    pub fn on_row_click(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row_click = Some(Arc::new(f));
        self
    }

    /// Public API item.
    pub fn on_row_select(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row_select = Some(Arc::new(f));
        self
    }

    /// Called with the new active row when Shift+navigation or Shift+Space extends the selection.
    pub fn on_row_extend(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row_extend = Some(Arc::new(f));
        self
    }

    /// Public API item.
    pub fn on_sort(
        mut self,
        f: impl Fn(SharedString, DataSortDirection, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_sort = Some(Arc::new(f));
        self
    }
}

impl Default for DataTable {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for DataTable {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let widths = window.use_keyed_state("data-table-widths", cx, |_, _| {
            self.columns
                .iter()
                .map(|column| column.width.unwrap_or(120.0).clamp(48.0, 600.0))
                .collect::<Vec<f32>>()
        });
        let current_widths = widths.read(cx).clone();
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let components = theme.components;
        let sorted_column_index = self.sort.as_ref().and_then(|(column_id, _)| {
            self.columns
                .iter()
                .position(|column| column.id == *column_id)
        });
        let mut rows = self.rows;

        if let (Some(col_idx), Some((_, direction))) = (sorted_column_index, self.sort.as_ref()) {
            let sort_kind = self
                .columns
                .get(col_idx)
                .map(|column| column.sort_kind)
                .unwrap_or_default();
            rows.sort_by(|a, b| {
                let left = a
                    .cells
                    .get(col_idx)
                    .map(|s| s.text.to_string())
                    .unwrap_or_default();
                let right = b
                    .cells
                    .get(col_idx)
                    .map(|s| s.text.to_string())
                    .unwrap_or_default();
                let ordering = match sort_kind {
                    DataSortKind::Text => left.cmp(&right),
                    DataSortKind::Number => {
                        let left = left.parse::<f64>().unwrap_or(f64::NAN);
                        let right = right.parse::<f64>().unwrap_or(f64::NAN);
                        left.partial_cmp(&right)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    }
                };
                match direction {
                    DataSortDirection::Ascending => ordering,
                    DataSortDirection::Descending => ordering.reverse(),
                }
            });
        }

        let row_ids: Vec<SharedString> = rows.iter().map(|row| row.id.clone()).collect();
        let selected_index = self
            .selected_id
            .as_ref()
            .and_then(|id| row_ids.iter().position(|row_id| row_id == id));
        let on_row_select = self.on_row_select.clone();
        let on_row_activate = self.on_row_click.clone();
        let on_row_extend = self.on_row_extend.clone();
        let key_row_ids = row_ids.clone();

        let mut root = div()
            .id("data-table")
            .debug_selector(|| "data-table".to_string())
            .role(Role::Grid)
            .aria_row_count(rows.len() + 1)
            .aria_column_count(self.columns.len())
            .tab_index(0)
            .key_context("DataTable")
            .on_key_down(move |ev: &KeyDownEvent, window, app| {
                if key_row_ids.is_empty() {
                    return;
                }
                let key = ev.keystroke.key.as_str();
                let mv = match key {
                    "up" => Some(RowMove::Up),
                    "down" => Some(RowMove::Down),
                    "home" => Some(RowMove::Home),
                    "end" => Some(RowMove::End),
                    "pageup" => Some(RowMove::PageUp),
                    "pagedown" => Some(RowMove::PageDown),
                    _ => None,
                };
                let extend = ev.keystroke.modifiers.shift;
                let handler = if extend {
                    &on_row_extend
                } else {
                    &on_row_select
                };
                if let Some(mv) = mv {
                    let next = move_target(selected_index, mv, 10, key_row_ids.len());
                    if let Some(handler) = handler {
                        handler(key_row_ids[next].clone(), window, app);
                    }
                    app.stop_propagation();
                    return;
                }
                match key {
                    "space" => {
                        if let (Some(handler), Some(current)) = (handler, selected_index) {
                            handler(key_row_ids[current].clone(), window, app);
                        } else if let Some(handler) = handler {
                            handler(key_row_ids[0].clone(), window, app);
                        }
                        app.stop_propagation();
                    }
                    "enter" | "return" => {
                        let current = selected_index.unwrap_or(0).min(key_row_ids.len() - 1);
                        if let Some(handler) = &on_row_activate {
                            let ev = ClickEvent::default();
                            handler(key_row_ids[current].clone(), &ev, window, app);
                        }
                        app.stop_propagation();
                    }
                    _ => {}
                }
            })
            .flex()
            .flex_col()
            .w_full()
            .border_1()
            .border_color(colors.stroke_neutral_subtle)
            .rounded(px(theme.radii.md))
            .overflow_hidden();

        let mut header = div()
            .id("data-header-row")
            .role(Role::Row)
            .aria_row_index(1)
            .flex()
            .flex_row()
            .h(px(components.content_tab_strip_height))
            .bg(colors.surface_dim)
            .border_b_1()
            .border_color(colors.stroke_neutral_subtle);

        for (col_idx, column) in self.columns.iter().enumerate() {
            let active_sort = sorted_column_index == Some(col_idx);
            let next_direction = self
                .sort
                .as_ref()
                .filter(|_| active_sort)
                .map(|(_, direction)| direction.toggle())
                .unwrap_or(DataSortDirection::Ascending);
            let mut label = div().truncate_label().child(column.title.clone());
            if active_sort {
                let arrow = match self.sort.as_ref().map(|(_, direction)| *direction) {
                    Some(DataSortDirection::Ascending) => " ↑",
                    Some(DataSortDirection::Descending) => " ↓",
                    None => "",
                };
                label = label.child(arrow);
            }

            let resize_widths = widths.clone();
            let key_resize_widths = resize_widths.clone();
            let resize_index = col_idx;
            let resize_handle = div()
                .id(("data-column-resize", col_idx as u64))
                .w(px(6.0))
                .h_full()
                .flex_none()
                .cursor_pointer()
                .on_click(move |_, _, cx| {
                    key_resize_widths.update(cx, |widths, cx| {
                        if let Some(width) = widths.get_mut(resize_index) {
                            *width = (*width + 8.0).clamp(48.0, 600.0);
                            cx.notify();
                        }
                    });
                })
                .on_key_down(move |ev, _, cx| {
                    let delta = match ev.keystroke.key.as_str() {
                        "left" => -8.0,
                        "right" => 8.0,
                        _ => return,
                    };
                    resize_widths.update(cx, |widths, cx| {
                        if let Some(width) = widths.get_mut(resize_index) {
                            *width = (*width + delta).clamp(48.0, 600.0);
                            cx.notify();
                        }
                    });
                });
            let mut cell = div()
                .id(("data-header", col_idx as u64))
                .role(Role::ColumnHeader)
                .aria_label(column.title.clone())
                .aria_column_index(col_idx + 1)
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing.xs))
                .px(px(spacing.md))
                .text_ramp(&typography.caption)
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(colors.on_subtle)
                .border_r_1()
                .border_color(colors.stroke_neutral_subtle)
                .child(label)
                .child(resize_handle);
            if column.sortable {
                if let Some(on_sort) = self.on_sort.clone() {
                    let column_id = column.id.clone();
                    cell = cell
                        .cursor_pointer()
                        .hover({
                            let hover = colors.neutral_hover;
                            move |s| s.bg(hover)
                        })
                        .on_click(move |ev, win, app| {
                            on_sort(column_id.clone(), next_direction, ev, win, app)
                        });
                }
            }
            let width = current_widths.get(col_idx).copied().unwrap_or(120.0);
            cell = cell.w(px(width)).flex_none();
            header = header.child(cell);
        }

        root = root.child(header);

        let status = match &self.state {
            DataTableState::Loading(m) => Some((DataStatusKind::Loading, m.clone())),
            DataTableState::Empty(m) => Some((DataStatusKind::Empty, m.clone())),
            DataTableState::NoResults(m) => Some((DataStatusKind::NoResults, m.clone())),
            DataTableState::Error(m) => Some((DataStatusKind::Error, m.clone())),
            DataTableState::Ready if rows.is_empty() => {
                Some((DataStatusKind::Empty, "No rows".into()))
            }
            DataTableState::Ready => None,
        };
        if let Some((kind, message)) = status {
            return root.child(status_block("data-table-status", kind, message, cx));
        }

        let count = rows.len();
        let columns = self.columns.clone();
        let selected_id = self.selected_id.clone();
        let on_row_click = self.on_row_click.clone();
        let colors_for_rows = colors.clone();
        let spacing_for_rows = spacing;
        let typography_for_rows = typography;
        let row_height = components.list_item_height;
        let build_row = move |idx: usize, _window: &mut Window, _cx: &mut App| {
            let row = rows[idx].clone();
            let is_selected = selected_id.as_ref() == Some(&row.id);
            let row_id = row.id.clone();
            let mut row_el = div()
                .id(("data-row", idx as u64))
                .role(Role::Row)
                .aria_row_index(idx + 2)
                .aria_selected(is_selected)
                .flex()
                .flex_row()
                .min_h(px(row_height))
                .bg(if is_selected {
                    colors_for_rows.neutral_selected
                } else {
                    colors_for_rows.surface
                })
                .border_b_1()
                .border_color(colors_for_rows.stroke_neutral_subtle);

            if let Some(handler) = on_row_click.clone() {
                row_el = row_el
                    .cursor_pointer()
                    .hover({
                        let hover = colors_for_rows.neutral_hover;
                        move |s| {
                            s.bg(if is_selected {
                                colors_for_rows.neutral_selected
                            } else {
                                hover
                            })
                        }
                    })
                    .on_click(move |ev, win, app| handler(row_id.clone(), ev, win, app));
            }

            for (col_idx, column) in columns.iter().enumerate() {
                let data_cell = row
                    .cells
                    .get(col_idx)
                    .cloned()
                    .unwrap_or_else(|| DataCell::new(""));
                let align = data_cell.align.unwrap_or(column.align);
                let mut cell = div()
                    .id(("data-cell", (idx * 1000 + col_idx) as u64))
                    .role(Role::Cell)
                    .aria_label(data_cell.text.clone())
                    .aria_column_index(col_idx + 1)
                    .flex()
                    .items_center()
                    .when(align == DataCellAlign::Center, |d| d.justify_center())
                    .when(align == DataCellAlign::End, |d| d.justify_end())
                    .px(px(spacing_for_rows.md))
                    .py(px(spacing_for_rows.xs))
                    .text_ramp(&typography_for_rows.body)
                    .text_color(colors_for_rows.on_neutral)
                    .border_r_1()
                    .border_color(colors_for_rows.stroke_neutral_subtle)
                    .child(div().truncate_label().child(data_cell.text));
                let width = current_widths.get(col_idx).copied().unwrap_or(120.0);
                cell = cell.w(px(width)).flex_none();
                row_el = row_el.child(cell);
            }

            row_el.into_any_element()
        };
        let mut body = div().id("data-table-body").flex().flex_col().h_full();
        if let Some(height) = self.max_body_height {
            body = body.h(px(height)).overflow_hidden();
        }
        body = body.child(
            VirtualList::new("data-table-virtual-list", count, build_row)
                .row_height(components.list_item_height)
                .reveal(selected_index),
        );
        root.child(body)
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Render, TestAppContext};

    use super::*;

    #[test]
    fn move_target_clamps_and_pages() {
        assert_eq!(move_target(Some(0), RowMove::Up, 5, 20), 0);
        assert_eq!(move_target(Some(19), RowMove::Down, 5, 20), 19);
        assert_eq!(move_target(Some(3), RowMove::PageDown, 5, 20), 8);
        assert_eq!(move_target(Some(18), RowMove::PageDown, 5, 20), 19);
        assert_eq!(move_target(Some(3), RowMove::PageUp, 5, 20), 0);
        assert_eq!(move_target(Some(7), RowMove::Home, 5, 20), 0);
        assert_eq!(move_target(Some(7), RowMove::End, 5, 20), 19);
        assert_eq!(move_target(None, RowMove::Down, 5, 20), 0);
        assert_eq!(move_target(None, RowMove::Up, 5, 20), 19);
    }

    #[derive(Default)]
    struct Log {
        selected: Option<SharedString>,
        extended: Vec<SharedString>,
        activated: Vec<SharedString>,
    }

    struct Host(Rc<RefCell<Log>>);

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (a, b, c) = (self.0.clone(), self.0.clone(), self.0.clone());
            let selected = self.0.borrow().selected.clone();
            let mut table = DataTable::new()
                .column(DataColumn::new("n", "N"))
                .rows((0..5).map(|i| DataRow::new(format!("r{i}"), [format!("{i}")])))
                .on_row_select(move |id, _, _| a.borrow_mut().selected = Some(id))
                .on_row_extend(move |id, _, _| b.borrow_mut().extended.push(id))
                .on_row_click(move |id, _, _, _| c.borrow_mut().activated.push(id));
            if let Some(id) = selected {
                table = table.selected(id);
            }
            table
        }
    }

    fn selected(log: &Rc<RefCell<Log>>) -> Option<String> {
        log.borrow().selected.as_ref().map(|s| s.to_string())
    }

    #[gpui::test]
    fn keys_move_extend_and_activate(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(Log::default()));
        let (host, cx) = open_fluent_test_window(cx, |_, _| Host(log.clone()));
        cx.click_selector("data-table");
        cx.type_keys("down");
        assert_eq!(selected(&log).as_deref(), Some("r0"));
        host.update(cx, |_, cx| cx.notify());
        cx.type_keys("down");
        assert_eq!(selected(&log).as_deref(), Some("r1"));
        cx.type_keys("end");
        assert_eq!(selected(&log).as_deref(), Some("r4"));
        cx.type_keys("home");
        assert_eq!(selected(&log).as_deref(), Some("r0"));
        cx.type_keys("pagedown");
        assert_eq!(selected(&log).as_deref(), Some("r4"));
        cx.type_keys("pageup");
        assert_eq!(selected(&log).as_deref(), Some("r0"));
        cx.type_keys("shift-down");
        assert_eq!(log.borrow().extended, vec![SharedString::from("r1")]);
        cx.type_keys("space");
        assert_eq!(selected(&log).as_deref(), Some("r0"));
        cx.type_keys("enter");
        assert_eq!(log.borrow().activated, vec![SharedString::from("r0")]);
    }

    /// Host that sorts through `on_sort` and keeps a keyed multi-selection fed by click modifiers.
    struct Model {
        sort: Option<(SharedString, DataSortDirection)>,
        sort_calls: Vec<(SharedString, DataSortDirection)>,
        selection: Vec<SharedString>,
        anchor: Option<SharedString>,
        rows: Vec<(&'static str, &'static str)>,
    }

    struct ModelHost(Rc<RefCell<Model>>);

    impl Render for ModelHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let m = self.0.borrow();
            let (sort_model, click_model) = (self.0.clone(), self.0.clone());
            let (sort_entity, click_entity) = (cx.entity(), cx.entity());
            let selected = m.selection.last().cloned();
            let mut table = DataTable::new()
                .columns([
                    DataColumn::new("name", "Name").width(120.0).sortable(true),
                    DataColumn::new("size", "Size")
                        .width(80.0)
                        .sortable(true)
                        .sort_kind(DataSortKind::Number),
                ])
                .rows(
                    m.rows
                        .iter()
                        .map(|(id, size)| DataRow::new(*id, [*id, *size])),
                )
                .on_sort(move |col, dir, _, _, app| {
                    let mut m = sort_model.borrow_mut();
                    m.sort = Some((col.clone(), dir));
                    m.sort_calls.push((col, dir));
                    drop(m);
                    sort_entity.update(app, |_, cx| cx.notify());
                })
                .on_row_click(move |id, ev, _, app| {
                    let mods = ev.modifiers();
                    let mut m = click_model.borrow_mut();
                    if mods.shift {
                        let ids: Vec<SharedString> =
                            m.rows.iter().map(|(i, _)| SharedString::from(*i)).collect();
                        let anchor = m.anchor.clone().unwrap_or_else(|| id.clone());
                        let a = ids.iter().position(|i| *i == anchor).unwrap_or(0);
                        let b = ids.iter().position(|i| *i == id).unwrap_or(0);
                        m.selection = ids[a.min(b)..=a.max(b)].to_vec();
                    } else if mods.control {
                        if let Some(pos) = m.selection.iter().position(|i| *i == id) {
                            m.selection.remove(pos);
                        } else {
                            m.selection.push(id.clone());
                        }
                        m.anchor = Some(id);
                    } else {
                        m.selection = vec![id.clone()];
                        m.anchor = Some(id);
                    }
                    drop(m);
                    click_entity.update(app, |_, cx| cx.notify());
                });
            if let Some((col, dir)) = m.sort.clone() {
                table = table.sort(col, dir);
            }
            if let Some(id) = selected {
                table = table.selected(id);
            }
            table
        }
    }

    fn model() -> Rc<RefCell<Model>> {
        Rc::new(RefCell::new(Model {
            sort: None,
            sort_calls: Vec::new(),
            selection: Vec::new(),
            anchor: None,
            rows: vec![("b", "20"), ("c", "3"), ("a", "100"), ("d", "7")],
        }))
    }

    /// Centre of header column `col` (widths 120, 80) in window coordinates.
    fn header_point(cx: &mut gpui::VisualTestContext, col: usize) -> gpui::Point<gpui::Pixels> {
        let b = cx.selector_bounds("data-table").expect("table painted");
        let x = [60.0, 160.0][col];
        gpui::point(b.origin.x + px(1.0 + x), b.origin.y + px(1.0 + 16.0))
    }

    /// Centre of body row `idx` as laid out (header 32px, rows 28px).
    fn row_point(cx: &mut gpui::VisualTestContext, idx: usize) -> gpui::Point<gpui::Pixels> {
        let b = cx.selector_bounds("data-table").expect("table painted");
        gpui::point(
            b.origin.x + px(40.0),
            b.origin.y + px(1.0 + 32.0 + 28.0 * idx as f32 + 14.0),
        )
    }

    #[gpui::test]
    fn header_click_sorts_ascending_then_descending(cx: &mut TestAppContext) {
        let m = model();
        let (_host, cx) = open_fluent_test_window(cx, |_, _| ModelHost(m.clone()));
        let p = header_point(cx, 0);
        cx.click_at(p, gpui::MouseButton::Left, gpui::Modifiers::none());
        cx.click_at(p, gpui::MouseButton::Left, gpui::Modifiers::none());
        let other = header_point(cx, 1);
        cx.click_at(other, gpui::MouseButton::Left, gpui::Modifiers::none());
        let calls = m.borrow().sort_calls.clone();
        assert_eq!(
            calls,
            vec![
                (SharedString::from("name"), DataSortDirection::Ascending),
                (SharedString::from("name"), DataSortDirection::Descending),
                (SharedString::from("size"), DataSortDirection::Ascending),
            ]
        );
    }

    #[gpui::test]
    fn sorted_rows_follow_column_order(cx: &mut TestAppContext) {
        let m = model();
        let (_host, cx) = open_fluent_test_window(cx, |_, _| ModelHost(m.clone()));
        // Numeric ascending on size: c(3) d(7) b(20) a(100). Click the first row.
        let p = header_point(cx, 1);
        cx.click_at(p, gpui::MouseButton::Left, gpui::Modifiers::none());
        let p = row_point(cx, 0);
        cx.click_at(p, gpui::MouseButton::Left, gpui::Modifiers::none());
        assert_eq!(m.borrow().selection, vec![SharedString::from("c")]);
        // Descending flips it: a(100) first.
        let p = header_point(cx, 1);
        cx.click_at(p, gpui::MouseButton::Left, gpui::Modifiers::none());
        let p = row_point(cx, 0);
        cx.click_at(p, gpui::MouseButton::Left, gpui::Modifiers::none());
        assert_eq!(m.borrow().selection, vec![SharedString::from("a")]);
    }

    #[gpui::test]
    fn click_ctrl_and_shift_select(cx: &mut TestAppContext) {
        let m = model();
        let (_host, cx) = open_fluent_test_window(cx, |_, _| ModelHost(m.clone()));
        let ids = |m: &Rc<RefCell<Model>>| -> Vec<String> {
            m.borrow().selection.iter().map(|s| s.to_string()).collect()
        };
        let none = gpui::Modifiers::none();
        let ctrl = gpui::Modifiers::control();
        let shift = gpui::Modifiers::shift();

        let p = row_point(cx, 0);
        cx.click_at(p, gpui::MouseButton::Left, none);
        assert_eq!(ids(&m), ["b"]);
        let p = row_point(cx, 2);
        cx.click_at(p, gpui::MouseButton::Left, ctrl);
        assert_eq!(ids(&m), ["b", "a"]);
        let p = row_point(cx, 0);
        cx.click_at(p, gpui::MouseButton::Left, ctrl);
        assert_eq!(ids(&m), ["a"]);
        let p = row_point(cx, 1);
        cx.click_at(p, gpui::MouseButton::Left, none);
        assert_eq!(ids(&m), ["c"]);
        let p = row_point(cx, 3);
        cx.click_at(p, gpui::MouseButton::Left, shift);
        assert_eq!(ids(&m), ["c", "a", "d"]);
    }

    #[gpui::test]
    fn selection_survives_refresh_by_row_key(cx: &mut TestAppContext) {
        let m = model();
        let (host, cx) = open_fluent_test_window(cx, |_, _| ModelHost(m.clone()));
        let p = row_point(cx, 2); // "a"
        cx.click_at(p, gpui::MouseButton::Left, gpui::Modifiers::none());
        assert_eq!(m.borrow().selection, vec![SharedString::from("a")]);
        // Refresh: reorder and replace the data, keeping row keys; "a" moves to the top.
        m.borrow_mut().rows = vec![("a", "101"), ("z", "1"), ("b", "21")];
        host.update(cx, |_, cx| cx.notify());
        cx.run_until_parked();
        assert_eq!(m.borrow().selection, vec![SharedString::from("a")]);
        // Clicking the key's new position selects the same key.
        let p = row_point(cx, 0);
        cx.click_at(p, gpui::MouseButton::Left, gpui::Modifiers::none());
        assert_eq!(m.borrow().selection, vec![SharedString::from("a")]);
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_grid_rows_and_cells(cx: &mut TestAppContext) {
        struct SnapHost;
        impl Render for SnapHost {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                DataTable::new()
                    .column(DataColumn::new("n", "Name"))
                    .column(DataColumn::new("s", "Size"))
                    .rows(
                        (0..2).map(|i| {
                            DataRow::new(format!("r{i}"), [format!("n{i}"), format!("{i}")])
                        }),
                    )
                    .selected("r1")
            }
        }
        let (_, vcx) = open_fluent_test_window(cx, |_, _| SnapHost);
        assert_eq!(
            vcx.accessibility_snapshot(),
            vec![
                "Window".to_string(),
                "Grid [row_count=3, column_count=2]".to_string(),
                "Row [row_index=1]".to_string(),
                "ColumnHeader \"Name\" [column_index=1]".to_string(),
                "ColumnHeader \"Size\" [column_index=2]".to_string(),
                "Row [selected=false, row_index=2]".to_string(),
                "Cell \"n0\" [column_index=1]".to_string(),
                "Cell \"0\" [column_index=2]".to_string(),
                "Row [selected=true, row_index=3]".to_string(),
                "Cell \"n1\" [column_index=1]".to_string(),
                "Cell \"1\" [column_index=2]".to_string(),
            ]
        );
    }
}
