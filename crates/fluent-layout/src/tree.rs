use fluent_core::TextRampExt;
use std::sync::Arc;

use crate::data_status::{status_block, DataStatusKind};
use fluent_core::{ThemeProvider as _, TruncateExt as _};
use gpui::{
    div, prelude::*, px, svg, App, ClickEvent, IntoElement, KeyDownEvent, MouseButton,
    MouseDownEvent, RenderOnce, Role, SharedString, Window,
};

type TreeSelectHandler = Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>;
type TreeActivateHandler = Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>;
type TreeToggleHandler =
    Arc<dyn Fn(SharedString, bool, &ClickEvent, &mut Window, &mut App) + 'static>;
type TreeContextHandler =
    Arc<dyn Fn(SharedString, &MouseDownEvent, &mut Window, &mut App) + 'static>;
type TreeCheckHandler =
    Arc<dyn Fn(SharedString, bool, &ClickEvent, &mut Window, &mut App) + 'static>;
type TreeLoadHandler = Arc<dyn Fn(SharedString, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
struct FlatTreeItem {
    id: SharedString,
    disabled: bool,
    expanded: bool,
    has_children: bool,
}

#[derive(Clone)]
struct TreeRenderState {
    selected_id: Option<SharedString>,
    on_select: Option<TreeSelectHandler>,
    on_activate: Option<TreeActivateHandler>,
    on_toggle: Option<TreeToggleHandler>,
    on_context_menu: Option<TreeContextHandler>,
    on_check: Option<TreeCheckHandler>,
    on_load_children: Option<TreeLoadHandler>,
    filter: Option<SharedString>,
}

/// Tri-state value of a row's checkbox. A folder's value is derived from the
/// leaf rows beneath it: all checked, none checked, or mixed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeCheckState {
    #[default]
    /// Public API item.
    Unchecked,
    /// Public API item.
    Checked,
    /// Public API item.
    Mixed,
}

#[derive(Clone, Debug)]
/// Public API item.
pub struct TreeItem {
    /// Public API item.
    pub id: SharedString,
    /// Public API item.
    pub label: SharedString,
    /// Public API item.
    pub icon: Option<SharedString>,
    /// Public API item.
    pub badge: Option<SharedString>,
    /// Public API item.
    pub expanded: bool,
    /// Public API item.
    pub disabled: bool,
    /// Public API item.
    pub children: Vec<TreeItem>,
    checkable: bool,
    checked: bool,
    /// Whether children are being fetched by a lazy data source.
    pub loading: bool,
    /// Whether this item can have children that are not loaded yet.
    pub lazy: bool,
}

impl TreeItem {
    /// Public API item.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            expanded: true,
            disabled: false,
            children: Vec::new(),
            checkable: false,
            checked: false,
            loading: false,
            lazy: false,
        }
    }

    /// Public API item.
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    /// Public API item.
    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    /// Public API item.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    /// Public API item.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Public API item.
    pub fn child(mut self, child: TreeItem) -> Self {
        self.children.push(child);
        self
    }

    /// Shows a checkbox slot on this row. Off by default.
    pub fn checkable(mut self, checkable: bool) -> Self {
        self.checkable = checkable;
        self
    }

    /// Initial checked value of a leaf row. Folders derive their value from
    /// their leaves and ignore this flag.
    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = checked;
        self
    }

    /// Marks this item as having children that may be loaded on demand.
    pub fn lazy(mut self, lazy: bool) -> Self {
        self.lazy = lazy;
        self
    }

    /// Shows a loading state while a lazy child request is in flight.
    pub fn loading(mut self, loading: bool) -> Self {
        self.loading = loading;
        self
    }

    /// Whether this row shows a checkbox slot.
    pub fn is_checkable(&self) -> bool {
        self.checkable
    }

    /// Tri-state checkbox value: a leaf reports its own flag, a folder reports
    /// checked when every leaf beneath it is checked, unchecked when none is,
    /// and mixed otherwise.
    pub fn check_state(&self) -> TreeCheckState {
        if self.children.is_empty() {
            return if self.checked {
                TreeCheckState::Checked
            } else {
                TreeCheckState::Unchecked
            };
        }
        let mut any_checked = false;
        let mut any_unchecked = false;
        for child in &self.children {
            match child.check_state() {
                TreeCheckState::Checked => any_checked = true,
                TreeCheckState::Unchecked => any_unchecked = true,
                TreeCheckState::Mixed => {
                    any_checked = true;
                    any_unchecked = true;
                }
            }
        }
        match (any_checked, any_unchecked) {
            (true, true) => TreeCheckState::Mixed,
            (true, false) => TreeCheckState::Checked,
            _ => TreeCheckState::Unchecked,
        }
    }

    /// Sets this row and every row beneath it to `checked`.
    fn set_checked_subtree(&mut self, checked: bool) {
        self.checked = checked;
        for child in &mut self.children {
            child.set_checked_subtree(checked);
        }
    }
}

