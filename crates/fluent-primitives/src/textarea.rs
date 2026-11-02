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
    field_validation: crate::FieldValidationState,
    focus_handle: Option<FocusHandle>,
    a11y_name: Option<SharedString>,
    a11y_description: Option<SharedString>,
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
            field_validation: crate::FieldValidationState::None,
            focus_handle: None,
            a11y_name: None,
            a11y_description: None,
            observing_theme: false,
            on_change: None,
        }
    }
}

impl Textarea {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new() -> Self {
        Self::default()
    }

    /// The accessible name announced by assistive technology. A [`crate::Field`]
    /// label overrides it.
    pub fn accessible_name(mut self, name: impl Into<SharedString>) -> Self {
        self.a11y_name = Some(name.into());
        self
    }

    /// Returns the current value.
    pub fn value(mut self, value: impl Into<SharedString>) -> Self {
        self.value = value.into().to_string();
        self
    }

    /// Sets the placeholder configuration for this control.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Sets the rows configuration for this control.
    pub fn rows(mut self, rows: usize) -> Self {
        self.rows = rows.max(2);
        self
    }

    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Registers a handler for change events.
    pub fn on_change(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_change = Some(Box::new(f));
        self
    }

    /// Returns the current text.
    pub fn text(&self) -> &str {
        &self.value
    }

    /// Sets the value value.
    pub fn set_value(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.value = value.into().to_string();
        cx.notify();
    }

    /// Sets the disabled value.
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
        let indicator = match self.field_validation {
            crate::FieldValidationState::Error => colors.status_error,
            crate::FieldValidationState::Warning => colors.status_warning,
            crate::FieldValidationState::Success => colors.status_success,
            crate::FieldValidationState::None if focused => colors.accent,
            crate::FieldValidationState::None if self.disabled => colors.stroke_neutral_disabled,
            crate::FieldValidationState::None => colors.stroke_neutral,
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

        let root = div()
            .id(("textarea", cx.entity_id().as_u64()))
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
            .border_1()
            .border_color(indicator)
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
            .child(lines);
        crate::field::input_semantics(
            root,
            crate::field::InputSemantics {
                role: gpui::accesskit::Role::MultilineTextInput,
                name: &self.a11y_name,
                description: &self.a11y_description,
                value: Some(SharedString::from(self.value.clone())),
                placeholder: Some(self.placeholder.clone()),
            },
        )
    }
}

impl crate::FieldControl for Textarea {
    fn set_field_semantics(
        &mut self,
        name: Option<SharedString>,
        description: Option<SharedString>,
    ) {
        self.a11y_name = name;
        self.a11y_description = description;
    }

    fn set_field_validation(&mut self, state: crate::FieldValidationState) {
        self.field_validation = state;
    }
}

impl Focusable for Textarea {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle().tab_stop(true))
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::TestAppContext;

    use super::*;

    #[gpui::test]
    fn typing_and_enter_edit_multiline_text(cx: &mut TestAppContext) {
        let (area, cx) = open_fluent_test_window(cx, |_, _| Textarea::new());
        let focus = area.read_with(cx, |area, _| {
            area.focus_handle
                .as_ref()
                .expect("rendered focus handle")
                .clone()
        });
        cx.update(|window, cx| window.focus(&focus, cx));

        cx.type_text("first");
        cx.type_keys("enter");
        cx.type_text("second");

        assert_eq!(
            area.read_with(cx, |area, _| area.text().to_owned()),
            "first\nsecond"
        );
    }
}
