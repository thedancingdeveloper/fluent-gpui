use fluent_core::{LayoutDirection, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, AnyElement, App, Entity, IntoElement, RenderOnce, SharedString, Window,
};

use crate::InfoLabel;

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

    /// Replace the visual validation state set by the owning field.
    fn set_field_validation(&mut self, _state: FieldValidationState) {}
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

type SemanticsApplier =
    Box<dyn FnOnce(Option<SharedString>, Option<SharedString>, FieldValidationState, &mut App)>;

/// Validation state for a `Field`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum FieldValidationState {
    #[default]
    /// The none option.
    None,
    /// The success option.
    Success,
    /// The warning option.
    Warning,
    /// The error option.
    Error,
}

/// A Fluent form field wrapper: label, required marker, control, helper text,
/// and validation message.
#[derive(IntoElement)]
pub struct Field {
    label: Option<SharedString>,
    info_label: Option<Entity<InfoLabel>>,
    required: bool,
    helper_text: Option<SharedString>,
    validation_message: Option<SharedString>,
    validation_state: FieldValidationState,
    label_width: Option<f32>,
    control: AnyElement,
    semantics: Option<SemanticsApplier>,
}

impl Field {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(control: impl IntoElement) -> Self {
        Self {
            label: None,
            info_label: None,
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
        field.semantics = Some(Box::new(move |name, description, validation, cx| {
            input.update(cx, |control, _| {
                control.set_field_semantics(name, description);
                control.set_field_validation(validation);
            });
        }));
        field
    }

    /// Returns the current label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self.info_label = None;
        self
    }

    /// Use an [`InfoLabel`] as this field's visible label.
    ///
    /// Its text remains the input's accessible name while its info button
    /// opens the supplementary explanation.
    pub fn info_label(mut self, label: Entity<InfoLabel>) -> Self {
        self.label = None;
        self.info_label = Some(label);
        self
    }

    /// Sets the required configuration for this control.
    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    /// Sets the helper text configuration for this control.
    pub fn helper_text(mut self, text: impl Into<SharedString>) -> Self {
        self.helper_text = Some(text.into());
        self
    }

    /// Sets the validation configuration for this control.
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
        let accessible_label = self
            .info_label
            .as_ref()
            .map(|label| label.read(cx).text())
            .or_else(|| self.label.clone());
        if let Some(apply) = self.semantics {
            let description = self
                .validation_message
                .clone()
                .or_else(|| self.helper_text.clone());
            apply(
                accessible_label.clone(),
                description,
                self.validation_state,
                cx,
            );
        }
        let theme = cx.theme();
        let colors = &theme.colors;
        let spacing = &theme.spacing;
        let typography = &theme.typography;
        let rtl = LayoutDirection::current(cx).is_rtl();

        let message_color = match self.validation_state {
            FieldValidationState::Error => colors.status_error,
            FieldValidationState::Warning => colors.status_warning,
            FieldValidationState::Success => colors.status_success,
            FieldValidationState::None => colors.neutral_foreground_2.default,
        };

        let mut body = div().flex().flex_col().gap(px(spacing.xs)).w_full();

        let label_el = accessible_label.map(|label| {
            let mut label_row = div()
                .flex()
                .when(rtl, |row| row.flex_row_reverse())
                .when(!rtl, |row| row.flex_row())
                .items_center()
                .gap(px(spacing.xs))
                .text_size(px(typography.body.size))
                .line_height(px(typography.body.line_height))
                .text_color(colors.on_neutral);

            label_row = if let Some(info_label) = self.info_label {
                label_row.child(info_label)
            } else {
                label_row.child(label)
            };

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
                body = body.child(validation_message(
                    message,
                    message_color,
                    self.validation_state,
                    spacing,
                    typography,
                ));
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
            root = root.child(validation_message(
                message,
                message_color,
                self.validation_state,
                spacing,
                typography,
            ));
        }

        root
    }
}

fn validation_message(
    message: SharedString,
    color: gpui::Hsla,
    state: FieldValidationState,
    spacing: &fluent_core::SpacingTokens,
    typography: &fluent_core::TypographyTokens,
) -> impl IntoElement {
    let icon = match state {
        FieldValidationState::Error => "×",
        FieldValidationState::Warning => "!",
        FieldValidationState::Success => "✓",
        FieldValidationState::None => "",
    };
    div()
        .flex()
        .items_center()
        .gap(px(spacing.xs))
        .text_size(px(typography.caption.size))
        .line_height(px(typography.caption.line_height))
        .text_color(color)
        .when(state != FieldValidationState::None, |row| {
            row.child(
                div()
                    .flex_none()
                    .w(px(12.0))
                    .text_align(gpui::TextAlign::Center)
                    .font_weight(gpui::FontWeight::SEMIBOLD)
                    .child(icon),
            )
        })
        .child(message)
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Entity, TestAppContext, VisualTestContext};

    use super::*;
    use crate::{Combobox, InfoLabel, Searchbox, TextInput, Textarea, TokenInput};

    struct ValidationProbe {
        state: Rc<RefCell<FieldValidationState>>,
    }

    struct ValidationHost {
        probe: Entity<ValidationProbe>,
    }

    struct InfoLabelHost {
        input: Entity<TextInput>,
        label: Entity<InfoLabel>,
    }

    impl gpui::Render for ValidationHost {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            Field::input(self.probe.clone())
                .label("Server")
                .required(true)
                .helper_text("Host or IP address")
                .validation(FieldValidationState::Warning, "Connection is unverified")
        }
    }

    impl gpui::Render for InfoLabelHost {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            Field::input(self.input.clone()).info_label(self.label.clone())
        }
    }

    impl gpui::Render for ValidationProbe {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            div()
        }
    }

    impl FieldControl for ValidationProbe {
        fn set_field_semantics(&mut self, _: Option<SharedString>, _: Option<SharedString>) {}

        fn set_field_validation(&mut self, state: FieldValidationState) {
            *self.state.borrow_mut() = state;
        }
    }

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

    #[gpui::test]
    fn field_propagates_validation_state_to_wrapped_control(cx: &mut TestAppContext) {
        let state = Rc::new(RefCell::new(FieldValidationState::None));
        let state_for_probe = state.clone();
        let (_, vcx) = open_fluent_test_window(cx, move |_, cx| {
            let probe = cx.new(|_| ValidationProbe {
                state: state_for_probe,
            });
            ValidationHost { probe }
        });

        vcx.update(|window, cx| window.draw(cx).clear(cx));
        assert_eq!(*state.borrow(), FieldValidationState::Warning);
    }

    #[gpui::test]
    fn field_accepts_info_label_and_preserves_accessible_name(cx: &mut TestAppContext) {
        let (_, vcx) = open_fluent_test_window(cx, |_, cx| InfoLabelHost {
            input: cx.new(|_| TextInput::new()),
            label: cx.new(|_| InfoLabel::new("server-info", "Server", "Where requests are sent.")),
        });

        vcx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(vcx
            .accessibility_snapshot()
            .iter()
            .any(|node| node == "TextInput \"Server\""));
        assert!(vcx.selector_bounds("server-info").is_some());
    }

    #[test]
    fn field_configures_required_hint_and_validation_message() {
        let field = Field::new(div())
            .required(true)
            .helper_text("Use the account name")
            .validation(FieldValidationState::Success, "Account found");

        assert!(field.required);
        assert_eq!(field.helper_text.as_deref(), Some("Use the account name"));
        assert_eq!(field.validation_message.as_deref(), Some("Account found"));
        assert_eq!(field.validation_state, FieldValidationState::Success);
    }
}
