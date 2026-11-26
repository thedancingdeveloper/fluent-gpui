use fluent_core::TextRampExt;
use std::sync::Arc;

use fluent_core::ThemeProvider as _;
use fluent_primitives::{focus_ring, is_keyboard_focused, FluentFocus as _};
use gpui::{
    canvas, div, prelude::*, px, svg, App, ClickEvent, ElementId, Entity, FocusHandle, IntoElement,
    KeyDownEvent, Pixels, RenderOnce, Role, SharedString, TextRun, Toggled, Window,
};

type ToolbarClick = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;
type KeyNav = Arc<dyn Fn(&KeyDownEvent, &mut Window, &mut App) + 'static>;
type ToolbarToggleClick = Arc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>;

/// Roving-focus wiring for one visible toolbar item.
struct Roving {
    handle: FocusHandle,
    tab_stop: bool,
    /// Draw the keyboard focus ring (keyboard focus only).
    ring: bool,
    on_key: KeyNav,
}

#[derive(Clone)]
/// Public API item.
pub enum ToolbarItem {
    /// Public API item.
    Action {
        /// Public API item.
        id: SharedString,
        /// Public API item.
        label: SharedString,
        /// Public API item.
        icon: Option<SharedString>,
        /// Public API item.
        disabled: bool,
        /// Public API item.
        on_click: ToolbarClick,
    },
    /// Public API item.
    Toggle {
        /// Public API item.
        id: SharedString,
        /// Public API item.
        label: SharedString,
        /// Public API item.
        icon: Option<SharedString>,
        /// Public API item.
        selected: bool,
        /// Public API item.
        disabled: bool,
        /// Public API item.
        on_click: ToolbarToggleClick,
    },
    /// Public API item.
    Separator,
}

impl ToolbarItem {
    /// Public API item.
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

    /// Public API item.
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

    /// Public API item.
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

    /// Public API item.
    pub fn separator() -> Self {
        Self::Separator
    }

