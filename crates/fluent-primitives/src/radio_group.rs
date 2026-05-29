use std::sync::Arc;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, Hsla, IntoElement, RenderOnce, SharedString,
    Window,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioGroupOrientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug)]
pub struct RadioOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl RadioOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A controlled Fluent radio group.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    options: Vec<RadioOption>,
    selected: Option<SharedString>,
    orientation: RadioGroupOrientation,
    disabled: bool,
    on_select: Option<Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl RadioGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            options: Vec::new(),
            selected: None,
            orientation: RadioGroupOrientation::Vertical,
            disabled: false,
            on_select: None,
        }
    }

    pub fn option(
        mut self,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.options.push(RadioOption::new(value, label));
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = RadioOption>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn orientation(mut self, orientation: RadioGroupOrientation) -> Self {
        self.orientation = orientation;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn on_select(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let spacing = &theme.spacing;
        let typography = &theme.typography;

        let mut root = div()
            .id(self.id)
            .flex()
            .gap(px(spacing.md))
            .when(self.orientation == RadioGroupOrientation::Horizontal, |d| {
                d.flex_row()
            })
            .when(self.orientation == RadioGroupOrientation::Vertical, |d| {
                d.flex_col()
            });

        for (idx, option) in self.options.into_iter().enumerate() {
            let checked = self.selected.as_ref() == Some(&option.value);
            let disabled = self.disabled || option.disabled;
            let on_select = self.on_select.clone();
            let value = option.value.clone();
            let fg = if disabled {
                colors.on_neutral_disabled
            } else {
                colors.on_neutral
            };
            let outer_border: Hsla = if checked {
                colors.accent
            } else {
                colors.stroke_neutral
            };

            let mut row = div()
                .id(("radio-option", idx as u64))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing.sm))
                .text_size(px(typography.body.size))
                .line_height(px(typography.body.line_height))
                .text_color(fg);

            if !disabled {
                row = row.cursor_pointer().hover({
                    let hover = colors.subtle_hover;
                    move |s| s.bg(hover)
                });
                if let Some(handler) = on_select {
                    row = row.on_click(move |ev, win, app| handler(value.clone(), ev, win, app));
                }
            }

            row = row
                .child(
                    div()
                        .size(px(16.0))
                        .rounded(px(9999.0))
                        .border_1()
                        .border_color(outer_border)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .size(px(if checked { 8.0 } else { 0.0 }))
                                .rounded(px(9999.0))
                                .bg(colors.accent),
                        ),
                )
                .child(option.label);

            root = root.child(row);
        }

        root
    }
}
