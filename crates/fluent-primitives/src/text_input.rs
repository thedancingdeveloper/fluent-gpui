use std::ops::Range;

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, fill, point, prelude::*, px, relative, rgba, size, App, Bounds, ClipboardItem, Context,
    CursorStyle, Element, ElementId, ElementInputHandler, Entity, EntityInputHandler, FocusHandle,
    Focusable, GlobalElementId, IntoElement, KeyDownEvent, LayoutId, MouseButton, MouseDownEvent,
    MouseMoveEvent, MouseUpEvent, PaintQuad, Pixels, Render, ShapedLine, SharedString, Style,
    TextRun, UTF16Selection, UnderlineStyle, Window,
};
use unicode_segmentation::UnicodeSegmentation as _;

/// A single-line editable text input with focus, selection, clipboard, and IME support.
#[allow(clippy::type_complexity)]
pub struct TextInput {
    value: String,
    placeholder: SharedString,
    disabled: bool,
    error: bool,
    /// When true, each character is displayed as `•` (password masking). The real
    /// value is still stored and returned by `text()`; only the visual is hidden.
    masked: bool,
    focus_handle: Option<FocusHandle>,
    selected_range: Range<usize>,
    selection_reversed: bool,
    marked_range: Option<Range<usize>>,
    last_layout: Option<ShapedLine>,
    last_bounds: Option<Bounds<Pixels>>,
    is_selecting: bool,
    observing_theme: bool,
    on_change: Option<Box<dyn Fn(SharedString, &mut App) + 'static>>,
    on_submit: Option<Box<dyn Fn(SharedString, &mut App) -> bool + 'static>>,
}

