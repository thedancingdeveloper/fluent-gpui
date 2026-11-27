use fluent_core::{TextRampExt, TruncateExt as _};
use std::{collections::HashMap, sync::Arc};

use fluent_core::{Theme, ThemeProvider as _};
use fluent_primitives::{focus_ring, is_keyboard_focused, FluentFocus as _};
use gpui::{
    div, prelude::*, px, svg, App, ClickEvent, Context, FocusHandle, FontWeight, IntoElement,
    Render, SharedString, Window,
};

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

/// A single item in a `SettingsNav` section.
#[derive(Clone, Debug)]
pub struct SettingsNavItem {
    /// Public API item.
    pub id: SharedString,
    /// Public API item.
    pub label: SharedString,
    /// Public API item.
    pub icon: Option<SharedString>,
}

impl SettingsNavItem {
    /// Public API item.
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
        }
    }

    /// Public API item.
    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }
}

/// A collapsible section in a `SettingsNav`.
#[derive(Clone, Debug)]
pub struct SettingsNavSection {
    /// Public API item.
    pub title: SharedString,
    /// Public API item.
    pub items: Vec<SettingsNavItem>,
    /// Public API item.
    pub expanded: bool,
}

impl SettingsNavSection {
    /// Public API item.
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            items: vec![],
            expanded: true,
        }
    }

    /// Public API item.
    pub fn item(mut self, item: SettingsNavItem) -> Self {
        self.items.push(item);
        self
    }

    /// Public API item.
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
    /// Public API item.
    pub sections: Vec<SettingsNavSection>,
    /// Public API item.
    pub active_id: Option<SharedString>,
    /// Public API item.
    pub on_select: Option<Arc<dyn Fn(SharedString, &mut App) + 'static>>,
    /// Focus handles of the section headers (`(section, None)`) and items
    /// (`(section, Some(item))`), created on first render.
    focus_handles: HashMap<(usize, Option<usize>), FocusHandle>,
}

impl SettingsNav {
    /// Public API item.
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self {
            sections: vec![],
            active_id: None,
            on_select: None,
            focus_handles: HashMap::new(),
        }
    }

    /// Public API item.
    pub fn section(mut self, section: SettingsNavSection) -> Self {
        if self.active_id.is_none() {
            if let Some(first) = section.items.first() {
                self.active_id = Some(first.id.clone());
            }
        }
        self.sections.push(section);
        self
    }

    /// Public API item.
    pub fn active(mut self, id: impl Into<SharedString>) -> Self {
        self.active_id = Some(id.into());
        self
    }

    /// Public API item.
    pub fn on_select(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }

    /// Public API item.
    pub fn set_active(&mut self, id: SharedString, cx: &mut Context<Self>) {
        self.active_id = Some(id);
        cx.notify();
    }
}

impl Render for SettingsNav {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
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
            .py(px(spacing.sm))
            // The focus ring sits 2px outside each row; keep it inside the
            // `overflow_hidden` clip.
            .px(px(2.0));

        for (si, section) in self.sections.iter().enumerate() {
            let expanded = section.expanded;
            let section_title = section.title.clone();

            // Section header (collapsible)
            let toggle_handler =
                cx.listener(move |nav: &mut SettingsNav, _: &ClickEvent, _, cx| {
                    nav.sections[si].expanded = !nav.sections[si].expanded;
                    cx.notify();
                });

            let header_focus = self
                .focus_handles
                .entry((si, None))
                .or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(true))
                .clone();
            let header_ring = is_keyboard_focused(&header_focus, window);

