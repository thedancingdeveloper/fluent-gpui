use crate::focus::{control_focus_handle, focus_ring, is_keyboard_focused, FluentFocus as _};
use crate::Spinner;
use fluent_core::{
    ControlStates, NeutralColorStates, TextRampExt as _, ThemeProvider as _, TruncateExt as _,
};
use gpui::{
    div, prelude::*, px, transparent_black, App, ClickEvent, ElementId, Hsla, IntoElement,
    RenderOnce, Role, SharedString, Window,
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

/// Box metrics shared by `Button` and `ToggleButton`, read from the component
/// and typography tokens so every button-shaped control has one sizing rule.
pub(crate) struct ButtonMetrics {
    pub height: f32,
    pub padding_x: f32,
    pub gap: f32,
    pub ramp: fluent_core::TypeRamp,
    pub icon_size: f32,
}

impl ButtonMetrics {
    pub(crate) fn new(size: ButtonSize, theme: &fluent_core::Theme) -> Self {
        let components = &theme.components;
        let typography = &theme.typography;
        match size {
            ButtonSize::Medium => Self {
                height: components.button_medium_height,
                padding_x: components.button_medium_padding_x,
                gap: components.button_medium_gap,
                ramp: typography.body,
                icon_size: components.button_medium_icon_size,
            },
            ButtonSize::Small => Self {
                height: components.button_small_height,
                padding_x: components.button_small_padding_x,
                gap: components.button_small_gap,
                ramp: typography.caption,
                icon_size: components.button_small_icon_size,
            },
            ButtonSize::Large => Self {
                height: components.button_large_height,
                padding_x: components.button_large_padding_x,
                gap: components.button_medium_gap,
                ramp: typography.subtitle,
                icon_size: components.button_large_icon_size,
            },
        }
    }
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
/// ```no_run
/// use fluent_primitives::{Button, ButtonAppearance};
///
/// let connect = Button::new("connect")
///     .label("Connect")
///     .appearance(ButtonAppearance::Accent)
///     .on_click(|_event, _window, _cx| {
///         // Start the connection here.
///     });
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
    loading: bool,
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
            loading: false,
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

    /// Shows a spinner in place of the leading icon (or before the label) and
    /// ignores clicks, Enter and Space. The button keeps its size, Tab stop and
    /// focus ring.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
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
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus =
            (!self.disabled).then(|| control_focus_handle(&self.id, self.tab_index, window, cx));
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;

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

        let ButtonMetrics {
            height,
            padding_x: px_val,
            gap,
            ramp,
            icon_size,
        } = ButtonMetrics::new(self.size, theme);

        let disabled = self.disabled;
        let loading = self.loading;
        let tab_index = self.tab_index;
        let on_click = self.on_click;
        let icon = self.icon;
        let icon_position = self.icon_position;
        let label = self.label;

        let debug_id = self.id.to_string();
        let icon_debug_id = debug_id.clone();
        let label_debug_id = debug_id.clone();
        let mut el = div()
            .id(self.id)
            .debug_selector(move || debug_id)
            .role(Role::Button)
            .when_some(label.clone(), |d, name| d.aria_label(name))
            .flex()
            .flex_row()
            .items_center()
            .justify_center()
            .gap(px(gap))
            .h(px(height))
            .px(px(px_val))
            .rounded(radius)
            .bg(bg)
            .text_ramp(&ramp)
            .text_color(fg)
            // Every appearance carries a 1px border (transparent when it has no
            // outline), so all appearances share one border-box height.
            .border_1()
            .border_color(border.unwrap_or_else(transparent_black));

        if !disabled {
            el = el
                .track_focus(focus.as_ref().expect("enabled control has a focus handle"))
                .key_context("Button")
                .fluent_tab_stop(tab_index.unwrap_or(0))
                .cursor_pointer()
                .hover(move |s| s.bg(bg_hover))
                .active(move |s| s.bg(bg_active));
        }

        if let Some(handler) = on_click.filter(|_| !disabled && !loading) {
            el = el.on_click(move |ev, win, app| handler(ev, win, app));
        }

        let leading_icon = icon
            .as_ref()
            .filter(|_| icon_position == ButtonIconPosition::Before);
        if loading {
            el = el.child(
                Spinner::new()
                    .id(format!("{icon_debug_id}-spinner"))
                    .size(icon_size),
            );
        } else if let Some(icon_path) = leading_icon {
            el = el.child(
                gpui::svg()
                    .debug_selector(move || format!("{icon_debug_id}-icon"))
                    .path(icon_path.clone())
                    .size(px(icon_size))
                    .text_color(fg),
            );
        }

        if let Some(text) = label {
            el = el.child(
                div()
                    .debug_selector(move || format!("{label_debug_id}-label"))
                    .truncate_label()
                    .child(text),
            );
        }

        if let Some(icon_path) = icon.filter(|_| icon_position == ButtonIconPosition::After) {
            el = el.child(
                gpui::svg()
                    .path(icon_path)
                    .size(px(icon_size))
                    .text_color(fg),
            );
        }

        let ring = focus
            .as_ref()
            .is_some_and(|h| is_keyboard_focused(h, window));
        focus_ring(el, ring, colors, radius)
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

    use fluent_test_utils::{
        assert_a11y_node, init_fluent_test, open_fluent_test_window, FluentInteractionExt as _,
    };
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

    const APPEARANCES: [ButtonAppearance; 5] = [
        ButtonAppearance::Primary,
        ButtonAppearance::Secondary,
        ButtonAppearance::Outline,
        ButtonAppearance::Subtle,
        ButtonAppearance::Transparent,
    ];

    #[gpui::test]
    fn every_appearance_and_size_has_the_same_height(cx: &mut TestAppContext) {
        let (_, vcx) = open_fluent_test_window(cx, |_, _| {
            A11yHost(Box::new(|| {
                let mut col = div().flex().flex_col().w(px(400.));
                for (i, appearance) in APPEARANCES.into_iter().enumerate() {
                    for (j, size) in [ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large]
                        .into_iter()
                        .enumerate()
                    {
                        col = col.child(
                            Button::new(format!("b-{i}-{j}"))
                                .label("Save")
                                .appearance(appearance)
                                .size(size),
                        );
                    }
                }
                col.into_any_element()
            }))
        });
        vcx.update(|window, app| window.draw(app).clear(app));
        let tokens = vcx.update(|_, app| app.theme().components);
        let expected = [
            tokens.button_small_height,
            tokens.button_medium_height,
            tokens.button_large_height,
        ];
        for i in 0..APPEARANCES.len() {
            for (j, want) in expected.into_iter().enumerate() {
                let bounds = vcx
                    .selector_bounds(Box::leak(format!("b-{i}-{j}").into_boxed_str()))
                    .expect("button laid out");
                assert_eq!(
                    f32::from(bounds.size.height),
                    want,
                    "appearance {i} size {j}"
                );
            }
        }
    }

    #[gpui::test]
    fn horizontal_padding_follows_tokens_for_every_appearance_and_size(cx: &mut TestAppContext) {
        let (_, vcx) = open_fluent_test_window(cx, |_, _| {
            A11yHost(Box::new(|| {
                // Shrink-wrapped (items_start), so the label's offset is the padding.
                let mut col = div().flex().flex_col().items_start().w(px(400.));
                for (i, appearance) in APPEARANCES.into_iter().enumerate() {
                    for (j, size) in [ButtonSize::Small, ButtonSize::Medium, ButtonSize::Large]
                        .into_iter()
                        .enumerate()
                    {
                        col = col.child(
                            Button::new(format!("p-{i}-{j}"))
                                .label("Save")
                                .appearance(appearance)
                                .size(size),
                        );
                    }
                }
                col.into_any_element()
            }))
        });
        vcx.update(|window, app| window.draw(app).clear(app));
        let t = vcx.update(|_, app| app.theme().components);
        // 12px normal, 8px compact, plus the 1px border every appearance carries.
        assert_eq!(
            (t.button_medium_padding_x, t.button_small_padding_x),
            (12.0, 8.0)
        );
        let expected = [
            t.button_small_padding_x,
            t.button_medium_padding_x,
            t.button_large_padding_x,
        ];
        for i in 0..APPEARANCES.len() {
            for (j, pad) in expected.into_iter().enumerate() {
                let button = vcx
                    .selector_bounds(Box::leak(format!("p-{i}-{j}").into_boxed_str()))
                    .expect("button");
                let label = vcx
                    .selector_bounds(Box::leak(format!("p-{i}-{j}-label").into_boxed_str()))
                    .expect("label");
                assert_eq!(
                    f32::from(label.left() - button.left()),
                    pad + 1.0,
                    "left padding, appearance {i} size {j}"
                );
                assert_eq!(
                    f32::from(button.right() - label.right()),
                    pad + 1.0,
                    "right padding, appearance {i} size {j}"
                );
            }
        }
    }

    #[gpui::test]
    fn long_label_is_truncated_to_one_line_inside_the_button(cx: &mut TestAppContext) {
        let (_, vcx) = open_fluent_test_window(cx, |_, _| {
            A11yHost(Box::new(|| {
                div()
                    .flex()
                    .flex_col()
                    .w(px(120.))
                    .child(
                        Button::new("long")
                            .label("A very long button label that cannot possibly fit on one line")
                            .icon("icons/none.svg"),
                    )
                    .into_any_element()
            }))
        });
        vcx.update(|window, app| window.draw(app).clear(app));
        let height = vcx.update(|_, app| app.theme().components.button_medium_height);
        let button = vcx.selector_bounds("long").expect("button");
        let label = vcx.selector_bounds("long-label").expect("label");
        let icon = vcx.selector_bounds("long-icon").expect("icon");
        assert_eq!(f32::from(button.size.height), height);
        assert!(label.right() <= button.right(), "{label:?} vs {button:?}");
        assert!(
            f32::from(label.size.height) <= height,
            "label wrapped: {label:?}"
        );
        assert!(f32::from(icon.size.width) > 0.0 && icon.right() <= label.left() + px(1.));
    }

    #[gpui::test]
    fn icon_and_label_are_centred_when_stretched(cx: &mut TestAppContext) {
        let (_, vcx) = open_fluent_test_window(cx, |_, _| {
            A11yHost(Box::new(|| {
                div()
                    .flex()
                    .flex_col()
                    .w(px(300.))
                    .child(Button::new("ic").label("Save").icon("icons/none.svg"))
                    .into_any_element()
            }))
        });
        vcx.update(|window, app| window.draw(app).clear(app));
        let button = vcx.selector_bounds("ic").expect("button");
        let icon = vcx.selector_bounds("ic-icon").expect("icon");
        let label = vcx.selector_bounds("ic-label").expect("label");
        assert_eq!(f32::from(button.size.width), 300.0);
        let left = f32::from(icon.origin.x - button.origin.x);
        let right = f32::from(button.right() - label.right());
        assert!(
            (left - right).abs() <= 1.0,
            "content not centred: {left} left vs {right} right"
        );
    }

    #[gpui::test]
    fn text_ramp_line_boxes_are_whole_pixels(cx: &mut TestAppContext) {
        init_fluent_test(cx);
        let typography = cx.update(|app| app.theme().typography);
        for ramp in [
            typography.caption,
            typography.body,
            typography.subtitle,
            typography.title,
            typography.display,
        ] {
            let line = f32::from(ramp.line_height_px());
            let size = f32::from(ramp.font_size_px());
            assert_eq!(line.fract(), 0.0, "{ramp:?}");
            assert_eq!(size.fract(), 0.0, "{ramp:?}");
            assert!(line >= size, "{ramp:?}");
        }
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_button(cx: &mut TestAppContext) {
        let snap = snapshot(cx, || Button::new("b").label("Save").into_any_element());
        assert_eq!(snap.len(), 2, "{snap:?}");
        assert_a11y_node(&snap, "Button", Some("Save"), &[]);
    }

    struct LoadingHost {
        clicks: Rc<Cell<usize>>,
        loading: bool,
    }

    impl Render for LoadingHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let clicks = self.clicks.clone();
            div()
                .flex()
                .flex_col()
                .w(px(300.))
                .child(Button::new("normal").label("Save"))
                .child(
                    Button::new("busy")
                        .label("Save")
                        .loading(self.loading)
                        .on_click(move |_, _, _| clicks.set(clicks.get() + 1)),
                )
        }
    }

    #[gpui::test]
    fn loading_button_ignores_click_enter_and_space_and_keeps_height(cx: &mut TestAppContext) {
        let clicks = Rc::new(Cell::new(0));
        let host_clicks = clicks.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| LoadingHost {
            clicks: host_clicks,
            loading: true,
        });
        cx.update(|window, app| window.draw(app).clear(app));
        cx.click_selector("busy");
        cx.update(|window, app| window.draw(app).clear(app));
        press(cx, "enter");
        press(cx, "space");
        assert_eq!(clicks.get(), 0);

        let normal = cx.selector_bounds("normal").expect("normal laid out");
        let busy = cx.selector_bounds("busy").expect("busy laid out");
        assert_eq!(busy.size.height, normal.size.height);
        let snap = cx.accessibility_snapshot();
        assert!(
            snap.iter().any(|n| n.starts_with("ProgressIndicator")),
            "{snap:?}"
        );
    }

    #[gpui::test]
    fn not_loading_button_still_activates(cx: &mut TestAppContext) {
        let clicks = Rc::new(Cell::new(0));
        let host_clicks = clicks.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| LoadingHost {
            clicks: host_clicks,
            loading: false,
        });
        cx.click_selector("busy");
        cx.update(|window, app| window.draw(app).clear(app));
        press(cx, "enter");
        assert_eq!(clicks.get(), 2);
    }
}
