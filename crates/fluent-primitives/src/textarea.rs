use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, App, ClipboardItem, Context, CursorStyle, FocusHandle, Focusable,
    IntoElement, KeyDownEvent, MouseButton, MouseDownEvent, Render, SharedString, Window,
};

/// A lightweight multiline text area with Fluent focus styling.
#[allow(clippy::type_complexity)]
pub struct Textarea {
    value: String,
    placeholder: SharedString,
    rows: usize,
    disabled: bool,
    focus_handle: Option<FocusHandle>,
    observing_theme: bool,
    on_change: Option<Box<dyn Fn(SharedString, &mut App) + 'static>>,
}

impl Default for Textarea {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: SharedString::default(),
            rows: 4,
            disabled: false,
            focus_handle: None,
            observing_theme: false,
            on_change: None,
        }
    }
}

impl Textarea {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into().to_string();
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(2);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_change(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    pub fn text(&self) -> &str {
        &self.value
    }

    pub fn set_value(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.value = value.into().to_string();
        cx.notify();
    }

    pub fn set_disabled(&mut self, disabled: bool, cx: &mut Context<Self>) {
        self.disabled = disabled;
        cx.notify();
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

    fn handle_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if crate::focus::handle_tab_navigation(ev, window, cx) {
            cx.notify();
            return;
        }

        if self.disabled {
            return;
        }

        let ctrl = ev.keystroke.modifiers.platform || ev.keystroke.modifiers.control;
        let key = ev.keystroke.key.as_str();

        // Standard clipboard + selection shortcuts.
        if ctrl {
            match key {
                "a" => {
                    // Select-all: copy entire value to clipboard.
                    if !self.value.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(self.value.clone()));
                    }
                    return;
                }
                "c" => {
                    if !self.value.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(self.value.clone()));
                    }
                    return;
                }
                "x" => {
                    if !self.value.is_empty() {
                        cx.write_to_clipboard(ClipboardItem::new_string(self.value.clone()));
                        self.value.clear();
                        self.emit_change(cx);
                        cx.notify();
                    }
                    return;
                }
                "v" => {
                    if let Some(item) = cx.read_from_clipboard() {
                        if let Some(text) = item.text() {
                            self.value.push_str(&text);
                            self.emit_change(cx);
                            cx.notify();
                        }
                    }
                    return;
                }
                "z" => {
                    // Undo is not yet implemented; absorb the keystroke so it
                    // doesn't insert a literal "z".
                    return;
                }
                _ => {}
            }
        }

        match key {
            "backspace" => {
                self.value.pop();
            }
            "enter" => {
                self.value.push('\n');
            }
            key if key.chars().count() == 1 && !ctrl => {
                self.value.push_str(key);
            }
            _ => return,
        }

        self.emit_change(cx);
        cx.notify();
    }

    fn on_mouse_down(&mut self, _: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }

        let focus_handle = self.focus_handle(cx);
        window.focus(&focus_handle, cx);
        cx.notify();
    }
}

impl Render for Textarea {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing_theme {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            self.observing_theme = true;
        }

        let focus_handle = self.focus_handle(cx);
        let theme = cx.theme();
        let colors = &theme.colors;
        let spacing = &theme.spacing;
        let typography = &theme.typography;
        let radii = &theme.radii;
        let components = &theme.components;
        let focused = focus_handle.is_focused(window);
        let indicator = if focused {
            colors.accent
        } else if self.disabled {
            colors.stroke_neutral_disabled
        } else {
            colors.stroke_neutral
        };
        let text = if self.value.is_empty() {
            self.placeholder.to_string()
        } else {
            self.value.clone()
        };
        let text_color = if self.disabled {
            colors.on_neutral_disabled
        } else if self.value.is_empty() {
            colors.on_subtle_disabled
        } else {
            colors.on_neutral
        };
        let min_height = typography.body.line_height * self.rows as f32 + spacing.md * 2.0;

        let mut lines = div()
            .flex()
            .flex_col()
            .gap(px(spacing.xs))
            .text_size(px(typography.body.size))
            .line_height(px(typography.body.line_height))
            .text_color(text_color);

        for line in text.split('\n') {
            let line = if line.is_empty() {
                SharedString::from(" ")
            } else {
                SharedString::from(line.to_owned())
            };
            lines = lines.child(line);
        }

        div()
            .key_context("Textarea")
            .track_focus(&focus_handle)
            .cursor(if self.disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_key_down(
                cx.listener(|area: &mut Textarea, ev: &KeyDownEvent, window, cx| {
                    area.handle_key(ev, window, cx);
                }),
            )
            .on_action(crate::focus::focus_next)
            .on_action(crate::focus::focus_previous)
            .relative()
            .w_full()
            .min_h(px(min_height))
            .px(px(spacing.md))
            .py(px(spacing.sm))
            .rounded(px(radii.md))
            .bg(if self.disabled {
                colors.neutral_disabled
            } else {
                colors.surface
            })
            .child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .bottom_0()
                    .h(px(if focused {
                        components.text_input_focus_indicator_height
                    } else {
                        1.0
                    }))
                    .bg(indicator),
            )
            .child(lines)
    }
}

impl Focusable for Textarea {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle().tab_stop(true))
    }
}
