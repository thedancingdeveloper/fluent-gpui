use fluent_core::TextRampExt;
use std::sync::Arc;

use fluent_core::ThemeProvider as _;
use gpui::{
    anchored, deferred, div, prelude::*, px, Anchor, App, ClickEvent, IntoElement, MouseDownEvent,
    Point, RenderOnce, SharedString, Window,
};

type SelectHandler = Arc<dyn Fn(SuggestionItem, &ClickEvent, &mut Window, &mut App) + 'static>;
type HoverHandler = Arc<dyn Fn(usize, &mut App) + 'static>;
type CloseHandler = Arc<dyn Fn(&MouseDownEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone, Debug, PartialEq, Eq)]
/// A Fluent primitive for representing suggestion item in an interface.
pub struct SuggestionItem {
    /// The id value.
    pub id: SharedString,
    /// The label value.
    pub label: SharedString,
    /// The detail value.
    pub detail: Option<SharedString>,
    /// The meta value.
    pub meta: Option<SharedString>,
    /// The disabled value.
    pub disabled: bool,
}

impl SuggestionItem {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            detail: None,
            meta: None,
            disabled: false,
        }
    }

    /// Sets the detail configuration for this control.
    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Sets the meta configuration for this control.
    pub fn meta(mut self, meta: impl Into<SharedString>) -> Self {
        self.meta = Some(meta.into());
        self
    }

    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// A Fluent primitive for representing suggestion list state in an interface.
pub struct SuggestionListState {
    highlighted: Option<usize>,
}

impl SuggestionListState {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the highlighted configuration for this control.
    pub fn highlighted(&self) -> Option<usize> {
        self.highlighted
    }

    /// Sets the highlighted value.
    pub fn set_highlighted(&mut self, highlighted: Option<usize>) {
        self.highlighted = highlighted;
    }

    /// Returns the highlighted item value.
    pub fn highlighted_item<'a>(&self, items: &'a [SuggestionItem]) -> Option<&'a SuggestionItem> {
        self.highlighted
            .and_then(|index| items.get(index))
            .filter(|item| !item.disabled)
    }

    /// Moves highlight.
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

    /// Moves the highlight to the first enabled suggestion.
    pub fn first(&mut self, items: &[SuggestionItem]) -> Option<usize> {
        self.highlighted = enabled_indices(items).first().copied();
        self.highlighted
    }

    /// Moves the highlight to the last enabled suggestion.
    pub fn last(&mut self, items: &[SuggestionItem]) -> Option<usize> {
        self.highlighted = enabled_indices(items).last().copied();
        self.highlighted
    }

    /// Updates suggestion state for a navigation key and returns the resulting action.
    pub fn handle_key(&mut self, key: &str, items: &[SuggestionItem]) -> SuggestionKeyAction {
        match key {
            "down" => SuggestionKeyAction::Move(self.move_highlight(items, 1)),
            "up" => SuggestionKeyAction::Move(self.move_highlight(items, -1)),
            "home" => SuggestionKeyAction::Move(self.first(items)),
            "end" => SuggestionKeyAction::Move(self.last(items)),
            "pageup" => SuggestionKeyAction::Move(self.move_highlight(items, -5)),
            "pagedown" => SuggestionKeyAction::Move(self.move_highlight(items, 5)),
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
/// Available suggestion key action options for configuring the control.
pub enum SuggestionKeyAction {
    /// The none option.
    None,
    /// The move option.
    Move(Option<usize>),
    /// The select option.
    Select(SharedString),
    /// The close option.
    Close,
}

#[derive(IntoElement)]
/// A Fluent primitive for representing suggestion popover in an interface.
pub struct SuggestionPopover {
    open: bool,
    exiting: bool,
    position: Point<gpui::Pixels>,
    anchor: Anchor,
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
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(position: Point<gpui::Pixels>) -> Self {
        Self {
            open: true,
            exiting: false,
            position,
            anchor: Anchor::TopLeft,
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

    /// Sets the open configuration for this control.
    pub fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    /// Sets the exiting configuration for this control.
    pub fn exiting(mut self, exiting: bool) -> Self {
        self.exiting = exiting;
        self
    }

    /// Sets the anchor configuration for this control.
    pub fn anchor(mut self, anchor: Anchor) -> Self {
        self.anchor = anchor;
        self
    }

    /// Sets the items configuration for this control.
    pub fn items(mut self, items: impl IntoIterator<Item = SuggestionItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Sets the item configuration for this control.
    pub fn item(mut self, item: SuggestionItem) -> Self {
        self.items.push(item);
        self
    }

    /// Sets the highlighted configuration for this control.
    pub fn highlighted(mut self, highlighted: impl Into<Option<usize>>) -> Self {
        self.highlighted = highlighted.into();
        self
    }

    /// Sets the empty text configuration for this control.
    pub fn empty_text(mut self, empty_text: impl Into<SharedString>) -> Self {
        self.empty_text = empty_text.into();
        self
    }

    /// Sets the min width configuration for this control.
    pub fn min_width(mut self, min_width: f32) -> Self {
        self.min_width = min_width;
        self
    }

    /// Sets the max height configuration for this control.
    pub fn max_height(mut self, max_height: f32) -> Self {
        self.max_height = max_height;
        self
    }

    /// Returns whether the key moves focus through the suggestion list.
    pub fn motion_key(mut self, motion_key: u64) -> Self {
        self.motion_key = motion_key;
        self
    }

    /// Registers a handler for select events.
    pub fn on_select(
        mut self,
        on_select: impl Fn(SuggestionItem, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(on_select));
        self
    }

    /// Registers a handler for hover events.
    pub fn on_hover(mut self, on_hover: impl Fn(usize, &mut App) + 'static) -> Self {
        self.on_hover = Some(Arc::new(on_hover));
        self
    }

    /// Registers a handler for close events.
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
                    .text_ramp(&typography.body)
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
                        .text_ramp(&typography.body)
                        .text_color(text_color)
                        .child(item.label),
                );
                if let Some(meta) = meta {
                    title_row = title_row.child(
                        div()
                            .text_ramp(&typography.caption)
                            .text_color(colors.on_subtle)
                            .child(meta),
                    );
                }

                row = row.child(title_row);
                if let Some(detail) = detail {
                    row = row.child(
                        div()
                            .pt(px(1.0))
                            .text_ramp(&typography.caption)
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
                .snap_to_window_with_margin(px(8.))
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
