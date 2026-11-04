use fluent_core::TextRampExt;
use std::{rc::Rc, sync::Arc};

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, App, ClickEvent, ElementId, FontWeight, IntoElement, KeyDownEvent,
    MouseButton, MouseDownEvent, RenderOnce, Role, SharedString, Window,
};

use crate::virtual_list::VirtualList;

type RowSource = Rc<dyn Fn(usize) -> SelectableListItem + 'static>;
type ListSelectHandler = Arc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;
type ListActivateHandler = Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>;
type ListContextHandler =
    Arc<dyn Fn(SharedString, &MouseDownEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, Default)]
/// Public API item.
pub enum SelectableListState {
    #[default]
    /// Public API item.
    Ready,
    /// Public API item.
    Loading(SharedString),
    /// Public API item.
    Empty(SharedString),
    /// Public API item.
    Error(SharedString),
}

#[derive(Clone, Debug)]
/// Public API item.
pub struct SelectableListItem {
    /// Public API item.
    pub id: SharedString,
    /// Public API item.
    pub title: SharedString,
    /// Public API item.
    pub subtitle: Option<SharedString>,
    /// Public API item.
    pub detail: Option<SharedString>,
    /// Public API item.
    pub meta: Option<SharedString>,
    /// Public API item.
    pub badge: Option<SharedString>,
    /// Public API item.
    pub icon: Option<SharedString>,
    /// Public API item.
    pub disabled: bool,
    /// Public API item.
    pub emphasized: bool,
}

impl SelectableListItem {
    /// Public API item.
    pub fn new(id: impl Into<SharedString>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            subtitle: None,
            detail: None,
            meta: None,
            badge: None,
            icon: None,
            disabled: false,
            emphasized: false,
        }
    }

    /// Public API item.
    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Public API item.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Public API item.
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// Public API item.
    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Public API item.
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Public API item.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Public API item.
    pub fn emphasized(mut self, emphasized: bool) -> Self {
        self.emphasized = emphasized;
        self
    }
}

/// A single-selection list with virtualised rows (only the visible rows plus
/// overscan are built), keyboard roving with scroll-into-view,
/// loading/empty/error states, and activation/context-menu callbacks.
///
/// Rows come either from [`SelectableList::items`] or, for large
/// collections, from [`SelectableList::rows`]: an item count plus a function
/// that produces the item at an index on demand. Rows are `row_height` tall
/// (uniform) unless variable heights are enabled with
/// [`SelectableList::variable_row_heights`], in which case each row is measured
/// and `row_height` is the estimate for rows not yet built.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct SelectableList {
    id: ElementId,
    items: Vec<SelectableListItem>,
    rows: Option<(usize, RowSource)>,
    selected_index: Option<usize>,
    variable_heights: bool,
    state: SelectableListState,
    empty_text: SharedString,
    selected_id: Option<SharedString>,
    row_height: f32,
    max_body_height: Option<f32>,
    on_select: Option<ListSelectHandler>,
    on_activate: Option<ListActivateHandler>,
    on_context_menu: Option<ListContextHandler>,
}

