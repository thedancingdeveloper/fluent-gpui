use std::sync::{Arc, Mutex};

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    canvas, div, point, prelude::*, px, App, Bounds, Context, CursorStyle, FocusHandle, Focusable,
    IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, Pixels, Point, Render, SharedString,
    Window,
};

use crate::{Chip, SuggestionItem, SuggestionPopover};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenInputToken {
    pub value: SharedString,
    pub label: SharedString,
    pub valid: bool,
}

impl TokenInputToken {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            valid: true,
        }
    }

    pub fn valid(mut self, valid: bool) -> Self {
        self.valid = valid;
        self
    }

    pub fn invalid(mut self) -> Self {
        self.valid = false;
        self
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TokenInputSuggestion {
    pub value: SharedString,
    pub label: SharedString,
    pub detail: Option<SharedString>,
}

impl TokenInputSuggestion {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            detail: None,
        }
    }

    pub fn detail(mut self, detail: impl Into<SharedString>) -> Self {
        self.detail = Some(detail.into());
        self
    }
}

#[allow(clippy::type_complexity)]
pub struct TokenInput {
    tokens: Vec<TokenInputToken>,
    suggestions: Vec<TokenInputSuggestion>,
    query: String,
    placeholder: SharedString,
    disabled: bool,
    error: bool,
    allow_freeform: bool,
    open: bool,
    highlighted: usize,
    focus_handle: Option<FocusHandle>,
    drop_origin: Point<Pixels>,
    trigger_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    observing_theme: bool,
    subscribed_focus_out: bool,
    on_change: Option<Box<dyn Fn(Vec<TokenInputToken>, &mut App) + 'static>>,
    on_query_change: Option<Box<dyn Fn(SharedString, &mut App) + 'static>>,
}

impl Default for TokenInput {
    fn default() -> Self {
        Self {
            tokens: Vec::new(),
            suggestions: Vec::new(),
            query: String::new(),
            placeholder: "Add item".into(),
            disabled: false,
            error: false,
            allow_freeform: true,
            open: false,
            highlighted: 0,
            focus_handle: None,
            drop_origin: point(px(0.0), px(0.0)),
            trigger_bounds: Arc::new(Mutex::new(None)),
            observing_theme: false,
            subscribed_focus_out: false,
            on_change: None,
            on_query_change: None,
        }
    }
}

