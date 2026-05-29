use std::sync::Arc;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, App, ClickEvent, IntoElement, KeyDownEvent, MouseButton,
    MouseDownEvent, RenderOnce, SharedString, Window,
};

type TreeSelectHandler = Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>;
type TreeActivateHandler = Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>;
type TreeToggleHandler =
    Arc<dyn Fn(SharedString, bool, &ClickEvent, &mut Window, &mut App) + 'static>;
type TreeContextHandler =
    Arc<dyn Fn(SharedString, &MouseDownEvent, &mut Window, &mut App) + 'static>;

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
    filter: Option<SharedString>,
}

#[derive(Clone, Debug)]
pub struct TreeItem {
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<SharedString>,
    pub badge: Option<SharedString>,
    pub expanded: bool,
    pub disabled: bool,
    pub children: Vec<TreeItem>,
}

impl TreeItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            badge: None,
            expanded: true,
            disabled: false,
            children: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn badge(mut self, badge: impl Into<SharedString>) -> Self {
        self.badge = Some(badge.into());
        self
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn child(mut self, child: TreeItem) -> Self {
        self.children.push(child);
        self
    }
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
}

impl Tree {
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
                    has_children: !item.children.is_empty(),
                });
            }

            if item.expanded {
                Self::flatten_visible_for_keys(&item.children, filter, out);
            }
        }
    }

    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            selected_id: None,
            filter: None,
            on_select: None,
            on_activate: None,
            on_toggle: None,
            on_context_menu: None,
        }
    }

    pub fn item(mut self, item: TreeItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = TreeItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }

    pub fn filter(mut self, query: impl Into<SharedString>) -> Self {
        let query = query.into();
        if !query.is_empty() {
            self.filter = Some(query);
        }
        self
    }

    pub fn on_select(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    pub fn on_activate(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_activate = Some(Arc::new(f));
        self
    }

    pub fn on_toggle(
        mut self,
        f: impl Fn(SharedString, bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Arc::new(f));
        self
    }

    pub fn on_context_menu(
        mut self,
        f: impl Fn(SharedString, &MouseDownEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_context_menu = Some(Arc::new(f));
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
        let has_children = !item.children.is_empty();
        let fg = if item.disabled {
            colors.on_neutral_disabled
        } else {
            colors.on_neutral
        };

        let mut row = div()
            .id(("tree-item", index as u64))
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.xs))
            .h(px(components.menu_item_height))
            .pl(px(spacing.md + depth as f32 * 16.0))
            .pr(px(spacing.md))
            .bg(if is_selected {
                colors.neutral_selected
            } else {
                colors.surface
            })
            .text_size(px(typography.body.size))
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
                    }
                })
                .child(chevron),
        );

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

        content = content.child(item.label);
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
        let mut node = div().flex().flex_col().child(row.child(content));
        if item.expanded {
            for (child_idx, child) in item.children.into_iter().enumerate() {
                let child = Self::render_item(
                    child,
                    depth + 1,
                    index * 100 + child_idx + 1,
                    state.clone(),
                    cx,
                );
                if let Some(child) = child {
                    node = node.child(child);
                }
            }
        }

        Some(node.into_any_element())
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
        let render_state = TreeRenderState {
            selected_id: self.selected_id.clone(),
            on_select: on_select.clone(),
            on_activate: on_activate.clone(),
            on_toggle: on_toggle.clone(),
            on_context_menu: on_context_menu.clone(),
            filter: self.filter.clone(),
        };
        let root_select = self.selected_id.clone();
        let flat_for_keys = flat.clone();
        let key_select = on_select.clone();
        let key_activate = on_activate.clone();
        let key_toggle = on_toggle.clone();
        let mut root = div()
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
        for (idx, item) in self.items.into_iter().enumerate() {
            if let Some(item) = Self::render_item(item, 0, idx, render_state.clone(), cx) {
                root = root.child(item);
            }
        }
        root
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
