use std::sync::Arc;

use gpui::{
    list, px, AnyElement, App, ElementId, FocusHandle, InteractiveElement, IntoElement,
    KeyDownEvent, ListAlignment, ListSizingBehavior, ListState, ParentElement, RenderOnce, Role,
    SharedString, StatefulInteractiveElement, Styled, Window,
};

use crate::virtual_list::DEFAULT_OVERSCAN;

/// A group in a [`GroupedVirtualList`].
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VirtualListGroup {
    /// Stable group identifier.
    pub id: SharedString,
    /// Text announced and shown in the group header.
    pub label: SharedString,
    /// Number of rows in this group.
    pub item_count: usize,
    /// Whether the group starts expanded.
    pub expanded: bool,
}

impl VirtualListGroup {
    /// Creates an expanded group.
    pub fn new(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        item_count: usize,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            item_count,
            expanded: true,
        }
    }

    /// Sets the initial expansion state.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
}

type RowBuilder = Box<dyn FnMut(usize, usize, &mut Window, &mut App) -> AnyElement + 'static>;
type HeaderBuilder =
    Arc<dyn Fn(usize, &VirtualListGroup, bool, &mut Window, &mut App) -> AnyElement + 'static>;

/// A virtual list partitioned into collapsible groups with sticky headers.
///
/// Group headers are exposed as `group` nodes and rows as `list_item` nodes.
/// Arrow keys move between visible rows; Home and End move to the first and
/// last visible row. Enter and Space toggle the focused group header.
#[derive(IntoElement)]
pub struct GroupedVirtualList {
    id: ElementId,
    groups: Vec<VirtualListGroup>,
    build_row: RowBuilder,
    header: Option<HeaderBuilder>,
    estimated_row_height: f32,
    header_height: f32,
    overscan: f32,
}