impl TokenInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tokens(mut self, tokens: impl IntoIterator<Item = TokenInputToken>) -> Self {
        self.tokens = tokens.into_iter().collect();
        self
    }

    pub fn token(mut self, value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        self.tokens.push(TokenInputToken::new(value, label));
        self
    }

    pub fn suggestions(
        mut self,
        suggestions: impl IntoIterator<Item = TokenInputSuggestion>,
    ) -> Self {
        self.suggestions = suggestions.into_iter().collect();
        self
    }

    pub fn suggestion(
        mut self,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.suggestions
            .push(TokenInputSuggestion::new(value, label));
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn allow_freeform(mut self, allow_freeform: bool) -> Self {
        self.allow_freeform = allow_freeform;
        self
    }

    pub fn on_change(mut self, f: impl Fn(Vec<TokenInputToken>, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn on_query_change(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_query_change = Some(Box::new(f));
        self
    }

    pub fn values(&self) -> &[TokenInputToken] {
        &self.tokens
    }

    pub fn query(&self) -> &str {
        &self.query
    }

    pub fn set_tokens(
        &mut self,
        tokens: impl IntoIterator<Item = TokenInputToken>,
        cx: &mut Context<Self>,
    ) {
        self.tokens = tokens.into_iter().collect();
        self.emit_change(cx);
        cx.notify();
    }

    /// Set tokens without firing the `on_change` callback. Use this for
    /// programmatic resets (e.g. sync from external state) to avoid
    /// re-entrant state updates.
    pub fn set_tokens_silent(
        &mut self,
        tokens: impl IntoIterator<Item = TokenInputToken>,
        cx: &mut Context<Self>,
    ) {
        self.set_tokens_silent_inner(tokens);
        cx.notify();
    }

    fn set_tokens_silent_inner(&mut self, tokens: impl IntoIterator<Item = TokenInputToken>) {
        self.tokens = tokens.into_iter().collect();
        self.query.clear();
        self.highlighted = 0;
        self.open = false;
    }

    pub fn set_suggestions(
        &mut self,
        suggestions: impl IntoIterator<Item = TokenInputSuggestion>,
        cx: &mut Context<Self>,
    ) {
        self.suggestions = suggestions.into_iter().collect();
        self.highlighted = self
            .highlighted
            .min(self.filtered_suggestions().len().saturating_sub(1));
        cx.notify();
    }

    pub fn set_query(&mut self, query: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.query = query.into().to_string();
        self.highlighted = 0;
        self.open = !self.filtered_suggestions().is_empty();
        self.emit_query_change(cx);
        cx.notify();
    }

    pub fn remove_token(&mut self, index: usize, cx: &mut Context<Self>) {
        if index >= self.tokens.len() {
            return;
        }
        self.tokens.remove(index);
        self.emit_change(cx);
        cx.notify();
    }

    fn focus_handle(&mut self, cx: &mut Context<Self>) -> FocusHandle {
        self.focus_handle
            .get_or_insert_with(|| cx.focus_handle().tab_stop(true))
            .clone()
    }

    fn emit_change(&self, cx: &mut Context<Self>) {
        if let Some(f) = &self.on_change {
            f(self.tokens.clone(), cx);
        }
    }

    fn emit_query_change(&self, cx: &mut Context<Self>) {
        if let Some(f) = &self.on_query_change {
            f(SharedString::from(self.query.clone()), cx);
        }
    }

    fn update_query(&mut self, query: String, cx: &mut Context<Self>) {
        self.query = query;
        self.highlighted = 0;
        self.open = !self.filtered_suggestions().is_empty();
        self.emit_query_change(cx);
        cx.notify();
    }

    fn push_token(&mut self, token: TokenInputToken, cx: &mut Context<Self>) {
        if self
            .tokens
            .iter()
            .any(|existing| existing.value == token.value)
        {
            self.query.clear();
            self.open = false;
            self.highlighted = 0;
            self.emit_query_change(cx);
            cx.notify();
            return;
        }

        self.tokens.push(token);
        self.query.clear();
        self.open = false;
        self.highlighted = 0;
        self.emit_query_change(cx);
        self.emit_change(cx);
        cx.notify();
    }

    fn select_suggestion(&mut self, suggestion: TokenInputSuggestion, cx: &mut Context<Self>) {
        self.push_token(TokenInputToken::new(suggestion.value, suggestion.label), cx);
    }

    fn commit_query(&mut self, cx: &mut Context<Self>) -> bool {
        let filtered = self.filtered_suggestions();
        if let Some(suggestion) = filtered.get(self.highlighted).cloned() {
            self.select_suggestion(suggestion, cx);
            return true;
        }

        let query = self.query.trim();
        if self.allow_freeform && !query.is_empty() {
            self.push_token(
                TokenInputToken::new(query.to_string(), query.to_string()),
                cx,
            );
            return true;
        }

        false
    }

    fn filtered_suggestions(&self) -> Vec<TokenInputSuggestion> {
        filter_suggestions(&self.suggestions, &self.tokens, &self.query)
    }

    fn update_origin(&mut self, fallback: Point<Pixels>) {
        self.drop_origin = self
            .trigger_bounds
            .lock()
            .ok()
            .and_then(|slot| *slot)
            .map(|bounds| point(bounds.origin.x, bounds.origin.y + bounds.size.height))
            .unwrap_or(fallback);
    }

    fn open_popup(&mut self, cx: &mut Context<Self>) {
        self.open = !self.filtered_suggestions().is_empty();
        cx.notify();
    }

    fn close_popup(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        self.highlighted = 0;
        cx.notify();
    }

    fn handle_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }

        if ev.keystroke.key.as_str() == "tab"
            && self.open
            && !self.filtered_suggestions().is_empty()
        {
            self.commit_query(cx);
            return;
        }

        if crate::focus::handle_tab_navigation(ev, window) {
            cx.notify();
            return;
        }

        let modifiers = &ev.keystroke.modifiers;
        if modifiers.control || modifiers.platform {
            return;
        }

        match ev.keystroke.key.as_str() {
            "backspace" => {
                if self.query.is_empty() {
                    if !self.tokens.is_empty() {
                        self.tokens.pop();
                        self.emit_change(cx);
                    }
                    cx.notify();
                } else {
                    let mut query = self.query.clone();
                    query.pop();
                    self.update_query(query, cx);
                }
            }
            "escape" => self.close_popup(cx),
            "down" => {
                let len = self.filtered_suggestions().len();
                if len > 0 {
                    self.open = true;
                    self.highlighted = (self.highlighted + 1).min(len - 1);
                    cx.notify();
                }
            }
            "up" => {
                if !self.filtered_suggestions().is_empty() {
                    self.open = true;
                    self.highlighted = self.highlighted.saturating_sub(1);
                    cx.notify();
                }
            }
            "enter" | "return" => {
                self.commit_query(cx);
            }
            _ => {
                let printable = ev
                    .keystroke
                    .key_char
                    .as_ref()
                    .map(ToString::to_string)
                    .unwrap_or_else(|| ev.keystroke.key.to_string());
                if printable.chars().count() == 1 {
                    if printable == "," {
                        self.commit_query(cx);
                    } else if !printable.chars().any(char::is_control) {
                        let mut query = self.query.clone();
                        query.push_str(&printable);
                        self.update_query(query, cx);
                    }
                }
            }
        }
    }

    fn on_mouse_down(&mut self, ev: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let handle = self.focus_handle(cx);
        window.focus(&handle);
        self.update_origin(point(
            ev.position.x,
            ev.position.y + px(cx.theme().components.dropdown_height),
        ));
        self.open_popup(cx);
    }
}

