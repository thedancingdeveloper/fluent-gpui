use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, relative, Animation, AnimationExt as _, App, Div, ElementId, Hsla,
    IntoElement, RenderOnce, Role, SharedString, Stateful, Window,
};

const MEDIUM_HEIGHT: f32 = 4.0;
const LARGE_HEIGHT: f32 = 8.0;
const INDETERMINATE_WIDTH: f32 = 0.3;

#[derive(Clone, Copy, Default)]
enum ProgressStatus {
    #[default]
    Neutral,
    Success,
    Warning,
    Error,
}

/// A horizontal progress bar.
///
/// Determinate values are clamped to `[0.0, 1.0]`. Use [`Self::indeterminate`]
/// when progress is active but its completion cannot be calculated.
#[derive(IntoElement)]
pub struct ProgressBar {
    id: ElementId,
    value: f32,
    indeterminate: bool,
    height: f32,
    status: ProgressStatus,
    value_text: Option<SharedString>,
}

impl ProgressBar {
    #[track_caller]
    pub fn new(value: f32) -> Self {
        Self {
            id: core::panic::Location::caller().into(),
            value: clamp_value(value),
            indeterminate: false,
            height: MEDIUM_HEIGHT,
            status: ProgressStatus::Neutral,
            value_text: None,
        }
    }

    /// Animate a segment across the track instead of showing a fixed value.
    pub fn indeterminate(mut self) -> Self {
        self.indeterminate = true;
        self
    }

    /// Human-readable value announced by assistive technology, e.g.
    /// `"412 of 900 MB"`. Defaults to a percentage for determinate bars.
    pub fn value_text(mut self, text: impl Into<SharedString>) -> Self {
        self.value_text = Some(text.into());
        self
    }

    /// Use the standard medium thickness.
    pub fn medium(mut self) -> Self {
        self.height = MEDIUM_HEIGHT;
        self
    }

    /// Use the standard large thickness.
    pub fn large(mut self) -> Self {
        self.height = LARGE_HEIGHT;
        self
    }

    /// Set a custom thickness in logical pixels.
    pub fn height(mut self, px: f32) -> Self {
        self.height = px;
        self
    }

    pub fn success(mut self) -> Self {
        self.status = ProgressStatus::Success;
        self
    }

    pub fn warning(mut self) -> Self {
        self.status = ProgressStatus::Warning;
        self
    }

    pub fn error(mut self) -> Self {
        self.status = ProgressStatus::Error;
        self
    }
}

fn clamp_value(value: f32) -> f32 {
    if value.is_nan() {
        0.0
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn default_value_text(value: f32) -> String {
    format!("{}%", (value * 100.0).round() as u32)
}

fn a11y_base(id: ElementId, value: f32, indeterminate: bool) -> Stateful<Div> {
    let progress = div()
        .id(id)
        .role(Role::ProgressIndicator)
        .aria_min_numeric_value(0.0)
        .aria_max_numeric_value(1.0);

    if indeterminate {
        progress
    } else {
        progress
            .aria_numeric_value(value.into())
            .aria_value(default_value_text(value))
    }
}

impl RenderOnce for ProgressBar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let indicator_color: Hsla = match self.status {
            ProgressStatus::Neutral => colors.accent,
            ProgressStatus::Success => colors.status_success,
            ProgressStatus::Warning => colors.status_warning,
            ProgressStatus::Error => colors.status_error,
        };

        let indicator = if self.indeterminate {
            let duration = theme.motion().popup_enter_duration() * 8;
            let animation_id = (self.id.clone(), "animation");
            div()
                .absolute()
                .h_full()
                .w(relative(INDETERMINATE_WIDTH))
                .rounded(px(9999.0))
                .bg(indicator_color)
                .with_animation(
                    animation_id,
                    Animation::new(duration).repeat(),
                    |segment, delta| {
                        segment.left(relative(
                            delta * (1.0 + INDETERMINATE_WIDTH) - INDETERMINATE_WIDTH,
                        ))
                    },
                )
                .into_any_element()
        } else {
            div()
                .h_full()
                .w(relative(self.value))
                .rounded(px(9999.0))
                .bg(indicator_color)
                .into_any_element()
        };

        let mut base = a11y_base(self.id, self.value, self.indeterminate);
        if let (Some(text), false) = (self.value_text, self.indeterminate) {
            base = base.aria_value(text);
        }
        base.relative()
            .w_full()
            .h(px(self.height))
            .overflow_hidden()
            .rounded(px(9999.0))
            .bg(colors.neutral)
            .child(indicator)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clamps_determinate_value() {
        assert_eq!(ProgressBar::new(-0.25).value, 0.0);
        assert_eq!(ProgressBar::new(0.4).value, 0.4);
        assert_eq!(ProgressBar::new(1.25).value, 1.0);
        assert_eq!(ProgressBar::new(f32::NAN).value, 0.0);
    }

    #[test]
    fn default_value_text_is_percentage() {
        assert_eq!(default_value_text(0.456), "46%");
    }

    #[test]
    fn exposes_progress_role_and_value() {
        let element = a11y_base("progress-test".into(), 0.4, false);
        assert_eq!(element.a11y_role(), Some(Role::ProgressIndicator));

        let mut node = gpui::accesskit::Node::new(Role::ProgressIndicator);
        element.write_a11y_info(&mut node);
        assert_eq!(node.numeric_value(), Some(f64::from(0.4_f32)));
        assert_eq!(node.min_numeric_value(), Some(0.0));
        assert_eq!(node.max_numeric_value(), Some(1.0));
    }
}
