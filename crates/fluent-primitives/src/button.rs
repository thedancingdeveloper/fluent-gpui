use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, Hsla, IntoElement, RenderOnce, SharedString,
    Window,
};

/// Visual style of a `Button`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonAppearance {
    #[default]
    Neutral,
    Accent,
    Danger,
    Subtle,
    Hyperlink,
}

/// Corner rounding of a `Button`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonShape {
    #[default]
    Rounded,
    Square,
    Circular,
}

/// Padding / height of a `Button`.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ButtonSize {
    #[default]
    Normal,
    Compact,
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
    appearance: ButtonAppearance,
    shape: ButtonShape,
    size: ButtonSize,
    disabled: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Button {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            label: None,
            icon: None,
            appearance: ButtonAppearance::default(),
            shape: ButtonShape::default(),
            size: ButtonSize::default(),
            disabled: false,
            on_click: None,
        }
    }

    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn appearance(mut self, appearance: ButtonAppearance) -> Self {
        self.appearance = appearance;
        self
    }

    pub fn shape(mut self, shape: ButtonShape) -> Self {
        self.shape = shape;
        self
    }

    pub fn size(mut self, size: ButtonSize) -> Self {
        self.size = size;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

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

        let (bg, bg_hover, bg_active, fg, border): (Hsla, Hsla, Hsla, Hsla, Option<Hsla>) =
            match self.appearance {
                ButtonAppearance::Accent => (
                    if self.disabled {
                        colors.accent_disabled
                    } else {
                        colors.accent
                    },
                    colors.accent_hover,
                    colors.accent_selected,
                    if self.disabled {
                        colors.on_accent_disabled
                    } else {
                        colors.on_accent
                    },
                    None,
                ),
                ButtonAppearance::Neutral => (
                    if self.disabled {
                        colors.neutral_disabled
                    } else {
                        colors.neutral
                    },
                    colors.neutral_hover,
                    colors.neutral_selected,
                    if self.disabled {
                        colors.on_neutral_disabled
                    } else {
                        colors.on_neutral
                    },
                    Some(colors.stroke_neutral),
                ),
                ButtonAppearance::Danger => (
                    if self.disabled {
                        colors.neutral_disabled
                    } else {
                        colors.status_error
                    },
                    colors.status_error_border,
                    colors.status_error_bg,
                    if self.disabled {
                        colors.on_neutral_disabled
                    } else {
                        colors.on_accent
                    },
                    None,
                ),
                ButtonAppearance::Subtle => (
                    colors.subtle,
                    colors.subtle_hover,
                    colors.subtle_selected,
                    if self.disabled {
                        colors.on_subtle_disabled
                    } else {
                        colors.on_subtle
                    },
                    None,
                ),
                ButtonAppearance::Hyperlink => (
                    colors.subtle,
                    colors.subtle_hover,
                    colors.subtle_selected,
                    if self.disabled {
                        colors.on_neutral_disabled
                    } else {
                        colors.on_neutral_accent
                    },
                    None,
                ),
            };

        let radius = match self.shape {
            ButtonShape::Rounded => px(radii.md),
            ButtonShape::Square => px(radii.sm),
            ButtonShape::Circular => px(radii.pill),
        };

        let (px_val, py_val, font_size, line_height) = match self.size {
            ButtonSize::Normal => (
                px(spacing.md),
                px(spacing.sm),
                px(typography.body.size),
                px(typography.body.line_height),
            ),
            ButtonSize::Compact => (
                px(spacing.sm),
                px(spacing.xs),
                px(typography.caption.size),
                px(typography.caption.line_height),
            ),
        };

        let disabled = self.disabled;
        let on_click = self.on_click;
        let icon = self.icon;
        let label = self.label;
        let has_icon = icon.is_some();

        let mut el = div()
            .id(self.id)
            .flex()
            .flex_row()
            .items_center()
            .when(!has_icon, |d| d.justify_center()) // center text when no icon
            .gap(px(spacing.sm))
            .px(px_val)
            .py(py_val)
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
                .cursor_pointer()
                .hover(move |s| s.bg(bg_hover))
                .active(move |s| s.bg(bg_active));
        }

        if let Some(handler) = on_click.filter(|_| !disabled) {
            el = el.on_click(move |ev, win, app| handler(ev, win, app));
        }

        if let Some(icon_path) = icon {
            let icon_size = match self.size {
                ButtonSize::Normal => 16.0,
                ButtonSize::Compact => 12.0,
            };
            el = el.child(
                gpui::svg()
                    .path(icon_path)
                    .size(px(icon_size))
                    .text_color(fg),
            );
        }

        if let Some(text) = label {
            el = el.child(text);
        }

        el
    }
}