            nav = nav.child(focus_ring(
                div()
                    .id(("nav-section", si as u64))
                    .debug_selector(move || format!("nav-section-{si}"))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.xs))
                    .px(px(spacing.md))
                    .h(px(components.list_item_height))
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg))
                    .track_focus(&header_focus)
                    .fluent_tab_stop(0)
                    .on_click(toggle_handler)
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .justify_center()
                            .size(px(components.popup_icon_slot))
                            .flex_none()
                            .child(
                                svg()
                                    .path(if expanded {
                                        "icons/chevron_down.svg"
                                    } else {
                                        "icons/chevron_right.svg"
                                    })
                                    .size(px(CHEVRON_GLYPH))
                                    .text_color(on_subtle),
                            ),
                    )
                    .child(
                        div()
                            .flex_1()
                            .debug_selector(move || format!("nav-section-label-{si}"))
                            .truncate_label()
                            .text_ramp(&typography.body)
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(on_neutral)
                            .child(section_title),
                    ),
                header_ring,
                &colors,
                px(0.0),
            ));

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

                let item_focus = self
                    .focus_handles
                    .entry((si, Some(ii)))
                    .or_insert_with(|| cx.focus_handle().tab_index(0).tab_stop(true))
                    .clone();
                let item_ring = is_keyboard_focused(&item_focus, window);

                // Active item: 3px accent bar on the left, offset by 16px indent
                let row = div()
                    .id(("nav-item", (si * 100 + ii) as u64))
                    .debug_selector(move || format!("nav-item-{si}-{ii}"))
                    .flex()
                    .flex_row()
                    .items_center()
                    .h(px(components.list_item_height))
                    .cursor_pointer()
                    .bg(if is_active {
                        active_bg
                    } else {
                        colors.panel_bg
                    })
                    .hover(move |s| s.bg(if is_active { active_bg } else { hover_bg }))
                    .track_focus(&item_focus)
                    .fluent_tab_stop(0)
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
                            .debug_selector(move || format!("nav-item-label-{si}-{ii}"))
                            .truncate_label()
                            .pl(px(components.settings_nav_item_indent))
                            .text_ramp(&typography.body)
                            .font_weight(if is_active {
                                FontWeight::SEMIBOLD
                            } else {
                                FontWeight::NORMAL
                            })
                            .text_color(if is_active { on_neutral } else { on_subtle })
                            .child(item_label),
                    );

                nav = nav.child(focus_ring(row, item_ring, &colors, px(0.0)));
            }
        }

        nav
    }
}

/// Section chevron glyph size.
pub(crate) const CHEVRON_GLYPH: f32 = 12.0;

#[cfg(test)]
mod tests {
    #[test]
    fn section_chevron_is_12px() {
        assert_eq!(super::CHEVRON_GLYPH, 12.0);
    }