impl SelectableList {
    /// Public API item.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            rows: None,
            selected_index: None,
            variable_heights: false,
            state: SelectableListState::Ready,
            empty_text: "No items".into(),
            selected_id: None,
            row_height: 56.0,
            max_body_height: None,
            on_select: None,
            on_activate: None,
            on_context_menu: None,
        }
    }

    /// Public API item.
    pub fn item(mut self, item: SelectableListItem) -> Self {
        self.items.push(item);
        self
    }

    /// Public API item.
    pub fn items(mut self, items: impl IntoIterator<Item = SelectableListItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Supplies `count` rows built on demand by `item_at`. Only the rows near
    /// the viewport are requested, so `count` may be very large. Replaces any
    /// items added with `item`/`items`.
    pub fn rows(
        mut self,
        count: usize,
        item_at: impl Fn(usize) -> SelectableListItem + 'static,
    ) -> Self {
        self.rows = Some((count, Rc::new(item_at)));
        self
    }

    /// Selects the row at `index` (for lists built with [`Self::rows`]).
    pub fn selected_index(mut self, index: usize) -> Self {
        self.selected_index = Some(index);
        self
    }

    /// Lets rows differ in height (for example when a row wraps its subtitle).
    /// Rows are measured as they are built.
    pub fn variable_row_heights(mut self, variable: bool) -> Self {
        self.variable_heights = variable;
        self
    }

    /// Public API item.
    pub fn state(mut self, state: SelectableListState) -> Self {
        self.state = state;
        self
    }

    /// Public API item.
    pub fn loading(mut self, message: impl Into<SharedString>) -> Self {
        self.state = SelectableListState::Loading(message.into());
        self
    }

    /// Public API item.
    pub fn empty_message(mut self, message: impl Into<SharedString>) -> Self {
        self.state = SelectableListState::Empty(message.into());
        self
    }

    /// Public API item.
    pub fn empty_text(mut self, message: impl Into<SharedString>) -> Self {
        self.empty_text = message.into();
        self
    }

    /// Public API item.
    pub fn error_message(mut self, message: impl Into<SharedString>) -> Self {
        self.state = SelectableListState::Error(message.into());
        self
    }

    /// Public API item.
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    /// Public API item.
    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(32.0);
        self
    }

    /// Public API item.
    pub fn max_body_height(mut self, height: f32) -> Self {
        self.max_body_height = Some(height.max(32.0));
        self
    }

    /// Public API item.
    pub fn on_select(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Public API item.
    pub fn on_activate(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Arc::new(f));
        self
    }

    /// Public API item.
    pub fn on_context_menu(
        mut self,
        f: impl Fn(SharedString, &MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_context_menu = Some(Arc::new(f));
        self
    }

    /// Index of the row a navigation key moves to, skipping disabled rows.
    /// Returns `None` when the key is not a navigation key or no row is enabled.
    fn key_target(
        count: usize,
        disabled: &dyn Fn(usize) -> bool,
        current: Option<usize>,
        key: &str,
    ) -> Option<usize> {
        if count == 0 {
            return None;
        }
        let last = count - 1;
        let cur = current.unwrap_or(0).min(last);
        let (goal, forward) = match key {
            "up" => (cur.checked_sub(1)?, false),
            "down" => (cur + 1, true),
            "home" => (0, true),
            "end" => (last, false),
            "pageup" => (cur.saturating_sub(5), false),
            "pagedown" => (cur + 5, true),
            _ => return None,
        };
        let goal = goal.min(last);
        let mut ix = goal;
        loop {
            if !disabled(ix) {
                return Some(ix);
            }
            if forward {
                if ix == last {
                    break;
                }
                ix += 1;
            } else {
                if ix == 0 {
                    break;
                }
                ix -= 1;
            }
        }
        // Nothing enabled beyond the goal in the travel direction.
        (!disabled(cur)).then_some(cur)
    }
}