    /// Public API item.
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

/// State the toolbar keeps between frames: the overflow menu's open flag and
/// the width the toolbar was last painted at.
#[derive(Default)]
struct ToolbarState {
    open: bool,
    /// Move focus into the overflow menu on the next frame it is shown.
    focus_menu: bool,
    /// Whole-pixel width of the toolbar when last painted.
    width: Option<f32>,
}

/// How many leading items fit in `budget` pixels, keeping room for a trailing
/// "More" button of `more` pixels when anything must collapse. Items are laid
/// out `gap` apart. Trailing separators are never left dangling at the end.
fn fit_count(widths: &[f32], is_separator: &[bool], gap: f32, budget: f32, more: f32) -> usize {
    let n = widths.len();
    let total: f32 = widths.iter().sum::<f32>() + gap * n.saturating_sub(1) as f32;
    if total <= budget {
        return n;
    }
    let mut used = more;
    let mut fit = 0;
    for w in widths {
        // Each item adds its own width and the gap before the next element.
        let next = used + w + gap;
        if next > budget {
            break;
        }
        used = next;
        fit += 1;
    }
    while fit > 0 && is_separator[fit - 1] {
        fit -= 1;
    }
    fit
}

fn text_width(window: &Window, text: &str, ramp: &fluent_core::TypeRamp) -> f32 {
    let run = TextRun {
        len: text.len(),
        font: window.text_style().font(),
        color: gpui::Hsla::default(),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    f32::from(
        window
            .text_system()
            .shape_line(text.to_string().into(), ramp.font_size_px(), &[run], None)
            .width,
    )
}

/// A compact command surface for panes, lists, and editor-like tools.
///
/// Items that do not fit the toolbar's width collapse, trailing first, into a
/// "More" menu. The More button is the last member of the toolbar's single
/// roving Tab stop: arrows reach it, and Enter, Space or Down open the menu.
#[derive(IntoElement)]
pub struct Toolbar {
    id: ElementId,
    items: Vec<ToolbarItem>,
    max_visible: Option<usize>,
    available_width: Option<Pixels>,
    overflow_open: bool,
    on_overflow_click: Option<ToolbarClick>,
}

impl Toolbar {
    /// Public API item.
    pub fn new() -> Self {
        Self {
            id: "toolbar".into(),
            items: Vec::new(),
            max_visible: None,
            available_width: None,
            overflow_open: false,
            on_overflow_click: None,
        }
    }

    /// Identifies the toolbar so its focus state survives between frames.
    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = id.into();
        self
    }

    /// Public API item.
    pub fn item(mut self, item: ToolbarItem) -> Self {
        self.items.push(item);
        self
    }

    /// Public API item.
    pub fn items(mut self, items: impl IntoIterator<Item = ToolbarItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    /// Caps the number of items shown before the rest collapse into the
    /// overflow menu, on top of the width-driven collapse.
    pub fn max_visible(mut self, max_visible: usize) -> Self {
        self.max_visible = Some(max_visible);
        self
    }

    /// Lays the toolbar out for this width instead of the width it was last
    /// painted at. Without it the toolbar collapses to its own measured width.
    pub fn available_width(mut self, width: Pixels) -> Self {
        self.available_width = Some(width);
        self
    }

    /// Keeps the overflow menu open from the caller's side; the toolbar also
    /// opens and closes it itself.
    pub fn overflow_open(mut self, open: bool) -> Self {
        self.overflow_open = open;
        self
    }

    /// Called when the More button is activated, besides toggling the menu.
    pub fn on_overflow_click(
        mut self,
        f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_overflow_click = Some(Arc::new(f));
        self
    }

    fn with_roving(
        el: gpui::Stateful<gpui::Div>,
        nav: Option<Roving>,
        colors: &fluent_core::ColorScheme,
        radius: gpui::Pixels,
    ) -> gpui::Stateful<gpui::Div> {
        match nav {
            Some(Roving {
                handle,
                tab_stop,
                ring,
                on_key,
            }) => {
                // A tracked handle carries its own tab settings; the toolbar is
                // one Tab stop, held by the roving item.
                let handle = handle.tab_index(0).tab_stop(tab_stop);
                let el = el
                    .track_focus(&handle)
                    .key_context("Toolbar")
                    .fluent_tab_stop(0)
                    .on_key_down(move |ev, win, app| on_key(ev, win, app));
                focus_ring(el, ring, colors, radius)
            }
            None => el,
        }
    }

    /// Natural width of `item` as `render_item` lays it out.
    fn item_width(item: &ToolbarItem, window: &Window, cx: &App) -> f32 {
        let theme = cx.theme();
        match item {
            ToolbarItem::Separator => 1.0 + theme.spacing.xs * 2.0,
            ToolbarItem::Action { label, icon, .. } | ToolbarItem::Toggle { label, icon, .. } => {
                let icon = if icon.is_some() {
                    theme.components.popup_icon_slot
                } else {
                    0.0
                };
                theme.spacing.sm * 2.0
                    + theme.spacing.xs
                    + icon
                    + text_width(window, label, &theme.typography.caption)
            }
        }
    }

    /// Renders one item. `in_menu` lays it out as a row of the overflow menu
    /// with menu roles instead of as a toolbar button.
    fn render_item(
        item: ToolbarItem,
        idx: usize,
        nav: Option<Roving>,
        in_menu: bool,
        cx: &mut App,
    ) -> gpui::AnyElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let components = theme.components;
        let row_h = if in_menu {
            components.list_item_height
        } else {
            components.dock_header_height
        };
        // One icon-to-label rule: menu rows follow the row density, the bar
        // itself has no compact mode.
        let icon_gap = if in_menu {
            components.row_icon_label_gap
        } else {
            components.icon_label_gap
        };

        match item {
            ToolbarItem::Separator if in_menu => div()
                .h(px(1.0))
                .mx(px(spacing.sm))
                .my(px(spacing.xs))
                .bg(colors.stroke_neutral_subtle)
                .into_any_element(),
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
                    .debug_selector(move || format!("toolbar-item-{idx}"))
                    .role(if in_menu {
                        Role::MenuItem
                    } else {
                        Role::Button
                    })
                    .aria_label(label.clone())
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(icon_gap))
                    .h(px(row_h))
                    .px(px(spacing.sm))
                    .rounded(px(radii.sm))
                    .text_ramp(&typography.caption)
                    .text_color(fg)
                    .child(match icon {
                        Some(icon) => svg()
                            .debug_selector(move || format!("toolbar-item-{idx}-icon"))
                            .path(icon)
                            .size(px(components.popup_icon_slot))
                            .text_color(fg)
                            .into_any_element(),
                        None => div().size(px(0.0)).into_any_element(),
                    })
                    .child(
                        div()
                            .debug_selector(move || format!("toolbar-item-{idx}-label"))
                            .child(label),
                    );
                if !disabled {
                    el = el
                        .cursor_pointer()
                        .hover({
                            let hover = colors.subtle_hover;
                            move |s| s.bg(hover)
                        })
                        .active({
                            let pressed = colors.subtle_selected;
                            move |s| s.bg(pressed)
                        })
                        // Enter/Space arrive as gpui's keyboard click on key-up.
                        .on_click(move |ev, win, app| on_click(ev, win, app));
                    el = Self::with_roving(el, nav, &colors, px(radii.sm));
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
                    .debug_selector(move || format!("toolbar-item-{idx}"))
                    .role(if in_menu {
                        Role::MenuItemCheckBox
                    } else {
                        Role::Button
                    })
                    .aria_toggled(if selected {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .aria_label(label.clone())
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(icon_gap))
                    .h(px(row_h))
                    .px(px(spacing.sm))
                    .rounded(px(radii.sm))
                    .bg(bg)
                    .text_ramp(&typography.caption)
                    .text_color(fg)
                    .child(match icon {
                        Some(icon) => svg()
                            .debug_selector(move || format!("toolbar-item-{idx}-icon"))
                            .path(icon)
                            .size(px(components.popup_icon_slot))
                            .text_color(fg)
                            .into_any_element(),
                        None => div().size(px(0.0)).into_any_element(),
                    })
                    .child(
                        div()
                            .debug_selector(move || format!("toolbar-item-{idx}-label"))
                            .child(label),
                    );
                if !disabled {
                    el = el
                        .cursor_pointer()
                        .hover({
                            let hover = colors.neutral_hover;
                            move |s| s.bg(hover)
                        })
                        .active({
                            let pressed = colors.neutral_selected;
                            move |s| s.bg(pressed)
                        })
                        // Enter/Space arrive as gpui's keyboard click on key-up.
                        .on_click(move |ev, win, app| on_click(!selected, ev, win, app));
                    el = Self::with_roving(el, nav, &colors, px(radii.sm));
                }
                let _ = id;
                el.into_any_element()
            }
        }
    }

    /// `item` with its activation also closing the overflow menu.
    #[allow(clippy::arc_with_non_send_sync)]
    fn closing_menu(item: ToolbarItem, state: &Entity<ToolbarState>) -> ToolbarItem {
        let close = {
            let state = state.clone();
            move |app: &mut App| {
                state.update(app, |s, cx| {
                    s.open = false;
                    cx.notify();
                })
            }
        };
        match item {
            ToolbarItem::Action {
                id,
                label,
                icon,
                disabled,
                on_click,
            } => ToolbarItem::Action {
                id,
                label,
                icon,
                disabled,
                on_click: Arc::new(move |ev, win, app| {
                    on_click(ev, win, app);
                    close(app);
                }),
            },
            ToolbarItem::Toggle {
                id,
                label,
                icon,
                selected,
                disabled,
                on_click,
            } => ToolbarItem::Toggle {
                id,
                label,
                icon,
                selected,
                disabled,
                on_click: Arc::new(move |v, ev, win, app| {
                    on_click(v, ev, win, app);
                    close(app);
                }),
            },
            ToolbarItem::Separator => ToolbarItem::Separator,
        }
    }
}

impl Default for Toolbar {
    fn default() -> Self {
        Self::new()
    }
}

fn is_enabled(item: &ToolbarItem) -> bool {
    match item {
        ToolbarItem::Action { disabled, .. } | ToolbarItem::Toggle { disabled, .. } => !disabled,
        ToolbarItem::Separator => false,
    }
}

impl RenderOnce for Toolbar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state((self.id.clone(), "toolbar-state"), cx, |_, _| {
            ToolbarState::default()
        });

