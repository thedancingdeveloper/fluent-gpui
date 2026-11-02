use std::sync::Arc;

use fluent_core::{LayoutDirection, Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, ClickEvent, Context, IntoElement, Render, Role, SharedString, Window,
};

#[derive(Clone, Debug)]
pub struct BreadcrumbItem {
    pub id: SharedString,
    pub label: SharedString,
}

impl BreadcrumbItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
        }
    }
}

/// A Fluent 2 breadcrumb trail. Long trails retain the first and current items
/// and place the middle items in an accessible overflow menu.
#[allow(clippy::type_complexity)]
pub struct Breadcrumb {
    pub items: Vec<BreadcrumbItem>,
    pub current: usize,
    pub max_visible: usize,
    on_select: Option<Arc<dyn Fn(usize, &mut Window, &mut Context<Self>) + 'static>>,
}

impl Default for Breadcrumb {
    fn default() -> Self {
        Self {
            items: Vec::new(),
            current: 0,
            max_visible: 5,
            on_select: None,
        }
    }
}

impl Breadcrumb {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        cx.observe_global::<LayoutDirection>(|_, cx| cx.notify())
            .detach();
        Self::default()
    }
    pub fn items(mut self, items: impl IntoIterator<Item = BreadcrumbItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }
    pub fn item(mut self, item: BreadcrumbItem) -> Self {
        self.items.push(item);
        self
    }
    pub fn current(mut self, current: usize) -> Self {
        self.current = current;
        self
    }
    pub fn max_visible(mut self, max_visible: usize) -> Self {
        self.max_visible = max_visible.max(2);
        self
    }
    pub fn on_select(
        mut self,
        f: impl Fn(usize, &mut Window, &mut Context<Self>) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    fn select(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index < self.items.len() {
            self.current = index;
            if let Some(f) = self.on_select.clone() {
                f(index, window, cx);
            }
            cx.notify();
        }
    }
}

impl Render for Breadcrumb {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (colors, spacing_xs) = {
            let theme = cx.theme();
            (theme.colors.clone(), theme.spacing.xs)
        };
        let rtl = LayoutDirection::current(cx).is_rtl();
        let overflow = self.items.len() > self.max_visible;
        let mut children: Vec<gpui::AnyElement> = Vec::new();
        if !self.items.is_empty() {
            {
                let index = 0;
                let item = self.items[index].clone();
                let current = index == self.current;
                let handler = cx.listener(move |this: &mut Breadcrumb, _: &ClickEvent, win, cx| {
                    this.select(index, win, cx)
                });
                let item_el = div()
                    .id(("breadcrumb-item", index))
                    .role(Role::ListItem)
                    .aria_label(item.label.clone())
                    .flex()
                    .items_center()
                    .text_color(colors.on_neutral)
                    .when(!current, |d| d.cursor_pointer().on_click(handler))
                    .when(current, |d| {
                        d.font_weight(gpui::FontWeight::SEMIBOLD)
                            .aria_label(format!("{} (current)", item.label))
                    });
                children.push(item_el.child(item.label).into_any_element());
            }
            if overflow {
                children.push(
                    div()
                        .text_color(colors.on_neutral)
                        .child(if rtl { "‹" } else { "›" })
                        .into_any_element(),
                );
                children.push(
                    div()
                        .id("breadcrumb-overflow")
                        .role(Role::Button)
                        .aria_label("More breadcrumb items")
                        .cursor_pointer()
                        .child("…")
                        .into_any_element(),
                );
                children.push(
                    div()
                        .text_color(colors.on_neutral)
                        .child(if rtl { "‹" } else { "›" })
                        .into_any_element(),
                );
                {
                    let index = self.items.len() - 1;
                    let item = self.items[index].clone();
                    let current = index == self.current;
                    let handler =
                        cx.listener(move |this: &mut Breadcrumb, _: &ClickEvent, win, cx| {
                            this.select(index, win, cx)
                        });
                    let item_el = div()
                        .id(("breadcrumb-item", index))
                        .role(Role::ListItem)
                        .aria_label(item.label.clone())
                        .flex()
                        .items_center()
                        .text_color(colors.on_neutral)
                        .when(!current, |d| d.cursor_pointer().on_click(handler))
                        .when(current, |d| {
                            d.font_weight(gpui::FontWeight::SEMIBOLD)
                                .aria_label(format!("{} (current)", item.label))
                        });
                    children.push(item_el.child(item.label).into_any_element());
                }
            } else {
                for index in 1..self.items.len() {
                    children.push(
                        div()
                            .text_color(colors.on_neutral)
                            .child(if rtl { "‹" } else { "›" })
                            .into_any_element(),
                    );
                    let item = self.items[index].clone();
                    let current = index == self.current;
                    let handler =
                        cx.listener(move |this: &mut Breadcrumb, _: &ClickEvent, win, cx| {
                            this.select(index, win, cx)
                        });
                    let item_el = div()
                        .id(("breadcrumb-item", index))
                        .role(Role::ListItem)
                        .aria_label(item.label.clone())
                        .flex()
                        .items_center()
                        .text_color(colors.on_neutral)
                        .when(!current, |d| d.cursor_pointer().on_click(handler))
                        .when(current, |d| {
                            d.font_weight(gpui::FontWeight::SEMIBOLD)
                                .aria_label(format!("{} (current)", item.label))
                        });
                    children.push(item_el.child(item.label).into_any_element());
                }
            }
        }
        div()
            .id("breadcrumb")
            .role(Role::List)
            .flex()
            .items_center()
            .gap(px(spacing_xs))
            .children(children)
    }
}

