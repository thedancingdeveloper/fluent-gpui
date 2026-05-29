use std::sync::Arc;

use fluent_core::ThemeProvider as _;
use gpui::{
    anchored, deferred, div, prelude::*, px, App, ClickEvent, Corner, IntoElement, MouseDownEvent,
    Point, RenderOnce, SharedString, Window,
};

type SelectHandler = Arc<dyn Fn(SuggestionItem, &ClickEvent, &mut Window, &mut App) + 'static>;
type HoverHandler = Arc<dyn Fn(usize, &mut App) + 'static>;
type CloseHandler = Arc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SuggestionItem {
    pub id: SharedString,
    pub label: SharedString,
    pub detail: Option<SharedString>,
    pub meta: Option<SharedString>,
    pub disabled: bool,
}

impl SuggestionItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: None,
            meta: None,
            disabled: false,
        }
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SuggestionListState {
    highlighted: Option<usize>,
}

impl SuggestionListState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    pub fn set_highlighted(&mut self, highlighted: Option<usize>) {
        self.highlighted = highlighted;
    }

    pub fn highlighted_item<'a>(&self, items: &'a [SuggestionItem]) -> Option<&'a SuggestionItem> {
        self.highlighted
            .and_then(|index| items.get(index))
            .filter(|item| !item.disabled)
    }

    pub fn move_highlight(&mut self, items: &[SuggestionItem], delta: isize) -> Option<usize> {
        let enabled = enabled_indices(items);
        if enabled.is_empty() {
            self.highlighted = None;
            return None;
        }

        let current_pos = self
            .highlighted
            .and_then(|index| enabled.iter().position(|enabled| *enabled == index));
        let next_pos = if delta < 0 {
            current_pos
                .unwrap_or(enabled.len())
                .saturating_sub(delta.unsigned_abs())
        } else {
            current_pos
                .map(|pos| (pos + delta as usize).min(enabled.len() - 1))
                .unwrap_or(0)
        };

        self.highlighted = Some(enabled[next_pos]);
        self.highlighted
    }

    pub fn first(&mut self, items: &[SuggestionItem]) -> Option<usize> {
        self.highlighted = enabled_indices(items).first().copied();
        self.highlighted
    }

    pub fn last(&mut self, items: &[SuggestionItem]) -> Option<usize> {
        self.highlighted = enabled_indices(items).last().copied();
        self.highlighted
    }

    pub fn handle_key(&mut self, key: &str, items: &[SuggestionItem]) -> SuggestionKeyAction {
        match key {
            "down" => SuggestionKeyAction::Move(self.move_highlight(items, 1)),
            "up" => SuggestionKeyAction::Move(self.move_highlight(items, -1)),
            "home" => SuggestionKeyAction::Move(self.first(items)),
            "end" => SuggestionKeyAction::Move(self.last(items)),
            "escape" => SuggestionKeyAction::Close,
            "enter" | "return" | "tab" => self
                .highlighted_item(items)
                .map(|item| SuggestionKeyAction::Select(item.id.clone()))
                .unwrap_or(SuggestionKeyAction::None),
            _ => SuggestionKeyAction::None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SuggestionKeyAction {
    None,
    Move(Option<usize>),
    Select(SharedString),
    Close,
}

#[derive(IntoElement)]
pub struct SuggestionPopover {
    open: bool,
    exiting: bool,
    position: Point<gpui::Pixels>,
    anchor: Corner,
    items: Vec<SuggestionItem>,
    highlighted: Option<usize>,
    empty_text: SharedString,
    min_width: f32,
    max_height: f32,
    motion_key: u64,
    on_select: Option<SelectHandler>,
    on_hover: Option<HoverHandler>,
    on_close: Option<CloseHandler>,
}

impl SuggestionPopover {
    pub fn new(position: Point<gpui::Pixels>) -> Self {
        Self {
            open: true,
            exiting: false,
            position,
            anchor: Corner::TopLeft,
            items: Vec::new(),
            highlighted: None,
            empty_text: "No suggestions".into(),
            min_width: 240.0,
            max_height: 260.0,
            motion_key: 0,
            on_select: None,
            on_hover: None,
            on_close: None,
        }
    }

    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    pub fn exiting(mut self, exiting: bool) -> Self {
        self.exiting = exiting;
        self
    }

    pub fn anchor(mut self, anchor: Corner) -> Self {
        self.anchor = anchor;
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = SuggestionItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn item(mut self, item: SuggestionItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn highlighted(mut self, highlighted: impl Into<Option<usize>>) -> Self {
        self.highlighted = highlighted.into();
        self
    }

    pub fn empty_text(mut self, empty_text: impl Into<SharedString>) -> Self {
        self.empty_text = empty_text.into();
        self
    }

    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }

    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    pub fn motion_key(mut self, motion_key: u64) -> Self {
        self.motion_key = motion_key;
        self
    }

    pub fn on_select(
        mut self,
        on_select: impl Fn(SuggestionItem, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(on_select));
        self
    }

    pub fn on_hover(mut self, on_hover: impl Fn(usize, &mut App) + 'static) -> Self {
        self.on_hover = Some(Arc::new(on_hover));
        self
    }

    pub fn on_close(
        mut self,
        on_close: impl Fn(&MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_close = Some(Arc::new(on_close));
        self
    }
}

impl RenderOnce for SuggestionPopover {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        if !self.open {
            return div();
        }

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let radii = theme.radii;
        let typography = theme.typography;
        let components = theme.components;

        let list_id = if self.motion_key != 0 {
            gpui::ElementId::from(("suggestion-popover", self.motion_key))
        } else {
            gpui::ElementId::from("suggestion-popover")
        };
        let mut list = div()
            .id(list_id)
            .flex()
            .flex_col()
            .min_w(px(self.min_width))
            .max_h(px(self.max_height))
            .overflow_y_scroll()
            .bg(colors.surface)
            .border_1()
            .border_color(colors.stroke_neutral)
            .rounded(px(radii.md))
            .shadow_md()
            .py(px(spacing.xs));

        if let Some(on_close) = self.on_close.clone() {
            list = list.on_mouse_down_out(move |ev, window, cx| {
                // Do NOT call stop_propagation here — it would swallow clicks on sibling
                // elements (e.g. Discard / Bcc / Settings) while the popup is open.
                on_close(ev, window, cx);
            });
        }

        if self.items.is_empty() {
            let empty_py = ((components.dropdown_option_height - typography.body.line_height)
                .max(spacing.xs))
                / 2.0;
            list = list.child(
                div()
                    .flex_shrink_0()
                    .px(px(spacing.md))
                    .py(px(empty_py))
                    .flex()
                    .items_center()
                    .text_size(px(typography.body.size))
                    .text_color(colors.on_subtle_disabled)
                    .child(self.empty_text),
            );
        } else {
            for (index, item) in self.items.into_iter().enumerate() {
                let highlighted = self.highlighted == Some(index);
                let disabled = item.disabled;
                let row_bg = if highlighted {
                    colors.neutral_selected
                } else {
                    colors.surface
                };
                let text_color = if disabled {
                    colors.on_neutral_disabled
                } else {
                    colors.on_neutral
                };
                let detail = item.detail.clone();
                let meta = item.meta.clone();
                let select_item = item.clone();
                let on_hover = self.on_hover.clone();
                let on_select = self.on_select.clone();

                let item_id = self.motion_key * 100_000 + index as u64;
                // Use natural height with explicit padding; avoid justify_center + min_h
                // inside overflow_y_scroll because flex layout can squash two-line rows.
                let has_detail = item.detail.is_some();
                let row_py = if has_detail {
                    spacing.xs
                } else {
                    (components.dropdown_option_height - typography.body.line_height)
                        .max(spacing.xs)
                        / 2.0
                };
                let mut row = div()
                    .id(("suggestion-popover-item", item_id))
                    .flex()
                    .flex_col()
                    .flex_shrink_0()
                    .px(px(spacing.md))
                    .py(px(row_py))
                    .bg(row_bg);

                if !disabled {
                    row = row
                        .cursor_pointer()
                        .hover({
                            let hover = colors.neutral_hover;
                            move |style| style.bg(hover)
                        })
                        .on_mouse_move(move |_, _, cx| {
                            if let Some(on_hover) = &on_hover {
                                on_hover(index, cx);
                            }
                        })
                        .on_click(move |ev, window, cx| {
                            if let Some(on_select) = &on_select {
                                on_select(select_item.clone(), ev, window, cx);
                            }
                        });
                }

                let mut title_row = div().flex().items_center().gap(px(spacing.md)).child(
                    div()
                        .flex_1()
                        .text_size(px(typography.body.size))
                        .text_color(text_color)
                        .child(item.label),
                );
                if let Some(meta) = meta {
                    title_row = title_row.child(
                        div()
                            .text_size(px(typography.caption.size))
                            .text_color(colors.on_subtle)
                            .child(meta),
                    );
                }

                row = row.child(title_row);
                if let Some(detail) = detail {
                    row = row.child(
                        div()
                            .pt(px(1.0))
                            .text_size(px(typography.caption.size))
                            .text_color(colors.on_subtle)
                            .child(detail),
                    );
                }

                list = list.child(row);
            }
        }

        let motion = theme.motion();
        let motion_id = ("suggestion-popover-motion", self.motion_key);
        let list = fluent_core::popup_motion_surface(motion_id, self.exiting, motion, list);

        div().child(deferred(
            anchored()
                .anchor(self.anchor)
                .position(self.position)
                .snap_to_window()
                .child(list),
        ))
    }
}

fn enabled_indices(items: &[SuggestionItem]) -> Vec<usize> {
    items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| (!item.disabled).then_some(index))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn items() -> Vec<SuggestionItem> {
        vec![
            SuggestionItem::new("one", "One"),
            SuggestionItem::new("two", "Two").disabled(true),
            SuggestionItem::new("three", "Three"),
        ]
    }

    #[test]
    fn keyboard_roving_skips_disabled_items() {
        let items = items();
        let mut state = SuggestionListState::new();

        assert_eq!(
            state.handle_key("down", &items),
            SuggestionKeyAction::Move(Some(0))
        );
        assert_eq!(
            state.handle_key("down", &items),
            SuggestionKeyAction::Move(Some(2))
        );
        assert_eq!(
            state.handle_key("up", &items),
            SuggestionKeyAction::Move(Some(0))
        );
    }

    #[test]
    fn keyboard_home_end_and_select_use_enabled_items() {
        let items = items();
        let mut state = SuggestionListState::new();

        assert_eq!(
            state.handle_key("end", &items),
            SuggestionKeyAction::Move(Some(2))
        );
        assert_eq!(
            state.handle_key("enter", &items),
            SuggestionKeyAction::Select("three".into())
        );
        assert_eq!(
            state.handle_key("home", &items),
            SuggestionKeyAction::Move(Some(0))
        );
        assert_eq!(
            state.handle_key("tab", &items),
            SuggestionKeyAction::Select("one".into())
        );
    }

    #[test]
    fn keyboard_escape_closes() {
        let mut state = SuggestionListState::new();

        assert_eq!(
            state.handle_key("escape", &items()),
            SuggestionKeyAction::Close
        );
    }
}
