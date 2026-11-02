use std::sync::Arc;

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, App, ClickEvent, Context, FontWeight, IntoElement, Render, SharedString,
    Window,
};

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A single item in a `SettingsNav` section.
#[derive(Clone, Debug)]
pub struct SettingsNavItem {
    pub id: SharedString,
    pub label: SharedString,
    pub icon: Option<SharedString>,
}

impl SettingsNavItem {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
        }
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// A collapsible section in a `SettingsNav`.
#[derive(Clone, Debug)]
pub struct SettingsNavSection {
    pub title: SharedString,
    pub items: Vec<SettingsNavItem>,
    pub expanded: bool,
}

impl SettingsNavSection {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            items: vec![],
            expanded: true,
        }
    }

    pub fn item(mut self, item: SettingsNavItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn collapsed(mut self) -> Self {
        self.expanded = false;
        self
    }
}

// ---------------------------------------------------------------------------
// SettingsNav entity
// ---------------------------------------------------------------------------

/// Left-side navigation for settings dialogs and data-entry modals.
///
/// The active item has a 3px accent bar on its left edge, matching the
/// Fluent 2 dialog navigation style. Sections are collapsible.
///
/// ```ignore
/// let nav = cx.new(|_| {
///     SettingsNav::new()
///         .section(SettingsNavSection::new("General")
///             .item(SettingsNavItem::new("general", "General"))
///             .item(SettingsNavItem::new("display", "Display Options")))
///         .section(SettingsNavSection::new("Advanced")
///             .item(SettingsNavItem::new("network", "Network")))
///         .active("general")
///         .on_select(|id, cx| { /* switch form panel */ })
/// });
/// ```
#[allow(clippy::type_complexity)]
pub struct SettingsNav {
    pub sections: Vec<SettingsNavSection>,
    pub active_id: Option<SharedString>,
    pub on_select: Option<Arc<dyn Fn(SharedString, &mut App) + 'static>>,
}

impl SettingsNav {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self {
            sections: vec![],
            active_id: None,
            on_select: None,
        }
    }

    pub fn section(mut self, section: SettingsNavSection) -> Self {
        if self.active_id.is_none() {
            if let Some(first) = section.items.first() {
                self.active_id = Some(first.id.clone());
            }
        }
        self.sections.push(section);
        self
    }

    pub fn active(mut self, id: impl Into<SharedString>) -> Self {
        self.active_id = Some(id.into());
        self
    }

    pub fn on_select(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    pub fn set_active(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.active_id = Some(id);
        cx.notify();
    }
}

impl Render for SettingsNav {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let components = theme.components;

        let accent = colors.accent;
        let on_neutral = colors.on_neutral;
        let on_subtle = colors.on_subtle;
        let hover_bg = colors.subtle_hover;
        let active_bg = colors.neutral_selected;
        let sep = colors.stroke_neutral_subtle;

        let mut nav = div()
            .flex()
            .flex_col()
            .w(px(components.settings_nav_width))
            .h_full()
            .overflow_hidden()
            .bg(colors.panel_bg)
            .border_r_1()
            .border_color(sep)
            .py(px(spacing.sm));

        for (si, section) in self.sections.iter().enumerate() {
            let expanded = section.expanded;
            let section_title = section.title.clone();

            // Section header (collapsible)
            let toggle_handler =
                cx.listener(move |nav: &mut SettingsNav, _: &ClickEvent, _, cx| {
                    nav.sections[si].expanded = !nav.sections[si].expanded;
                    cx.notify();
                });

            nav = nav.child(
                div()
                    .id(("nav-section", si as u64))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.xs))
                    .px(px(spacing.md))
                    .h(px(components.settings_nav_section_height))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .on_click(toggle_handler)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(components.popup_icon_slot))
                            .flex_none()
                            .text_size(px(9.0))
                            .text_color(on_subtle)
                            .child(if expanded { "▼" } else { "▶" }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(typography.body.size))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(on_neutral)
                            .child(section_title),
                    ),
            );

            if !expanded {
                continue;
            }

            for (ii, item) in section.items.iter().enumerate() {
                let item_id = item.id.clone();
                let item_label = item.label.clone();
                let is_active =
                    self.active_id.as_ref().map(|s| s.as_ref()) == Some(item_id.as_ref());

                let select_handler = {
                    let sel_id = item_id.clone();
                    let cb = self.on_select.clone();
                    cx.listener(move |nav: &mut SettingsNav, _: &ClickEvent, _, cx| {
                        nav.active_id = Some(sel_id.clone());
                        if let Some(f) = &cb {
                            f(sel_id.clone(), cx);
                        }
                        cx.notify();
                    })
                };

                // Active item: 3px accent bar on the left, offset by 16px indent
                let row = div()
                    .id(("nav-item", (si * 100 + ii) as u64))
                    .debug_selector(move || format!("nav-item-{si}-{ii}"))
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(components.settings_nav_item_height))
                    .cursor_pointer()
                    .bg(if is_active {
                        active_bg
                    } else {
                        colors.panel_bg
                    })
                    .hover(move |s| s.bg(if is_active { active_bg } else { hover_bg }))
                    .on_click(select_handler)
                    // Left accent bar (3px, accent colour when active)
                    .child(
                        div()
                            .w(px(components.settings_nav_accent_width))
                            .h_full()
                            .bg(if is_active { accent } else { colors.panel_bg }),
                    )
                    .child(
                        div()
                            .flex_1()
                            .pl(px(components.settings_nav_item_indent))
                            .text_size(px(typography.body.size))
                            .font_weight(if is_active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(if is_active { on_neutral } else { on_subtle })
                            .child(item_label),
                    );

                nav = nav.child(row);
            }
        }

        nav
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::TestAppContext;

    use super::*;

    fn nav(cx: &mut Context<SettingsNav>, log: Rc<RefCell<Vec<String>>>) -> SettingsNav {
        SettingsNav::new(cx)
            .section(
                SettingsNavSection::new("General")
                    .item(SettingsNavItem::new("a", "A"))
                    .item(SettingsNavItem::new("b", "B")),
            )
            .on_select(move |id, _| log.borrow_mut().push(id.to_string()))
    }

    #[gpui::test]
    fn first_item_is_active_by_default(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        nav.read_with(cx, |n, _| assert_eq!(n.active_id.as_deref(), Some("a")));
    }

    #[gpui::test]
    fn clicking_item_changes_selection_and_fires_callback(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        cx.click_selector("nav-item-0-1");
        nav.read_with(cx, |n, _| assert_eq!(n.active_id.as_deref(), Some("b")));
        assert_eq!(*log.borrow(), vec!["b".to_string()]);
    }

    #[gpui::test]
    fn set_active_changes_selection_without_callback(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        nav.update(cx, |n, cx| n.set_active("b".into(), cx));
        nav.read_with(cx, |n, _| assert_eq!(n.active_id.as_deref(), Some("b")));
        assert!(log.borrow().is_empty());
    }
}
