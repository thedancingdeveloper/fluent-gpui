use std::time::Duration;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, AnimationExt, App, ElementId, IntoElement, RenderOnce, SharedString,
    Window,
};
use gpui::{pulsating_between, Animation};

/// Shimmering placeholder block — fades between `neutral` and `neutral_hover`
/// to indicate content is loading. Size is supplied by the caller.
#[derive(IntoElement)]
pub struct Skeleton {
    id: ElementId,
    width: Option<f32>,
    height: f32,
    rounded: f32,
}

impl Skeleton {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            width: None,
            height: 12.0,
            rounded: 4.0,
        }
    }

    /// Set an explicit width in logical px. Omit for `flex_1` (full-width).
    pub fn w(mut self, width: f32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn h(mut self, height: f32) -> Self {
        self.height = height;
        self
    }

    pub fn rounded(mut self, r: f32) -> Self {
        self.rounded = r;
        self
    }
}

impl RenderOnce for Skeleton {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let base = colors.neutral;
        let peak = colors.neutral_hover;

        let mut block = div().h(px(self.height)).rounded(px(self.rounded));
        block = match self.width {
            Some(w) => block.w(px(w)),
            None => block.flex_1(),
        };

        block.bg(base).with_animation(
            self.id,
            Animation::new(Duration::from_millis(1500))
                .repeat()
                .with_easing(pulsating_between(0.0, 1.0)),
            move |this, delta| {
                let mut c = base;
                c.l = base.l + (peak.l - base.l) * delta;
                this.bg(c)
            },
        )
    }
}

/// Convenience: render a horizontal "list row" skeleton — circle avatar +
/// stacked text bars. Used by Messages/Notifications first-paint.
#[derive(IntoElement)]
pub struct SkeletonRow {
    id_prefix: SharedString,
    avatar_size: f32,
}

impl SkeletonRow {
    pub fn new(id_prefix: impl Into<SharedString>) -> Self {
        Self {
            id_prefix: id_prefix.into(),
            avatar_size: 32.0,
        }
    }

    pub fn avatar_size(mut self, size: f32) -> Self {
        self.avatar_size = size;
        self
    }
}

/// Conditional shimmer wrapper — renders `content` when `loaded`, otherwise a
/// same-sized `Skeleton` placeholder. Pass an explicit `size` (or a width/
/// height pair) so the placeholder doesn't collapse before content arrives.
///
/// ```ignore
/// Skeleton::wrap("hero-title", title_loaded, Some(180.0), 24.0,
///     Label::new(title).size(LabelSize::Subtitle))
/// ```
#[derive(IntoElement)]
pub struct SkeletonWrap<E: IntoElement + 'static> {
    id: ElementId,
    loaded: bool,
    width: Option<f32>,
    height: f32,
    content: E,
}

impl Skeleton {
    /// Convenience constructor for [`SkeletonWrap`].
    pub fn wrap<E: IntoElement + 'static>(
        id: impl Into<ElementId>,
        loaded: bool,
        width: Option<f32>,
        height: f32,
        content: E,
    ) -> SkeletonWrap<E> {
        SkeletonWrap {
            id: id.into(),
            loaded,
            width,
            height,
            content,
        }
    }
}

impl<E: IntoElement + 'static> RenderOnce for SkeletonWrap<E> {
    fn render(self, _: &mut Window, _cx: &mut App) -> impl IntoElement {
        if self.loaded {
            self.content.into_any_element()
        } else {
            let mut sk = Skeleton::new(self.id).h(self.height);
            if let Some(w) = self.width {
                sk = sk.w(w);
            }
            sk.into_any_element()
        }
    }
}

impl RenderOnce for SkeletonRow {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let colors = &cx.theme().colors;
        let base = colors.neutral;
        let lightness_delta = colors.neutral_hover.l - base.l;
        let prefix = self.id_prefix.clone();
        let circle_id: ElementId = SharedString::from(format!("{}-avatar", prefix)).into();
        let bar1_id: ElementId = SharedString::from(format!("{}-line1", prefix)).into();
        let bar2_id: ElementId = SharedString::from(format!("{}-line2", prefix)).into();

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(10.0))
            .child(
                div()
                    .size(px(self.avatar_size))
                    .rounded(px(9999.0))
                    .bg(base)
                    .with_animation(
                        circle_id,
                        Animation::new(Duration::from_millis(1500))
                            .repeat()
                            .with_easing(pulsating_between(0.0, 1.0)),
                        move |this, delta| {
                            let mut c = base;
                            c.l = base.l + lightness_delta * delta;
                            this.bg(c)
                        },
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.0))
                    .flex_1()
                    .child(Skeleton::new(bar1_id).w(140.0).h(10.0))
                    .child(Skeleton::new(bar2_id).w(220.0).h(10.0)),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skeleton_defaults_to_full_width_bar() {
        let skeleton = Skeleton::new("loading");

        assert!(skeleton.width.is_none());
        assert_eq!(skeleton.height, 12.0);
        assert_eq!(skeleton.rounded, 4.0);
    }

    #[test]
    fn skeleton_builders_record_size() {
        let skeleton = Skeleton::new("loading").w(180.0).h(20.0).rounded(8.0);

        assert_eq!(skeleton.width, Some(180.0));
        assert_eq!(skeleton.height, 20.0);
        assert_eq!(skeleton.rounded, 8.0);
    }

    #[test]
    fn skeleton_row_defaults_and_avatar_builder() {
        let row = SkeletonRow::new("row").avatar_size(24.0);

        assert_eq!(row.id_prefix.as_ref(), "row");
        assert_eq!(row.avatar_size, 24.0);
    }

    #[test]
    fn wrap_records_loading_contract() {
        let wrap = Skeleton::wrap("title", false, Some(120.0), 18.0, div());

        assert!(!wrap.loaded);
        assert_eq!(wrap.width, Some(120.0));
        assert_eq!(wrap.height, 18.0);
    }
}
