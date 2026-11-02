use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, svg, App, Context, CursorStyle, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, Render, SharedString, Window,
};

/// A compact search input with a search icon and clear affordance.
#[allow(clippy::type_complexity)]
pub struct Searchbox {
    value: String,
    placeholder: SharedString,
    disabled: bool,
    field_validation: crate::FieldValidationState,
    focus_handle: Option<FocusHandle>,
    a11y_name: Option<SharedString>,
    a11y_description: Option<SharedString>,
    observing_theme: bool,
    on_change: Option<Box<dyn Fn(SharedString, &mut App) + 'static>>,
    on_submit: Option<Box<dyn Fn(SharedString, &mut App) + 'static>>,
}

impl Default for Searchbox {
    fn default() -> Self {
        Self {
            value: String::new(),
            placeholder: "Search".into(),
            disabled: false,
            field_validation: crate::FieldValidationState::None,
            focus_handle: None,
            a11y_name: None,
            a11y_description: None,
            observing_theme: false,
            on_change: None,
            on_submit: None,
        }
    }
}

impl Searchbox {
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

    /// Registers a handler for submit events.
    pub fn on_submit(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_submit = Some(Box::new(f));
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

        match ev.keystroke.key.as_str() {
            "escape" => {
                self.clear(cx);
            }
            "backspace" => {
                self.value.pop();
                self.emit_change(cx);
                cx.notify();
            }
            "enter" => {
                if let Some(f) = &self.on_submit {
                    f(SharedString::from(self.value.clone()), cx);
                }
            }
            key if key.chars().count() == 1 => {
                self.value.push_str(key);
                self.emit_change(cx);
                cx.notify();
            }
            _ => {}
        }
    }

    fn clear(&mut self, cx: &mut Context<Self>) {
        self.value.clear();
        self.emit_change(cx);
        cx.notify();
    }

    fn on_mouse_down(&mut self, _: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled {
            return;
        }
        let handle = self.focus_handle(cx);
        window.focus(&handle, cx);
        cx.notify();
    }
}

impl Render for Searchbox {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing_theme {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            self.observing_theme = true;
        }

        let focus_handle = self.focus_handle(cx);
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let components = theme.components;
        let focused = focus_handle.is_focused(window);
        let fg = if self.disabled {
            colors.on_neutral_disabled
        } else if self.value.is_empty() {
            colors.on_subtle_disabled
        } else {
            colors.on_neutral
        };
        let indicator = match self.field_validation {
            crate::FieldValidationState::Error => colors.status_error,
            crate::FieldValidationState::Warning => colors.status_warning,
            crate::FieldValidationState::Success => colors.status_success,
            crate::FieldValidationState::None if focused => colors.accent,
            crate::FieldValidationState::None => colors.stroke_neutral,
        };
        let text = if self.value.is_empty() {
            self.placeholder.clone()
        } else {
            SharedString::from(self.value.clone())
        };

        let root = div()
            .id(("searchbox", cx.entity_id().as_u64()))
            .key_context("Searchbox")
            .track_focus(&focus_handle)
            .on_key_down(
                cx.listener(|search: &mut Searchbox, ev: &KeyDownEvent, window, cx| {
                    search.handle_key(ev, window, cx);
                }),
            )
            .on_action(crate::focus::focus_next)
            .on_action(crate::focus::focus_previous)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.sm))
            .w_full()
            .h(px(components.text_input_height))
            .px(px(spacing.md))
            .rounded(px(radii.md))
            .border_1()
            .border_color(indicator)
            .bg(if self.disabled {
                colors.neutral_disabled
            } else {
                colors.surface
            })
            .cursor(if self.disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .child(
                svg()
                    .path("icons/search.svg")
                    .size(px(components.command_palette_icon_slot))
                    .text_color(fg),
            )
            .child(
                div()
                    .flex_1()
                    .text_size(px(typography.body.size))
                    .line_height(px(typography.body.line_height))
                    .text_color(fg)
                    .child(text),
            )
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
            );
        let mut root = crate::field::input_semantics(
            root,
            crate::field::InputSemantics {
                role: gpui::accesskit::Role::SearchInput,
                name: &self.a11y_name,
                description: &self.a11y_description,
                value: Some(SharedString::from(self.value.clone())),
                placeholder: Some(self.placeholder.clone()),
            },
        );

        if !self.value.is_empty() && !self.disabled {
            root = root.child(
                div()
                    .id("search-clear")
                    .flex_none()
                    .size(px(components.command_palette_icon_slot))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .rounded(px(radii.sm))
                    .hover({
                        let hover = colors.subtle_hover;
                        move |s| s.bg(hover)
                    })
                    .on_click(cx.listener(|search: &mut Searchbox, _, _, cx| {
                        search.clear(cx);
                    }))
                    .child(
                        svg()
                            .path("icons/dismiss.svg")
                            .size(px(10.0))
                            .text_color(colors.on_subtle),
                    ),
            );
        }

        root
    }
}

impl crate::FieldControl for Searchbox {
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

impl Focusable for Searchbox {
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
    fn escape_clears_search_text(cx: &mut TestAppContext) {
        let (search, cx) = open_fluent_test_window(cx, |_, _| Searchbox::new().value("needle"));
        let focus = search.read_with(cx, |search, _| {
            search
                .focus_handle
                .as_ref()
                .expect("rendered focus handle")
                .clone()
        });
        cx.update(|window, cx| window.focus(&focus, cx));

        cx.type_keys("escape");

        assert_eq!(
            search.read_with(cx, |search, _| search.text().to_owned()),
            ""
        );
    }
}
