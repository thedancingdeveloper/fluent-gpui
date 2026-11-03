//! Fluent 2 side navigation with categories, nested groups, and roving focus.

use std::sync::Arc;

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, App, ClickEvent, Context, IntoElement, KeyDownEvent, Render, Role,
    SharedString, Window,
};

#[derive(Clone, Debug)]
/// Public API item.
pub struct NavItem {
    /// Public API item.
    pub id: SharedString,
    /// Public API item.
    pub label: SharedString,
    /// Public API item.
    pub icon: Option<SharedString>,
    /// Public API item.
    pub children: Vec<NavItem>,
    /// Public API item.
    pub expanded: bool,
}

impl NavItem {
    /// Public API item.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            children: Vec::new(),
            expanded: true,
        }
    }

    /// Public API item.
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }
    /// Public API item.
    pub fn child(mut self, child: NavItem) -> Self {
        self.children.push(child);
        self
    }
    /// Public API item.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
    /// Public API item.
    pub fn collapsed(self) -> Self {
        self.expanded(false)
    }
}

#[derive(Clone, Debug)]
/// Public API item.
pub struct NavCategory {
    /// Public API item.
    pub title: SharedString,
    /// Public API item.
    pub items: Vec<NavItem>,
    /// Public API item.
    pub expanded: bool,
}

impl NavCategory {
    /// Public API item.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            items: Vec::new(),
            expanded: true,
        }
    }
    /// Public API item.
    pub fn item(mut self, item: NavItem) -> Self {
        self.items.push(item);
        self
    }
    /// Public API item.
    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }
    /// Public API item.
    pub fn collapsed(self) -> Self {
        self.expanded(false)
    }
}

#[derive(Clone)]
struct FlatItem {
    id: SharedString,
    expandable: bool,
    expanded: bool,
}

/// Fluent 2 side navigation. Keyboard focus stays on the tree while the
/// selected row is moved with Up/Down (roving focus); Left/Right collapse and
/// expand groups. The generated AccessKit tree uses `Tree` and `TreeItem` roles.
#[allow(clippy::type_complexity)]
pub struct Nav {
    /// Public API item.
    pub categories: Vec<NavCategory>,
    /// Public API item.
    pub selected_id: Option<SharedString>,
    /// Public API item.
    pub compact: bool,
    /// Public API item.
    pub on_select: Option<Arc<dyn Fn(SharedString, &mut App) + 'static>>,
}

impl Nav {
    /// Public API item.
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self {
            categories: Vec::new(),
            selected_id: None,
            compact: false,
            on_select: None,
        }
    }
    /// Public API item.
    pub fn category(mut self, category: NavCategory) -> Self {
        if self.selected_id.is_none() {
            self.selected_id = first_visible(&category.items).map(|item| item.id.clone());
        }
        self.categories.push(category);
        self
    }
    /// Public API item.
    pub fn item(self, item: NavItem) -> Self {
        self.category(NavCategory::new("").item(item))
    }
    /// Public API item.
    pub fn selected(mut self, id: impl Into<SharedString>) -> Self {
        self.selected_id = Some(id.into());
        self
    }
    /// Public API item.
    pub fn active(self, id: impl Into<SharedString>) -> Self {
        self.selected(id)
    }
    /// Public API item.
    pub fn compact(mut self, compact: bool) -> Self {
        self.compact = compact;
        self
    }
    /// Public API item.
    pub fn on_select(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }
    /// Public API item.
    pub fn set_selected(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.selected_id = Some(id);
        cx.notify();
    }
    /// Public API item.
    pub fn set_active(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.set_selected(id, cx);
    }
    /// Public API item.
    pub fn toggle(&mut self, id: &str, cx: &mut Context<Self>) -> bool {
        let changed = toggle_items(&mut self.categories, id);
        if changed {
            cx.notify();
        }
        changed
    }
}

fn first_visible(items: &[NavItem]) -> Option<&NavItem> {
    items.first()
}

fn flatten(items: &[NavItem], out: &mut Vec<FlatItem>) {
    for item in items {
        out.push(FlatItem {
            id: item.id.clone(),
            expandable: !item.children.is_empty(),
            expanded: item.expanded,
        });
        if item.expanded {
            flatten(&item.children, out);
        }
    }
}

fn flatten_categories(categories: &[NavCategory]) -> Vec<FlatItem> {
    let mut out = Vec::new();
    for category in categories {
        if category.expanded {
            flatten(&category.items, &mut out);
        }
    }
    out
}

fn toggle_items(categories: &mut [NavCategory], id: &str) -> bool {
    fn visit(items: &mut [NavItem], id: &str) -> bool {
        for item in items {
            if item.id.as_ref() == id {
                item.expanded = !item.expanded;
                return true;
            }
            if visit(&mut item.children, id) {
                return true;
            }
        }
        false
    }
    for category in categories {
        if category.title.as_ref() == id {
            category.expanded = !category.expanded;
            return true;
        }
        if visit(&mut category.items, id) {
            return true;
        }
    }
    false
}