        let (sm, xs) = {
            let spacing = cx.theme().spacing;
            (spacing.sm, spacing.xs)
        };
        let budget = self
            .available_width
            .map(f32::from)
            .or(state.read(cx).width)
            .map(|w| w - sm * 2.0);

        let widths: Vec<f32> = self
            .items
            .iter()
            .map(|item| Self::item_width(item, window, cx))
            .collect();
        let separators: Vec<bool> = self
            .items
            .iter()
            .map(|item| matches!(item, ToolbarItem::Separator))
            .collect();
        let mut visible_count = self.items.len();
        if let Some(budget) = budget {
            let more = sm * 2.0 + text_width(window, "More", &cx.theme().typography.caption);
            visible_count = fit_count(&widths, &separators, xs, budget, more);
        }
        if let Some(max) = self.max_visible {
            visible_count = visible_count.min(max);
        }
        let visible = visible_count;
        let has_overflow = visible < self.items.len();
        let menu_open = has_overflow && (self.overflow_open || state.read(cx).open);

        // Visible items, then the More button, share one roving Tab stop.
        let mut enabled: Vec<bool> = self.items.iter().take(visible).map(is_enabled).collect();
        if has_overflow {
            enabled.push(true);
        }
        let handles = crate::roving::handles(window, cx, &self.id, enabled.len());
        let stop = crate::roving::stop_index(window, &handles, &enabled, None);