impl RenderOnce for SelectableList {
    fn render(mut self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let typography = theme.typography;
        let row_height = self.row_height;
        let variable_heights = self.variable_heights;
        let (count, item_at): (usize, RowSource) = match self.rows.take() {
            Some(rows) => rows,
            None => {
                let items = Rc::new(std::mem::take(&mut self.items));
                (items.len(), Rc::new(move |ix| items[ix].clone()))
            }
        };
        let selected_ix = match (self.selected_index, self.selected_id.as_ref()) {
            (Some(ix), _) => Some(ix),
            (None, Some(id)) => (0..count).find(|ix| &item_at(*ix).id == id),
            (None, None) => None,
        };
        let on_select = self.on_select.clone();
        let on_activate = self.on_activate.clone();
        let key_item_at = item_at.clone();

        let mut root = div()
            .id(self.id.clone())
            .role(Role::List)
            .aria_size_of_set(count)
            .tab_index(0)
            .key_context("SelectableList")
            .on_key_down(
                move |ev: &KeyDownEvent, window, app| match ev.keystroke.key.as_str() {
                    "up" | "down" | "home" | "end" | "pageup" | "pagedown" => {
                        let target = Self::key_target(
                            count,
                            &|ix| key_item_at(ix).disabled,
                            selected_ix,
                            &ev.keystroke.key,
                        );
                        if let (Some(handler), Some(ix)) = (on_select.as_ref(), target) {
                            handler(key_item_at(ix).id, window, app);
                        }
                    }
                    "return" | "space" => {
                        if count == 0 {
                            return;
                        }
                        let ix = selected_ix.unwrap_or(0).min(count - 1);
                        let item = key_item_at(ix);
                        if let (Some(handler), false) = (on_activate.as_ref(), item.disabled) {
                            let ev = ClickEvent::default();
                            handler(item.id, &ev, window, app);
                        }
                    }
                    _ => {}
                },
            )
            .flex()
            .flex_col()
            .w_full()
            .min_h_0();

        match &self.state {
            SelectableListState::Loading(message)
            | SelectableListState::Empty(message)
            | SelectableListState::Error(message) => {
                let color = match &self.state {
                    SelectableListState::Error(_) => colors.status_error,
                    _ => colors.on_subtle,
                };
                return root.child(status_panel(
                    message.clone(),
                    color,
                    row_height,
                    &typography,
                ));
            }
            SelectableListState::Ready if count == 0 => {
                return root.child(status_panel(
                    self.empty_text,
                    colors.on_subtle_disabled,
                    row_height,
                    &typography,
                ));
            }
            SelectableListState::Ready => {}
        }

        let activate_handler = self.on_activate.clone();
        let select_handler = self.on_select.clone();
        let context_handler = self.on_context_menu.clone();
        let build_row = move |idx: usize, _window: &mut Window, cx: &mut App| {
            render_row(
                idx,
                count,
                item_at(idx),
                selected_ix == Some(idx),
                row_height,
                variable_heights,
                activate_handler.clone(),
                select_handler.clone(),
                context_handler.clone(),
                cx,
            )
            .into_any_element()
        };
        let mut list = div().w_full().child(
            VirtualList::new(("selectable-list-body", 0u64), count, build_row)
                .row_height(row_height)
                .reveal(selected_ix),
        );

        if let Some(height) = self.max_body_height {
            list = list.h(px(height).min(px(row_height * count as f32)));
        } else {
            list = list.h_full();
        }

        root = root.child(list);
        root
    }
}

fn status_panel(
    message: SharedString,
    color: gpui::Hsla,
    row_height: f32,
    typography: &fluent_core::TypographyTokens,
) -> gpui::Div {
    div()
        .min_h(px(row_height * 2.0))
        .flex()
        .items_center()
        .justify_center()
        .text_ramp(&typography.body)
        .text_color(color)
        .child(message)
}