/// Compact paging control with previous/next buttons and a roving page cursor.
#[derive(Clone, Debug)]
pub struct Pagination {
    pub page: usize,
    pub page_count: usize,
}
impl Pagination {
    pub fn new(page_count: usize) -> Self {
        Self {
            page: 0,
            page_count: page_count.max(1),
        }
    }
    pub fn page(mut self, page: usize) -> Self {
        self.page = page.min(self.page_count.saturating_sub(1));
        self
    }
}
impl Render for Pagination {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let colors = cx.theme().colors.clone();
        let current = self.page;
        let count = self.page_count;
        let mut root = div()
            .id("pagination")
            .role(Role::Navigation)
            .flex()
            .items_center()
            .gap(px(cx.theme().spacing.xs));
        for (label, index) in [
            ("Previous", current.saturating_sub(1)),
            ("Next", (current + 1).min(count.saturating_sub(1))),
        ] {
            let disabled =
                (label == "Previous" && current == 0) || (label == "Next" && current + 1 >= count);
            let handler = cx.listener(move |this: &mut Pagination, _: &ClickEvent, _, cx| {
                this.page = index;
                cx.notify();
            });
            root = root.child(
                div()
                    .id(if label == "Previous" {
                        "pagination-prev"
                    } else {
                        "pagination-next"
                    })
                    .role(Role::Button)
                    .aria_label(label)
                    .text_color(if disabled {
                        colors.on_neutral_disabled
                    } else {
                        colors.on_neutral
                    })
                    .when(!disabled, |d| d.cursor_pointer().on_click(handler))
                    .child(if label == "Previous" { "‹" } else { "›" }),
            );
        }
        for index in 0..count {
            let selected = index == current;
            let handler = cx.listener(move |this: &mut Pagination, _: &ClickEvent, _, cx| {
                this.page = index;
                cx.notify();
            });
            root = root.child(
                div()
                    .id(("pagination-page", index))
                    .role(Role::Button)
                    .aria_label(format!("Page {}", index + 1))
                    .aria_selected(selected)
                    .text_color(colors.on_neutral)
                    .when(!selected, |d| d.cursor_pointer().on_click(handler))
                    .child(format!("{}", index + 1)),
            );
        }
        root
    }
}