        let overflow: Vec<ToolbarItem> = self
            .items
            .iter()
            .skip(visible)
            .cloned()
            .map(|item| Self::closing_menu(item, &state))
            .collect();
        let menu_enabled: Vec<bool> = overflow.iter().map(is_enabled).collect();
        let menu_id = ElementId::from((self.id.clone(), "overflow-menu"));
        let menu_handles = crate::roving::handles(window, cx, &menu_id, overflow.len());

        if menu_open && state.read(cx).focus_menu {
            state.update(cx, |s, _| s.focus_menu = false);
            if let Some(first) = menu_enabled.iter().position(|&e| e) {
                let handle = menu_handles[first].clone();
                window.defer(cx, move |win, app| handle.focus(win, app));
            }
        }

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let components = theme.components;
        let radii = theme.radii;
        let typography = theme.typography;

        let measure = {
            let state = state.clone();
            canvas(
                move |bounds, _, cx| {
                    let width = f32::from(bounds.size.width).round();
                    if state.read(cx).width != Some(width) {
                        state.update(cx, |s, cx| {
                            s.width = Some(width);
                            cx.notify();
                        });
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .size_full()
        };

        let mut root = div()
            .id(self.id.clone())
            .role(Role::Toolbar)
            .aria_orientation(gpui::accesskit::Orientation::Horizontal)
            .flex()
            .flex_row()
            .items_center()
            .relative()
            .gap(px(spacing.xs))
            .h(px(components.dock_header_height))
            .px(px(spacing.sm))
            .bg(colors.surface_dim)
            .border_b_1()
            .border_color(colors.stroke_neutral_subtle)
            .child(measure);

        for (idx, item) in self.items.iter().take(visible).cloned().enumerate() {
            let (nav_handles, nav_enabled) = (handles.clone(), enabled.clone());
            let nav = Roving {
                handle: handles[idx].clone(),
                tab_stop: stop == Some(idx),
                ring: enabled[idx] && is_keyboard_focused(&handles[idx], window),
                on_key: Arc::new(move |ev, win, app| {
                    if let Some(next) =
                        crate::roving::target(ev.keystroke.key.as_str(), idx, &nav_enabled)
                    {
                        nav_handles[next].focus(win, app);
                        app.stop_propagation();
                    }
                }),
            };
            root = root.child(Self::render_item(item, idx, Some(nav), false, cx));
        }

        if has_overflow {
            let more_idx = visible;
            let more_handle = handles[more_idx].clone();
            let (nav_handles, nav_enabled) = (handles.clone(), enabled.clone());
            let open_state = state.clone();
            let nav = Roving {
                handle: more_handle.clone(),
                tab_stop: stop == Some(more_idx),
                ring: is_keyboard_focused(&more_handle, window),
                on_key: Arc::new(move |ev, win, app| {
                    let key = ev.keystroke.key.as_str();
                    if key == "down" {
                        open_state.update(app, |s, cx| {
                            s.open = true;
                            s.focus_menu = true;
                            cx.notify();
                        });
                        app.stop_propagation();
                    } else if let Some(next) = crate::roving::target(key, more_idx, &nav_enabled) {
                        nav_handles[next].focus(win, app);
                        app.stop_propagation();
                    }
                }),
            };

            let click_state = state.clone();
            let handler = self.on_overflow_click.clone();
            let overflow_button = div()
                .id("toolbar-overflow")
                .debug_selector(|| "toolbar-overflow".to_string())
                .role(Role::Button)
                .aria_label("More")
                .aria_expanded(menu_open)
                .flex()
                .items_center()
                .justify_center()
                .h(px(components.dock_header_height))
                .px(px(spacing.sm))
                .rounded(px(radii.sm))
                .text_ramp(&typography.caption)
                .text_color(colors.on_neutral)
                .cursor_pointer()
                .hover({
                    let hover = colors.subtle_hover;
                    move |s| s.bg(hover)
                })
                .active({
                    let pressed = colors.subtle_selected;
                    move |s| s.bg(pressed)
                })
                .child("More")
                // Enter/Space arrive as gpui's keyboard click on key-up.
                .on_click(move |ev, win, app| {
                    click_state.update(app, |s, cx| {
                        s.open = !s.open;
                        cx.notify();
                    });
                    if let Some(handler) = &handler {
                        handler(ev, win, app);
                    }
                });
            let overflow_button =
                Self::with_roving(overflow_button, Some(nav), &colors, px(radii.sm));
            root = root.child(overflow_button);

            if menu_open {
                let mut menu = div()
                    .id("toolbar-overflow-menu")
                    .role(Role::Menu)
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
                    let (nav_handles, nav_enabled) = (menu_handles.clone(), menu_enabled.clone());
                    let (close_state, back) = (state.clone(), more_handle.clone());
                    let nav = Roving {
                        handle: menu_handles[idx].clone(),
                        // Items are reached by arrows from the More button's
                        // Down key, not by Tab.
                        tab_stop: false,
                        ring: menu_enabled[idx] && is_keyboard_focused(&menu_handles[idx], window),
                        on_key: Arc::new(move |ev, win, app| {
                            let key = ev.keystroke.key.as_str();
                            if key == "escape" {
                                close_state.update(app, |s, cx| {
                                    s.open = false;
                                    cx.notify();
                                });
                                back.focus(win, app);
                                app.stop_propagation();
                            } else if let Some(next) = crate::roving::target(key, idx, &nav_enabled)
                            {
                                if matches!(key, "up" | "down" | "home" | "end") {
                                    nav_handles[next].focus(win, app);
                                    app.stop_propagation();
                                }
                            }
                        }),
                    };
                    menu = menu.child(Self::render_item(item, idx + visible, Some(nav), true, cx));
                }
                root = root.child(menu);
            }
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Render, TestAppContext};

    use super::*;
    use crate::roving::test_support::{outer_ring_quads, press, tab};

    struct Host(Rc<Cell<usize>>);

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let hits = self.0.clone();
            Toolbar::new()
                .item(ToolbarItem::action("a", "A", |_, _, _| {}))
                .item(ToolbarItem::separator())
                .item(ToolbarItem::action("b", "B", |_, _, _| {}).disabled(true))
                .item(ToolbarItem::action("c", "C", move |_, _, _| {
                    hits.set(hits.get() + 1)
                }))
        }
    }