#[allow(clippy::too_many_arguments)]
fn render_row(
    idx: usize,
    count: usize,
    item: SelectableListItem,
    selected: bool,
    row_height: f32,
    variable_height: bool,
    activate_handler: Option<ListActivateHandler>,
    select_handler: Option<ListSelectHandler>,
    context_handler: Option<ListContextHandler>,
    cx: &mut App,
) -> impl IntoElement {
    let theme = cx.theme();
    let colors = theme.colors.clone();
    let spacing = theme.spacing;
    let typography = theme.typography;
    let components = theme.components;
    let fg = if item.disabled {
        colors.on_neutral_disabled
    } else {
        colors.on_neutral
    };
    let mut row = div()
        .id(("selectable-list-row", idx as u64))
        .role(Role::ListItem)
        .aria_label(item.title.clone())
        .aria_selected(selected)
        .aria_position_in_set(idx + 1)
        .aria_size_of_set(count)
        .flex()
        .flex_row()
        .items_center()
        .gap(px(spacing.sm))
        .when(variable_height, |row| row.min_h(px(row_height)))
        .when(!variable_height, |row| row.h(px(row_height)))
        .w_full()
        .px(px(spacing.md))
        .bg(if selected {
            colors.neutral_selected
        } else if item.emphasized {
            colors.subtle_hover
        } else {
            colors.surface
        })
        .border_b_1()
        .border_color(colors.stroke_neutral_subtle)
        .text_color(fg);

    if !item.disabled {
        row = row.cursor_pointer().hover({
            let hover = colors.neutral_hover;
            move |s| {
                s.bg(if selected {
                    colors.neutral_selected
                } else {
                    hover
                })
            }
        });

        if select_handler.is_some() || activate_handler.is_some() {
            let id = item.id.clone();
            let select = select_handler.clone();
            let activate = activate_handler.clone();
            row = row.on_click(move |ev, win, app| {
                if let Some(handler) = &select {
                    handler(id.clone(), win, app);
                }
                if ev.click_count() >= 2 {
                    if let Some(handler) = &activate {
                        handler(id.clone(), ev, win, app);
                    }
                }
            });
        }

        if let Some(handler) = context_handler {
            let id = item.id.clone();
            row = row.on_mouse_down(MouseButton::Right, move |ev, win, app| {
                handler(id.clone(), ev, win, app);
            });
        }
    }

    if let Some(icon) = item.icon.clone() {
        row = row.child(
            div()
                .flex_none()
                .size(px(components.popup_icon_slot))
                .flex()
                .items_center()
                .justify_center()
                .child(
                    svg()
                        .path(icon)
                        .size(px(components.popup_icon_slot))
                        .text_color(fg),
                ),
        );
    }

    let mut text = div()
        .flex()
        .flex_col()
        .justify_center()
        .gap(px(2.0))
        .flex_1()
        .min_w_0()
        .overflow_hidden()
        .child(
            div()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .text_ramp(&typography.body)
                .font_weight(if item.emphasized {
                    FontWeight::SEMIBOLD
                } else {
                    FontWeight::NORMAL
                })
                .child(item.title.clone()),
        );

    if item.subtitle.is_some() || item.detail.is_some() {
        let mut secondary = String::new();
        if let Some(subtitle) = item.subtitle.as_ref() {
            secondary.push_str(subtitle.as_ref());
        }
        if let Some(detail) = item.detail.as_ref() {
            if !secondary.is_empty() {
                secondary.push_str(" - ");
            }
            secondary.push_str(detail.as_ref());
        }
        text = text.child(
            div()
                .overflow_hidden()
                .when(!variable_height, |d| d.whitespace_nowrap().text_ellipsis())
                .py(px(if variable_height { 4.0 } else { 0.0 }))
                .text_ramp(&typography.caption)
                .text_color(if item.disabled {
                    colors.on_neutral_disabled
                } else {
                    colors.on_subtle
                })
                .child(secondary),
        );
    }

    row = row.child(text);

    if let Some(meta) = item.meta {
        row = row.child(
            div()
                .flex_none()
                .text_ramp(&typography.caption)
                .text_color(colors.on_subtle)
                .child(meta),
        );
    }

    if let Some(badge) = item.badge {
        row = row.child(
            div()
                .flex_none()
                .px(px(5.0))
                .py(px(1.0))
                .rounded(px(3.0))
                .bg(colors.neutral)
                .text_size(px(10.0))
                .text_color(colors.on_subtle)
                .child(badge),
        );
    }

    row
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use fluent_test_utils::*;
    use gpui::{Context, IntoElement, Render, Window};

    use super::*;

    fn none_disabled(_: usize) -> bool {
        false
    }

    #[test]
    fn key_target_skips_disabled_rows() {
        let disabled = |ix: usize| ix == 1;
        assert_eq!(
            SelectableList::key_target(3, &disabled, Some(0), "down"),
            Some(2)
        );
        assert_eq!(
            SelectableList::key_target(3, &disabled, Some(2), "up"),
            Some(0)
        );
    }

    #[test]
    fn key_target_clamps_home_end_and_pages() {
        assert_eq!(
            SelectableList::key_target(3, &none_disabled, Some(1), "home"),
            Some(0)
        );
        assert_eq!(
            SelectableList::key_target(3, &none_disabled, Some(1), "end"),
            Some(2)
        );
        assert_eq!(
            SelectableList::key_target(3, &none_disabled, Some(1), "pagedown"),
            Some(2)
        );
        assert_eq!(
            SelectableList::key_target(100_000, &none_disabled, Some(10), "pagedown"),
            Some(15)
        );
        assert_eq!(
            SelectableList::key_target(0, &none_disabled, None, "down"),
            None
        );
    }

    thread_local! {
        static BUILT: Cell<usize> = const { Cell::new(0) };
    }

    struct Host {
        selected: usize,
        variable: bool,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let view = cx.entity();
            let mut list = SelectableList::new("big")
                .rows(100_000, |ix| {
                    BUILT.with(|b| b.set(b.get() + 1));
                    let item = SelectableListItem::new(format!("r{ix}"), format!("Row {ix}"));
                    if ix % 3 == 0 {
                        item.subtitle("with a second line")
                    } else {
                        item
                    }
                })
                .selected_index(self.selected)
                .max_body_height(320.0)
                .on_select(move |id, _, cx| {
                    let ix: usize = id[1..].parse().unwrap();
                    view.update(cx, |host, cx| {
                        host.selected = ix;
                        cx.notify();
                    });
                });
            if self.variable {
                list = list.variable_row_heights(true);
            }
            list
        }
    }

    fn host(cx: &mut gpui::TestAppContext, variable: bool) -> &mut gpui::VisualTestContext {
        BUILT.with(|b| b.set(0));
        let (_, cx) = open_fluent_test_window(cx, |_, _| Host {
            selected: 0,
            variable,
        });
        cx.run_until_parked();
        cx
    }

    #[gpui::test]
    fn builds_only_visible_rows_plus_overscan(cx: &mut gpui::TestAppContext) {
        let _cx = host(cx, false);
        let built = BUILT.with(Cell::get);
        // 320px viewport / 56px rows is 6 visible rows; overscan adds a few more.
        assert!(built > 0 && built <= 20, "built {built} rows of 100000");
    }

    #[gpui::test]
    fn variable_heights_also_virtualise(cx: &mut gpui::TestAppContext) {
        let _cx = host(cx, true);
        let built = BUILT.with(Cell::get);
        assert!(built > 0 && built <= 30, "built {built} rows of 100000");
    }

    #[gpui::test]
    fn keyboard_navigation_selects_and_scrolls_into_view(cx: &mut gpui::TestAppContext) {
        let cx = host(cx, false);
        cx.update(|window, cx| {
            window.focus_next(cx);
        });
        cx.type_keys("down down down");
        cx.type_keys("end");
        cx.run_until_parked();
        assert_a11y_node(
            &cx.accessibility_snapshot(),
            "ListItem",
            Some("Row 99999"),
            &["selected=true", "position_in_set=100000"],
        );
        let built = BUILT.with(Cell::get);
        assert!(built < 200, "built {built} rows after scrolling to the end");
    }

    #[gpui::test]
    fn exposes_set_size_and_position(cx: &mut gpui::TestAppContext) {
        let cx = host(cx, false);
        let snapshot = cx.accessibility_snapshot();
        assert_a11y_node(&snapshot, "List", None, &["size_of_set=100000"]);
        assert_a11y_node(
            &snapshot,
            "ListItem",
            Some("Row 0"),
            &["selected=true", "position_in_set=1", "size_of_set=100000"],
        );
    }
}
