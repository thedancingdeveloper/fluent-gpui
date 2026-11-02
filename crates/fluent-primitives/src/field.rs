use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, AnyElement, App, Entity, IntoElement, RenderOnce, SharedString, Window,
};

/// An input a [`Field`] can name and describe for assistive technology.
///
/// [`Field::input`] pushes the field's label (the accessible name) and its
/// validation message, or helper text when there is none (the accessible
/// description), into the control on every render.
pub trait FieldControl: 'static {
    /// Replace the accessible name and description set by the owning field.
    fn set_field_semantics(
        &mut self,
        name: Option<SharedString>,
        description: Option<SharedString>,
    );
}

/// What an input reports to assistive technology.
pub(crate) struct InputSemantics<'a> {
    pub role: gpui::accesskit::Role,
    pub name: &'a Option<SharedString>,
    pub description: &'a Option<SharedString>,
    pub value: Option<SharedString>,
    pub placeholder: Option<SharedString>,
}

/// Apply role, name, description, value and placeholder to an input's root.
pub(crate) fn input_semantics(
    root: gpui::Stateful<gpui::Div>,
    semantics: InputSemantics<'_>,
) -> gpui::Stateful<gpui::Div> {
    let mut root = root.role(semantics.role);
    if let Some(name) = semantics.name {
        root = root.aria_label(name.clone());
    }
    if let Some(description) = semantics.description {
        root = root.aria_description(description.clone());
    }
    if let Some(value) = semantics.value {
        root = root.aria_value(value);
    }
    if let Some(placeholder) = semantics.placeholder.filter(|p| !p.is_empty()) {
        root = root.aria_placeholder(placeholder);
    }
    root
}

type SemanticsApplier = Box<dyn FnOnce(Option<SharedString>, Option<SharedString>, &mut App)>;

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
    semantics: Option<SemanticsApplier>,
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
            semantics: None,
        }
    }

    /// A field around an input entity. The label becomes the input's
    /// accessible name and the validation message (else the helper text) its
    /// accessible description.
    pub fn input<T: FieldControl + gpui::Render>(input: Entity<T>) -> Self {
        let mut field = Self::new(input.clone());
        field.semantics = Some(Box::new(move |name, description, cx| {
            input.update(cx, |control, _| {
                control.set_field_semantics(name, description)
            });
        }));
        field
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
        if let Some(apply) = self.semantics {
            let description = self
                .validation_message
                .clone()
                .or_else(|| self.helper_text.clone());
            apply(self.label.clone(), description, cx);
        }
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

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Entity, TestAppContext, VisualTestContext};

    use super::*;
    use crate::{Combobox, Searchbox, TextInput, Textarea, TokenInput};

    struct Host {
        text: Entity<TextInput>,
        password: Entity<TextInput>,
        area: Entity<Textarea>,
        search: Entity<Searchbox>,
        combo: Entity<Combobox>,
        tokens: Entity<TokenInput>,
    }

    impl gpui::Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            div()
                .child(
                    Field::input(self.text.clone())
                        .label("Email")
                        .validation(FieldValidationState::Error, "Enter a valid address"),
                )
                .child(Field::input(self.password.clone()).label("Password"))
                .child(Field::input(self.area.clone()).label("Notes"))
                .child(Field::input(self.search.clone()).label("Find"))
                .child(
                    Field::input(self.combo.clone())
                        .label("Country")
                        .helper_text("Pick one"),
                )
                .child(Field::input(self.tokens.clone()).label("Recipients"))
        }
    }

    fn open(cx: &mut TestAppContext) -> &mut VisualTestContext {
        let (_, vcx) = open_fluent_test_window(cx, |_, cx| Host {
            text: cx.new(|_| TextInput::new().value("ada@example.com")),
            password: cx.new(|_| TextInput::new().value("hunter2").masked(true)),
            area: cx.new(|_| Textarea::new().value("line one")),
            search: cx.new(|_| Searchbox::new().value("needle")),
            combo: cx.new(|_| Combobox::new()),
            tokens: cx.new(|_| TokenInput::new()),
        });
        vcx
    }

    #[gpui::test]
    fn field_names_inputs_by_role_and_label(cx: &mut TestAppContext) {
        let vcx = open(cx);
        vcx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(
            vcx.accessibility_snapshot(),
            vec![
                "Window".to_string(),
                "TextInput \"Email\"".to_string(),
                "PasswordInput \"Password\"".to_string(),
                "MultilineTextInput \"Notes\"".to_string(),
                "SearchInput \"Find\" [placeholder=Search]".to_string(),
                "ComboBox \"Country\" [placeholder=Select or type, expanded=false]".to_string(),
                "TextInput \"Recipients\" [placeholder=Add item]".to_string(),
            ]
        );
    }

    #[gpui::test]
    fn field_wires_description_value_and_protection(cx: &mut TestAppContext) {
        let vcx = open(cx);
        vcx.accessibility_snapshot();
        let tree = vcx
            .update(|window, _| window.debug_a11y_tree_json())
            .expect("tree captured");
        assert!(tree.contains("Enter a valid address"), "{tree}");
        assert!(tree.contains("Pick one"), "{tree}");
        assert!(tree.contains("ada@example.com"), "{tree}");
        assert!(tree.contains("line one"), "{tree}");
        assert!(tree.contains("needle"), "{tree}");
        assert!(!tree.contains("hunter2"), "password value leaked: {tree}");
    }
}
