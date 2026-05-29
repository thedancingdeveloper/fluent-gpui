use std::sync::Arc;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, uniform_list, App, ClickEvent, ElementId, FontWeight, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, RenderOnce, SharedString, Window,
};

type ListSelectHandler = Arc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;
type ListActivateHandler = Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>;
type ListContextHandler =
    Arc<dyn Fn(SharedString, &MouseDownEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, Default)]
pub enum SelectableListState {
    #[default]
    Ready,
    Loading(SharedString),
    Empty(SharedString),
    Error(SharedString),
}

#[derive(Clone, Debug)]
pub struct SelectableListItem {
    pub id: SharedString,
    pub title: SharedString,
    pub subtitle: Option<SharedString>,
    pub detail: Option<SharedString>,
    pub meta: Option<SharedString>,
    pub badge: Option<SharedString>,
    pub icon: Option<SharedString>,
    pub disabled: bool,
    pub emphasized: bool,
}

impl SelectableListItem {
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

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn emphasized(mut self, emphasized: bool) -> Self {
        self.emphasized = emphasized;
        self
    }
}

#[derive(Clone, Debug)]
struct SelectableKeyItem {
    id: SharedString,
}

/// A single-selection list with lazy uniform-row rendering, keyboard roving,
/// loading/empty/error states, and activation/context-menu callbacks.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct SelectableList {
    id: ElementId,
    items: Vec<SelectableListItem>,
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
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
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

    pub fn item(mut self, item: SelectableListItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SelectableListItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn state(mut self, state: SelectableListState) -> Self {
        self.state = state;
        self
    }

    pub fn loading(mut self, message: impl Into<SharedString>) -> Self {
        self.state = SelectableListState::Loading(message.into());
        self
    }

    pub fn empty_message(mut self, message: impl Into<SharedString>) -> Self {
        self.state = SelectableListState::Empty(message.into());
        self
    }

    pub fn empty_text(mut self, message: impl Into<SharedString>) -> Self {
        self.empty_text = message.into();
        self
    }

    pub fn error_message(mut self, message: impl Into<SharedString>) -> Self {
        self.state = SelectableListState::Error(message.into());
        self
    }

    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    pub fn row_height(mut self, height: f32) -> Self {
        self.row_height = height.max(32.0);
        self
    }

    pub fn max_body_height(mut self, height: f32) -> Self {
        self.max_body_height = Some(height.max(32.0));
        self
    }

    pub fn on_select(mut self, f: impl Fn(SharedString, &mut Window, &mut App) + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    pub fn on_activate(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Arc::new(f));
        self
    }

    pub fn on_context_menu(
        mut self,
        f: impl Fn(SharedString, &MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_context_menu = Some(Arc::new(f));
        self
    }

    fn key_items(items: &[SelectableListItem]) -> Vec<SelectableKeyItem> {
        items
            .iter()
            .filter(|item| !item.disabled)
            .map(|item| SelectableKeyItem {
                id: item.id.clone(),
            })
            .collect()
    }

    fn selected_key_index(
        key_items: &[SelectableKeyItem],
        selected_id: Option<&SharedString>,
    ) -> Option<usize> {
        selected_id.and_then(|id| key_items.iter().position(|item| &item.id == id))
    }

    fn key_target(
        key_items: &[SelectableKeyItem],
        selected_id: Option<&SharedString>,
        key: &str,
    ) -> Option<SharedString> {
        if key_items.is_empty() {
            return None;
        }
        let current = Self::selected_key_index(key_items, selected_id).unwrap_or(0);
        let next = match key {
            "up" => current.saturating_sub(1),
            "down" => (current + 1).min(key_items.len() - 1),
            "home" => 0,
            "end" => key_items.len() - 1,
            "pageup" => current.saturating_sub(5),
            "pagedown" => (current + 5).min(key_items.len() - 1),
            _ => return None,
        };
        Some(key_items[next].id.clone())
    }
}

impl RenderOnce for SelectableList {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let typography = theme.typography;
        let row_height = self.row_height;
        let key_items = Self::key_items(&self.items);
        let selected_id = self.selected_id.clone();
        let on_select = self.on_select.clone();
        let on_activate = self.on_activate.clone();

        let mut root = div()
            .tab_index(0)
            .key_context("SelectableList")
            .on_key_down(
                move |ev: &KeyDownEvent, window, app| match ev.keystroke.key.as_str() {
                    "up" | "down" | "home" | "end" | "pageup" | "pagedown" => {
                        if let (Some(handler), Some(id)) = (
                            on_select.as_ref(),
                            Self::key_target(&key_items, selected_id.as_ref(), &ev.keystroke.key),
                        ) {
                            handler(id, window, app);
                        }
                    }
                    "return" | "space" => {
                        let current = Self::selected_key_index(&key_items, selected_id.as_ref())
                            .unwrap_or(0)
                            .min(key_items.len().saturating_sub(1));
                        if let (Some(handler), Some(item)) =
                            (on_activate.as_ref(), key_items.get(current))
                        {
                            let ev = ClickEvent::default();
                            handler(item.id.clone(), &ev, window, app);
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
            SelectableListState::Ready if self.items.is_empty() => {
                return root.child(status_panel(
                    self.empty_text,
                    colors.on_subtle_disabled,
                    row_height,
                    &typography,
                ));
            }
            SelectableListState::Ready => {}
        }

        let items = self.items;
        let selected_id = self.selected_id.clone();
        let activate_handler = self.on_activate.clone();
        let select_handler = self.on_select.clone();
        let context_handler = self.on_context_menu.clone();
        let mut list = uniform_list(self.id, items.len(), move |range, _window, cx| {
            range
                .map(|idx| {
                    render_row(
                        idx,
                        items[idx].clone(),
                        selected_id.as_ref() == Some(&items[idx].id),
                        row_height,
                        activate_handler.clone(),
                        select_handler.clone(),
                        context_handler.clone(),
                        cx,
                    )
                })
                .collect::<Vec<_>>()
        })
        .w_full();

        if let Some(height) = self.max_body_height {
            list = list.max_h(px(height));
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
        .text_size(px(typography.body.size))
        .text_color(color)
        .child(message)
}

#[allow(clippy::too_many_arguments)]
fn render_row(
    idx: usize,
    item: SelectableListItem,
    selected: bool,
    row_height: f32,
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
        .flex()
        .flex_row()
        .items_center()
        .gap(px(spacing.sm))
        .h(px(row_height))
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
                .text_size(px(typography.body.size))
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
                .whitespace_nowrap()
                .text_ellipsis()
                .text_size(px(typography.caption.size))
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
                .text_size(px(typography.caption.size))
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
    use super::*;

    fn ids(items: &[SelectableKeyItem]) -> Vec<&str> {
        items.iter().map(|item| item.id.as_ref()).collect()
    }

    #[test]
    fn key_items_skip_disabled_rows() {
        let items = vec![
            SelectableListItem::new("one", "One"),
            SelectableListItem::new("two", "Two").disabled(true),
            SelectableListItem::new("three", "Three"),
        ];

        assert_eq!(
            ids(&SelectableList::key_items(&items)),
            vec!["one", "three"]
        );
    }

    #[test]
    fn key_target_moves_from_selected_row() {
        let items = SelectableList::key_items(&[
            SelectableListItem::new("one", "One"),
            SelectableListItem::new("two", "Two"),
            SelectableListItem::new("three", "Three"),
        ]);
        let selected = SharedString::from("two");

        assert_eq!(
            SelectableList::key_target(&items, Some(&selected), "down")
                .map(|id| id.to_string())
                .as_deref(),
            Some("three")
        );
        assert_eq!(
            SelectableList::key_target(&items, Some(&selected), "up")
                .map(|id| id.to_string())
                .as_deref(),
            Some("one")
        );
    }

    #[test]
    fn key_target_clamps_home_end_and_pages() {
        let items = SelectableList::key_items(&[
            SelectableListItem::new("one", "One"),
            SelectableListItem::new("two", "Two"),
            SelectableListItem::new("three", "Three"),
        ]);
        let selected = SharedString::from("two");

        assert_eq!(
            SelectableList::key_target(&items, Some(&selected), "home")
                .map(|id| id.to_string())
                .as_deref(),
            Some("one")
        );
        assert_eq!(
            SelectableList::key_target(&items, Some(&selected), "end")
                .map(|id| id.to_string())
                .as_deref(),
            Some("three")
        );
        assert_eq!(
            SelectableList::key_target(&items, Some(&selected), "pagedown")
                .map(|id| id.to_string())
                .as_deref(),
            Some("three")
        );
    }
}
