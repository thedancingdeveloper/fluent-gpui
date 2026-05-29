use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, AnyView, App, Context, CursorStyle, DragMoveEvent, IntoElement,
    MouseButton, MouseDownEvent, Render, Window,
};

const DEFAULT_MIN_RATIO: f32 = 0.1;
const DEFAULT_MAX_RATIO: f32 = 0.9;
const DIVIDER_SIZE: f32 = 5.0;

/// Direction for a pane split.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

/// Pixel constraints for a [`PaneGroup`].
///
/// These constraints are generic layout hints. For example, a mail app can use
/// them for message-list and reading-pane widths, while an admin app can use the
/// same type for navigation/details or editor/preview splits.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PaneConstraints {
    pub first_min: f32,
    pub second_min: f32,
    pub first_max: Option<f32>,
    pub second_max: Option<f32>,
}

impl Default for PaneConstraints {
    fn default() -> Self {
        Self {
            first_min: 96.0,
            second_min: 96.0,
            first_max: None,
            second_max: None,
        }
    }
}

impl PaneConstraints {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn first_min(mut self, px: f32) -> Self {
        self.first_min = px.max(0.0);
        self
    }

    pub fn second_min(mut self, px: f32) -> Self {
        self.second_min = px.max(0.0);
        self
    }

    pub fn first_max(mut self, px: impl Into<Option<f32>>) -> Self {
        self.first_max = px.into().map(|px| px.max(0.0));
        self
    }

    pub fn second_max(mut self, px: impl Into<Option<f32>>) -> Self {
        self.second_max = px.into().map(|px| px.max(0.0));
        self
    }
}

/// A resizable split layout of two content areas.
///
/// `PaneGroup` stores the split as a ratio so applications can persist it in
/// their own settings. Pixel constraints are enforced while dragging and also
/// applied as layout min/max constraints when rendering.
#[allow(clippy::type_complexity)]
pub struct PaneGroup {
    pub axis: SplitAxis,
    pub first: Option<AnyView>,
    pub second: Option<AnyView>,
    pub ratio: f32,
    pub constraints: PaneConstraints,
    pub on_resize: Option<Box<dyn Fn(f32, &mut App) + 'static>>,
    drag_start: Option<(f32, f32)>,
    observing_theme: bool,
}

pub type ResizablePanelGroup = PaneGroup;

impl Default for PaneGroup {
    fn default() -> Self {
        Self {
            axis: SplitAxis::Horizontal,
            first: None,
            second: None,
            ratio: 0.5,
            constraints: PaneConstraints::default(),
            on_resize: None,
            drag_start: None,
            observing_theme: false,
        }
    }
}

pub(crate) struct PaneSplitDrag;

impl Render for PaneSplitDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

impl PaneGroup {
    pub fn horizontal() -> Self {
        Self {
            axis: SplitAxis::Horizontal,
            ..Self::default()
        }
    }

    pub fn vertical() -> Self {
        Self {
            axis: SplitAxis::Vertical,
            ..Self::default()
        }
    }

    pub fn first(mut self, view: AnyView) -> Self {
        self.first = Some(view);
        self
    }

    pub fn second(mut self, view: AnyView) -> Self {
        self.second = Some(view);
        self
    }

    pub fn ratio(mut self, ratio: f32) -> Self {
        self.ratio = clamp_ratio_without_extent(ratio);
        self
    }

    pub fn constraints(mut self, constraints: PaneConstraints) -> Self {
        self.constraints = constraints;
        self
    }

    pub fn min_first(mut self, px: f32) -> Self {
        self.constraints.first_min = px.max(0.0);
        self
    }

    pub fn min_second(mut self, px: f32) -> Self {
        self.constraints.second_min = px.max(0.0);
        self
    }

    pub fn max_first(mut self, px: impl Into<Option<f32>>) -> Self {
        self.constraints.first_max = px.into().map(|px| px.max(0.0));
        self
    }

    pub fn max_second(mut self, px: impl Into<Option<f32>>) -> Self {
        self.constraints.second_max = px.into().map(|px| px.max(0.0));
        self
    }

    pub fn on_resize(mut self, f: impl Fn(f32, &mut App) + 'static) -> Self {
        self.on_resize = Some(Box::new(f));
        self
    }

    pub fn ratio_value(&self) -> f32 {
        self.ratio
    }

    pub fn set_first(&mut self, view: AnyView, cx: &mut Context<Self>) {
        self.first = Some(view);
        cx.notify();
    }

