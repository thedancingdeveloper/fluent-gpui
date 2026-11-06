use std::rc::Rc;
use std::time::{Duration, Instant};

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, Bounds, Context, DragMoveEvent, ElementId, IntoElement, ListState,
    MouseButton, MouseDownEvent, Orientation, Pixels, Point, Render, RenderOnce, Role,
    ScrollHandle, Window,
};

/// Width of the hit area (and expanded track) across the scroll axis.
const TRACK: f32 = 12.0;
/// Thumb thickness while the scrollbar is collapsed (idle, overlay).
const THUMB_THIN: f32 = 2.0;
/// Thumb thickness while the pointer is over the scrollbar or it is dragged.
const THUMB_WIDE: f32 = 6.0;
/// Gap between the thumb and the edge of the viewport.
const INSET: f32 = 3.0;
/// The shortest the thumb is allowed to get, so it stays grabbable.
const MIN_THUMB: f32 = 24.0;
/// How long an auto-hide scrollbar stays visible after the last scroll.
const HIDE_AFTER: Duration = Duration::from_millis(1000);
/// The last part of [`HIDE_AFTER`] is spent fading out.
const FADE: Duration = Duration::from_millis(250);

/// The thumb's size and position along the track, in logical pixels.
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ThumbGeometry {
    pub length: f32,
    pub start: f32,
}

/// Thumb length and start for a viewport of `viewport` pixels over content
/// that can scroll `max_offset` pixels, currently scrolled `scrolled` pixels.
/// `None` when nothing can scroll.
pub(crate) fn thumb_geometry(
    viewport: f32,
    max_offset: f32,
    scrolled: f32,
    min_thumb: f32,
) -> Option<ThumbGeometry> {
    if viewport.is_nan() || max_offset.is_nan() || viewport <= 0.0 || max_offset <= 0.5 {
        return None;
    }
    let content = viewport + max_offset;
    let length = (viewport * viewport / content).max(min_thumb).min(viewport);
    let travel = viewport - length;
    let fraction = (scrolled / max_offset).clamp(0.0, 1.0);
    Some(ThumbGeometry {
        length,
        start: travel * fraction,
    })
}

/// The scroll distance that puts the thumb's start at `thumb_start`.
pub(crate) fn scroll_for_thumb_start(
    thumb_start: f32,
    viewport: f32,
    max_offset: f32,
    thumb_length: f32,
) -> f32 {
    let travel = viewport - thumb_length;
    if travel.is_nan() || max_offset.is_nan() || travel <= 0.0 || max_offset <= 0.0 {
        return 0.0;
    }
    thumb_start.clamp(0.0, travel) / travel * max_offset
}

/// The scroll distance after a click on the track at `position` (relative to
/// the track start): one viewport towards the click, never past the ends.
pub(crate) fn page_target(
    position: f32,
    thumb: ThumbGeometry,
    scrolled: f32,
    viewport: f32,
    max_offset: f32,
) -> f32 {
    let target = if position < thumb.start {
        scrolled - viewport
    } else if position > thumb.start + thumb.length {
        scrolled + viewport
    } else {
        scrolled
    };
    target.clamp(0.0, max_offset.max(0.0))
}

/// Opacity of an auto-hiding scrollbar `since_activity` after it last moved.
pub(crate) fn auto_hide_opacity(since_activity: Duration) -> f32 {
    if since_activity >= HIDE_AFTER {
        return 0.0;
    }
    let remaining = HIDE_AFTER - since_activity;
    if remaining >= FADE {
        1.0
    } else {
        remaining.as_secs_f32() / FADE.as_secs_f32()
    }
}

/// Scroll state a [`Scrollbar`] can show and drive.
///
/// Offsets follow GPUI's convention: the scroll offset is zero at the start
/// and negative once content has scrolled, down to `-max_offset()`.
pub trait ScrollbarSource {
    /// Where the scrolling viewport is painted, as of the last frame.
    fn viewport_bounds(&self) -> Bounds<Pixels>;
    /// The current scroll offset (zero or negative on each axis).
    fn scroll_offset(&self) -> Point<Pixels>;
    /// The furthest the content can scroll (zero or positive on each axis).
    fn max_offset(&self) -> Point<Pixels>;
    /// Scrolls to `offset` (zero or negative on each axis).
    fn set_scroll_offset(&self, offset: Point<Pixels>);
    /// Called when a thumb drag begins.
    fn drag_started(&self) {}
    /// Called when a thumb drag ends.
    fn drag_ended(&self) {}
}