impl Render for Nav {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let components = theme.components;
        let selected = self.selected_id.clone();
        let compact = self.compact;
        let flat = flatten_categories(&self.categories);
        let on_select = self.on_select.clone();
        let key_selected = selected.clone();
        let key_flat = flat.clone();
        let key_callback = on_select.clone();
        let mut root = div()
            .id("fluent-nav")
            .role(Role::Tree)
            .tab_index(0)
            .key_context("Nav")
            .on_key_down(
                cx.listener(move |nav: &mut Nav, ev: &KeyDownEvent, _window, cx| {
                    let Some(current) = key_selected
                        .as_ref()
                        .and_then(|id| key_flat.iter().position(|item| &item.id == id))
                    else {
                        return;
                    };
                    let next = match ev.keystroke.key.as_str() {
                        "up" => current.checked_sub(1),
                        "down" => (current + 1 < key_flat.len()).then_some(current + 1),
                        "left" if key_flat[current].expandable && key_flat[current].expanded => {
                            nav.toggle(key_flat[current].id.as_ref(), cx);
                            None
                        }
                        "right" if key_flat[current].expandable && !key_flat[current].expanded => {
                            nav.toggle(key_flat[current].id.as_ref(), cx);
                            None
                        }
                        "return" | "space" => {
                            if let Some(cb) = &key_callback {
                                cb(key_flat[current].id.clone(), cx);
                            }
                            None
                        }
                        _ => None,
                    };
                    if let Some(next) = next {
                        nav.selected_id = Some(key_flat[next].id.clone());
                        cx.notify();
                    }
                }),
            )
            .flex()
            .flex_col()
            .w(if compact {
                px(56.)
            } else {
                px(components.settings_nav_width)
            })
            .h_full()
            .overflow_hidden()
            .bg(colors.panel_bg)
            .border_r_1()
            .border_color(colors.stroke_neutral_subtle)
            .py(px(spacing.sm));

        for (ci, category) in self.categories.iter().enumerate() {
            if !category.title.is_empty() {
                let title = category.title.clone();
                let expanded = category.expanded;
                let toggle_title = title.clone();
                let toggle = cx.listener(move |nav: &mut Nav, _: &ClickEvent, _, cx| {
                    nav.toggle(toggle_title.as_ref(), cx);
                });
                root = root.child(
                    div()
                        .id(("nav-category", ci as u64))
                        .role(Role::List)
                        .flex()
                        .items_center()
                        .h(px(components.settings_nav_section_height))
                        .px(px(spacing.md))
                        .cursor_pointer()
                        .on_click(toggle)
                        .aria_expanded(expanded)
                        .child(if compact { "".into() } else { title }),
                );
            }
            if category.expanded {
                for (ii, item) in category.items.iter().enumerate() {
                    root = root.child(render_item(
                        item,
                        0,
                        ci * 10000 + ii,
                        self,
                        cx,
                        &colors,
                        spacing,
                        typography,
                        compact,
                        selected.as_ref(),
                        on_select.clone(),
                    ));
                }
            }
        }
        root
    }
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn render_item(
    item: &NavItem,
    depth: usize,
    index: usize,
    nav: &Nav,
    cx: &mut Context<Nav>,
    colors: &fluent_core::ColorScheme,
    spacing: fluent_core::SpacingTokens,
    typography: fluent_core::TypographyTokens,
    compact: bool,
    selected: Option<&SharedString>,
    callback: Option<Arc<dyn Fn(SharedString, &mut App) + 'static>>,
) -> gpui::AnyElement {
    let id = item.id.clone();
    let label = item.label.clone();
    let active = selected == Some(&id);
    let expandable = !item.children.is_empty();
    let expanded = item.expanded;
    let select = id.clone();
    let click = cx.listener(move |nav: &mut Nav, _: &ClickEvent, _, cx| {
        nav.selected_id = Some(select.clone());
        if let Some(f) = &callback {
            f(select.clone(), cx);
        }
        cx.notify();
    });
    let mut row = div()
        .id(("nav-item", index as u64))
        .role(Role::TreeItem)
        .aria_label(label.clone())
        .aria_selected(active)
        .aria_expanded(expanded)
        .flex()
        .items_center()
        .gap(px(spacing.xs))
        .h(px(components_height(nav)))
        .pl(px(spacing.md + depth as f32 * 16.))
        .pr(px(spacing.sm))
        .cursor_pointer()
        .bg(if active {
            colors.neutral_selected
        } else {
            colors.panel_bg
        })
        .hover(|s| s.bg(colors.subtle_hover))
        .on_click(click);
    if compact {
        let tooltip_label = label.clone();
        row = row.tooltip(move |_, cx| {
            cx.new(|_| NavTooltip {
                text: tooltip_label.clone(),
            })
            .into()
        });
    }
    if expandable {
        row = row.child(
            div()
                .size(px(16.))
                .text_size(px(10.))
                .text_color(colors.on_subtle)
                .child(if expanded { "▼" } else { "▶" }),
        );
    } else {
        row = row.child(div().size(px(16.)));
    }
    if let Some(icon) = &item.icon {
        row = row.child(div().size(px(20.)).child(icon.clone()));
    }
    if !compact {
        row = row.child(
            div()
                .flex_1()
                .text_size(px(typography.body.size))
                .text_color(if active {
                    colors.on_neutral
                } else {
                    colors.on_subtle
                })
                .child(label),
        );
    }
    if expanded {
        for (child_index, child) in item.children.iter().enumerate() {
            row = row.child(render_item(
                child,
                depth + 1,
                index * 100 + child_index + 1,
                nav,
                cx,
                colors,
                spacing,
                typography,
                compact,
                selected,
                nav.on_select.clone(),
            ));
        }
    }
    row.into_any_element()
}

fn components_height(nav: &Nav) -> f32 {
    nav.categories.first().map(|_| 32.).unwrap_or(32.)
}

struct NavTooltip {
    text: SharedString,
}
impl Render for NavTooltip {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_2()
            .py_1()
            .bg(cx.theme().colors.surface_blur_layer)
            .text_color(cx.theme().colors.on_neutral)
            .child(self.text.clone())
    }
}