impl Default for TextInput {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: SharedString::default(),
            disabled: false,
            error: false,
            masked: false,
            focus_handle: None,
            selected_range: 0..0,
            selection_reversed: false,
            marked_range: None,
            last_layout: None,
            last_bounds: None,
            is_selecting: false,
            observing_theme: false,
            on_change: None,
            on_submit: None,
        }
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.set_value_inner(value.into());
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

    pub fn masked(mut self, masked: bool) -> Self {
        self.masked = masked;
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn on_change(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Register a submit callback fired by Enter/Return. Return `true` to clear
    /// the input after the callback runs.
    pub fn on_submit(mut self, f: impl Fn(SharedString, &mut App) -> bool + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
        self
    }

    pub fn set_on_submit(&mut self, f: impl Fn(SharedString, &mut App) -> bool + 'static) {
        self.on_submit = Some(Box::new(f));
    }

    pub fn text(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.set_value_inner(value.into());
        cx.notify();
    }

    pub fn set_placeholder(&mut self, placeholder: impl Into<SharedString>) {
        self.placeholder = placeholder.into();
    }

    pub fn is_masked(&self) -> bool {
        self.masked
    }

    pub fn set_masked(&mut self, masked: bool, cx: &mut Context<Self>) {
        self.masked = masked;
        cx.notify();
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        cx.notify();
    }

    pub fn set_error(&mut self, error: bool, cx: &mut Context<Self>) {
        self.error = error;
        cx.notify();
    }

    fn set_value_inner(&mut self, value: SharedString) {
        self.value = value.to_string();
        self.selected_range = self.value.len()..self.value.len();
        self.selection_reversed = false;
        self.marked_range = None;
    }

    fn focus_handle(&mut self, cx: &mut Context<Self>) -> FocusHandle {
        self.focus_handle
            .get_or_insert_with(|| cx.focus_handle().tab_stop(true))
            .clone()
    }

    fn emit_change(&self, cx: &mut Context<Self>) {
        if let Some(f) = &self.on_change {
            f(SharedString::from(self.value.clone()), cx);
        }
    }

    fn cursor_offset(&self) -> usize {
        if self.selection_reversed {
            self.selected_range.start
        } else {
            self.selected_range.end
        }
    }

    fn move_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = offset.min(self.value.len());
        self.selected_range = offset..offset;
        self.selection_reversed = false;
        cx.notify();
    }

    fn select_to(&mut self, offset: usize, cx: &mut Context<Self>) {
        let offset = offset.min(self.value.len());
        if self.selection_reversed {
            self.selected_range.start = offset;
        } else {
            self.selected_range.end = offset;
        }
        if self.selected_range.end < self.selected_range.start {
            self.selection_reversed = !self.selection_reversed;
            self.selected_range = self.selected_range.end..self.selected_range.start;
        }
        cx.notify();
    }

    fn previous_boundary(&self, offset: usize) -> usize {
        self.value
            .grapheme_indices(true)
            .rev()
            .find_map(|(idx, _)| (idx < offset).then_some(idx))
            .unwrap_or(0)
    }

    fn next_boundary(&self, offset: usize) -> usize {
        self.value
            .grapheme_indices(true)
            .find_map(|(idx, _)| (idx > offset).then_some(idx))
            .unwrap_or(self.value.len())
    }

    fn index_for_mouse_position(&self, position: gpui::Point<Pixels>) -> usize {
        if self.value.is_empty() {
            return 0;
        }

        let (Some(bounds), Some(line)) = (self.last_bounds.as_ref(), self.last_layout.as_ref())
        else {
            return self.value.len();
        };

        if position.y < bounds.top() {
            return 0;
        }
        if position.y > bounds.bottom() {
            return self.value.len();
        }
        let idx = line.closest_index_for_x(position.x - bounds.left());
        if self.masked {
            masked_to_real_offset(&self.value, idx)
        } else {
            idx
        }
    }

    fn offset_from_utf16(&self, offset: usize) -> usize {
        let mut utf8_offset = 0;
        let mut utf16_count = 0;

        for ch in self.value.chars() {
            if utf16_count >= offset {
                break;
            }
            utf16_count += ch.len_utf16();
            utf8_offset += ch.len_utf8();
        }

        utf8_offset
    }

    fn offset_to_utf16(&self, offset: usize) -> usize {
        let mut utf16_offset = 0;
        let mut utf8_count = 0;

        for ch in self.value.chars() {
            if utf8_count >= offset {
                break;
            }
            utf8_count += ch.len_utf8();
            utf16_offset += ch.len_utf16();
        }

        utf16_offset
    }

    fn range_to_utf16(&self, range: &Range<usize>) -> Range<usize> {
        self.offset_to_utf16(range.start)..self.offset_to_utf16(range.end)
    }

    fn range_from_utf16(&self, range_utf16: &Range<usize>) -> Range<usize> {
        self.offset_from_utf16(range_utf16.start)..self.offset_from_utf16(range_utf16.end)
    }

    fn replace_selected_text(&mut self, text: &str, cx: &mut Context<Self>) {
        let range = self
            .marked_range
            .clone()
            .unwrap_or_else(|| self.selected_range.clone());
        self.value =
            self.value[0..range.start].to_owned() + text + &self.value[range.end..self.value.len()];
        let cursor = range.start + text.len();
        self.selected_range = cursor..cursor;
        self.selection_reversed = false;
        self.marked_range = None;
        self.emit_change(cx);
        cx.notify();
    }

    fn handle_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if crate::focus::handle_tab_navigation(ev, window, cx) {
            cx.notify();
            return;
        }

        if self.disabled {
            return;
        }

        let modifiers = &ev.keystroke.modifiers;
        let command = modifiers.control || modifiers.platform;

        if command {
            match ev.keystroke.key.as_str() {
                "a" => {
                    self.move_to(0, cx);
                    self.select_to(self.value.len(), cx);
                }
                "c" if !self.selected_range.is_empty() => {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        self.value[self.selected_range.clone()].to_string(),
                    ));
                }
                "x" if !self.selected_range.is_empty() => {
                    cx.write_to_clipboard(ClipboardItem::new_string(
                        self.value[self.selected_range.clone()].to_string(),
                    ));
                    self.replace_selected_text("", cx);
                }
                "v" => {
                    if let Some(text) = cx.read_from_clipboard().and_then(|item| item.text()) {
                        self.replace_selected_text(&text.replace('\n', " "), cx);
                    }
                }
                _ => {}
            }
            return;
        }

        match ev.keystroke.key.as_str() {
            "backspace" => {
                if self.selected_range.is_empty() {
                    self.select_to(self.previous_boundary(self.cursor_offset()), cx);
                }
                self.replace_selected_text("", cx);
            }
            "delete" => {
                if self.selected_range.is_empty() {
                    self.select_to(self.next_boundary(self.cursor_offset()), cx);
                }
                self.replace_selected_text("", cx);
            }
            "left" => {
                if modifiers.shift {
                    self.select_to(self.previous_boundary(self.cursor_offset()), cx);
                } else if self.selected_range.is_empty() {
                    self.move_to(self.previous_boundary(self.cursor_offset()), cx);
                } else {
                    self.move_to(self.selected_range.start, cx);
                }
            }
            "right" => {
                if modifiers.shift {
                    self.select_to(self.next_boundary(self.cursor_offset()), cx);
                } else if self.selected_range.is_empty() {
                    self.move_to(self.next_boundary(self.selected_range.end), cx);
                } else {
                    self.move_to(self.selected_range.end, cx);
                }
            }
            "home" => {
                if modifiers.shift {
                    self.select_to(0, cx);
                } else {
                    self.move_to(0, cx);
                }
            }
            "end" => {
                if modifiers.shift {
                    self.select_to(self.value.len(), cx);
                } else {
                    self.move_to(self.value.len(), cx);
                }
            }
            "enter" | "return" => {
                if let Some(on_submit) = &self.on_submit {
                    let submitted = on_submit(SharedString::from(self.value.clone()), cx);
                    if submitted {
                        self.set_value_inner(SharedString::default());
                        cx.notify();
                    }
                }
            }
            _ => {
                let _ = window;
            }
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let focus_handle = self.focus_handle(cx);
        window.focus(&focus_handle, cx);
        self.is_selecting = true;

        if event.modifiers.shift {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        } else {
            self.move_to(self.index_for_mouse_position(event.position), cx);
        }
    }

    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }

    fn on_mouse_move(&mut self, event: &MouseMoveEvent, _: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting && !self.disabled {
            self.select_to(self.index_for_mouse_position(event.position), cx);
        }
    }
}

