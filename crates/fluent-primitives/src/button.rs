use crate::focus::FluentFocus as _;
use fluent_core::{ControlStates, NeutralColorStates, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, Hsla, IntoElement, RenderOnce, Role,
    SharedString, Window,
};

/// Visual style of a `Button`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonAppearance {
    #[default]
    /// The secondary option.
    Secondary,
    /// The primary option.
    Primary,
    /// The outline option.
    Outline,
    /// The subtle option.
    Subtle,
    /// The transparent option.
    Transparent,
}

impl ButtonAppearance {
    /// Compatibility name for the former default appearance.
    #[allow(non_upper_case_globals)]
    pub const Neutral: Self = Self::Secondary;
    /// Compatibility name for the former accent appearance.
    #[allow(non_upper_case_globals)]
    pub const Accent: Self = Self::Primary;
    /// Compatibility name retained for applications migrating to the matrix.
    #[allow(non_upper_case_globals)]
    pub const Danger: Self = Self::Primary;
    /// Compatibility name for text-only buttons.
    #[allow(non_upper_case_globals)]
    pub const Hyperlink: Self = Self::Transparent;
}

/// Corner rounding of a `Button`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonShape {
    #[default]
    /// The rounded option.
    Rounded,
    /// The square option.
    Square,
    /// The circular option.
    Circular,
}

/// Padding / height of a `Button`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    /// The medium option.
    Medium,
    /// The small option.
    Small,
    /// The large option.
    Large,
}

impl ButtonSize {
    #[allow(non_upper_case_globals)]
    /// The normal constant.
    pub const Normal: Self = Self::Medium;
    #[allow(non_upper_case_globals)]
    /// The compact constant.
    pub const Compact: Self = Self::Small;
}

/// Placement of an optional icon relative to the label.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonIconPosition {
    #[default]
    /// The before option.
    Before,
    /// The after option.
    After,
}