impl ScrollbarSource for ScrollHandle {
    fn viewport_bounds(&self) -> Bounds<Pixels> {
        self.bounds()
    }
    fn scroll_offset(&self) -> Point<Pixels> {
        self.offset()
    }
    fn max_offset(&self) -> Point<Pixels> {
        ScrollHandle::max_offset(self)
    }
    fn set_scroll_offset(&self, offset: Point<Pixels>) {
        self.set_offset(offset);
    }
}

impl ScrollbarSource for ListState {
    fn viewport_bounds(&self) -> Bounds<Pixels> {
        ListState::viewport_bounds(self)
    }
    fn scroll_offset(&self) -> Point<Pixels> {
        self.scroll_px_offset_for_scrollbar()
    }
    fn max_offset(&self) -> Point<Pixels> {
        self.max_offset_for_scrollbar()
    }
    fn set_scroll_offset(&self, offset: Point<Pixels>) {
        self.set_offset_from_scrollbar(offset);
    }
    fn drag_started(&self) {
        self.scrollbar_drag_started();
    }
    fn drag_ended(&self) {
        self.scrollbar_drag_ended();
    }
}

/// When a [`Scrollbar`] is shown.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarVisibility {
    /// Fluent 2 overlay behaviour: shown while scrolling or while the pointer
    /// is over it, then faded out.
    #[default]
    Auto,
    /// Always shown while the content overflows.
    Always,
}

/// Which axis a [`Scrollbar`] scrolls.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ScrollbarAxis {
    /// Scrolls vertically; sits on the right edge of the viewport.
    #[default]
    Vertical,
    /// Scrolls horizontally; sits on the bottom edge of the viewport.
    Horizontal,
}

#[derive(Default)]
struct ScrollbarState {
    hovered: bool,
    thumb_hovered: bool,
    /// Distance from the thumb's start to where it was grabbed.
    grab: Option<f32>,
    last_scrolled: f32,
    last_activity: Option<Instant>,
}

/// Drag payload; only the gesture matters, so the ghost is empty.
struct ScrollbarDrag;

impl Render for ScrollbarDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
    }
}

/// A Fluent 2 scrollbar: a thin overlay thumb that widens on hover, drags,
/// pages when the track is clicked and follows wheel or programmatic scrolling.
///
/// Place it as a sibling of the scrolling element, inside a `relative()`
/// parent that has the same bounds; it positions itself along the right
/// (vertical) or bottom (horizontal) edge. Give it a [`ScrollHandle`] that the
/// scrolling element tracks (`track_scroll`), or a [`ListState`].
#[derive(IntoElement)]
pub struct Scrollbar {
    id: ElementId,
    source: Rc<dyn ScrollbarSource>,
    axis: ScrollbarAxis,
    visibility: ScrollbarVisibility,
}

impl Scrollbar {
    /// Creates a vertical scrollbar over `source`.
    pub fn new(id: impl Into<ElementId>, source: impl ScrollbarSource + 'static) -> Self {
        Self {
            id: id.into(),
            source: Rc::new(source),
            axis: ScrollbarAxis::Vertical,
            visibility: ScrollbarVisibility::Auto,
        }
    }

    /// Sets the axis.
    pub fn axis(mut self, axis: ScrollbarAxis) -> Self {
        self.axis = axis;
        self
    }

    /// Scrolls horizontally.
    pub fn horizontal(self) -> Self {
        self.axis(ScrollbarAxis::Horizontal)
    }

    /// Sets when the scrollbar is shown.
    pub fn visibility(mut self, visibility: ScrollbarVisibility) -> Self {
        self.visibility = visibility;
        self
    }

    /// Keeps the scrollbar visible whenever the content overflows.
    pub fn always_visible(self) -> Self {
        self.visibility(ScrollbarVisibility::Always)
    }
}

/// An axis-specific view of a [`ScrollbarSource`], in plain `f32` pixels with
/// scroll distances positive.
#[derive(Clone, Copy)]
struct AxisMetrics {
    origin: f32,
    viewport: f32,
    max: f32,
    scrolled: f32,
}

fn along(axis: ScrollbarAxis, p: Point<Pixels>) -> f32 {
    match axis {
        ScrollbarAxis::Vertical => f32::from(p.y),
        ScrollbarAxis::Horizontal => f32::from(p.x),
    }
}