const MASK_CHAR: char = '•';
const MASK_CHAR_BYTES: usize = '•'.len_utf8();

fn real_to_masked_offset(value: &str, real_offset: usize) -> usize {
    value[..real_offset].chars().count() * MASK_CHAR_BYTES
}

fn masked_to_real_offset(value: &str, masked_offset: usize) -> usize {
    let char_count = masked_offset / MASK_CHAR_BYTES;
    value
        .char_indices()
        .nth(char_count)
        .map(|(i, _)| i)
        .unwrap_or(value.len())
}

impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        range_utf16: Range<usize>,
        actual_range: &mut Option<Range<usize>>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<String> {
        let range = self.range_from_utf16(&range_utf16);
        actual_range.replace(self.range_to_utf16(&range));
        Some(self.value[range].to_string())
    }

    fn selected_text_range(
        &mut self,
        ignore_disabled_input: bool,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        if self.disabled && !ignore_disabled_input {
            return None;
        }
        Some(UTF16Selection {
            range: self.range_to_utf16(&self.selected_range),
            reversed: self.selection_reversed,
        })
    }

    fn marked_text_range(
        &self,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Range<usize>> {
        self.marked_range
            .as_ref()
            .map(|range| self.range_to_utf16(range))
    }

    fn unmark_text(&mut self, _window: &mut Window, _cx: &mut Context<Self>) {
        self.marked_range = None;
    }

    fn replace_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());

        self.value = self.value[0..range.start].to_owned() + new_text + &self.value[range.end..];
        let cursor = range.start + new_text.len();
        self.selected_range = cursor..cursor;
        self.selection_reversed = false;
        self.marked_range = None;
        self.emit_change(cx);
        cx.notify();
    }

    fn replace_and_mark_text_in_range(
        &mut self,
        range_utf16: Option<Range<usize>>,
        new_text: &str,
        new_selected_range_utf16: Option<Range<usize>>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.disabled {
            return;
        }
        let range = range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .or(self.marked_range.clone())
            .unwrap_or_else(|| self.selected_range.clone());

        self.value = self.value[0..range.start].to_owned() + new_text + &self.value[range.end..];
        self.marked_range =
            (!new_text.is_empty()).then_some(range.start..range.start + new_text.len());
        self.selected_range = new_selected_range_utf16
            .as_ref()
            .map(|range_utf16| self.range_from_utf16(range_utf16))
            .map(|new_range| new_range.start + range.start..new_range.end + range.start)
            .unwrap_or_else(|| range.start + new_text.len()..range.start + new_text.len());
        self.emit_change(cx);
        cx.notify();
    }

    fn bounds_for_range(
        &mut self,
        range_utf16: Range<usize>,
        bounds: Bounds<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let last_layout = self.last_layout.as_ref()?;
        let range = self.range_from_utf16(&range_utf16);
        Some(Bounds::from_corners(
            point(
                bounds.left() + last_layout.x_for_index(range.start),
                bounds.top(),
            ),
            point(
                bounds.left() + last_layout.x_for_index(range.end),
                bounds.bottom(),
            ),
        ))
    }

    fn character_index_for_point(
        &mut self,
        point: gpui::Point<Pixels>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) -> Option<usize> {
        let line_point = self.last_bounds?.localize(&point)?;
        let last_layout = self.last_layout.as_ref()?;
        let utf8_index = last_layout.index_for_x(point.x - line_point.x)?;
        Some(self.offset_to_utf16(utf8_index))
    }
}