impl GroupedVirtualList {
    /// Creates a grouped list from its ordered groups and a row builder.
    pub fn new(
        id: impl Into<ElementId>,
        groups: Vec<VirtualListGroup>,
        build_row: impl FnMut(usize, usize, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        Self {
            id: id.into(),
            groups,
            build_row: Box::new(build_row),
            header: None,
            estimated_row_height: 32.,
            header_height: 36.,
            overscan: DEFAULT_OVERSCAN,
        }
    }

    /// Supplies a custom header renderer. The callback receives group index,
    /// group data, and its current expansion state.
    pub fn header(
        mut self,
        header: impl Fn(usize, &VirtualListGroup, bool, &mut Window, &mut App) -> AnyElement + 'static,
    ) -> Self {
        self.header = Some(Arc::new(header));
        self
    }
    /// Sets the estimated row height.
    pub fn row_height(mut self, height: f32) -> Self {
        self.estimated_row_height = height.max(1.);
        self
    }
    /// Sets the header height estimate.
    pub fn header_height(mut self, height: f32) -> Self {
        self.header_height = height.max(1.);
        self
    }
    /// Sets list overscan.
    pub fn overscan(mut self, overscan: f32) -> Self {
        self.overscan = overscan.max(0.);
        self
    }
}

struct GroupedState {
    list: ListState,
    expanded: Vec<bool>,
    focus: Vec<FocusHandle>,
    count: usize,
    hinted: bool,
    pending_focus: Option<usize>,
    pending_frames: u8,
}

/// Frames a keyboard-navigation target may take to render before the request
/// is abandoned, so an unreachable target cannot animate forever.
const SETTLE_FRAMES: u8 = 3;

/// Advances the pending keyboard-focus request by one render. Returns the item
/// to reveal and whether another frame is needed. The target is clamped to the
/// current count and dropped when the viewport is unmeasured, the list is
/// empty, or the frame budget is spent.
fn settle_pending(
    pending: &mut Option<usize>,
    frames: &mut u8,
    count: usize,
    measured: bool,
) -> (Option<usize>, bool) {
    let Some(target) = *pending else {
        return (None, false);
    };
    if count == 0 || !measured || *frames == 0 {
        *pending = None;
        *frames = 0;
        return (None, false);
    }
    let target = target.min(count - 1);
    *pending = Some(target);
    *frames -= 1;
    (Some(target), true)
}

#[cfg(test)]
thread_local! {
    static FRAME_REQUESTS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static TEST_STATE: std::cell::RefCell<Option<gpui::Entity<GroupedState>>> =
        const { std::cell::RefCell::new(None) };
}

impl RenderOnce for GroupedVirtualList {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let groups = self.groups;
        let group_count = groups.len();
        let expanded_defaults: Vec<bool> = groups.iter().map(|g| g.expanded).collect();
        let estimate = self.estimated_row_height;
        let header_height = self.header_height;
        let overscan = self.overscan;
        let header = self.header;
        let state = window.use_keyed_state(self.id.clone(), cx, |_, _| GroupedState {
            list: ListState::new(0, ListAlignment::Top, px(overscan)),
            expanded: expanded_defaults,
            focus: Vec::new(),
            count: 0,
            hinted: false,
            pending_focus: None,
            pending_frames: 0,
        });
        #[cfg(test)]
        TEST_STATE.with(|t| *t.borrow_mut() = Some(state.clone()));
        let (list_state, expanded, focus, count, settling) = state.update(cx, |s, cx| {
            if s.expanded.len() != group_count {
                s.expanded = groups.iter().map(|g| g.expanded).collect();
            }
            let count = groups
                .iter()
                .enumerate()
                .map(|(i, g)| 1 + if s.expanded[i] { g.item_count } else { 0 })
                .sum();
            if s.count != count {
                s.list.reset_with_uniform_height(count, px(estimate));
                s.count = count;
                s.hinted = false;
            }
            if !s.hinted && s.list.viewport_bounds().size.height > px(0.) {
                s.list.reset_with_uniform_height(count, px(estimate));
                s.hinted = true;
            }
            let measured = s.hinted;
            let (reveal, settling) =
                settle_pending(&mut s.pending_focus, &mut s.pending_frames, count, measured);
            if let Some(target) = reveal {
                s.list.scroll_to_reveal_item(target);
            }
            while s.focus.len() < count {
                s.focus.push(cx.focus_handle());
            }
            s.focus.truncate(count);
            (
                s.list.clone(),
                s.expanded.clone(),
                s.focus.clone(),
                count,
                settling,
            )
        });
        if settling {
            #[cfg(test)]
            FRAME_REQUESTS.with(|c| c.set(c.get() + 1));
            window.request_animation_frame();
        }
        let mut offsets = Vec::with_capacity(group_count);
        let mut ix = 0;
        for (i, g) in groups.iter().enumerate() {
            offsets.push(ix);
            ix += 1 + if expanded[i] { g.item_count } else { 0 };
        }
        let mut row_builder = self.build_row;
        let header_for_rows = header.clone();
        let groups_for_rows = groups.clone();
        let state_for_rows = state.clone();
        let expanded_for_rows = expanded.clone();
        let focus_for_rows = focus.clone();
        let list_for_rows = list_state.clone();
        let build = move |flat: usize, w: &mut Window, app: &mut App| {
            let (gi, item) = locate(&groups_for_rows, &expanded_for_rows, flat);
            let focus_handle = focus_for_rows[flat].clone();
            let navigation_focus = focus_for_rows.clone();
            let navigation_list = list_for_rows.clone();
            let navigation_state = state_for_rows.clone();
            let navigate = move |event: &KeyDownEvent, _window: &mut Window, cx: &mut App| {
                let target = match event.keystroke.key.as_str() {
                    "up" => flat.checked_sub(1),
                    "down" => (flat + 1 < navigation_focus.len()).then_some(flat + 1),
                    "home" => (!navigation_focus.is_empty()).then_some(0),
                    "end" => navigation_focus.len().checked_sub(1),
                    _ => None,
                };
                if let Some(target) = target {
                    navigation_state.update(cx, |state, cx| {
                        state.pending_focus = Some(target);
                        state.pending_frames = SETTLE_FRAMES;
                        cx.notify();
                    });
                    navigation_list.scroll_to_reveal_item(target);
                }
            };
            let pending_focus = state_for_rows.read(app).pending_focus;
            if pending_focus == Some(flat) {
                focus_handle.focus(w, app);
                state_for_rows.update(app, |state, cx| {
                    if state.pending_focus == Some(flat) {
                        state.pending_focus = None;
                        cx.notify();
                    }
                });
            }
            if item.is_none() {
                let group = &groups_for_rows[gi];
                let expanded_now = expanded_for_rows[gi];
                let state = state_for_rows.clone();
                let key_state = state.clone();
                let default = gpui::div()
                    .id(("group-header", gi as u64))
                    .debug_selector(move || format!("group-header-{gi}"))
                    .role(Role::Group)
                    .aria_label(group.label.clone())
                    .aria_expanded(expanded_now)
                    .track_focus(&focus_handle)
                    .tab_index(if flat == 0 { 0 } else { -1 })
                    .on_click(move |_, _, cx| {
                        state.update(cx, |s, cx| {
                            s.expanded[gi] = !s.expanded[gi];
                            s.pending_focus = None;
                            cx.notify();
                        });
                    })
                    .on_key_down(move |event: &KeyDownEvent, window, cx| {
                        if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                            key_state.update(cx, |s, cx| {
                                s.expanded[gi] = !s.expanded[gi];
                                s.pending_focus = None;
                                cx.notify();
                            });
                        } else {
                            navigate(event, window, cx);
                        }
                    });
                if let Some(ref make) = header_for_rows {
                    default
                        .h(px(header_height))
                        .child(make(gi, group, expanded_now, w, app))
                        .into_any_element()
                } else {
                    default
                        .child(default_header(group, expanded_now, header_height))
                        .into_any_element()
                }
            } else if let Some(item) = item {
                gpui::div()
                    .id(("group-row", flat as u64))
                    .debug_selector(move || format!("group-row-{flat}"))
                    .role(Role::ListItem)
                    .track_focus(&focus_handle)
                    .tab_index(-1)
                    .on_key_down(navigate)
                    .child(row_builder(gi, item, w, app))
                    .into_any_element()
            } else {
                gpui::div().into_any_element()
            }
        };
        let list_element = list(list_state, build)
            .with_sizing_behavior(ListSizingBehavior::Auto)
            .size_full();
        let mut root = gpui::div().relative().size_full().child(list_element);
        // The first visible group header is repeated at the viewport edge. It
        // has no accessibility role, so the source header remains the only
        // group node and keyboard/accessibility order stays deterministic.
        if let Some(gi) =
            sticky_group_index(&offsets, state.read(cx).list.logical_scroll_top().item_ix)
        {
            let g = groups[gi].clone();
            let expanded_now = expanded[gi];
            let sticky_header = header.clone();
            root = root.child(
                gpui::div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(header_height))
                    .id(("sticky-group-header", gi as u64))
                    .debug_selector(move || format!("sticky-group-header-{gi}"))
                    .child(if let Some(make) = sticky_header {
                        make(gi, &g, expanded_now, window, cx)
                    } else {
                        default_header(&g, expanded_now, header_height)
                    }),
            );
        }
        let _ = count;
        root
    }
}