/// A Fluent 2-styled push button with optional icon.
///
/// ```ignore
/// Button::new("connect")
///     .label("Connect")
///     .appearance(ButtonAppearance::Accent)
///     .on_click(|_, _, cx| cx.dispatch_action(Connect))
/// ```
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Button {
    id: ElementId,
    label: Option<SharedString>,
    icon: Option<SharedString>,
    icon_position: ButtonIconPosition,
    appearance: ButtonAppearance,
    shape: ButtonShape,
    size: ButtonSize,
    disabled: bool,
    tab_index: Option<isize>,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Button {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            icon_position: ButtonIconPosition::default(),
            appearance: ButtonAppearance::default(),
            shape: ButtonShape::default(),
            size: ButtonSize::default(),
            disabled: false,
            tab_index: None,
            on_click: None,
        }
    }

    /// Returns the current label.
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    /// Sets the icon configuration for this control.
    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    /// Sets the icon position configuration for this control.
    pub fn icon_position(mut self, position: ButtonIconPosition) -> Self {
        self.icon_position = position;
        self
    }

    /// Sets the appearance configuration for this control.
    pub fn appearance(mut self, appearance: ButtonAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    /// Sets the shape configuration for this control.
    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    /// Sets the size configuration for this control.
    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    /// Explicit position in the Tab order (lower first). Defaults to document order.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = Some(index);
        self
    }

    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Registers a handler for click events.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Button {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let spacing = &theme.spacing;
        let radii = &theme.radii;
        let typography = &theme.typography;

        let (states, border): (ControlStates, Option<Hsla>) = match self.appearance {
            ButtonAppearance::Primary => (colors.accent_control(), None),
            ButtonAppearance::Secondary => (
                colors.neutral_control(),
                Some(if self.disabled {
                    colors.neutral_stroke_1.disabled
                } else {
                    colors.neutral_stroke_1.default
                }),
            ),
            ButtonAppearance::Outline => (
                colors.subtle_control(),
                Some(if self.disabled {
                    colors.neutral_stroke_1.disabled
                } else {
                    colors.neutral_stroke_1.default
                }),
            ),
            ButtonAppearance::Subtle => (colors.subtle_control(), None),
            ButtonAppearance::Transparent => {
                let mut s = colors.subtle_control();
                s.foreground = NeutralColorStates {
                    default: colors.on_neutral_accent,
                    hover: colors.on_neutral_accent,
                    pressed: colors.on_neutral_accent,
                    selected: colors.on_neutral_accent,
                    disabled: colors.neutral_foreground_1.disabled,
                };
                (s, None)
            }
        };
        let (bg, fg) = if self.disabled {
            (states.background.disabled, states.foreground.disabled)
        } else {
            (states.background.default, states.foreground.default)
        };
        let (bg_hover, bg_active) = (states.background.hover, states.background.pressed);

        let radius = match self.shape {
            ButtonShape::Rounded => px(radii.md),
            ButtonShape::Square => px(radii.sm),
            ButtonShape::Circular => px(radii.pill),
        };

        let (height, px_val, font_size, line_height, icon_size) = match self.size {
            ButtonSize::Medium => (
                theme.components.button_medium_height,
                spacing.md,
                px(typography.body.size),
                px(typography.body.line_height),
                theme.components.button_medium_icon_size,
            ),
            ButtonSize::Small => (
                theme.components.button_small_height,
                spacing.sm,
                px(typography.caption.size),
                px(typography.caption.line_height),
                theme.components.button_small_icon_size,
            ),
            ButtonSize::Large => (
                theme.components.button_large_height,
                spacing.lg,
                px(typography.subtitle.size),
                px(typography.subtitle.line_height),
                theme.components.button_large_icon_size,
            ),
        };

        let disabled = self.disabled;
        let tab_index = self.tab_index;
        let focus_ring = colors.focus_ring;
        let on_click = self.on_click;
        let icon = self.icon;
        let icon_position = self.icon_position;
        let label = self.label;
        let has_icon = icon.is_some();

        let debug_id = self.id.to_string();
        let mut el = div()
            .id(self.id)
            .debug_selector(move || debug_id)
            .role(Role::Button)
            .when_some(label.clone(), |d, name| d.aria_label(name))
            .flex()
            .flex_row()
            .items_center()
            .when(!has_icon, |d| d.justify_center()) // center text when no icon
            .gap(px(spacing.sm))
            .h(px(height))
            .px(px(px_val))
            .rounded(radius)
            .bg(bg)
            .text_size(font_size)
            .line_height(line_height)
            .text_color(fg);

        if let Some(border_color) = border {
            el = el.border_1().border_color(border_color);
        }

        if !disabled {
            el = el
                .focusable()
                .tab_stop(true)
                .fluent_focus(tab_index, focus_ring)
                .cursor_pointer()
                .hover(move |s| s.bg(bg_hover))
                .active(move |s| s.bg(bg_active));
        }

        if let Some(handler) = on_click.filter(|_| !disabled) {
            el = el.on_click(move |ev, win, app| handler(ev, win, app));
        }

        if let Some(icon_path) = icon
            .as_ref()
            .filter(|_| icon_position == ButtonIconPosition::Before)
        {
            el = el.child(
                gpui::svg()
                    .path(icon_path.clone())
                    .size(px(icon_size))
                    .text_color(fg),
            );
        }

        if let Some(text) = label {
            el = el.child(text);
        }

        if let Some(icon_path) = icon.filter(|_| icon_position == ButtonIconPosition::After) {
            el = el.child(
                gpui::svg()
                    .path(icon_path)
                    .size(px(icon_size))
                    .text_color(fg),
            );
        }

        el
    }
}

