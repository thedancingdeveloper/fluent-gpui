use fluent_core::ThemeProvider as _;
use gpui::{prelude::*, px, svg, App, Hsla, IntoElement, RenderOnce, SharedString, Window};

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

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let size = px(self.size.px());
        let color = self.color.unwrap_or_else(|| cx.theme().colors.on_neutral);
        svg().path(self.path).size(size).text_color(color)
    }
}