    pub fn set_second(&mut self, view: AnyView, cx: &mut Context<Self>) {
        self.second = Some(view);
        cx.notify();
    }

    pub fn set_ratio(&mut self, ratio: f32, cx: &mut Context<Self>) {
        self.apply_ratio(ratio, None, cx);
    }

    pub fn set_constraints(&mut self, constraints: PaneConstraints, cx: &mut Context<Self>) {
        self.constraints = constraints;
        self.apply_ratio(self.ratio, None, cx);
    }

    fn apply_ratio(&mut self, ratio: f32, extent: Option<f32>, cx: &mut Context<Self>) {
        let next = constrain_ratio(ratio, extent, self.constraints);
        if (self.ratio - next).abs() <= f32::EPSILON {
            return;
        }

        self.ratio = next;
        if let Some(on_resize) = &self.on_resize {
            on_resize(self.ratio, cx);
        }
        cx.notify();
    }
}

impl Render for PaneGroup {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing_theme {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            self.observing_theme = true;
        }

        let theme = cx.theme();
        let colors = theme.colors.clone();

        let divider_color = colors.stroke_neutral_subtle;
        let divider_hover = colors.stroke_neutral_dim;
        let axis = self.axis;
        let extent = split_extent(axis, window).map(|extent| (extent - DIVIDER_SIZE).max(1.0));
        let render_ratio = constrain_ratio(self.ratio, extent, self.constraints);
        let constraints = self.constraints;

        let mut container = match self.axis {
            SplitAxis::Horizontal => div().flex().flex_row().flex_1().min_h_0().min_w_0(),
            SplitAxis::Vertical => div().flex().flex_col().flex_1().min_h_0().min_w_0(),
        };

        // First pane
        let first_div = div()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .when(axis == SplitAxis::Horizontal, |d| {
                let d = d
                    .w(gpui::relative(render_ratio))
                    .min_w(px(constraints.first_min))
                    .flex_none();
                if let Some(max) = constraints.first_max {
                    d.max_w(px(max))
                } else {
                    d
                }
            })
            .when(axis == SplitAxis::Vertical, |d| {
                let d = d
                    .h(gpui::relative(render_ratio))
                    .min_h(px(constraints.first_min))
                    .flex_none();
                if let Some(max) = constraints.first_max {
                    d.max_h(px(max))
                } else {
                    d
                }
            });
        let first_div = if let Some(v) = &self.first {
            first_div.child(v.clone())
        } else {
            first_div
        };
        container = container.child(first_div);

        let start_handler =
            cx.listener(move |group: &mut PaneGroup, ev: &MouseDownEvent, _, _cx| {
                let coord = match axis {
                    SplitAxis::Horizontal => f32::from(ev.position.x),
                    SplitAxis::Vertical => f32::from(ev.position.y),
                };
                group.drag_start = Some((group.ratio, coord));
            });

        let move_handler = cx.listener(
            move |group: &mut PaneGroup, ev: &DragMoveEvent<PaneSplitDrag>, window, cx| {
                if let Some((initial_ratio, start_coord)) = group.drag_start {
                    let Some(extent) = split_extent(axis, window) else {
                        return;
                    };
                    let extent = (extent - DIVIDER_SIZE).max(1.0);
                    let current = match axis {
                        SplitAxis::Horizontal => f32::from(ev.event.position.x),
                        SplitAxis::Vertical => f32::from(ev.event.position.y),
                    };
                    let delta = (current - start_coord) / extent;
                    group.apply_ratio(initial_ratio + delta, Some(extent), cx);
                }
            },
        );

        // Divider — flex_none prevents it being shrunk to 0 in the flex container
        let divider = match self.axis {
            SplitAxis::Horizontal => div()
                .w(px(DIVIDER_SIZE))
                .flex_none()
                .cursor(CursorStyle::ResizeLeftRight)
                .bg(divider_color),
            SplitAxis::Vertical => div()
                .h(px(DIVIDER_SIZE))
                .flex_none()
                .cursor(CursorStyle::ResizeUpDown)
                .bg(divider_color),
        };
        container = container.child(
            divider
                .id("pane-split-divider")
                .hover(move |s| s.bg(divider_hover))
                .on_mouse_down(MouseButton::Left, start_handler)
                .on_drag(PaneSplitDrag, |_, _, _, cx| cx.new(|_| PaneSplitDrag)),
        );

