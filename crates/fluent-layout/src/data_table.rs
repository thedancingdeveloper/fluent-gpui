use std::sync::Arc;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, IntoElement, KeyDownEvent, RenderOnce, SharedString,
    Window,
};

type RowClickHandler = Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>;
type RowSelectHandler = Arc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;
type SortHandler =
    Arc<dyn Fn(SharedString, DataSortDirection, &ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataSortDirection {
    Ascending,
    Descending,
}

impl DataSortDirection {
    pub fn toggle(self) -> Self {
        match self {
            Self::Ascending => Self::Descending,
            Self::Descending => Self::Ascending,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataCellAlign {
    #[default]
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DataSortKind {
    #[default]
    Text,
    Number,
}

#[derive(Clone, Debug)]
pub struct DataCell {
    pub text: SharedString,
    pub align: Option<DataCellAlign>,
}

impl DataCell {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            align: None,
        }
    }

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
pub enum DataTableState {
    Ready,
    Loading(SharedString),
    Empty(SharedString),
    Error(SharedString),
}

#[derive(Clone, Debug)]
pub struct DataColumn {
    pub id: SharedString,
    pub title: SharedString,
    pub width: Option<f32>,
    pub sortable: bool,
    pub align: DataCellAlign,
    pub sort_kind: DataSortKind,
}

impl DataColumn {
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

    pub fn width(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn align(mut self, align: DataCellAlign) -> Self {
        self.align = align;
        self
    }

    pub fn sort_kind(mut self, kind: DataSortKind) -> Self {
        self.sort_kind = kind;
        self
    }
}

#[derive(Clone, Debug)]
pub struct DataRow {
    pub id: SharedString,
    pub cells: Vec<DataCell>,
}

impl DataRow {
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
    on_sort: Option<SortHandler>,
}

impl DataTable {
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
            on_sort: None,
        }
    }

    pub fn column(mut self, column: DataColumn) -> Self {
        self.columns.push(column);
        self
    }

    pub fn columns(mut self, columns: impl IntoIterator<Item = DataColumn>) -> Self {
        self.columns = columns.into_iter().collect();
        self
    }

    pub fn row(mut self, row: DataRow) -> Self {
        self.rows.push(row);
        self
    }

    pub fn rows(mut self, rows: impl IntoIterator<Item = DataRow>) -> Self {
        self.rows = rows.into_iter().collect();
        self
    }

    pub fn state(mut self, state: DataTableState) -> Self {
        self.state = state;
        self
    }

    pub fn loading(mut self, message: impl Into<SharedString>) -> Self {
        self.state = DataTableState::Loading(message.into());
        self
    }

    pub fn empty_message(mut self, message: impl Into<SharedString>) -> Self {
        self.state = DataTableState::Empty(message.into());
        self
    }

    pub fn error_message(mut self, message: impl Into<SharedString>) -> Self {
        self.state = DataTableState::Error(message.into());
        self
    }

    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    pub fn sort(
        mut self,
        column_id: impl Into<SharedString>,
        direction: DataSortDirection,
    ) -> Self {
        self.sort = Some((column_id.into(), direction));
        self
    }

    pub fn max_body_height(mut self, height: f32) -> Self {
        self.max_body_height = Some(height);
        self
    }

    pub fn on_row_click(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row_click = Some(Arc::new(f));
        self
    }

    pub fn on_row_select(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_row_select = Some(Arc::new(f));
        self
    }

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
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
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
        let key_row_ids = row_ids.clone();

        let mut root = div()
            .tab_index(0)
            .key_context("DataTable")
            .on_key_down(move |ev: &KeyDownEvent, window, app| {
                if key_row_ids.is_empty() {
                    return;
                }
                match ev.keystroke.key.as_str() {
                    "up" | "down" => {
                        let current = selected_index.unwrap_or(0);
                        let next = if ev.keystroke.key.as_str() == "up" {
                            current.saturating_sub(1)
                        } else {
                            (current + 1).min(key_row_ids.len() - 1)
                        };
                        if let Some(handler) = &on_row_select {
                            handler(key_row_ids[next].clone(), window, app);
                        }
                    }
                    "return" | "space" => {
                        let current = selected_index.unwrap_or(0).min(key_row_ids.len() - 1);
                        if let Some(handler) = &on_row_activate {
                            let ev = ClickEvent::default();
                            handler(key_row_ids[current].clone(), &ev, window, app);
                        }
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
            let mut label = div().child(column.title.clone());
            if active_sort {
                let arrow = match self.sort.as_ref().map(|(_, direction)| *direction) {
                    Some(DataSortDirection::Ascending) => " ↑",
                    Some(DataSortDirection::Descending) => " ↓",
                    None => "",
                };
                label = label.child(arrow);
            }

            let mut cell = div()
                .id(("data-header", col_idx as u64))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing.xs))
                .px(px(spacing.md))
                .text_size(px(typography.caption.size))
                .font_weight(gpui::FontWeight::SEMIBOLD)
                .text_color(colors.on_subtle)
                .border_r_1()
                .border_color(colors.stroke_neutral_subtle)
                .child(label);
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
            if let Some(width) = column.width {
                cell = cell.w(px(width)).flex_none();
            } else {
                cell = cell.flex_1();
            }
            header = header.child(cell);
        }

        root = root.child(header);

        match &self.state {
            DataTableState::Loading(message)
            | DataTableState::Empty(message)
            | DataTableState::Error(message) => {
                let color = match &self.state {
                    DataTableState::Error(_) => colors.status_error,
                    _ => colors.on_subtle,
                };
                return root.child(
                    div()
                        .min_h(px(components.menu_item_height * 3.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(typography.body.size))
                        .text_color(color)
                        .child(message.clone()),
                );
            }
            DataTableState::Ready if rows.is_empty() => {
                return root.child(
                    div()
                        .min_h(px(components.menu_item_height * 3.0))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(typography.body.size))
                        .text_color(colors.on_subtle_disabled)
                        .child("No rows"),
                );
            }
            DataTableState::Ready => {}
        }

        let mut body = div().id("data-table-body").flex().flex_col();
        if let Some(height) = self.max_body_height {
            body = body.overflow_y_scroll().max_h(px(height));
        }

        for (idx, row) in rows.into_iter().enumerate() {
            let is_selected = self.selected_id.as_ref() == Some(&row.id);
            let row_id = row.id.clone();
            let mut row_el = div()
                .id(("data-row", idx as u64))
                .flex()
                .flex_row()
                .min_h(px(components.menu_item_height))
                .bg(if is_selected {
                    colors.neutral_selected
                } else {
                    colors.surface
                })
                .border_b_1()
                .border_color(colors.stroke_neutral_subtle);

            if let Some(handler) = self.on_row_click.clone() {
                row_el = row_el
                    .cursor_pointer()
                    .hover({
                        let hover = colors.neutral_hover;
                        move |s| {
                            s.bg(if is_selected {
                                colors.neutral_selected
                            } else {
                                hover
                            })
                        }
                    })
                    .on_click(move |ev, win, app| handler(row_id.clone(), ev, win, app));
            }

            for (col_idx, column) in self.columns.iter().enumerate() {
                let data_cell = row
                    .cells
                    .get(col_idx)
                    .cloned()
                    .unwrap_or_else(|| DataCell::new(""));
                let align = data_cell.align.unwrap_or(column.align);
                let mut cell = div()
                    .flex()
                    .items_center()
                    .when(align == DataCellAlign::Center, |d| d.justify_center())
                    .when(align == DataCellAlign::End, |d| d.justify_end())
                    .px(px(spacing.md))
                    .py(px(spacing.xs))
                    .text_size(px(typography.body.size))
                    .text_color(colors.on_neutral)
                    .border_r_1()
                    .border_color(colors.stroke_neutral_subtle)
                    .child(data_cell.text);
                if let Some(width) = column.width {
                    cell = cell.w(px(width)).flex_none();
                } else {
                    cell = cell.flex_1();
                }
                row_el = row_el.child(cell);
            }

            body = body.child(row_el);
        }

        root.child(body)
    }
}