struct TextElement {
    input: Entity<TextInput>,
}

struct PrepaintState {
    line: Option<ShapedLine>,
    cursor: Option<PaintQuad>,
    selection: Option<PaintQuad>,
}

impl IntoElement for TextElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TextElement {
    type RequestLayoutState = ();
    type PrepaintState = PrepaintState;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let mut style = Style::default();
        style.size.width = relative(1.).into();
        style.size.height = window.line_height().into();
        (window.request_layout(style, [], cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        let input = self.input.read(cx);
        let colors = &cx.theme().colors;
        let value = input.value.clone();
        let masked = input.masked;
        let selected_range = input.selected_range.clone();
        let cursor_real = input.cursor_offset();
        let style = window.text_style();

        let raw_content = SharedString::from(value.clone());
        let masked_content: SharedString = if masked && !value.is_empty() {
            SharedString::from(MASK_CHAR.to_string().repeat(value.chars().count()))
        } else {
            raw_content.clone()
        };

        let (display_text, text_color) = if value.is_empty() {
            (input.placeholder.clone(), colors.on_subtle_disabled)
        } else if input.disabled {
            (masked_content, colors.on_neutral_disabled)
        } else {
            (masked_content, style.color)
        };

        // Translate real-value byte offsets to display-text byte offsets when masked.
        let display_cursor = if masked {
            real_to_masked_offset(&value, cursor_real)
        } else {
            cursor_real
        };
        let display_sel = if masked {
            real_to_masked_offset(&value, selected_range.start)
                ..real_to_masked_offset(&value, selected_range.end)
        } else {
            selected_range.clone()
        };

        let run = TextRun {
            len: display_text.len(),
            font: style.font(),
            color: text_color,
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        // Masked fields don't use IME marked ranges, so skip the marked-range split.
        let runs = if !masked {
            if let Some(marked_range) = input.marked_range.as_ref() {
                vec![
                    TextRun {
                        len: marked_range.start,
                        ..run.clone()
                    },
                    TextRun {
                        len: marked_range.end - marked_range.start,
                        underline: Some(UnderlineStyle {
                            color: Some(run.color),
                            thickness: px(1.0),
                            wavy: false,
                        }),
                        ..run.clone()
                    },
                    TextRun {
                        len: display_text.len().saturating_sub(marked_range.end),
                        ..run
                    },
                ]
                .into_iter()
                .filter(|run| run.len > 0)
                .collect()
            } else {
                vec![run]
            }
        } else {
            vec![run]
        };

        let font_size = style.font_size.to_pixels(window.rem_size());
        let line = window
            .text_system()
            .shape_line(display_text, font_size, &runs, None);

        let cursor_pos = line.x_for_index(display_cursor);
        let selection_color = rgba(0x3B82F633);
        let (selection, cursor) = if display_sel.is_empty() {
            (
                None,
                Some(fill(
                    Bounds::new(
                        point(bounds.left() + cursor_pos, bounds.top()),
                        size(px(1.5), bounds.bottom() - bounds.top()),
                    ),
                    colors.accent,
                )),
            )
        } else {
            (
                Some(fill(
                    Bounds::from_corners(
                        point(
                            bounds.left() + line.x_for_index(display_sel.start),
                            bounds.top(),
                        ),
                        point(
                            bounds.left() + line.x_for_index(display_sel.end),
                            bounds.bottom(),
                        ),
                    ),
                    selection_color,
                )),
                None,
            )
        };

        PrepaintState {
            line: Some(line),
            cursor,
            selection,
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let (focus_handle, disabled) = self
            .input
            .update(cx, |input, cx| (input.focus_handle(cx), input.disabled));
        if !disabled {
            window.handle_input(
                &focus_handle,
                ElementInputHandler::new(bounds, self.input.clone()),
                cx,
            );
        }
        if let Some(selection) = prepaint.selection.take() {
            window.paint_quad(selection);
        }
        let line = prepaint.line.take().unwrap();
        line.paint(
            bounds.origin,
            window.line_height(),
            gpui::TextAlign::Left,
            None,
            window,
            cx,
        )
        .unwrap();

        if focus_handle.is_focused(window) && !disabled {
            if let Some(cursor) = prepaint.cursor.take() {
                window.paint_quad(cursor);
            }
        }

        self.input.update(cx, |input, _cx| {
            input.last_layout = Some(line);
            input.last_bounds = Some(bounds);
        });
    }
}

impl Render for TextInput {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing_theme {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            self.observing_theme = true;
        }

        let focus_handle = self.focus_handle(cx);
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;
        let spacing = &theme.spacing;
        let typography = &theme.typography;
        let components = &theme.components;

        let bg = if self.disabled {
            colors.neutral_disabled
        } else {
            colors.surface
        };
        let indicator = if self.error {
            colors.stroke_accent
        } else if self.disabled {
            colors.stroke_neutral_disabled
        } else if focus_handle.is_focused(_window) {
            colors.accent
        } else {
            colors.stroke_neutral
        };

        div()
            .key_context("TextInput")
            .track_focus(&focus_handle)
            .cursor(if self.disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .on_key_down(
                cx.listener(|input: &mut TextInput, ev: &KeyDownEvent, window, cx| {
                    input.handle_key(ev, window, cx);
                }),
            )
            .on_action(crate::focus::focus_next)
            .on_action(crate::focus::focus_previous)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_move(cx.listener(Self::on_mouse_move))
            .relative()
            .flex()
            .items_center()
            .w_full()
            .h(px(components.text_input_height))
            .px(px(spacing.md))
            .rounded(px(radii.md))
            .bg(bg)
            .text_size(px(typography.body.size))
            .line_height(px(typography.body.line_height))
            .text_color(colors.on_neutral)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .h(px(if self.error || focus_handle.is_focused(_window) {
                        components.text_input_focus_indicator_height
                    } else {
                        1.0
                    }))
                    .bg(indicator),
            )
            .child(TextElement { input: cx.entity() })
    }
}

impl Focusable for TextInput {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle().tab_stop(true))
    }
}