/// A Fluent 2 button with a primary action and a separate menu action.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct SplitButton {
    id: ElementId,
    label: SharedString,
    appearance: ButtonAppearance,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_menu_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl SplitButton {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            appearance: ButtonAppearance::Secondary,
            disabled: false,
            on_click: None,
            on_menu_click: None,
        }
    }

    /// Sets the appearance configuration for this control.
    pub fn appearance(mut self, appearance: ButtonAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Registers a handler for click events.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    /// Registers a handler for menu click events.
    pub fn on_menu_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_menu_click = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for SplitButton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let main_id = ElementId::Name(format!("{}-primary", self.id).into());
        let menu_id = ElementId::Name(format!("{}-menu", self.id).into());
        let mut primary = Button::new(main_id)
            .label(self.label)
            .appearance(self.appearance)
            .shape(ButtonShape::Square)
            .disabled(self.disabled);
        if let Some(handler) = self.on_click {
            primary = primary.on_click(handler);
        }
        let mut menu = Button::new(menu_id)
            .label("More options")
            .appearance(self.appearance)
            .shape(ButtonShape::Square)
            .disabled(self.disabled);
        if let Some(handler) = self.on_menu_click {
            menu = menu.on_click(handler);
        }
        div()
            .id(self.id)
            .flex()
            .flex_row()
            .child(primary)
            .child(menu)
    }
}

/// A two-line Fluent 2 button for an action that benefits from description.
#[derive(IntoElement)]
pub struct CompoundButton {
    button: Button,
}

impl CompoundButton {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        secondary_content: impl Into<SharedString>,
    ) -> Self {
        let label = label.into();
        let secondary_content = secondary_content.into();
        Self {
            button: Button::new(id)
                .label(format!("{label}\n{secondary_content}"))
                .size(ButtonSize::Large),
        }
    }

    /// Sets the appearance configuration for this control.
    pub fn appearance(mut self, appearance: ButtonAppearance) -> Self {
        self.button = self.button.appearance(appearance);
        self
    }

    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.button = self.button.disabled(disabled);
        self
    }

    /// Registers a handler for click events.
    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.button = self.button.on_click(handler);
        self
    }
}

impl RenderOnce for CompoundButton {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        self.button
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use fluent_test_utils::{assert_a11y_node, open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, InputEvent as _, KeyUpEvent, Keystroke, Render, TestAppContext};

    use super::*;

    struct Host {
        disabled: bool,
        clicks: Rc<Cell<usize>>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            Button::new("button")
                .label("Save")
                .disabled(self.disabled)
                .on_click(move |_, _, _| clicks.set(clicks.get() + 1))
        }
    }

    fn open(
        cx: &mut TestAppContext,
        disabled: bool,
    ) -> (Rc<Cell<usize>>, &mut gpui::VisualTestContext) {
        let clicks = Rc::new(Cell::new(0));
        let host_clicks = clicks.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| Host {
            disabled,
            clicks: host_clicks,
        });
        (clicks, cx)
    }

    fn press(cx: &mut gpui::VisualTestContext, key: &'static str) {
        cx.type_keys(key);
        let keystroke = Keystroke::parse(key).expect("valid test keystroke");
        cx.update(|window, app| {
            window.dispatch_event(
                KeyUpEvent {
                    keystroke,
                    scan_code: None,
                }
                .to_platform_input(),
                app,
            );
        });
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
    }

    #[gpui::test]
    fn click_enter_and_space_activate_when_enabled(cx: &mut TestAppContext) {
        let (clicks, cx) = open(cx, false);
        cx.click_selector("button");
        cx.update(|window, app| window.draw(app).clear(app));
        press(cx, "enter");
        press(cx, "space");
        assert_eq!(clicks.get(), 3);
    }

    #[gpui::test]
    fn disabled_button_ignores_clicks(cx: &mut TestAppContext) {
        let (clicks, cx) = open(cx, true);
        cx.click_selector("button");
        assert_eq!(clicks.get(), 0);
    }

    struct A11yHost(Box<dyn Fn() -> gpui::AnyElement>);

    impl Render for A11yHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    fn snapshot(
        cx: &mut TestAppContext,
        build: impl Fn() -> gpui::AnyElement + 'static,
    ) -> Vec<String> {
        let (_, cx) = open_fluent_test_window(cx, move |_, _| A11yHost(Box::new(build)));
        cx.accessibility_snapshot()
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_button(cx: &mut TestAppContext) {
        let snap = snapshot(cx, || Button::new("b").label("Save").into_any_element());
        assert_eq!(snap.len(), 2, "{snap:?}");
        assert_a11y_node(&snap, "Button", Some("Save"), &[]);
    }
}