    #[gpui::test]
    fn arrows_skip_disabled_and_home_end_jump(cx: &mut TestAppContext) {
        let hits = Rc::new(Cell::new(0));
        let (_host, cx) = open_fluent_test_window(cx, |_, _| Host(hits.clone()));
        cx.click_selector("toolbar-item-0");
        cx.type_keys("right"); // skips the separator and disabled item
        press(cx, "enter");
        assert_eq!(hits.get(), 1);
        cx.type_keys("home right");
        press(cx, "enter");
        assert_eq!(hits.get(), 2);
        cx.type_keys("end");
        press(cx, "enter");
        assert_eq!(hits.get(), 3);
    }

    struct Toggles {
        action_hits: Rc<Cell<usize>>,
        toggled: Rc<Cell<Option<bool>>>,
        disabled_hits: Rc<Cell<usize>>,
    }

    impl Render for Toggles {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (a, t, d) = (
                self.action_hits.clone(),
                self.toggled.clone(),
                self.disabled_hits.clone(),
            );
            Toolbar::new()
                .item(ToolbarItem::action("a", "A", move |_, _, _| {
                    a.set(a.get() + 1)
                }))
                .item(ToolbarItem::toggle("t", "T", false, move |v, _, _, _| {
                    t.set(Some(v))
                }))
                .item(
                    ToolbarItem::action("d", "D", move |_, _, _| d.set(d.get() + 1)).disabled(true),
                )
        }
    }

    #[gpui::test]
    fn clicking_dispatches_action_and_toggle_but_not_disabled(cx: &mut TestAppContext) {
        let action_hits = Rc::new(Cell::new(0));
        let toggled = Rc::new(Cell::new(None));
        let disabled_hits = Rc::new(Cell::new(0));
        let (_host, cx) = open_fluent_test_window(cx, |_, _| Toggles {
            action_hits: action_hits.clone(),
            toggled: toggled.clone(),
            disabled_hits: disabled_hits.clone(),
        });
        cx.click_selector("toolbar-item-0");
        assert_eq!(action_hits.get(), 1);
        cx.click_selector("toolbar-item-1");
        assert_eq!(toggled.get(), Some(true));
        cx.click_selector("toolbar-item-2");
        assert_eq!(disabled_hits.get(), 0);
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_toolbar_roles(cx: &mut TestAppContext) {
        let (_host, cx) = open_fluent_test_window(cx, |_, _| Toggles {
            action_hits: Rc::new(Cell::new(0)),
            toggled: Rc::new(Cell::new(None)),
            disabled_hits: Rc::new(Cell::new(0)),
        });
        let snap = cx.accessibility_snapshot();
        let joined = snap.join("\n");
        assert!(joined.contains("Toolbar"), "{joined}");
        assert!(joined.contains("Button \"A\""), "{joined}");
        assert!(joined.contains("Button \"T\""), "{joined}");
        assert!(joined.contains("Button \"D\""), "{joined}");
    }

    /// `[toolbar: A, T(toggle), D(disabled)] [checkbox "after"]`.
    struct Form {
        action: Rc<Cell<usize>>,
        toggle: Rc<Cell<usize>>,
        after: Rc<Cell<usize>>,
    }

    impl Render for Form {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let (a, t, after) = (self.action.clone(), self.toggle.clone(), self.after.clone());
            div()
                .child(
                    Toolbar::new()
                        .item(ToolbarItem::action("a", "A", move |_, _, _| {
                            a.set(a.get() + 1)
                        }))
                        .item(ToolbarItem::toggle("t", "T", false, move |_, _, _, _| {
                            t.set(t.get() + 1)
                        }))
                        .item(ToolbarItem::action("d", "D", |_, _, _| {}).disabled(true)),
                )
                .child(
                    fluent_primitives::Checkbox::new("after")
                        .on_click(move |_, _, _, _| after.set(after.get() + 1)),
                )
        }
    }

    fn open_form(cx: &mut TestAppContext) -> ([Rc<Cell<usize>>; 3], &mut gpui::VisualTestContext) {
        let counters: [Rc<Cell<usize>>; 3] = Default::default();
        let c = counters.clone();
        let (_host, cx) = open_fluent_test_window(cx, move |_, _| {
            let [action, toggle, after] = c;
            Form {
                action,
                toggle,
                after,
            }
        });
        (counters, cx)
    }

    #[gpui::test]
    fn toolbar_is_one_tab_stop_and_enter_activates_once(cx: &mut TestAppContext) {
        let ([action, toggle, after], cx) = open_form(cx);
        tab(cx, 1);
        press(cx, "enter");
        assert_eq!(action.get(), 1, "Enter activates exactly once");
        press(cx, "space");
        assert_eq!(action.get(), 2, "Space activates");
        press(cx, "right");
        press(cx, "space");
        assert_eq!(toggle.get(), 1, "arrow moves to the toggle");
        // One Tab leaves the whole toolbar (the disabled item is no stop).
        cx.type_keys("tab");
        press(cx, "space");
        assert_eq!(after.get(), 1, "Tab left the toolbar for the checkbox");
        assert_eq!(toggle.get(), 1);
    }

    #[gpui::test]
    fn toolbar_ring_only_for_keyboard_focus(cx: &mut TestAppContext) {
        let (_, cx) = open_form(cx);
        assert_eq!(outer_ring_quads(cx), 0, "nothing focused");
        tab(cx, 1);
        cx.type_keys("right"); // a key press switches to keyboard modality
        assert_eq!(outer_ring_quads(cx), 1, "keyboard focus draws the ring");
        cx.click_selector("toolbar-item-1");
        cx.update(|window, app| window.draw(app).clear(app));
        assert_eq!(outer_ring_quads(cx), 0, "mouse focus draws no ring");
    }

    #[test]
    fn fit_count_keeps_everything_when_it_fits_and_collapses_the_tail_otherwise() {
        let w = [50.0, 50.0, 50.0, 50.0];
        let sep = [false; 4];
        assert_eq!(fit_count(&w, &sep, 4.0, 212.0, 40.0), 4);
        // 150 wide: More (40) plus two items (2 x 54) fit, three do not.
        assert_eq!(fit_count(&w, &sep, 4.0, 150.0, 40.0), 2);
        assert_eq!(fit_count(&w, &sep, 4.0, 10.0, 40.0), 0);
        // A separator never ends the visible run.
        let sep = [false, true, false, false];
        assert_eq!(fit_count(&w, &sep, 4.0, 150.0, 40.0), 1);
    }

    /// Eight actions (the seventh disabled), in a toolbar `width` wide.
    struct Narrow {
        width: f32,
        /// Pass the width to the toolbar instead of boxing it in a parent.
        explicit: bool,
        hits: Rc<RefCell<Vec<usize>>>,
    }

    impl Render for Narrow {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let mut toolbar = Toolbar::new().id("narrow");
            for i in 0..8usize {
                let hits = self.hits.clone();
                let item =
                    ToolbarItem::action(format!("a{i}"), format!("Action {i}"), move |_, _, _| {
                        hits.borrow_mut().push(i)
                    });
                toolbar = toolbar.item(if i == 6 { item.disabled(true) } else { item });
            }
            if self.explicit {
                toolbar.available_width(px(self.width)).into_any_element()
            } else {
                div()
                    .w(px(self.width))
                    .child(toolbar)
                    .child(fluent_primitives::Checkbox::new("after"))
                    .into_any_element()
            }
        }
    }

    fn open_narrow(
        cx: &mut TestAppContext,
        width: f32,
        explicit: bool,
    ) -> (Rc<RefCell<Vec<usize>>>, &mut gpui::VisualTestContext) {
        let hits = Rc::new(RefCell::new(Vec::new()));
        let h = hits.clone();
        let (_host, cx) = open_fluent_test_window(cx, move |_, _| Narrow {
            width,
            explicit,
            hits: h,
        });
        // The measured width arrives after the first paint.
        for _ in 0..3 {
            cx.update(|window, app| window.draw(app).clear(app));
        }
        (hits, cx)
    }

    #[gpui::test]
    fn narrow_toolbar_collapses_trailing_items_into_more(cx: &mut TestAppContext) {
        let (_, cx) = open_narrow(cx, 260.0, true);
        assert!(cx.selector_bounds("toolbar-item-0").is_some());
        assert!(
            cx.selector_bounds("toolbar-overflow").is_some(),
            "More shown"
        );
        assert!(
            cx.selector_bounds("toolbar-item-7").is_none(),
            "tail collapsed"
        );
    }

    #[gpui::test]
    fn toolbar_collapses_to_its_own_measured_width(cx: &mut TestAppContext) {
        let (_, cx) = open_narrow(cx, 260.0, false);
        assert!(
            cx.selector_bounds("toolbar-overflow").is_some(),
            "More shown"
        );
        assert!(cx.selector_bounds("toolbar-item-7").is_none());
        let (_, cx) = open_narrow(cx, 1600.0, false);
        assert!(
            cx.selector_bounds("toolbar-item-7").is_some(),
            "wide: all fit"
        );
        assert!(
            cx.selector_bounds("toolbar-overflow").is_none(),
            "no More when all fit"
        );
    }

    #[gpui::test]
    fn more_button_is_a_roving_stop_with_a_keyboard_only_ring(cx: &mut TestAppContext) {
        let (hits, cx) = open_narrow(cx, 260.0, true);
        assert_eq!(outer_ring_quads(cx), 0, "nothing focused");
        tab(cx, 1);
        // The toolbar is one Tab stop: End moves to the last stop, More.
        cx.type_keys("end");
        assert_eq!(
            outer_ring_quads(cx),
            1,
            "keyboard focus on More draws the ring"
        );
        assert!(
            cx.selector_bounds("toolbar-item-7").is_none(),
            "menu still closed"
        );
        press(cx, "enter");
        let joined = cx.accessibility_snapshot().join("\n");
        assert!(joined.contains("Menu"), "{joined}");
        assert!(joined.contains("MenuItem \"Action 7\""), "{joined}");
        assert!(hits.borrow().is_empty());
        // Mouse focus draws no ring.
        cx.click_selector("toolbar-overflow");
        cx.update(|window, app| window.draw(app).clear(app));
        assert_eq!(outer_ring_quads(cx), 0, "mouse focus draws no ring");
    }

    #[gpui::test]
    fn down_opens_overflow_menu_and_enter_activates_a_collapsed_item(cx: &mut TestAppContext) {
        let (hits, cx) = open_narrow(cx, 260.0, true);
        tab(cx, 1);
        cx.type_keys("end down");
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
        // Focus is in the menu; Down skips the disabled seventh item.
        let first_hidden = (0..8)
            .find(|i| {
                let sel: &'static str = Box::leak(format!("toolbar-item-{i}").into_boxed_str());
                cx.selector_bounds(sel).is_some() && *i > 0 && {
                    let joined = cx.accessibility_snapshot().join("\n");
                    joined.contains(&format!("MenuItem \"Action {i}\""))
                }
            })
            .expect("a menu item is shown");
        cx.type_keys("down");
        press(cx, "enter");
        let hits = hits.borrow();
        assert_eq!(hits.len(), 1, "one activation");
        assert!(
            hits[0] > first_hidden,
            "Down moved to the next item: {hits:?}"
        );
        assert_ne!(hits[0], 6, "disabled item skipped");
        drop(hits);
        let joined = cx.accessibility_snapshot().join("\n");
        assert!(
            !joined.contains("MenuItem"),
            "menu closed after activation: {joined}"
        );
    }

    #[gpui::test]
    fn escape_closes_overflow_menu_and_returns_focus_to_more(cx: &mut TestAppContext) {
        let (_, cx) = open_narrow(cx, 260.0, true);
        tab(cx, 1);
        cx.type_keys("end down");
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
        cx.type_keys("escape");
        cx.update(|window, app| window.draw(app).clear(app));
        let joined = cx.accessibility_snapshot().join("\n");
        assert!(!joined.contains("MenuItem"), "{joined}");
        assert_eq!(outer_ring_quads(cx), 1, "focus is back on More");
    }

    struct IconToolbar;

    impl Render for IconToolbar {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            Toolbar::new().item(ToolbarItem::action_with_icon(
                "a",
                "Alpha",
                "icons/none.svg",
                |_, _, _| {},
            ))
        }
    }

    /// The icon-to-label gap of a toolbar item (WI-3257: 8px, one rule).
    #[gpui::test]
    fn toolbar_icon_to_label_gap_is_the_shared_token(cx: &mut TestAppContext) {
        let (_host, cx) = open_fluent_test_window(cx, |_, _| IconToolbar);
        cx.run_until_parked();
        let icon = cx.selector_bounds("toolbar-item-0-icon").expect("icon");
        let label = cx.selector_bounds("toolbar-item-0-label").expect("label");
        let gap = f32::from(label.origin.x - icon.right());
        assert_eq!(gap, 8.0, "toolbar icon-to-label gap");
    }
}
