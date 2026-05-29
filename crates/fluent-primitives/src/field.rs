use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, AnyElement, App, IntoElement, RenderOnce, SharedString, Window};

/// Validation state for a `Field`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldValidationState {
    #[default]
    None,
    Success,
    Warning,
    Error,
}

/// A Fluent form field wrapper: label, required marker, control, helper text,
/// and validation message.
#[derive(IntoElement)]
pub struct Field {
    label: Option<SharedString>,
    required: bool,
    helper_text: Option<SharedString>,
    validation_message: Option<SharedString>,
    validation_state: FieldValidationState,
    label_width: Option<f32>,
    control: AnyElement,
}

impl Field {
    pub fn new(control: impl IntoElement) -> Self {
        Self {
            label: None,
            required: false,
            helper_text: None,
            validation_message: None,
            validation_state: FieldValidationState::None,
            label_width: None,
            control: control.into_any_element(),
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    pub fn validation(
        mut self,
        state: FieldValidationState,
        message: impl Into<SharedString>,
    ) -> Self {
        self.validation_state = state;
        self.validation_message = Some(message.into());
        self
    }

    /// Render the label in a fixed-width leading column.
    pub fn horizontal(mut self, label_width: f32) -> Self {
        self.label_width = Some(label_width);
        self
    }
}

impl RenderOnce for Field {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let spacing = &theme.spacing;
        let typography = &theme.typography;

        let message_color = match self.validation_state {
            FieldValidationState::Error => colors.status_error,
            FieldValidationState::Warning => colors.status_warning,
            FieldValidationState::Success => colors.status_success,
            FieldValidationState::None => colors.on_subtle,
        };

        let mut body = div().flex().flex_col().gap(px(spacing.xs)).w_full();

        let label_el = self.label.map(|label| {
            let mut label_row = div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing.xs))
                .text_size(px(typography.body.size))
                .line_height(px(typography.body.line_height))
                .text_color(colors.on_neutral)
                .child(label);

            if self.required {
                label_row = label_row.child(
                    div()
                        .text_color(colors.status_error)
                        .font_weight(gpui::FontWeight::SEMIBOLD)
                        .child("*"),
                );
            }

            label_row
        });

        if let Some(label_width) = self.label_width {
            let mut root = div()
                .flex()
                .flex_row()
                .items_start()
                .gap(px(spacing.md))
                .w_full();
            root = root.child(
                div()
                    .flex_none()
                    .w(px(label_width))
                    .pt(px(5.0))
                    .children(label_el),
            );
            body = body.child(self.control);
            if let Some(helper) = self.helper_text {
                body = body.child(
                    div()
                        .text_size(px(typography.caption.size))
                        .line_height(px(typography.caption.line_height))
                        .text_color(colors.on_subtle)
                        .child(helper),
                );
            }
            if let Some(message) = self.validation_message {
                body = body.child(
                    div()
                        .text_size(px(typography.caption.size))
                        .line_height(px(typography.caption.line_height))
                        .text_color(message_color)
                        .child(message),
                );
            }
            return root.child(body);
        }

        let mut root = div().flex().flex_col().gap(px(spacing.xs)).w_full();
        root = root.children(label_el).child(self.control);

        if let Some(helper) = self.helper_text {
            root = root.child(
                div()
                    .text_size(px(typography.caption.size))
                    .line_height(px(typography.caption.line_height))
                    .text_color(colors.on_subtle)
                    .child(helper),
            );
        }

        if let Some(message) = self.validation_message {
            root = root.child(
                div()
                    .text_size(px(typography.caption.size))
                    .line_height(px(typography.caption.line_height))
                    .text_color(message_color)
                    .child(message),
            );
        }

        root
    }
}