        // Second pane
        let second_div = div()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .overflow_hidden()
            .when(axis == SplitAxis::Horizontal, |d| {
                let d = d.min_w(px(constraints.second_min));
                if let Some(max) = constraints.second_max {
                    d.max_w(px(max))
                } else {
                    d
                }
            })
            .when(axis == SplitAxis::Vertical, |d| {
                let d = d.min_h(px(constraints.second_min));
                if let Some(max) = constraints.second_max {
                    d.max_h(px(max))
                } else {
                    d
                }
            });
        let second_div = if let Some(v) = &self.second {
            second_div.child(v.clone())
        } else {
            second_div
        };
        container
            .on_drag_move::<PaneSplitDrag>(move_handler)
            .child(second_div)
    }
}

fn split_extent(axis: SplitAxis, window: &Window) -> Option<f32> {
    let bounds = window.bounds();
    let extent = match axis {
        SplitAxis::Horizontal => f32::from(bounds.size.width),
        SplitAxis::Vertical => f32::from(bounds.size.height),
    };
    extent.is_finite().then_some(extent.max(1.0))
}

fn clamp_ratio_without_extent(ratio: f32) -> f32 {
    if ratio.is_finite() {
        ratio.clamp(DEFAULT_MIN_RATIO, DEFAULT_MAX_RATIO)
    } else {
        0.5
    }
}

fn constrain_ratio(ratio: f32, extent: Option<f32>, constraints: PaneConstraints) -> f32 {
    let ratio = clamp_ratio_without_extent(ratio);
    let Some(extent) = extent else {
        return ratio;
    };

    let extent = extent.max(1.0);
    let lower_from_first_min = constraints.first_min / extent;
    let lower_from_second_max = constraints
        .second_max
        .map(|second_max| 1.0 - second_max / extent)
        .unwrap_or(DEFAULT_MIN_RATIO);
    let upper_from_first_max = constraints
        .first_max
        .map(|first_max| first_max / extent)
        .unwrap_or(DEFAULT_MAX_RATIO);
    let upper_from_second_min = 1.0 - constraints.second_min / extent;

    let lower = DEFAULT_MIN_RATIO
        .max(lower_from_first_min)
        .max(lower_from_second_max)
        .clamp(0.0, 1.0);
    let upper = DEFAULT_MAX_RATIO
        .min(upper_from_first_max)
        .min(upper_from_second_min)
        .clamp(0.0, 1.0);

    if lower <= upper {
        ratio.clamp(lower, upper)
    } else {
        ((lower + upper) * 0.5).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::{constrain_ratio, PaneConstraints};

    #[test]
    fn ratio_without_extent_uses_default_guardrails() {
        assert_eq!(constrain_ratio(0.01, None, PaneConstraints::default()), 0.1);
        assert_eq!(constrain_ratio(0.99, None, PaneConstraints::default()), 0.9);
        assert_eq!(
            constrain_ratio(f32::NAN, None, PaneConstraints::default()),
            0.5
        );
    }

    #[test]
    fn first_min_constrains_lower_ratio() {
        let constraints = PaneConstraints::new().first_min(240.0).second_min(120.0);
        assert_eq!(constrain_ratio(0.2, Some(1000.0), constraints), 0.24);
    }

    #[test]
    fn second_min_constrains_upper_ratio() {
        let constraints = PaneConstraints::new().first_min(120.0).second_min(300.0);
        assert_eq!(constrain_ratio(0.8, Some(1000.0), constraints), 0.7);
    }

    #[test]
    fn first_max_constrains_upper_ratio() {
        let constraints = PaneConstraints::new()
            .first_min(100.0)
            .second_min(100.0)
            .first_max(Some(400.0));

        assert_eq!(constrain_ratio(0.8, Some(1000.0), constraints), 0.4);
    }

    #[test]
    fn second_max_constrains_lower_ratio() {
        let constraints = PaneConstraints::new()
            .first_min(100.0)
            .second_min(100.0)
            .second_max(Some(500.0));

        assert_eq!(constrain_ratio(0.4, Some(1000.0), constraints), 0.5);
    }

    #[test]
    fn impossible_constraints_fall_back_to_middle_of_available_range() {
        let constraints = PaneConstraints::new().first_min(800.0).second_min(800.0);
        assert_eq!(constrain_ratio(0.25, Some(1000.0), constraints), 0.5);
    }
}
