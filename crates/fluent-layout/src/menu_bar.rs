use std::sync::{Arc, Mutex};

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    anchored, canvas, deferred, div, point, prelude::*, px, svg, App, Bounds, ClickEvent, Context,
    FocusHandle, IntoElement, KeyDownEvent, MouseButton, Pixels, Render, SharedString, WeakEntity,
    Window,
};

use crate::context_menu::ContextMenuItem;
use crate::menu_tree::{self, MenuRenderEnv};

#[derive(Clone)]
pub struct MenuDef {
    pub label: SharedString,
    pub items: Vec<ContextMenuItem>,
}

/// Builder facade for menu bar items.
///
/// `MenuBar` uses `ContextMenuItem` as its canonical item model so app menus,
/// context menus, and cascades share the same semantics.
pub struct MenuItemDef;

impl MenuItemDef {
    pub fn action(
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> ContextMenuItem {
        ContextMenuItem::action(label, on_click)
    }

    pub fn action_with_shortcut(
        label: impl Into<SharedString>,
        shortcut: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> ContextMenuItem {
        ContextMenuItem::action(label, on_click).shortcut(shortcut)
    }

    pub fn action_with_icon(
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> ContextMenuItem {
        ContextMenuItem::action_with_icon(label, icon, on_click)
    }

    pub fn checkbox(
        label: impl Into<SharedString>,
        checked: bool,
        on_click: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> ContextMenuItem {
        ContextMenuItem::checkbox(label, checked, on_click)
    }

    pub fn radio(
        label: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> ContextMenuItem {
        ContextMenuItem::radio(label, selected, on_click)
    }

    pub fn submenu(label: impl Into<SharedString>, items: Vec<ContextMenuItem>) -> ContextMenuItem {
        ContextMenuItem::submenu(label, items)
    }

    pub fn separator() -> ContextMenuItem {
        ContextMenuItem::Separator
    }
}

/// A horizontal application menu bar (File | Edit | View | ...).
///
/// Top-level menus render attached dropdowns. Items use the same model as
/// `ContextMenu`, including actions, checkbox/radio state, disabled rows,
/// separators, and cascaded submenus.
pub struct MenuBar {
    pub menus: Vec<MenuDef>,
    pub open_index: Option<usize>,
    closing: bool,
    motion_epoch: u64,
    active_index: usize,
    selected_path: Vec<usize>,
    open_path: Vec<usize>,
    focus_handle: FocusHandle,
    trigger_bounds: Vec<Arc<Mutex<Option<Bounds<Pixels>>>>>,
}

impl MenuBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self {
            menus: vec![],
            open_index: None,
            closing: false,
            motion_epoch: 0,
            active_index: 0,
            selected_path: Vec::new(),
            open_path: Vec::new(),
            focus_handle: cx.focus_handle().tab_stop(true),
            trigger_bounds: Vec::new(),
        }
    }

    pub fn menu(mut self, label: impl Into<SharedString>, items: Vec<ContextMenuItem>) -> Self {
        self.menus.push(MenuDef {
            label: label.into(),
            items,
        });
        self.trigger_bounds.push(Arc::new(Mutex::new(None)));
        self
    }

    pub(crate) fn ensure_front_menu(
        &mut self,
        label: impl Into<SharedString>,
        items: Vec<ContextMenuItem>,
    ) {
        let label = label.into();
        if self.menus.first().is_some_and(|menu| menu.label == label) {
            return;
        }

        self.menus.insert(0, MenuDef { label, items });
        self.trigger_bounds.insert(0, Arc::new(Mutex::new(None)));

        if let Some(open_index) = &mut self.open_index {
            *open_index += 1;
            self.active_index += 1;
        } else {
            self.active_index = 0;
        }
    }

    fn close(&mut self, cx: &mut Context<Self>) {
        if self.open_index.is_none() || self.closing {
            return;
        }

        self.closing = true;
        self.motion_epoch = self.motion_epoch.wrapping_add(1);
        let epoch = self.motion_epoch;
        let duration = cx.theme().motion().popup_exit_duration();

        cx.spawn(
            move |bar: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    cx.background_executor().timer(duration).await;
                    bar.update(&mut cx, move |bar, cx| {
                        if bar.motion_epoch == epoch && bar.closing {
                            bar.open_index = None;
                            bar.closing = false;
                            bar.selected_path.clear();
                            bar.open_path.clear();
                            cx.notify();
                        }
                    })
                    .ok();
                }
            },
        )
        .detach();

        cx.notify();
    }

    fn current_items(&self) -> Option<&[ContextMenuItem]> {
        if self.closing {
            return None;
        }
        let idx = self.open_index?;
        Some(&self.menus.get(idx)?.items)
    }

    fn open_menu(&mut self, idx: usize, select_last: bool) {
        if self.menus.is_empty() {
            return;
        }
        let idx = idx.min(self.menus.len() - 1);
        self.active_index = idx;
        self.closing = false;
        self.motion_epoch = self.motion_epoch.wrapping_add(1);
        self.open_index = Some(idx);
        self.open_path.clear();
        self.selected_path = if select_last {
            menu_tree::last_enabled_path(&self.menus[idx].items, &[]).unwrap_or_default()
        } else {
            menu_tree::first_enabled_path(&self.menus[idx].items, &[]).unwrap_or_default()
        };
    }

    fn move_top_menu(&mut self, delta: isize) {
        if self.menus.is_empty() {
            return;
        }
        let len = self.menus.len();
        let next = if delta < 0 {
            if self.active_index == 0 {
                len - 1
            } else {
                self.active_index - 1
            }
        } else {
            (self.active_index + 1) % len
        };
        if self.open_index.is_some() && !self.closing {
            self.open_menu(next, false);
        } else {
            self.active_index = next;
        }
    }

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        if self.closing {
            return;
        }
        let Some(open_idx) = self.open_index else {
            return;
        };
        let Some(menu) = self.menus.get(open_idx) else {
            return;
        };
        if menu_tree::move_selection(&menu.items, &mut self.selected_path, delta) {
            self.open_path = menu_tree::parent_path(&self.selected_path);
            cx.notify();
        }
    }

