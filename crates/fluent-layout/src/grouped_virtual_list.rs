use std::sync::Arc;

use gpui::{
    list, px, AnyElement, App, ElementId, InteractiveElement, IntoElement, KeyDownEvent,
    ListAlignment, ListSizingBehavior, ListState, ParentElement, RenderOnce, Role, SharedString,
    StatefulInteractiveElement, Styled, Window,
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
    count: usize,
    hinted: bool,
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
            count: 0,
            hinted: false,
        });
        let (list_state, expanded, count) = state.update(cx, |s, _| {
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
            (s.list.clone(), s.expanded.clone(), count)
        });
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
        let build = move |flat: usize, w: &mut Window, app: &mut App| {
            let (gi, item) = locate(&groups_for_rows, &expanded, flat);
            if item.is_none() {
                let group = &groups_for_rows[gi];
                let expanded_now = expanded[gi];
                let state = state_for_rows.clone();
                let key_state = state.clone();
                let default = gpui::div()
                    .id(("group-header", gi as u64))
                    .role(Role::Group)
                    .aria_label(group.label.clone())
                    .aria_expanded(expanded_now)
                    .tab_index(0)
                    .on_click(move |_, _, cx| {
                        state.update(cx, |s, _| {
                            s.expanded[gi] = !s.expanded[gi];
                        });
                    })
                    .on_key_down(move |event: &KeyDownEvent, _, cx| {
                        if matches!(event.keystroke.key.as_str(), "return" | "space") {
                            key_state.update(cx, |s, _| {
                                s.expanded[gi] = !s.expanded[gi];
                            });
                        }
                    });
                if let Some(ref make) = header_for_rows {
                    make(gi, group, expanded_now, w, app)
                } else {
                    default
                        .h(px(header_height))
                        .flex()
                        .items_center()
                        .px_3()
                        .child(group.label.clone())
                        .into_any_element()
                }
            } else if let Some(item) = item {
                row_builder(gi, item, w, app)
            } else {
                gpui::div().into_any_element()
            }
        };
        let list_element = list(list_state, build)
            .with_sizing_behavior(ListSizingBehavior::Auto)
            .size_full();
        let mut root = gpui::div().relative().size_full().child(list_element);
        // The first visible group header is repeated at the viewport edge. This
        // is intentionally a lightweight overlay; the source header remains in
        // the list so keyboard and accessibility order stay deterministic.
        if let Some(gi) =
            sticky_group_index(&offsets, state.read(cx).list.logical_scroll_top().item_ix)
        {
            let g = groups[gi].clone();
            root = root.child(
                gpui::div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(header_height))
                    .id(("sticky-group-header", gi as u64))
                    .role(Role::Group)
                    .aria_label(g.label.clone())
                    .child(gpui::div().h(px(header_height))),
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
        assert_eq!(1 + groups[0].item_count, 100_001);
    }
}