impl Render for TokenInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing_theme {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            self.observing_theme = true;
        }

        let focus_handle = self.focus_handle(cx);

        // Close the suggestion popup when focus leaves this input, so it can
        // never block clicks on sibling elements (e.g. Discard / Bcc buttons).
        if !self.subscribed_focus_out {
            cx.on_focus_out(&focus_handle, window, |input: &mut TokenInput, _, _, cx| {
                input.close_popup(cx);
            })
            .detach();
            self.subscribed_focus_out = true;
        }
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let components = theme.components;
        let focused = focus_handle.is_focused(window);
        let invalid = self.error || self.tokens.iter().any(|token| !token.valid);

        let trigger_bounds_arc = Arc::clone(&self.trigger_bounds);
        let measure_trigger = canvas(
            move |bounds, _, _| {
                if let Ok(mut slot) = trigger_bounds_arc.lock() {
                    *slot = Some(bounds);
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0();

        let entity = cx.entity();
        let mut input_row = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .items_center()
            .gap(px(spacing.xs))
            .flex_1();

        for (index, token) in self.tokens.iter().enumerate() {
            let entity = entity.clone();
            input_row = input_row.child(
                Chip::new(("token-input-token", index), token.label.clone()).on_dismiss(
                    move |_, _, cx| {
                        entity.update(cx, |input, cx| {
                            input.remove_token(index, cx);
                        });
                    },
                ),
            );
        }

        let query_text = if self.query.is_empty() && self.tokens.is_empty() {
            self.placeholder.clone()
        } else if self.query.is_empty() {
            SharedString::from("")
        } else {
            SharedString::from(format!("{}_", self.query))
        };
        let query_color = if self.query.is_empty() {
            colors.on_subtle_disabled
        } else {
            colors.on_neutral
        };

        input_row = input_row.child(
            div()
                .min_w(px(120.0))
                .flex_1()
                .text_size(px(typography.body.size))
                .text_color(query_color)
                .child(query_text),
        );

        let mut root = div()
            .id("token-input")
            .key_context("TokenInput")
            .track_focus(&focus_handle)
            .on_key_down(
                cx.listener(|input: &mut TokenInput, ev: &KeyDownEvent, window, cx| {
                    input.handle_key(ev, window, cx);
                }),
            )
            .on_action(crate::focus::focus_next)
            .on_action(crate::focus::focus_previous)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .relative()
            .flex()
            .items_center()
            .w_full()
            .min_h(px(components.dropdown_height))
            .px(px(spacing.sm))
            .py(px(spacing.xs))
            .rounded(px(radii.md))
            .bg(if self.disabled {
                colors.neutral_disabled
            } else {
                colors.neutral
            })
            .border_1()
            .border_color(if invalid {
                colors.status_error
            } else if focused {
                colors.accent
            } else {
                colors.stroke_neutral
            })
            .cursor(if self.disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .child(input_row)
            .child(measure_trigger);

        if self.open && !self.disabled {
            let suggestions = self.filtered_suggestions();
            if self.highlighted >= suggestions.len().max(1) {
                self.highlighted = 0;
            }

            let popup_min_width = self
                .trigger_bounds
                .lock()
                .ok()
                .and_then(|slot| *slot)
                .map(|bounds| f32::from(bounds.size.width).max(components.dropdown_min_width))
                .unwrap_or(components.dropdown_min_width);

            let items = suggestions.into_iter().map(|suggestion| {
                let mut item = SuggestionItem::new(suggestion.value, suggestion.label);
                if let Some(detail) = suggestion.detail {
                    item = item.detail(detail);
                }
                item
            });

            // Use the entity ID as the motion key so each TokenInput's popover
            // gets unique GPUI element IDs, preventing rendering conflicts when
            // multiple inputs exist in the same view.
            let popover_key = cx.entity().entity_id().as_u64();
            let hover_entity = cx.entity();
            let select_entity = cx.entity();
            let close_entity = cx.entity();
            root = root.child(
                SuggestionPopover::new(self.drop_origin)
                    .items(items)
                    .highlighted(self.highlighted)
                    .min_width(popup_min_width)
                    .max_height(components.dropdown_max_height)
                    .empty_text("No matching suggestions")
                    .motion_key(popover_key)
                    .on_hover(move |index, app_cx| {
                        hover_entity.update(app_cx, |input, cx| {
                            if input.highlighted != index {
                                input.highlighted = index;
                                cx.notify();
                            }
                        });
                    })
                    .on_select(move |item, _, _, app_cx| {
                        select_entity.update(app_cx, |input, cx| {
                            input.select_suggestion(
                                TokenInputSuggestion {
                                    value: item.id,
                                    label: item.label,
                                    detail: item.detail,
                                },
                                cx,
                            );
                        });
                    })
                    .on_close(move |_, _, app_cx| {
                        close_entity.update(app_cx, |input, cx| {
                            input.close_popup(cx);
                        });
                    }),
            );
        }

        root
    }
}

impl Focusable for TokenInput {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle().tab_stop(true))
    }
}