    fn enter_submenu(&mut self, cx: &mut Context<Self>) {
        if self.closing {
            return;
        }
        let Some(open_idx) = self.open_index else {
            return;
        };
        let Some(menu) = self.menus.get(open_idx) else {
            return;
        };
        if menu_tree::enter_submenu(&menu.items, &mut self.selected_path, &mut self.open_path) {
            cx.notify();
        }
    }

    fn leave_submenu(&mut self, cx: &mut Context<Self>) {
        menu_tree::leave_submenu(&mut self.selected_path, &mut self.open_path);
        cx.notify();
    }

    fn invoke_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(items) = self.current_items() else {
            return;
        };
        let Some(item) = menu_tree::enabled_item_at_path(items, &self.selected_path).cloned()
        else {
            return;
        };

        match item {
            ContextMenuItem::Action {
                disabled, on_click, ..
            } if !disabled => {
                let ev = ClickEvent::default();
                on_click(&ev, window, cx);
                self.close(cx);
            }
            ContextMenuItem::Checkbox {
                checked,
                disabled,
                on_click,
                ..
            } if !disabled => {
                let ev = ClickEvent::default();
                on_click(!checked, &ev, window, cx);
                self.close(cx);
            }
            ContextMenuItem::Radio {
                disabled, on_click, ..
            } if !disabled => {
                let ev = ClickEvent::default();
                on_click(&ev, window, cx);
                self.close(cx);
            }
            ContextMenuItem::Submenu { disabled, .. } if !disabled => self.enter_submenu(cx),
            _ => {}
        }
    }

    fn handle_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.closing {
            return;
        }

        match ev.keystroke.key.as_str() {
            "escape" => self.close(cx),
            "left" => {
                if self.selected_path.len() > 1 {
                    self.leave_submenu(cx);
                } else {
                    self.move_top_menu(-1);
                    cx.notify();
                }
            }
            "right" => {
                let submenu_selected = self
                    .current_items()
                    .and_then(|items| menu_tree::item_at_path(items, &self.selected_path))
                    .is_some_and(|item| matches!(item, ContextMenuItem::Submenu { .. }));
                if submenu_selected {
                    self.enter_submenu(cx);
                } else {
                    self.move_top_menu(1);
                    cx.notify();
                }
            }
            "down" => {
                if self.open_index.is_none() {
                    self.open_menu(self.active_index, false);
                    cx.notify();
                } else {
                    self.move_selection(1, cx);
                }
            }
            "up" => {
                if self.open_index.is_none() {
                    self.open_menu(self.active_index, true);
                    cx.notify();
                } else {
                    self.move_selection(-1, cx);
                }
            }
            "return" | "space" => {
                if self.open_index.is_some() {
                    self.invoke_selected(window, cx);
                } else {
                    self.open_menu(self.active_index, false);
                    cx.notify();
                }
            }
            _ => {}
        }
    }

    fn render_items(
        &self,
        items: &[ContextMenuItem],
        path_prefix: Vec<usize>,
        self_weak: WeakEntity<Self>,
        env: &MenuRenderEnv,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let mut menu = div()
            .relative()
            .flex()
            .flex_col()
            .bg(env.colors.surface)
            .border_1()
            .border_color(env.colors.stroke_neutral)
            .rounded(px(env.radii.md))
            .py(px(env.spacing.xs))
            .min_w(px(env.components.menu_min_width))
            .shadow_md();

        let mut row_top = env.spacing.xs;
        for (idx, item) in items.iter().enumerate() {
            let mut path = path_prefix.clone();
            path.push(idx);
            let selected = self.selected_path == path;

            match item {
                ContextMenuItem::Action {
                    label,
                    shortcut,
                    icon,
                    disabled,
                    on_click,
                } => {
                    let cb = on_click.clone();
                    let closer = self_weak.clone();
                    let hover_parent = menu_tree::parent_path(&path);
                    let select_path = path.clone();
                    let fg = if *disabled {
                        env.colors.on_neutral_disabled
                    } else {
                        env.colors.on_neutral
                    };
                    let mut row = menu_tree::row_base("menu-item", &path, selected, fg, env)
                        .on_mouse_move(cx.listener(move |bar: &mut MenuBar, _, _, cx| {
                            if bar.closing {
                                return;
                            }
                            bar.selected_path = select_path.clone();
                            bar.open_path = hover_parent.clone();
                            cx.notify();
                        }));
                    if !disabled && !self.closing {
                        row = row
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_mouse_down(MouseButton::Left, move |_, win, cx| {
                                if closer.update(cx, |bar, _| bar.closing).unwrap_or(true) {
                                    return;
                                }
                                cx.stop_propagation();
                                let ev = ClickEvent::default();
                                cb(&ev, win, cx);
                                closer.update(cx, |bar, cx| bar.close(cx)).ok();
                            });
                    }
                    row = row
                        .child(menu_tree::icon_slot(icon.clone(), fg, env))
                        .child(div().flex_1().child(label.clone()));
                    if let Some(shortcut) = shortcut.clone() {
                        row = row.child(
                            div()
                                .flex_none()
                                .pl(px(env.spacing.xl))
                                .text_size(px(env.typography.caption.size))
                                .text_color(env.colors.on_subtle_disabled)
                                .child(shortcut),
                        );
                    }
                    menu = menu.child(row);
                    row_top += env.components.menu_item_height;
                }
                ContextMenuItem::Checkbox {
                    label,
                    checked,
                    disabled,
                    on_click,
                } => {
                    let cb = on_click.clone();
                    let closer = self_weak.clone();
                    let hover_parent = menu_tree::parent_path(&path);
                    let select_path = path.clone();
                    let next = !checked;
                    let fg = if *disabled {
                        env.colors.on_neutral_disabled
                    } else {
                        env.colors.on_neutral
                    };
                    let mut row = menu_tree::row_base("menu-item", &path, selected, fg, env)
                        .on_mouse_move(cx.listener(move |bar: &mut MenuBar, _, _, cx| {
                            if bar.closing {
                                return;
                            }
                            bar.selected_path = select_path.clone();
                            bar.open_path = hover_parent.clone();
                            cx.notify();
                        }));
                    if !disabled && !self.closing {
                        row = row
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_mouse_down(MouseButton::Left, move |_, win, cx| {
                                if closer.update(cx, |bar, _| bar.closing).unwrap_or(true) {
                                    return;
                                }
                                cx.stop_propagation();
                                let ev = ClickEvent::default();
                                cb(next, &ev, win, cx);
                                closer.update(cx, |bar, cx| bar.close(cx)).ok();
                            });
                    }
                    let check = if *checked {
                        svg()
                            .path("icons/checkmark.svg")
                            .size(px(env.components.popup_icon_slot))
                            .text_color(fg)
                            .into_any_element()
                    } else {
                        div()
                            .size(px(env.components.popup_icon_slot))
                            .into_any_element()
                    };
                    menu = menu.child(
                        row.child(
                            div()
                                .flex_none()
                                .size(px(env.components.popup_icon_slot))
                                .child(check),
                        )
                        .child(div().flex_1().child(label.clone())),
                    );
                    row_top += env.components.menu_item_height;
                }
                ContextMenuItem::Radio {
                    label,
                    selected: checked,
                    disabled,
                    on_click,
                } => {
                    let cb = on_click.clone();
                    let closer = self_weak.clone();
                    let hover_parent = menu_tree::parent_path(&path);
                    let select_path = path.clone();
                    let fg = if *disabled {
                        env.colors.on_neutral_disabled
                    } else {
                        env.colors.on_neutral
                    };
                    let mut row = menu_tree::row_base("menu-item", &path, selected, fg, env)
                        .on_mouse_move(cx.listener(move |bar: &mut MenuBar, _, _, cx| {
                            if bar.closing {
                                return;
                            }
                            bar.selected_path = select_path.clone();
                            bar.open_path = hover_parent.clone();
                            cx.notify();
                        }));
                    if !disabled && !self.closing {
                        row = row
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_mouse_down(MouseButton::Left, move |_, win, cx| {
                                if closer.update(cx, |bar, _| bar.closing).unwrap_or(true) {
                                    return;
                                }
                                cx.stop_propagation();
                                let ev = ClickEvent::default();
                                cb(&ev, win, cx);
                                closer.update(cx, |bar, cx| bar.close(cx)).ok();
                            });
                    }
                    menu = menu.child(
                        row.child(
                            div()
                                .flex_none()
                                .size(px(env.components.popup_icon_slot))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child(
                                    div()
                                        .size(px(if *checked { 6.0 } else { 0.0 }))
                                        .rounded(px(env.radii.pill))
                                        .bg(fg),
                                ),
                        )
                        .child(div().flex_1().child(label.clone())),
                    );
                    row_top += env.components.menu_item_height;
                }
                ContextMenuItem::Submenu {
                    label,
                    icon,
                    disabled,
                    items,
                } => {
                    let select_path = path.clone();
                    let fg = if *disabled {
                        env.colors.on_neutral_disabled
                    } else {
                        env.colors.on_neutral
                    };
                    let mut row = menu_tree::row_base("menu-item", &path, selected, fg, env);
                    if !disabled && !self.closing {
                        row = row
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_mouse_move(cx.listener(move |bar: &mut MenuBar, _, _, cx| {
                                if bar.closing {
                                    return;
                                }
                                bar.selected_path = select_path.clone();
                                bar.open_path = select_path.clone();
                                cx.notify();
                            }));
                    }
                    menu = menu.child(
                        row.child(menu_tree::icon_slot(icon.clone(), fg, env))
                            .child(div().flex_1().child(label.clone()))
                            .child(
                                svg()
                                    .path("icons/chevron_right.svg")
                                    .size(px(12.0))
                                    .text_color(fg),
                            ),
                    );
                    if !disabled && menu_tree::path_starts_with(&self.open_path, &path) {
                        let submenu =
                            self.render_items(items, path.clone(), self_weak.clone(), env, cx);
                        menu = menu
                            .child(div().absolute().left_full().top(px(row_top)).child(submenu));
                    }
                    row_top += env.components.menu_item_height;
                }
                ContextMenuItem::Separator => {
                    menu = menu.child(
                        div()
                            .h(px(env.components.popup_separator_height))
                            .mx(px(env.spacing.sm))
                            .my(px(env.spacing.xs))
                            .bg(env.colors.stroke_neutral_subtle),
                    );
                    row_top += env.components.popup_separator_height + env.spacing.xs * 2.0;
                }
            }
        }

        menu.into_any_element()
    }
}