    use std::{cell::RefCell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::TestAppContext;

    use super::*;
    use crate::roving::test_support::{clipped_ring_quads, press};

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
    fn long_titles_are_truncated_to_one_line(cx: &mut TestAppContext) {
        let long = "A very long settings page title that would otherwise wrap onto several lines";
        let (_nav, cx) = open_fluent_test_window(cx, |_, cx| {
            SettingsNav::new(cx)
                .section(SettingsNavSection::new(long).item(SettingsNavItem::new("a", long)))
        });
        cx.accessibility_snapshot();
        let row_h = cx.update(|_, app| app.theme().components.list_item_height);
        let nav_bounds = cx
            .selector_bounds("nav-section-0")
            .expect("section painted");
        for selector in ["nav-section-label-0", "nav-item-label-0-0"] {
            let label = cx.selector_bounds(selector).expect("label painted");
            assert!(
                f32::from(label.size.height) < row_h,
                "{selector} wrapped to several lines: {label:?}"
            );
            assert!(
                label.right() <= nav_bounds.right(),
                "{selector} overflows the nav: {label:?} vs {nav_bounds:?}"
            );
        }
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
    fn tab_then_enter_selects_item(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        cx.update(|_, app| fluent_primitives::install_focus_keybindings(app));
        cx.update(|window, app| window.draw(app).clear(app));
        // Section header, item A, item B are the Tab stops.
        for _ in 0..3 {
            cx.update(|window, app| window.focus_next(app));
            cx.run_until_parked();
            cx.update(|window, app| window.draw(app).clear(app));
        }
        // A press is a key-down and a key-up; it selects exactly once.
        press(cx, "enter");
        nav.read_with(cx, |n, _| assert_eq!(n.active_id.as_deref(), Some("b")));
        assert_eq!(*log.borrow(), vec!["b".to_string()]);
    }

    /// Focuses the `n`th Tab stop (header, item A, item B, ...) the way a
    /// keyboard user does: the first stop via the window, the rest via Tab, so
    /// the window ends in keyboard input modality.
    fn tab_to(cx: &mut gpui::VisualTestContext, stops: usize) {
        cx.update(|_, app| fluent_primitives::install_focus_keybindings(app));
        cx.update(|window, app| window.draw(app).clear(app));
        cx.update(|window, app| window.focus_next(app));
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
        for _ in 1..stops {
            cx.type_keys("tab");
            cx.update(|window, app| window.draw(app).clear(app));
        }
    }

    /// Number of distinct painted rings in the Fluent 2 outer focus-ring colour.
    fn outer_ring_quads(cx: &mut gpui::VisualTestContext) -> usize {
        cx.update(|window, app| window.draw(app).clear(app));
        cx.update(|window, app| {
            let outer = app.theme().colors.stroke_focus_outer;
            // Identical quads from repeated draws count once.
            let mut rings: Vec<String> = window
                .painted_quads()
                .iter()
                .filter(|q| q.border_color == outer)
                .map(|q| format!("{:?}", q.bounds))
                .collect();
            rings.sort();
            rings.dedup();
            rings.len()
        })
    }

    #[gpui::test]
    fn tab_then_space_selects_item(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        tab_to(cx, 3);
        press(cx, "space");
        nav.read_with(cx, |n, _| assert_eq!(n.active_id.as_deref(), Some("b")));
        assert_eq!(*log.borrow(), vec!["b".to_string()]);
    }

    #[gpui::test]
    fn keyboard_toggles_section_header(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        tab_to(cx, 1);
        nav.read_with(cx, |n, _| assert!(n.sections[0].expanded));
        // Each press (key-down and key-up) toggles exactly once.
        press(cx, "enter");
        nav.read_with(cx, |n, _| assert!(!n.sections[0].expanded));
        press(cx, "space");
        nav.read_with(cx, |n, _| assert!(n.sections[0].expanded));
        // Toggling a header never selects an item.
        assert!(log.borrow().is_empty());
    }

    #[gpui::test]
    fn keyboard_focus_draws_the_ring(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (_nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        assert_eq!(
            outer_ring_quads(cx),
            0,
            "no ring before anything is focused"
        );
        tab_to(cx, 2);
        assert_eq!(outer_ring_quads(cx), 1, "one ring, on the focused item");
    }

    #[gpui::test]
    fn ring_is_not_clipped_by_the_nav(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (_nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        // First item, last item, then the section header (shift-tab twice):
        // the nav's `overflow_hidden` clips to its own bounds, so each ring
        // must lie inside them.
        tab_to(cx, 2);
        let check = |cx: &mut gpui::VisualTestContext, what: &str| {
            assert_eq!(outer_ring_quads(cx), 1, "one ring on {what}");
            let clipped = clipped_ring_quads(cx);
            assert!(clipped.is_empty(), "ring clipped on {what}: {clipped:?}");
        };
        check(cx, "item A");
        cx.type_keys("tab");
        check(cx, "item B");
        cx.type_keys("shift-tab");
        cx.type_keys("shift-tab");
        check(cx, "the section header");
    }

    #[gpui::test]
    fn mouse_click_does_not_draw_the_ring(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (_nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        cx.click_selector("nav-item-0-1");
        cx.update(|window, app| window.draw(app).clear(app));
        // The click did focus the row; only the ring is withheld.
        assert!(cx.update(|window, app| window.focused(app).is_some()));
        assert_eq!(outer_ring_quads(cx), 0, "mouse focus must not draw a ring");
        // Tabbing afterwards (keyboard modality) does draw one.
        cx.type_keys("tab");
        assert_eq!(outer_ring_quads(cx), 1);
    }

    #[gpui::test]
    fn set_active_changes_selection_without_callback(cx: &mut TestAppContext) {
        let log = Rc::new(RefCell::new(vec![]));
        let (nav, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        nav.update(cx, |n, cx| n.set_active("b".into(), cx));
        nav.read_with(cx, |n, _| assert_eq!(n.active_id.as_deref(), Some("b")));
        assert!(log.borrow().is_empty());
    }

    fn nav_row_heights(compact: bool, cx: &mut TestAppContext) -> (f32, f32) {
        let log = Rc::new(RefCell::new(vec![]));
        let (nav_entity, cx) = open_fluent_test_window(cx, |_, cx| nav(cx, log.clone()));
        if compact {
            cx.update(|_, app| {
                let theme = app.global_mut::<fluent_core::Theme>();
                theme.components = theme
                    .components
                    .with_row_density(fluent_core::RowDensity::Compact);
            });
            nav_entity.update(cx, |_, cx| cx.notify());
        }
        cx.run_until_parked();
        let section = cx
            .selector_bounds("nav-section-0")
            .expect("section painted");
        let item = cx.selector_bounds("nav-item-0-0").expect("item painted");
        (f32::from(section.size.height), f32::from(item.size.height))
    }

    #[gpui::test]
    fn nav_section_and_item_rows_match_list_item_height(cx: &mut TestAppContext) {
        assert_eq!(nav_row_heights(false, cx), (32.0, 32.0));
    }

    #[gpui::test]
    fn nav_section_and_items_follow_compact_density(cx: &mut TestAppContext) {
        assert_eq!(nav_row_heights(true, cx), (28.0, 28.0));
    }
}