/// Sets the checked value of the row `id` and all of its descendants, wherever
/// it sits in `items`. Returns `false` when no row has that id.
///
/// Use this from an `on_check` handler: it updates the model, and the caller
/// re-renders the tree with the new items.
pub fn set_tree_checked(items: &mut [TreeItem], id: &str, checked: bool) -> bool {
    for item in items {
        if item.id.as_ref() == id {
            item.set_checked_subtree(checked);
            return true;
        }
        if set_tree_checked(&mut item.children, id, checked) {
            return true;
        }
    }
    false
}

/// The value a click on a checkbox in `state` requests: a checked row becomes
/// unchecked; an unchecked or mixed row becomes checked.
fn next_checked(state: TreeCheckState) -> bool {
    state != TreeCheckState::Checked
}

/// A generic Fluent-style hierarchical tree.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Tree {
    items: Vec<TreeItem>,
    selected_id: Option<SharedString>,
    filter: Option<SharedString>,
    on_select: Option<TreeSelectHandler>,
    on_activate: Option<TreeActivateHandler>,
    on_toggle: Option<TreeToggleHandler>,
    on_context_menu: Option<TreeContextHandler>,
    on_check: Option<TreeCheckHandler>,
    on_load_children: Option<TreeLoadHandler>,
    status: Option<(DataStatusKind, SharedString)>,
}

impl Tree {
    fn flatten_visible(
        items: &[TreeItem],
        filter: Option<&str>,
        depth: usize,
    ) -> Vec<(TreeItem, usize)> {
        let mut out = Vec::new();
        for item in items {
            if filter.is_some_and(|query| !Self::item_matches_filter(item, query)) {
                continue;
            }
            out.push((item.clone(), depth));
            if item.expanded {
                out.extend(Self::flatten_visible(&item.children, filter, depth + 1));
            }
        }
        out
    }

    fn flatten_visible_for_keys(
        items: &[TreeItem],
        filter: Option<&str>,
        out: &mut Vec<FlatTreeItem>,
    ) {
        for item in items {
            if let Some(query) = filter {
                if !Self::item_matches_filter(item, query) {
                    continue;
                }
            }

            if !item.disabled {
                out.push(FlatTreeItem {
                    id: item.id.clone(),
                    disabled: item.disabled,
                    expanded: item.expanded,
                    has_children: !item.children.is_empty() || item.lazy,
                });
            }

            if item.expanded {
                Self::flatten_visible_for_keys(&item.children, filter, out);
            }
        }
    }