impl Render for MenuBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let components = theme.components;
        let motion = theme.motion();
        let bar_height = components.menu_bar_height;
        let env = MenuRenderEnv::from_theme(theme);

        let self_weak = cx.entity().downgrade();
        let open_idx = self.open_index;
        let active_idx = self.active_index.min(self.menus.len().saturating_sub(1));

        let bar_bg = colors.surface_dim;
        let trigger_hover_bg = colors.subtle_hover;
        let trigger_active_bg = colors.neutral;
        let trigger_text = colors.on_neutral;

        // Build the trigger buttons row.
        let mut triggers_row = div()
            .flex()
            .flex_row()
            .items_center()
            .h(px(bar_height))
            .bg(bar_bg)
            .px(px(spacing.sm));

        for (idx, menu_def) in self.menus.iter().enumerate() {
            let label = menu_def.label.clone();
            let is_open = open_idx == Some(idx);
            let sw_click = self_weak.clone();
            let is_active = active_idx == idx;
            let trigger_bg = if is_open || is_active {
                trigger_active_bg
            } else {
                bar_bg
            };
            let focus_handle = self.focus_handle.clone();

            // 0×0 absolute overlay on the trigger — prepaint captures window-coordinate bounds.
            let bounds_arc = self.trigger_bounds.get(idx).cloned().unwrap_or_default();
            let tracker = canvas(
                move |bounds, _, _| {
                    if let Ok(mut slot) = bounds_arc.lock() {
                        *slot = Some(bounds);
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0();

            let trigger = div()
                .id(SharedString::from(format!("menu-trigger-{idx}")))
                .relative()
                .flex()
                .items_center()
                .px(px(spacing.md))
                .h_full()
                .text_size(px(typography.body.size))
                .text_color(trigger_text)
                .bg(trigger_bg)
                .rounded(px(radii.sm))
                .cursor_pointer()
                .hover(move |s| {
                    if open_idx != Some(idx) {
                        s.bg(trigger_hover_bg)
                    } else {
                        s
                    }
                })
                .on_mouse_down(MouseButton::Left, move |_, win, cx| {
                    win.focus(&focus_handle);
                    sw_click
                        .update(cx, |bar, cx| {
                            if bar.open_index == Some(idx) && !bar.closing {
                                bar.close(cx);
                            } else if bar.open_index != Some(idx) {
                                bar.open_menu(idx, false);
                                cx.notify();
                            }
                        })
                        .ok();
                })
                .child(label)
                .child(tracker);

            triggers_row = triggers_row.child(trigger);
        }

        // Build the dropdown overlay for the currently open menu (if any).
        // Rendered via deferred + anchored so it paints above all other content at the
        // correct window-coordinate position regardless of the menu bar's depth in the tree.
        let overlay = if let Some(oi) = self.open_index {
            let anchor = self
                .trigger_bounds
                .get(oi)
                .and_then(|arc| arc.lock().ok())
                .and_then(|g| *g)
                .map(|b: Bounds<Pixels>| point(b.origin.x, b.origin.y + b.size.height))
                .unwrap_or_else(|| point(px(0.0), px(bar_height)));

            let items = self
                .menus
                .get(oi)
                .map(|m| m.items.clone())
                .unwrap_or_default();
            let item_list = self.render_items(&items, Vec::new(), self_weak.clone(), &env, cx);
            let item_list = fluent_core::popup_motion_surface(
                ("menu-bar-popup-motion", self.motion_epoch),
                self.closing,
                motion,
                item_list,
            );

            let vp = window.viewport_size();
            let mut popup = div();
            if !self.closing {
                let closer = self_weak.clone();
                popup = popup.child(
                    anchored().position(point(px(0.0), px(0.0))).child(
                        div()
                            .id("menu-backdrop")
                            .w(vp.width)
                            .h(vp.height)
                            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                                cx.stop_propagation();
                                closer.update(cx, |bar, cx| bar.close(cx)).ok();
                            }),
                    ),
                );
            }
            Some(
                deferred(popup.child(anchored().position(anchor).child(item_list)))
                    .with_priority(2),
            )
        } else {
            None
        };

        // Root wrapper. No .relative() here — adding it would make this div the CSS containing
        // block for the absolutely-positioned deferred overlay, constraining the backdrop to
        // bar_height px instead of the full window. Without it, absolute().inset_0() in the
        // deferred overlay stretches to the viewport, giving the backdrop full-window coverage.
        let mut root = div()
            .key_context("MenuBar")
            .track_focus(&self.focus_handle)
            .on_key_down(
                cx.listener(|bar: &mut MenuBar, ev: &KeyDownEvent, window, cx| {
                    bar.handle_key(ev, window, cx);
                }),
            )
            .flex()
            .flex_col()
            .h(px(bar_height));

        root = root.child(triggers_row);
        if let Some(ov) = overlay {
            root = root.child(ov);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use fluent_test_utils::init_fluent_test;
    use gpui::{AppContext as _, TestAppContext};

    use super::*;

    #[gpui::test]
    fn close_keeps_menu_bar_paths_until_exit_duration(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let bar = cx.new(|cx| {
            MenuBar::new(cx).menu(
                "File",
                vec![ContextMenuItem::submenu(
                    "More",
                    vec![ContextMenuItem::action("Open", |_, _, _| {})],
                )],
            )
        });

        bar.update(cx, |bar, cx| {
            bar.open_menu(0, false);
            bar.enter_submenu(cx);
            assert_eq!(bar.open_path, vec![0]);
            assert_eq!(bar.selected_path, vec![0, 0]);

            bar.close(cx);
            assert_eq!(bar.open_index, Some(0));
            assert!(bar.closing);
            assert_eq!(bar.open_path, vec![0]);
            assert_eq!(bar.selected_path, vec![0, 0]);
        });

        cx.executor().advance_clock(Duration::from_millis(159));
        bar.update(cx, |bar, _| {
            assert_eq!(bar.open_index, Some(0));
            assert!(bar.closing);
            assert_eq!(bar.open_path, vec![0]);
            assert_eq!(bar.selected_path, vec![0, 0]);
        });

        cx.executor().advance_clock(Duration::from_millis(1));
        bar.update(cx, |bar, _| {
            assert_eq!(bar.open_index, None);
            assert!(!bar.closing);
            assert!(bar.open_path.is_empty());
            assert!(bar.selected_path.is_empty());
        });
    }

    #[gpui::test]
    fn ensure_front_menu_keeps_trigger_bounds_aligned(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let bar = cx.new(|cx| {
            MenuBar::new(cx)
                .menu("Edit", vec![ContextMenuItem::action("Undo", |_, _, _| {})])
                .menu("View", vec![ContextMenuItem::action("Zoom", |_, _, _| {})])
        });

        bar.update(cx, |bar, _| {
            bar.ensure_front_menu("File", vec![ContextMenuItem::action("Exit", |_, _, _| {})]);

            assert_eq!(bar.menus.len(), 3);
            assert_eq!(bar.trigger_bounds.len(), 3);
            assert_eq!(bar.menus[0].label.as_ref(), "File");
            assert_eq!(bar.menus[1].label.as_ref(), "Edit");
            assert_eq!(bar.menus[2].label.as_ref(), "View");
            assert_eq!(bar.active_index, 0);
        });
    }

    #[gpui::test]
    fn ensure_front_menu_is_idempotent(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let bar = cx.new(|cx| {
            MenuBar::new(cx).menu("File", vec![ContextMenuItem::action("Open", |_, _, _| {})])
        });

        bar.update(cx, |bar, _| {
            bar.ensure_front_menu("File", vec![ContextMenuItem::action("Exit", |_, _, _| {})]);

            assert_eq!(bar.menus.len(), 1);
            assert_eq!(bar.trigger_bounds.len(), 1);
            assert_eq!(bar.menus[0].items.len(), 1);
        });
    }
}