fn metrics(source: &dyn ScrollbarSource, axis: ScrollbarAxis) -> AxisMetrics {
    let bounds = source.viewport_bounds();
    AxisMetrics {
        origin: along(axis, bounds.origin),
        viewport: match axis {
            ScrollbarAxis::Vertical => f32::from(bounds.size.height),
            ScrollbarAxis::Horizontal => f32::from(bounds.size.width),
        },
        max: along(axis, source.max_offset()),
        scrolled: -along(axis, source.scroll_offset()),
    }
}

/// Scrolls `source` to `distance` along `axis`, leaving the other axis alone.
fn scroll_to(source: &dyn ScrollbarSource, axis: ScrollbarAxis, distance: f32) {
    let mut offset = source.scroll_offset();
    match axis {
        ScrollbarAxis::Vertical => offset.y = px(-distance),
        ScrollbarAxis::Horizontal => offset.x = px(-distance),
    }
    source.set_scroll_offset(offset);
}

impl RenderOnce for Scrollbar {
    fn render(self, window: &mut Window, cx: &mut gpui::App) -> impl IntoElement {
        let (axis, source) = (self.axis, self.source);
        let vertical = axis == ScrollbarAxis::Vertical;
        let state = window.use_keyed_state((self.id.clone(), "state"), cx, |_, _| {
            ScrollbarState::default()
        });

        let m = metrics(source.as_ref(), axis);
        if m.viewport <= 0.0 {
            // No layout yet; look again next frame.
            window.request_animation_frame();
        }

        // A drag ends when GPUI drops the drag payload (button released).
        if state.read(cx).grab.is_some() && !cx.has_active_drag() {
            source.drag_ended();
            state.update(cx, |s, _| s.grab = None);
        }
        let now = Instant::now();
        let (hovered, thumb_hovered, dragging, since_activity) = state.update(cx, |s, _| {
            if (m.scrolled - s.last_scrolled).abs() > 0.5 {
                s.last_scrolled = m.scrolled;
                s.last_activity = Some(now);
            }
            (
                s.hovered,
                s.thumb_hovered,
                s.grab.is_some(),
                s.last_activity.map(|t| now.duration_since(t)),
            )
        });

        let geometry = thumb_geometry(m.viewport, m.max, m.scrolled, MIN_THUMB);
        let engaged = hovered || dragging;
        let opacity = match self.visibility {
            ScrollbarVisibility::Always => 1.0,
            ScrollbarVisibility::Auto if engaged => 1.0,
            ScrollbarVisibility::Auto => match since_activity {
                Some(d) => {
                    if d < HIDE_AFTER {
                        window.request_animation_frame();
                    }
                    auto_hide_opacity(d)
                }
                None => 0.0,
            },
        };

        let colors = cx.theme().colors.clone();
        let fg = &colors.neutral_foreground_3;
        let thumb_color = if dragging {
            fg.pressed
        } else if thumb_hovered || hovered {
            fg.hover
        } else {
            fg.default
        };
        let thickness = if engaged { THUMB_WIDE } else { THUMB_THIN };
        let radius = px(cx.theme().radii.pill);

        let mut root = div()
            .id(self.id.clone())
            .debug_selector({
                let name = self.id.to_string();
                move || name
            })
            .absolute()
            .role(Role::ScrollBar)
            .aria_orientation(if vertical {
                Orientation::Vertical
            } else {
                Orientation::Horizontal
            })
            .aria_min_numeric_value(0.0)
            .aria_max_numeric_value(f64::from(m.max))
            .aria_numeric_value(f64::from(m.scrolled));
        root = if vertical {
            root.top_0().bottom_0().right_0().w(px(TRACK))
        } else {
            root.left_0().right_0().bottom_0().h(px(TRACK))
        };
        let Some(thumb) = geometry else {
            return root;
        };
        if engaged && opacity > 0.0 {
            root = root.bg(colors.neutral_background_2.hover);
        }

        let hover_state = state.clone();
        let click_state = state.clone();
        let click_source = source.clone();
        let mut root = root
            .opacity(opacity)
            .on_hover(move |over, window, cx| {
                hover_state.update(cx, |s, cx| {
                    s.hovered = *over;
                    cx.notify();
                });
                window.refresh();
            })
            .on_mouse_down(MouseButton::Left, move |ev: &MouseDownEvent, window, cx| {
                // Only reached for clicks on the bare track: the thumb stops
                // propagation of its own presses.
                let m = metrics(click_source.as_ref(), axis);
                let Some(thumb) = thumb_geometry(m.viewport, m.max, m.scrolled, MIN_THUMB) else {
                    return;
                };
                let target = page_target(
                    along(axis, ev.position) - m.origin,
                    thumb,
                    m.scrolled,
                    m.viewport,
                    m.max,
                );
                scroll_to(click_source.as_ref(), axis, target);
                click_state.update(cx, |s, _| s.last_activity = Some(Instant::now()));
                window.refresh();
            });

        let drag_state = state.clone();
        let drag_source = source.clone();
        root = root.on_drag_move::<ScrollbarDrag>(
            move |ev: &DragMoveEvent<ScrollbarDrag>, window, cx| {
                let Some(grab) = drag_state.read(cx).grab else {
                    return;
                };
                let m = metrics(drag_source.as_ref(), axis);
                let Some(thumb) = thumb_geometry(m.viewport, m.max, m.scrolled, MIN_THUMB) else {
                    return;
                };
                let start = along(axis, ev.event.position) - m.origin - grab;
                let target = scroll_for_thumb_start(start, m.viewport, m.max, thumb.length);
                scroll_to(drag_source.as_ref(), axis, target);
                window.refresh();
            },
        );

        let thumb_hover_state = state.clone();
        let press_state = state;
        let press_source = source;
        let mut thumb_el = div()
            .id((self.id, "thumb"))
            .absolute()
            .rounded(radius)
            .bg(thumb_color)
            .on_hover(move |over, window, cx| {
                thumb_hover_state.update(cx, |s, cx| {
                    s.thumb_hovered = *over;
                    cx.notify();
                });
                window.refresh();
            })
            .on_mouse_down(MouseButton::Left, move |ev: &MouseDownEvent, _, cx| {
                let m = metrics(press_source.as_ref(), axis);
                let Some(thumb) = thumb_geometry(m.viewport, m.max, m.scrolled, MIN_THUMB) else {
                    return;
                };
                let grab = along(axis, ev.position) - m.origin - thumb.start;
                press_source.drag_started();
                press_state.update(cx, |s, cx| {
                    s.grab = Some(grab);
                    cx.notify();
                });
                cx.stop_propagation();
            })
            .on_drag(ScrollbarDrag, |_, _, _, cx| cx.new(|_| ScrollbarDrag));
        thumb_el = if vertical {
            thumb_el
                .top(px(thumb.start))
                .h(px(thumb.length))
                .right(px(INSET))
                .w(px(thickness))
        } else {
            thumb_el
                .left(px(thumb.start))
                .w(px(thumb.length))
                .bottom(px(INSET))
                .h(px(thickness))
        };
        root.child(thumb_el)
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{assert_a11y_node, open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{point, TestAppContext};

    use super::*;

    fn geo(viewport: f32, max: f32, scrolled: f32) -> ThumbGeometry {
        thumb_geometry(viewport, max, scrolled, 24.0).unwrap()
    }

    #[test]
    fn nothing_to_scroll_has_no_thumb() {
        assert_eq!(thumb_geometry(300.0, 0.0, 0.0, 24.0), None);
        assert_eq!(thumb_geometry(0.0, 100.0, 0.0, 24.0), None);
        assert_eq!(thumb_geometry(f32::NAN, 100.0, 0.0, 24.0), None);
    }

    #[test]
    fn thumb_length_is_the_visible_fraction() {
        // Content is 4x the viewport: the thumb covers a quarter of the track.
        let g = geo(200.0, 600.0, 0.0);
        assert_eq!(g.length, 50.0);
        assert_eq!(g.start, 0.0);
    }

    #[test]
    fn thumb_length_has_a_floor_and_a_ceiling() {
        assert_eq!(geo(200.0, 1_000_000.0, 0.0).length, 24.0);
        assert_eq!(thumb_geometry(10.0, 5.0, 0.0, 24.0).unwrap().length, 10.0);
    }

    #[test]
    fn thumb_position_tracks_scroll_and_clamps() {
        let end = geo(200.0, 600.0, 600.0);
        assert_eq!(end.start + end.length, 200.0);
        assert_eq!(geo(200.0, 600.0, 300.0).start, 75.0);
        assert_eq!(geo(200.0, 600.0, -50.0).start, 0.0);
        assert_eq!(geo(200.0, 600.0, 9_000.0).start, 150.0);
    }

    #[test]
    fn thumb_start_maps_back_to_scroll_distance() {
        for scrolled in [0.0, 120.0, 300.0, 600.0] {
            let g = geo(200.0, 600.0, scrolled);
            let back = scroll_for_thumb_start(g.start, 200.0, 600.0, g.length);
            assert!((back - scrolled).abs() < 1e-3, "{scrolled} -> {back}");
        }
        assert_eq!(scroll_for_thumb_start(-30.0, 200.0, 600.0, 50.0), 0.0);
        assert_eq!(scroll_for_thumb_start(500.0, 200.0, 600.0, 50.0), 600.0);
        // A thumb as long as the track cannot move.
        assert_eq!(scroll_for_thumb_start(10.0, 200.0, 600.0, 200.0), 0.0);
    }

    #[test]
    fn track_click_pages_towards_the_click() {
        let g = geo(200.0, 600.0, 200.0);
        assert_eq!(page_target(5.0, g, 200.0, 200.0, 600.0), 0.0);
        assert_eq!(page_target(190.0, g, 200.0, 200.0, 600.0), 400.0);
        // Clicking the thumb itself does not move.
        let mid = g.start + 1.0;
        assert_eq!(page_target(mid, g, 200.0, 200.0, 600.0), 200.0);
    }

    #[test]
    fn paging_stops_at_the_ends() {
        let near_end = geo(200.0, 600.0, 500.0);
        assert_eq!(page_target(199.0, near_end, 500.0, 200.0, 600.0), 600.0);
        let near_start = geo(200.0, 600.0, 50.0);
        assert_eq!(page_target(0.0, near_start, 50.0, 200.0, 600.0), 0.0);
    }

    #[test]
    fn auto_hide_holds_then_fades_then_hides() {
        assert_eq!(auto_hide_opacity(Duration::ZERO), 1.0);
        assert_eq!(auto_hide_opacity(Duration::from_millis(700)), 1.0);
        let fading = auto_hide_opacity(Duration::from_millis(875));
        assert!(fading > 0.0 && fading < 1.0, "{fading}");
        assert_eq!(auto_hide_opacity(HIDE_AFTER), 0.0);
        assert_eq!(auto_hide_opacity(Duration::from_secs(60)), 0.0);
    }

    #[test]
    fn scroll_handle_source_converts_axes() {
        let handle = ScrollHandle::new();
        handle.set_offset(point(px(-5.0), px(-40.0)));
        scroll_to(&handle, ScrollbarAxis::Vertical, 10.0);
        assert_eq!(handle.offset(), point(px(-5.0), px(-10.0)));
        scroll_to(&handle, ScrollbarAxis::Horizontal, 20.0);
        assert_eq!(handle.offset(), point(px(-20.0), px(-10.0)));
    }

    struct Host {
        handle: ScrollHandle,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .relative()
                .size(px(200.0))
                .child(
                    div()
                        .id("scroller")
                        .size_full()
                        .overflow_y_scroll()
                        .track_scroll(&self.handle)
                        .child(div().h(px(800.0)).w_full()),
                )
                .child(Scrollbar::new("bar", self.handle.clone()).always_visible())
        }
    }

    #[gpui::test]
    fn track_click_pages_the_scroll_handle(cx: &mut TestAppContext) {
        let handle = ScrollHandle::new();
        let h = handle.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| Host { handle: h });
        cx.update(|w, a| w.draw(a).clear(a));
        assert_eq!(handle.max_offset().y, px(600.0));
        let bar = cx.selector_bounds("bar").expect("scrollbar is laid out");
        // The bottom of the track is below the thumb, so it pages forward.
        let bottom = point(bar.center().x, bar.bottom() - px(2.0));
        cx.click_at(bottom, MouseButton::Left, Default::default());
        assert_eq!(handle.offset().y, px(-200.0));
        cx.click_at(bottom, MouseButton::Left, Default::default());
        assert_eq!(handle.offset().y, px(-400.0));
        let snap = cx.accessibility_snapshot();
        assert_a11y_node(&snap, "ScrollBar", None, &["orientation=Vertical"]);
    }
}