    /// Public API item.
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_id: None,
            filter: None,
            on_select: None,
            on_activate: None,
            on_toggle: None,
            on_context_menu: None,
            on_check: None,
            on_load_children: None,
            status: None,
        }
    }

    /// Show a loading, empty or error state instead of the items.
    pub fn status(mut self, kind: DataStatusKind, message: impl Into<SharedString>) -> Self {
        self.status = Some((kind, message.into()));
        self
    }

    /// Public API item.
    pub fn item(mut self, item: TreeItem) -> Self {
        self.items.push(item);
        self
    }

    /// Public API item.
    pub fn items(mut self, items: impl IntoIterator<Item = TreeItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Public API item.
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    /// Public API item.
    pub fn filter(mut self, query: impl Into<SharedString>) -> Self {
        let query = query.into();
        if !query.is_empty() {
            self.filter = Some(query);
        }
        self
    }

    /// Public API item.
    pub fn on_select(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Public API item.
    pub fn on_activate(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Arc::new(f));
        self
    }

    /// Public API item.
    pub fn on_toggle(
        mut self,
        f: impl Fn(SharedString, bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Arc::new(f));
        self
    }

    /// Public API item.
    pub fn on_context_menu(
        mut self,
        f: impl Fn(SharedString, &MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_context_menu = Some(Arc::new(f));
        self
    }

    /// Called when a row's checkbox is clicked, with the requested checked
    /// value (the new state, not the old one). Folders report the value that
    /// applies to all their leaves. Apply it with [`set_tree_checked`].
    pub fn on_check(
        mut self,
        f: impl Fn(SharedString, bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_check = Some(Arc::new(f));
        self
    }

    /// Requests children for a lazy item when it is expanded. The owner should
    /// update its model with the children and clear [`TreeItem::loading`].
    pub fn on_load_children(
        mut self,
        f: impl Fn(SharedString, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_load_children = Some(Arc::new(f));
        self
    }

    fn item_matches_filter(item: &TreeItem, query: &str) -> bool {
        let query = query.to_ascii_lowercase();
        item.label.to_ascii_lowercase().contains(&query)
            || item
                .children
                .iter()
                .any(|child| Self::item_matches_filter(child, &query))
    }

    fn render_item(
        item: TreeItem,
        depth: usize,
        index: usize,
        state: TreeRenderState,
        cx: &mut App,
    ) -> Option<gpui::AnyElement> {
        if let Some(query) = state.filter.as_ref() {
            if !Self::item_matches_filter(&item, query.as_ref()) {
                return None;
            }
        }

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let components = theme.components;
        let is_selected = state.selected_id.as_ref() == Some(&item.id);
        let has_children = !item.children.is_empty() || item.lazy;
        let fg = if item.disabled {
            colors.on_neutral_disabled
        } else {
            colors.on_neutral
        };

        let mut row = div()
            .id(("tree-item", index as u64))
            .role(Role::TreeItem)
            .aria_label(item.label.clone())
            .aria_level(depth + 1)
            .aria_selected(is_selected)
            .when(has_children, |d| d.aria_expanded(item.expanded))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.xs))
            .h(px(components.list_item_height))
            .pl(px(spacing.md + depth as f32 * 16.0))
            .pr(px(spacing.md))
            .bg(if is_selected {
                colors.neutral_selected
            } else {
                colors.surface
            })
            .text_ramp(&typography.body)
            .text_color(fg);

        if !item.disabled {
            row = row.cursor_pointer().hover({
                let hover = colors.neutral_hover;
                move |s| {
                    s.bg(if is_selected {
                        colors.neutral_selected
                    } else {
                        hover
                    })
                }
            });

            if state.on_select.is_some() || state.on_activate.is_some() {
                let id = item.id.clone();
                let select_handler = state.on_select.clone();
                let activate_handler = state.on_activate.clone();
                row = row.on_click(move |ev, win, app| {
                    if let Some(handler) = &select_handler {
                        handler(id.clone(), ev, win, app);
                    }
                    if ev.click_count() >= 2 {
                        if let Some(handler) = &activate_handler {
                            handler(id.clone(), ev, win, app);
                        }
                    }
                });
            }
        }

        if !item.disabled {
            if let Some(handler) = state.on_context_menu.clone() {
                let id = item.id.clone();
                row = row.on_mouse_down(MouseButton::Right, move |ev, win, app| {
                    handler(id.clone(), ev, win, app);
                });
            }
        }

        let toggle_id = item.id.clone();
        let toggle_handler = state.on_toggle.clone();
        let load_handler = state.on_load_children.clone();
        let chevron = if has_children {
            let path = if item.expanded {
                "icons/chevron_down.svg"
            } else {
                "icons/chevron_right.svg"
            };
            svg()
                .path(path)
                .size(px(12.0))
                .text_color(colors.on_subtle)
                .into_any_element()
        } else {
            div().size(px(12.0)).into_any_element()
        };
        row = row.child(
            div()
                .id(("tree-toggle", index as u64))
                .flex_none()
                .size(px(components.popup_icon_slot))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .on_click(move |ev, win, app| {
                    if has_children {
                        if let Some(handler) = &toggle_handler {
                            handler(toggle_id.clone(), !item.expanded, ev, win, app);
                        }
                        if item.lazy && !item.expanded && !item.loading {
                            if let Some(handler) = &load_handler {
                                handler(toggle_id.clone(), win, app);
                            }
                        }
                    }
                })
                .child(chevron),
        );

        if item.checkable {
            let check_state = item.check_state();
            let filled = check_state != TreeCheckState::Unchecked;
            let box_disabled = item.disabled;
            let box_fill = if filled {
                if box_disabled {
                    colors.accent_disabled
                } else {
                    colors.accent
                }
            } else {
                colors.surface
            };
            let box_border = if filled {
                colors.accent
            } else if box_disabled {
                colors.stroke_neutral_disabled
            } else {
                colors.stroke_neutral
            };
            let mark_color = if box_disabled {
                colors.on_accent_disabled
            } else {
                colors.on_accent
            };
            let radius = theme.radii.sm;
            let check_id = item.id.clone();
            let check_handler = state.on_check.clone();
            let next_value = next_checked(check_state);
            let mut check_box = div()
                .id(("tree-check", index as u64))
                .flex_none()
                .size(px(16.0))
                .rounded(px(radius))
                .border_1()
                .border_color(box_border)
                .bg(box_fill)
                .flex()
                .items_center()
                .justify_center();
            if !box_disabled {
                check_box = check_box.cursor_pointer().hover({
                    let hover_border = colors.stroke_accent;
                    move |s| s.border_color(hover_border)
                });
                if let Some(handler) = check_handler {
                    check_box = check_box.on_click(move |ev, win, app| {
                        handler(check_id.clone(), next_value, ev, win, app);
                        // The row itself selects on click; a checkbox press should not.
                        app.stop_propagation();
                    });
                }
            }
            let mark = match check_state {
                TreeCheckState::Checked => svg()
                    .path("icons/checkmark.svg")
                    .size(px(12.0))
                    .text_color(mark_color)
                    .into_any_element(),
                TreeCheckState::Mixed => div()
                    .w(px(8.0))
                    .h(px(2.0))
                    .rounded(px(radius))
                    .bg(mark_color)
                    .into_any_element(),
                TreeCheckState::Unchecked => div().into_any_element(),
            };
            row = row.child(check_box.child(mark));
        }

        let mut content = div()
            .id(("tree-content", index as u64))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.xs))
            .flex_1()
            .h_full();

        if let Some(icon) = item.icon.clone() {
            content = content.child(
                svg()
                    .path(icon)
                    .size(px(components.popup_icon_slot))
                    .text_color(fg),
            );
        }

        content = content.child(div().truncate_label().child(item.label));
        if item.loading {
            content = content.child(div().text_color(colors.on_subtle).child("Loading…"));
        }
        if let Some(badge) = item.badge {
            content = content.child(
                div()
                    .flex_none()
                    .px(px(4.0))
                    .rounded(px(3.0))
                    .bg(colors.neutral)
                    .text_size(px(10.0))
                    .text_color(colors.on_subtle)
                    .child(badge),
            );
        }
        // Children are separate rows in the flattened virtual list. Keeping
        // this element to one row is what makes deep and wide trees cheap.
        Some(
            div()
                .flex()
                .flex_col()
                .child(row.child(content))
                .into_any_element(),
        )
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Tree {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let mut flat = Vec::new();
        Self::flatten_visible_for_keys(
            &self.items,
            self.filter.as_ref().map(|filter| filter.as_ref()),
            &mut flat,
        );
        let on_select = self.on_select.clone();
        let on_activate = self.on_activate.clone();
        let on_toggle = self.on_toggle.clone();
        let on_context_menu = self.on_context_menu.clone();
        let on_check = self.on_check.clone();
        let render_state = TreeRenderState {
            selected_id: self.selected_id.clone(),
            on_select: on_select.clone(),
            on_activate: on_activate.clone(),
            on_toggle: on_toggle.clone(),
            on_context_menu: on_context_menu.clone(),
            on_check,
            on_load_children: self.on_load_children.clone(),
            filter: self.filter.clone(),
        };
        let root_select = self.selected_id.clone();
        let flat_for_keys = flat.clone();
        let key_select = on_select.clone();
        let key_activate = on_activate.clone();
        let key_toggle = on_toggle.clone();
        let root = div()
            .id("tree")
            .role(Role::Tree)
            .tab_index(0)
            .key_context("Tree")
            .on_key_down(move |ev: &KeyDownEvent, win, app| {
                if flat_for_keys.is_empty() {
                    return;
                }
                let current = root_select
                    .as_ref()
                    .and_then(|id| flat_for_keys.iter().position(|item| &item.id == id))
                    .unwrap_or(0);
                match ev.keystroke.key.as_str() {
                    "up" => {
                        let next = current.saturating_sub(1);
                        if let Some(handler) = &key_select {
                            let ev = ClickEvent::default();
                            handler(flat_for_keys[next].id.clone(), &ev, win, app);
                        }
                    }
                    "down" => {
                        let next = (current + 1).min(flat_for_keys.len() - 1);
                        if let Some(handler) = &key_select {
                            let ev = ClickEvent::default();
                            handler(flat_for_keys[next].id.clone(), &ev, win, app);
                        }
                    }
                    "right" => {
                        let item = &flat_for_keys[current];
                        if item.has_children && !item.expanded {
                            if let Some(handler) = &key_toggle {
                                let ev = ClickEvent::default();
                                handler(item.id.clone(), true, &ev, win, app);
                            }
                        }
                    }
                    "left" => {
                        let item = &flat_for_keys[current];
                        if item.has_children && item.expanded {
                            if let Some(handler) = &key_toggle {
                                let ev = ClickEvent::default();
                                handler(item.id.clone(), false, &ev, win, app);
                            }
                        }
                    }
                    "return" | "space" => {
                        let item = &flat_for_keys[current];
                        if !item.disabled {
                            if let Some(handler) = &key_activate {
                                let ev = ClickEvent::default();
                                handler(item.id.clone(), &ev, win, app);
                            } else if let Some(handler) = &key_select {
                                let ev = ClickEvent::default();
                                handler(item.id.clone(), &ev, win, app);
                            }
                        }
                    }
                    _ => {}
                }
            })
            .flex()
            .flex_col()
            .w_full();
        let status = match self.status {
            Some(status) => Some(status),
            None if self.items.is_empty() => Some((DataStatusKind::Empty, "No items".into())),
            None if flat.is_empty() && self.filter.is_some() => Some((
                DataStatusKind::NoResults,
                "No items match the filter".into(),
            )),
            None => None,
        };
        if let Some((kind, message)) = status {
            return root.child(status_block("tree-status", kind, message, cx));
        }
        let rows = Self::flatten_visible(&self.items, self.filter.as_deref(), 0);
        let count = rows.len();
        let build_row = move |index: usize, _window: &mut Window, app: &mut App| {
            let (item, depth) = rows[index].clone();
            Self::render_item(item, depth, index, render_state.clone(), app)
                .unwrap_or_else(|| div().into_any_element())
        };
        root.child(
            crate::virtual_list::VirtualList::new("tree-virtual-list", count, build_row)
                .row_height(cx.theme().components.list_item_height)
                .reveal(
                    flat.iter()
                        .position(|item| self.selected_id.as_ref() == Some(&item.id)),
                ),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ids(items: &[FlatTreeItem]) -> Vec<&str> {
        items.iter().map(|item| item.id.as_ref()).collect()
    }

    #[test]
    fn filtered_key_model_only_contains_visible_matching_branch() {
        let items = vec![
            TreeItem::new("projects", "Projects")
                .child(TreeItem::new("alpha", "Alpha"))
                .child(TreeItem::new("beta", "Beta")),
            TreeItem::new("archive", "Archive").child(TreeItem::new("old", "Old")),
        ];
        let mut flat = Vec::new();

        Tree::flatten_visible_for_keys(&items, Some("beta"), &mut flat);

        assert_eq!(ids(&flat), vec!["projects", "beta"]);
    }

    #[test]
    fn disabled_rows_are_excluded_from_key_model() {
        let items = vec![
            TreeItem::new("disabled", "Disabled").disabled(true),
            TreeItem::new("enabled", "Enabled"),
            TreeItem::new("disabled-parent", "Disabled Parent")
                .disabled(true)
                .child(TreeItem::new("enabled-child", "Enabled Child")),
        ];
        let mut flat = Vec::new();

        Tree::flatten_visible_for_keys(&items, None, &mut flat);

        assert_eq!(ids(&flat), vec!["enabled", "enabled-child"]);
    }

    #[test]
    fn collapsed_matching_parent_does_not_expose_child_to_key_model() {
        let items = vec![TreeItem::new("parent", "Parent")
            .expanded(false)
            .child(TreeItem::new("child", "Needle"))];
        let mut flat = Vec::new();

        Tree::flatten_visible_for_keys(&items, Some("needle"), &mut flat);

        assert_eq!(ids(&flat), vec!["parent"]);
    }
}

#[cfg(test)]
mod check_tests {
    use super::*;

    fn leaf(id: &'static str, checked: bool) -> TreeItem {
        TreeItem::new(id, id).checkable(true).checked(checked)
    }

    fn folder(id: &'static str, children: Vec<TreeItem>) -> TreeItem {
        let mut item = TreeItem::new(id, id).checkable(true);
        for child in children {
            item = item.child(child);
        }
        item
    }

    #[test]
    fn checkable_is_off_by_default() {
        assert!(!TreeItem::new("a", "A").is_checkable());
        assert!(TreeItem::new("a", "A").checkable(true).is_checkable());
    }

    #[test]
    fn leaf_reports_its_own_flag() {
        assert_eq!(leaf("a", true).check_state(), TreeCheckState::Checked);
        assert_eq!(leaf("a", false).check_state(), TreeCheckState::Unchecked);
    }

    #[test]
    fn folder_is_checked_when_every_leaf_is_checked() {
        let f = folder("f", vec![leaf("a", true), leaf("b", true)]);
        assert_eq!(f.check_state(), TreeCheckState::Checked);
    }

    #[test]
    fn folder_is_unchecked_when_no_leaf_is_checked() {
        let f = folder("f", vec![leaf("a", false), leaf("b", false)]);
        assert_eq!(f.check_state(), TreeCheckState::Unchecked);
    }

    #[test]
    fn folder_is_mixed_when_leaves_disagree() {
        let f = folder("f", vec![leaf("a", true), leaf("b", false)]);
        assert_eq!(f.check_state(), TreeCheckState::Mixed);
    }

    #[test]
    fn tri_state_rolls_up_through_nested_folders() {
        // The outer folder mixes a fully-checked subfolder with an unchecked leaf.
        let inner = folder("inner", vec![leaf("a", true), leaf("b", true)]);
        let outer = folder("outer", vec![inner, leaf("c", false)]);
        assert_eq!(outer.check_state(), TreeCheckState::Mixed);

        // A folder with only a mixed child is itself mixed.
        let mixed_child = folder("m", vec![leaf("x", true), leaf("y", false)]);
        let wrapper = folder("w", vec![mixed_child]);
        assert_eq!(wrapper.check_state(), TreeCheckState::Mixed);
    }

    #[test]
    fn empty_folder_is_unchecked() {
        let f = folder("f", vec![]);
        assert_eq!(f.check_state(), TreeCheckState::Unchecked);
    }

    #[test]
    fn click_requests_the_opposite_of_the_derived_state() {
        assert!(next_checked(TreeCheckState::Unchecked));
        assert!(next_checked(TreeCheckState::Mixed));
        assert!(!next_checked(TreeCheckState::Checked));
    }

    #[test]
    fn toggling_a_folder_sets_all_descendants() {
        let mut items = vec![folder(
            "root",
            vec![
                folder("dir", vec![leaf("a", false), leaf("b", false)]),
                leaf("c", false),
            ],
        )];

        assert!(set_tree_checked(&mut items, "dir", true));
        assert_eq!(items[0].children[0].check_state(), TreeCheckState::Checked);
        assert_eq!(
            items[0].children[1].check_state(),
            TreeCheckState::Unchecked
        );
        assert_eq!(items[0].check_state(), TreeCheckState::Mixed);

        assert!(set_tree_checked(&mut items, "root", false));
        assert_eq!(items[0].check_state(), TreeCheckState::Unchecked);
    }

    #[test]
    fn toggling_a_leaf_updates_only_that_leaf() {
        let mut items = vec![folder("dir", vec![leaf("a", false), leaf("b", false)])];

        assert!(set_tree_checked(&mut items, "a", true));
        assert_eq!(items[0].check_state(), TreeCheckState::Mixed);
    }

    #[test]
    fn toggling_an_unknown_id_changes_nothing() {
        let mut items = vec![leaf("a", false)];
        assert!(!set_tree_checked(&mut items, "missing", true));
        assert_eq!(items[0].check_state(), TreeCheckState::Unchecked);
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_tree_levels_and_expansion(cx: &mut gpui::TestAppContext) {
        struct SnapHost;
        impl gpui::Render for SnapHost {
            fn render(&mut self, _: &mut Window, _: &mut gpui::Context<Self>) -> impl IntoElement {
                Tree::new()
                    .item(
                        TreeItem::new("root", "Root")
                            .expanded(true)
                            .child(TreeItem::new("leaf", "Leaf"))
                            .child(
                                TreeItem::new("folder", "Folder").child(TreeItem::new("x", "X")),
                            ),
                    )
                    .selected("leaf")
            }
        }
        let (_, vcx) = fluent_test_utils::open_fluent_test_window(cx, |_, _| SnapHost);
        use fluent_test_utils::FluentInteractionExt as _;
        assert_eq!(
            vcx.accessibility_snapshot(),
            vec![
                "Window".to_string(),
                "Tree".to_string(),
                "TreeItem \"Root\" [selected=false, expanded=true, level=1]".to_string(),
                "TreeItem \"Leaf\" [selected=true, level=2]".to_string(),
                "TreeItem \"Folder\" [selected=false, expanded=true, level=2]".to_string(),
                "TreeItem \"X\" [selected=false, level=3]".to_string(),
            ]
        );
    }
}