fn sticky_group_index(offsets: &[usize], item: usize) -> Option<usize> {
    offsets
        .iter()
        .enumerate()
        .rev()
        .find(|(_, offset)| **offset <= item)
        .map(|(index, _)| index)
}

fn default_header(group: &VirtualListGroup, expanded: bool, header_height: f32) -> AnyElement {
    gpui::div()
        .h(px(header_height))
        .flex()
        .items_center()
        .gap_2()
        .px_3()
        .child(if expanded { "▾" } else { "▸" })
        .child(group.label.clone())
        .child(format!("({})", group.item_count))
        .into_any_element()
}

fn locate(groups: &[VirtualListGroup], expanded: &[bool], flat: usize) -> (usize, Option<usize>) {
    let mut at = 0;
    for (i, g) in groups.iter().enumerate() {
        if flat == at {
            return (i, None);
        }
        at += 1;
        if expanded[i] {
            if flat < at + g.item_count {
                return (i, Some(flat - at));
            }
            at += g.item_count;
        }
    }
    (groups.len().saturating_sub(1), Some(0))
}

#[cfg(test)]
fn flattened_count(groups: &[VirtualListGroup], expanded: &[bool]) -> usize {
    groups
        .iter()
        .enumerate()
        .map(|(i, group)| 1 + if expanded[i] { group.item_count } else { 0 })
        .sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flattening_skips_collapsed_items() {
        let groups = vec![
            VirtualListGroup::new("a", "A", 3),
            VirtualListGroup::new("b", "B", 2),
        ];
        assert_eq!(locate(&groups, &[true, false], 0), (0, None));
        assert_eq!(locate(&groups, &[true, false], 3), (0, Some(2)));
        assert_eq!(locate(&groups, &[true, false], 4), (1, None));
    }
    #[test]
    fn sticky_header_tracks_last_group_at_or_before_row() {
        assert_eq!(sticky_group_index(&[0, 4, 8], 0), Some(0));
        assert_eq!(sticky_group_index(&[0, 4, 8], 7), Some(1));
        assert_eq!(sticky_group_index(&[0, 4, 8], 100), Some(2));
    }

    #[test]
    fn large_group_has_one_header_and_all_rows() {
        let groups = [VirtualListGroup::new("large", "Large", 100_000)];
        assert_eq!(flattened_count(&groups, &[true]), 100_001);
        assert_eq!(locate(&groups, &[true], 99_999), (0, Some(99_998)));
        assert_eq!(locate(&groups, &[true], 100_000), (0, Some(99_999)));
    }

    #[gpui::test]
    fn toggling_header_rebuilds_visible_rows(cx: &mut gpui::TestAppContext) {
        use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
        use gpui::{Context, Render};

        struct Host;
        impl Render for Host {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                GroupedVirtualList::new(
                    "test-grouped-list",
                    vec![VirtualListGroup::new("a", "Alpha", 2)],
                    |_, row, _, _| {
                        gpui::div()
                            .h(px(24.))
                            .child(format!("row {row}"))
                            .into_any_element()
                    },
                )
            }
        }

        let (_, cx) = open_fluent_test_window(cx, |_, _| Host);
        assert!(cx.selector_bounds("group-row-1").is_some());
        assert!(cx.selector_bounds("group-row-2").is_some());
        cx.click_selector("group-header-0");
        assert!(cx.selector_bounds("group-row-1").is_none());
        assert!(cx.selector_bounds("group-row-2").is_none());
    }

    #[gpui::test]
    fn header_enter_toggles_group(cx: &mut gpui::TestAppContext) {
        use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
        use gpui::{Context, Render};

        struct Host;
        impl Render for Host {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                GroupedVirtualList::new(
                    "test-keyboard-grouped-list",
                    vec![VirtualListGroup::new("a", "Alpha", 1)],
                    |_, _, _, _| gpui::div().h(px(24.)).into_any_element(),
                )
            }
        }

        let (_, cx) = open_fluent_test_window(cx, |_, _| Host);
        cx.click_selector("group-header-0");
        cx.type_keys("enter");
        assert!(cx.selector_bounds("group-row-1").is_some());
    }

    fn state() -> gpui::Entity<GroupedState> {
        TEST_STATE.with(|t| t.borrow().clone().expect("list rendered"))
    }

    fn frames() -> usize {
        FRAME_REQUESTS.with(|c| c.get())
    }

    fn focused_index(cx: &mut gpui::VisualTestContext) -> Option<usize> {
        let state = state();
        cx.update(|window, app| {
            state
                .read(app)
                .focus
                .iter()
                .position(|f| f.is_focused(window))
        })
    }

    fn settle(cx: &mut gpui::VisualTestContext) {
        for _ in 0..8 {
            cx.run_until_parked();
        }
    }

    struct SharedHost {
        id: &'static str,
        items: std::rc::Rc<std::cell::Cell<usize>>,
    }
    impl gpui::Render for SharedHost {
        fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
            GroupedVirtualList::new(
                self.id,
                vec![VirtualListGroup::new("a", "Alpha", self.items.get())],
                |_, row, _, _| {
                    gpui::div()
                        .h(px(24.))
                        .child(format!("row {row}"))
                        .into_any_element()
                },
            )
        }
    }

    #[gpui::test]
    fn keyboard_navigation_moves_focus_and_reveals_end(cx: &mut gpui::TestAppContext) {
        use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};

        let items = std::rc::Rc::new(std::cell::Cell::new(10_000));
        let host = items.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| SharedHost {
            id: "test-keyboard-navigation",
            items: host,
        });
        let first = state().read_with(cx, |s, _| s.focus[0].clone());
        cx.update(|window, app| first.focus(window, app));
        settle(cx);
        assert_eq!(focused_index(cx), Some(0), "header focused");

        cx.type_keys("up");
        settle(cx);
        assert_eq!(focused_index(cx), Some(0), "up at top keeps focus");
        cx.type_keys("down");
        settle(cx);
        assert_eq!(focused_index(cx), Some(1));
        cx.type_keys("down");
        settle(cx);
        assert_eq!(focused_index(cx), Some(2));
        cx.type_keys("up");
        settle(cx);
        assert_eq!(focused_index(cx), Some(1));
        cx.type_keys("home");
        settle(cx);
        assert_eq!(focused_index(cx), Some(0), "home focuses first header");

        cx.type_keys("end");
        settle(cx);
        assert_eq!(focused_index(cx), Some(10_000), "end focuses last row");
        assert!(
            cx.selector_bounds("group-row-10000").is_some(),
            "last row is rendered"
        );
        cx.type_keys("down");
        settle(cx);
        assert_eq!(
            focused_index(cx),
            Some(10_000),
            "down at bottom keeps focus"
        );
        cx.type_keys("up");
        settle(cx);
        assert_eq!(focused_index(cx), Some(9_999));
    }

    #[gpui::test]
    fn unreachable_target_stops_requesting_frames(cx: &mut gpui::TestAppContext) {
        use fluent_test_utils::open_fluent_test_window;

        let items = std::rc::Rc::new(std::cell::Cell::new(5));
        let host = items.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| SharedHost {
            id: "test-shrink-pending",
            items: host,
        });
        settle(cx);
        // Shrink the list below a pending target and re-render.
        let state = state();
        items.set(2);
        cx.update(|_, app| {
            state.update(app, |s, cx| {
                s.pending_focus = Some(5);
                s.pending_frames = SETTLE_FRAMES;
                cx.notify();
            });
        });
        settle(cx);
        let after = frames();
        settle(cx);
        assert_eq!(frames(), after, "no frames requested once settled");
        cx.update(|_, app| assert_eq!(state.read(app).pending_focus, None));
    }

    #[gpui::test]
    fn unmeasured_viewport_does_not_request_frames(cx: &mut gpui::TestAppContext) {
        use fluent_test_utils::open_fluent_test_window;
        use gpui::{Context, Render};

        struct ZeroHeight;
        impl Render for ZeroHeight {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                gpui::div().h(px(0.)).child(GroupedVirtualList::new(
                    "test-zero-height",
                    vec![VirtualListGroup::new("a", "Alpha", 5)],
                    |_, _, _, _| gpui::div().h(px(24.)).into_any_element(),
                ))
            }
        }

        let (_, cx) = open_fluent_test_window(cx, |_, _| ZeroHeight);
        settle(cx);
        let state = state();
        let before = frames();
        cx.update(|_, app| {
            state.update(app, |s, cx| {
                s.pending_focus = Some(3);
                s.pending_frames = SETTLE_FRAMES;
                cx.notify();
            });
        });
        settle(cx);
        assert_eq!(frames(), before, "unmeasured viewport requests no frames");
        cx.update(|_, app| assert_eq!(state.read(app).pending_focus, None));
    }

    #[gpui::test]
    fn collapse_while_pending_clears_target(cx: &mut gpui::TestAppContext) {
        use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};

        let items = std::rc::Rc::new(std::cell::Cell::new(5));
        let host = items.clone();
        let (_, cx) = open_fluent_test_window(cx, move |_, _| SharedHost {
            id: "test-collapse-pending",
            items: host,
        });
        settle(cx);
        let state = state();
        cx.update(|_, app| {
            state.update(app, |s, cx| {
                s.pending_focus = Some(3);
                s.pending_frames = SETTLE_FRAMES;
                cx.notify();
            });
        });
        cx.click_selector("group-header-0");
        settle(cx);
        let after = frames();
        settle(cx);
        assert_eq!(frames(), after, "no frames requested after collapse");
        cx.update(|_, app| assert_eq!(state.read(app).pending_focus, None));
        assert!(cx.selector_bounds("group-row-3").is_none());
    }

    #[test]
    fn settle_pending_is_bounded_and_clamped() {
        let (mut p, mut f) = (Some(50), SETTLE_FRAMES);
        assert_eq!(settle_pending(&mut p, &mut f, 10, true), (Some(9), true));
        assert_eq!(p, Some(9));
        assert_eq!(settle_pending(&mut p, &mut f, 10, true), (Some(9), true));
        assert_eq!(settle_pending(&mut p, &mut f, 10, true), (Some(9), true));
        assert_eq!(settle_pending(&mut p, &mut f, 10, true), (None, false));
        assert_eq!(p, None);
        let (mut p, mut f) = (Some(1), SETTLE_FRAMES);
        assert_eq!(settle_pending(&mut p, &mut f, 10, false), (None, false));
        assert_eq!(p, None);
        let (mut p, mut f) = (Some(1), SETTLE_FRAMES);
        assert_eq!(settle_pending(&mut p, &mut f, 0, true), (None, false));
        assert_eq!(p, None);
    }
}
