use fluent_core::{LayoutDirection, LayoutDirectionProvider as _, ThemeProvider as _};
use gpui::{
    prelude::*, px, size, svg, App, Hsla, IntoElement, RenderOnce, SharedString, Transformation,
    Window,
};

/// Canonical icon display sizes.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IconSize {
    Sm,
    #[default]
    Md,
    Lg,
    Xl,
}

impl IconSize {
    pub fn px(self) -> f32 {
        match self {
            Self::Sm => 16.0,
            Self::Md => 20.0,
            Self::Lg => 24.0,
            Self::Xl => 32.0,
        }
    }
}

/// An SVG icon element.
///
/// Pass an asset path to an SVG file. The colour defaults to `on_neutral` from
/// the active theme; override it with `.color()`.
///
/// ```ignore
/// Icon::new("icons/add.svg").size(IconSize::Sm)
/// ```
#[derive(IntoElement)]
pub struct Icon {
    path: SharedString,
    size: IconSize,
    color: Option<Hsla>,
}

impl Icon {
    pub fn new(path: impl Into<SharedString>) -> Self {
        Self {
            path: path.into(),
            size: IconSize::default(),
            color: None,
        }
    }

    pub fn size(mut self, size: IconSize) -> Self {
        self.size = size;
        self
    }

    pub fn color(mut self, color: impl Into<Hsla>) -> Self {
        self.color = Some(color.into());
        self
    }
}

fn is_directional_icon(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let name = name.strip_suffix(".svg").unwrap_or(name);
    [
        "chevron_left",
        "chevron_right",
        "arrow_left",
        "arrow_right",
        "back",
        "forward",
    ]
    .contains(&name)
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let icon_size = px(self.size.px());
        let color = self.color.unwrap_or_else(|| cx.theme().colors.on_neutral);
        let mut icon = svg()
            .path(self.path.clone())
            .size(icon_size)
            .text_color(color);
        if cx.layout_direction() == LayoutDirection::Rtl && is_directional_icon(&self.path) {
            icon = icon.with_transformation(Transformation::scale(size(-1.0, 1.0)));
        }
        icon
    }
}

#[cfg(test)]
mod tests {
    use super::is_directional_icon;
    use fluent_core::LayoutDirection;

    #[test]
    fn identifies_directional_icon_names() {
        assert!(is_directional_icon("icons/chevron_right.svg"));
        assert!(is_directional_icon("icons/arrow_left.svg"));
        assert!(is_directional_icon("icons/back.svg"));
        assert!(is_directional_icon("forward.svg"));
        assert!(!is_directional_icon("icons/chevron_down.svg"));
        assert!(!is_directional_icon("icons/search.svg"));
    }

    #[test]
    fn layout_direction_defaults_to_ltr() {
        assert_eq!(LayoutDirection::default(), LayoutDirection::Ltr);
        assert!(!LayoutDirection::default().is_rtl());
        assert!(LayoutDirection::Rtl.is_rtl());
    }
}
