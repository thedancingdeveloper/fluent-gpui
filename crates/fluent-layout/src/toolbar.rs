use std::sync::Arc;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, App, ClickEvent, IntoElement, RenderOnce, SharedString, Window,
};

type ToolbarClick = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type ToolbarToggleClick = Arc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>;

#[derive(Clone)]
pub enum ToolbarItem {
    Action {
        id: SharedString,
        label: SharedString,
        icon: Option<SharedString>,
        disabled: bool,
        on_click: ToolbarClick,
    },
    Toggle {
        id: SharedString,
        label: SharedString,
        icon: Option<SharedString>,
        selected: bool,
        disabled: bool,
        on_click: ToolbarToggleClick,
    },
    Separator,
}

impl ToolbarItem {
    pub fn action(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Action {
            id: id.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    pub fn action_with_icon(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Action {
            id: id.into(),
            label: label.into(),
            icon: Some(icon.into()),
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    pub fn toggle(
        id: impl Into<SharedString>,
        label: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Toggle {
            id: id.into(),
            label: label.into(),
            icon: None,
            selected,
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    pub fn separator() -> Self {
        Self::Separator
    }

    pub fn disabled(self, disabled: bool) -> Self {
        match self {
            Self::Action {
                id,
                label,
                icon,
                on_click,
                ..
            } => Self::Action {
                id,
                label,
                icon,
                disabled,
                on_click,
            },
            Self::Toggle {
                id,
                label,
                icon,
                selected,
                on_click,
                ..
            } => Self::Toggle {
                id,
                label,
                icon,
                selected,
                disabled,
                on_click,
            },
            Self::Separator => Self::Separator,
        }
    }
}

/// A compact command surface for panes, lists, and editor-like tools.
#[derive(IntoElement)]
pub struct Toolbar {
    items: Vec<ToolbarItem>,
    max_visible: Option<usize>,
    overflow_open: bool,
    on_overflow_click: Option<ToolbarClick>,
}

impl Toolbar {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            max_visible: None,
            overflow_open: false,
            on_overflow_click: None,
        }
    }

    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn max_visible(mut self, max_visible: usize) -> Self {
        self.max_visible = Some(max_visible);
        self
    }

    pub fn overflow_open(mut self, open: bool) -> Self {
        self.overflow_open = open;
        self
    }

    pub fn on_overflow_click(
        mut self,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_overflow_click = Some(Arc::new(f));
        self
    }

    fn render_item(item: ToolbarItem, idx: usize, cx: &mut App) -> gpui::AnyElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let components = theme.components;

        match item {
            ToolbarItem::Separator => div()
                .w(px(1.0))
                .h(px(components.dock_header_height - spacing.sm * 2.0))
                .mx(px(spacing.xs))
                .bg(colors.stroke_neutral_subtle)
                .into_any_element(),
            ToolbarItem::Action {
                id,
                label,
                icon,
                disabled,
                on_click,
            } => {
                let fg = if disabled {
                    colors.on_neutral_disabled
                } else {
                    colors.on_neutral
                };
                let mut el = div()
                    .id(("toolbar-action", idx as u64))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.xs))
                    .h(px(components.dock_header_height))
                    .px(px(spacing.sm))
                    .rounded(px(radii.sm))
                    .text_size(px(typography.caption.size))
                    .text_color(fg)
                    .child(match icon {
                        Some(icon) => svg()
                            .path(icon)
                            .size(px(components.popup_icon_slot))
                            .text_color(fg)
                            .into_any_element(),
                        None => div().size(px(0.0)).into_any_element(),
                    })
                    .child(label);
                if !disabled {
                    el = el
                        .cursor_pointer()
                        .hover({
                            let hover = colors.subtle_hover;
                            move |s| s.bg(hover)
                        })
                        .on_click(move |ev, win, app| on_click(ev, win, app));
                }
                let _ = id;
                el.into_any_element()
            }
            ToolbarItem::Toggle {
                id,
                label,
                icon,
                selected,
                disabled,
                on_click,
            } => {
                let bg = if selected {
                    colors.neutral_selected
                } else {
                    colors.subtle
                };
                let fg = if disabled {
                    colors.on_neutral_disabled
                } else {
                    colors.on_neutral
                };
                let mut el = div()
                    .id(("toolbar-toggle", idx as u64))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.xs))
                    .h(px(components.dock_header_height))
                    .px(px(spacing.sm))
                    .rounded(px(radii.sm))
                    .bg(bg)
                    .text_size(px(typography.caption.size))
                    .text_color(fg)
                    .child(match icon {
                        Some(icon) => svg()
                            .path(icon)
                            .size(px(components.popup_icon_slot))
                            .text_color(fg)
                            .into_any_element(),
                        None => div().size(px(0.0)).into_any_element(),
                    })
                    .child(label);
                if !disabled {
                    el = el
                        .cursor_pointer()
                        .hover({
                            let hover = colors.neutral_hover;
                            move |s| s.bg(hover)
                        })
                        .on_click(move |ev, win, app| on_click(!selected, ev, win, app));
                }
                let _ = id;
                el.into_any_element()
            }
        }
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Toolbar {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let components = theme.components;
        let radii = theme.radii;
        let typography = theme.typography;

        let visible_count = self.max_visible.unwrap_or(self.items.len());
        let mut root = div()
            .flex()
            .flex_row()
            .items_center()
            .relative()
            .gap(px(spacing.xs))
            .h(px(components.dock_header_height))
            .px(px(spacing.sm))
            .bg(colors.surface_dim)
            .border_b_1()
            .border_color(colors.stroke_neutral_subtle);

        for (idx, item) in self.items.iter().take(visible_count).cloned().enumerate() {
            root = root.child(Self::render_item(item, idx, cx));
        }

        let overflow: Vec<ToolbarItem> = self.items.into_iter().skip(visible_count).collect();
        if !overflow.is_empty() {
            let mut overflow_button = div()
                .id("toolbar-overflow")
                .flex()
                .items_center()
                .justify_center()
                .h(px(components.dock_header_height))
                .px(px(spacing.sm))
                .rounded(px(radii.sm))
                .text_size(px(typography.caption.size))
                .text_color(colors.on_neutral)
                .cursor_pointer()
                .hover({
                    let hover = colors.subtle_hover;
                    move |s| s.bg(hover)
                })
                .child("More");

            if let Some(handler) = self.on_overflow_click {
                overflow_button =
                    overflow_button.on_click(move |ev, win, app| handler(ev, win, app));
            }

            root = root.child(overflow_button);

            if self.overflow_open {
                let mut menu = div()
                    .absolute()
                    .top(px(components.dock_header_height))
                    .right_0()
                    .min_w(px(180.0))
                    .flex()
                    .flex_col()
                    .bg(colors.surface)
                    .border_1()
                    .border_color(colors.stroke_neutral)
                    .rounded(px(radii.md))
                    .shadow_md()
                    .py(px(spacing.xs));
                for (idx, item) in overflow.into_iter().enumerate() {
                    menu = menu.child(Self::render_item(item, idx + visible_count, cx));
                }
                root = root.child(menu);
            }
        }

        root
    }
}