fn filter_suggestions(
    suggestions: &[TokenInputSuggestion],
    tokens: &[TokenInputToken],
    query: &str,
) -> Vec<TokenInputSuggestion> {
    let query = query.trim().to_lowercase();
    suggestions
        .iter()
        .filter(|suggestion| !tokens.iter().any(|token| token.value == suggestion.value))
        .filter(|suggestion| {
            if query.is_empty() {
                return true;
            }
            suggestion.label.to_lowercase().contains(&query)
                || suggestion.value.to_lowercase().contains(&query)
                || suggestion
                    .detail
                    .as_ref()
                    .is_some_and(|detail| detail.to_lowercase().contains(&query))
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filters_by_label_value_and_detail() {
        let suggestions = vec![
            TokenInputSuggestion::new("ops@example.com", "Ops"),
            TokenInputSuggestion::new("sales@example.com", "Sales").detail("Revenue team"),
        ];

        assert_eq!(
            filter_suggestions(&suggestions, &[], "ops")
                .into_iter()
                .map(|suggestion| suggestion.value.to_string())
                .collect::<Vec<_>>(),
            vec!["ops@example.com"]
        );
        assert_eq!(
            filter_suggestions(&suggestions, &[], "revenue")
                .into_iter()
                .map(|suggestion| suggestion.value.to_string())
                .collect::<Vec<_>>(),
            vec!["sales@example.com"]
        );
        assert_eq!(
            filter_suggestions(&suggestions, &[], "example")
                .into_iter()
                .map(|suggestion| suggestion.value.to_string())
                .collect::<Vec<_>>(),
            vec!["ops@example.com", "sales@example.com"]
        );
    }

    #[test]
    fn selected_tokens_are_removed_from_suggestions() {
        let suggestions = vec![
            TokenInputSuggestion::new("ops@example.com", "Ops"),
            TokenInputSuggestion::new("sales@example.com", "Sales"),
        ];
        let tokens = vec![TokenInputToken::new("ops@example.com", "Ops")];

        assert_eq!(
            filter_suggestions(&suggestions, &tokens, "")
                .into_iter()
                .map(|suggestion| suggestion.value.to_string())
                .collect::<Vec<_>>(),
            vec!["sales@example.com"]
        );
    }

    #[test]
    fn token_validity_builder_sets_invalid_state() {
        let token = TokenInputToken::new("bad", "bad").invalid();
        assert!(!token.valid);
        assert!(TokenInputToken::new("ok", "ok").valid(true).valid);
    }

    #[test]
    fn silent_token_reset_clears_transient_query_state() {
        let mut input = TokenInput::new()
            .suggestion("ops@example.com", "Ops")
            .suggestion("sales@example.com", "Sales");
        input.query = "ops".to_string();
        input.highlighted = 1;
        input.open = true;

        input.set_tokens_silent_inner(vec![TokenInputToken::new("a@example.com", "A")]);

        assert_eq!(input.values()[0].value.as_ref(), "a@example.com");
        assert_eq!(input.query(), "");
        assert_eq!(input.highlighted, 0);
        assert!(!input.open);
    }
}
