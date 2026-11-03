use gpui::{
    list, px, AnyElement, App, ElementId, IntoElement, ListAlignment, ListSizingBehavior,
    ListState, RenderOnce, Styled as _, Window,
};

type RowBuilder = Box<dyn FnMut(usize, &mut Window, &mut App) -> AnyElement + 'static>;

/// Default extra space rendered above and below the viewport.
pub const DEFAULT_OVERSCAN: f32 = 96.0;

struct VirtualListState {
    list: ListState,
    count: usize,
    revealed: Option<usize>,
    /// The first layout discards size hints, so the estimate is applied again
    /// once the list has a viewport.
    hinted: bool,
}

/// A lazily rendered vertical list over an item count and a row builder.
///
/// Only the rows that intersect the viewport, plus the `overscan` band above
/// and below it, are built. Rows may have different heights: each built row is
/// measured, and `row_height` is only the estimate used for rows not built yet
/// (it sizes the scroll range). Scroll position is kept across frames by the
/// element id.
#[derive(IntoElement)]
pub struct VirtualList {
    id: ElementId,
    count: usize,
    build_row: RowBuilder,
    estimated_row_height: f32,
    overscan: f32,
    reveal: Option<usize>,
}

impl VirtualList {
    /// Public API item.
    pub fn new(
        id: impl Into<ElementId>,
        count: usize,
        build_row: impl FnMut(usize, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            count,
            build_row: Box::new(build_row),
            estimated_row_height: 32.0,
            overscan: DEFAULT_OVERSCAN,
            reveal: None,
        }
    }

    /// Height used for rows that have not been built yet. For uniform rows
    /// this is the row height.
    pub fn row_height(mut self, height: f32) -> Self {
        self.estimated_row_height = height.max(1.0);
        self
    }

    /// Extra pixels built above and below the viewport.
    pub fn overscan(mut self, overscan: f32) -> Self {
        self.overscan = overscan.max(0.0);
        self
    }

    /// Scrolls so row `index` is fully visible when this value changes
    /// between frames (for example after keyboard navigation).
    pub fn reveal(mut self, index: Option<usize>) -> Self {
        self.reveal = index;
        self
    }
}

impl RenderOnce for VirtualList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (count, estimate, overscan) = (self.count, self.estimated_row_height, self.overscan);
        let state = window.use_keyed_state(self.id, cx, |_, _| VirtualListState {
            list: ListState::new(count, ListAlignment::Top, px(overscan)),
            count,
            revealed: None,
            hinted: false,
        });
        let (list_state, settling) = state.update(cx, |state, _| {
            if state.count != count {
                state.list.reset_with_uniform_height(count, px(estimate));
                state.count = count;
                state.revealed = None;
            }
            if !state.hinted && state.list.viewport_bounds().size.height > px(0.) {
                state.list.reset_with_uniform_height(count, px(estimate));
                state.hinted = true;
            }
            // Scroll positions are only meaningful once estimates are in place.
            if state.hinted && self.reveal != state.revealed {
                if let Some(ix) = self.reveal.filter(|ix| *ix < count) {
                    state.list.scroll_to_reveal_item(ix);
                }
                state.revealed = self.reveal;
            }
            (state.list.clone(), !state.hinted)
        });
        if settling {
            window.request_animation_frame();
        }
        list(list_state, self.build_row)
            .with_sizing_behavior(ListSizingBehavior::Auto)
            .size_full()
    }
}
