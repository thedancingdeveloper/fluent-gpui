use std::sync::{Arc, Mutex};

use fluent_core::{LayoutDirection, Theme, ThemeProvider as _};
use gpui::{
    anchored, canvas, deferred, div, point, prelude::*, px, svg, App, Bounds, ClickEvent, Context,
    FocusHandle, IntoElement, KeyDownEvent, MouseButton, Pixels, Render, Role, SharedString,
    Subscription, WeakEntity, WeakFocusHandle, Window, WindowId,
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
    window_id: Option<WindowId>,
    prior_focus: Option<WeakFocusHandle>,
    _key_subscriptions: Vec<Subscription>,
}

impl MenuBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();

        // Window-wide keys: F10 and Alt+<access key> (intercepted so they work
        // wherever focus is), and a lone Alt tap (observed on release).
        let weak = cx.entity().downgrade();
        let intercept = cx.intercept_keystrokes({
            move |ev, window, cx| {
                let consumed = weak
                    .update(cx, |bar, cx| bar.global_key(&ev.keystroke, window, cx))
                    .unwrap_or(false);
                if consumed {
                    cx.stop_propagation();
                }
            }
        });
        let observe = cx.observe_keystrokes(|bar, ev, window, cx| {
            if ev.keystroke.key == "alt"
                && bar.in_window(window)
                && !bar.focus_handle.is_focused(window)
            {
                bar.activate(window, cx);
            }
        });

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
            window_id: None,
            prior_focus: None,
            _key_subscriptions: vec![intercept, observe],
        }
    }

    fn in_window(&self, window: &Window) -> bool {
        self.window_id == Some(window.window_handle().window_id())
    }

    /// Moves focus to the menu bar, highlighting the first menu, as F10 or a
    /// tap of Alt does. Escape returns focus to where it was.
    pub fn activate(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.menus.is_empty() {
            return;
        }
        if !self.focus_handle.is_focused(window) {
            self.prior_focus = window.focused(cx).map(|h| h.downgrade());
            window.focus(&self.focus_handle, cx);
        }
        if self.open_index.is_none() {
            self.active_index = 0;
        }
        cx.notify();
    }

    /// F10 and Alt+<access key>, from anywhere in the window. Returns whether
    /// the keystroke was consumed.
    fn global_key(
        &mut self,
        keystroke: &gpui::Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self.in_window(window) || self.menus.is_empty() || self.closing {
            return false;
        }
        let m = &keystroke.modifiers;
        if keystroke.key == "f10" && !m.shift && !m.alt && !m.control && !m.platform {
            if self.focus_handle.is_focused(window) {
                self.dismiss(window, cx);
            } else {
                self.activate(window, cx);
            }
            return true;
        }
        if m.alt && !self.focus_handle.is_focused(window) {
            if let Some(idx) = self.top_level_for_key(keystroke) {
                self.activate(window, cx);
                self.open_menu(idx, false);
                cx.notify();
                return true;
            }
        }
        false
    }

    fn top_level_for_key(&self, keystroke: &gpui::Keystroke) -> Option<usize> {
        let ch = menu_tree::typed_access_char(keystroke)?
            .to_lowercase()
            .next()?;
        let start = self.active_index + 1;
        (0..self.menus.len())
            .map(|offset| (start + offset) % self.menus.len())
            .find(|&idx| menu_tree::access_key(&self.menus[idx].label) == Some(ch))
    }

    /// Closes any open menu and hands focus back to where it came from.
    fn dismiss(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close(cx);
        match self.prior_focus.take().and_then(|h| h.upgrade()) {
            Some(prior) => window.focus(&prior, cx),
            None => window.blur(cx),
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
            "escape" => {
                if self.open_index.is_some() {
                    self.close(cx);
                } else {
                    self.dismiss(window, cx);
                }
            }
            "f10" if !ev.keystroke.modifiers.shift => self.dismiss(window, cx),
            "left" if !self.is_rtl(cx) => {
                if self.selected_path.len() > 1 {
                    self.leave_submenu(cx);
                } else {
                    self.move_top_menu(-1);
                    cx.notify();
                }
            }
            "right" if !self.is_rtl(cx) => {
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
            _ => {
                if matches!(ev.keystroke.key.as_str(), "left" | "right") {
                    if ev.keystroke.key == "left" {
                        self.enter_or_move_right(cx);
                    } else {
                        self.leave_or_move_left(cx);
                    }
                    return;
                }
                if let Some(ch) = menu_tree::typed_access_char(&ev.keystroke) {
                    self.press_access_key(ch, &ev.keystroke, window, cx);
                }
            }
        }
    }

    fn is_rtl(&self, cx: &Context<Self>) -> bool {
        cx.global::<LayoutDirection>().is_rtl()
    }

    fn enter_or_move_right(&mut self, cx: &mut Context<Self>) {
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

    fn leave_or_move_left(&mut self, cx: &mut Context<Self>) {
        if self.selected_path.len() > 1 {
            self.leave_submenu(cx);
        } else {
            self.move_top_menu(-1);
            cx.notify();
        }
    }

    /// An access key typed while the bar has focus: with a menu open (and no
    /// Alt) it picks an item at the open level, otherwise it opens a menu.
    fn press_access_key(
        &mut self,
        ch: char,
        keystroke: &gpui::Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !keystroke.modifiers.alt && !self.closing {
            if let Some(open_idx) = self.open_index {
                let parent = menu_tree::parent_path(&self.selected_path);
                let current = self.selected_path.last().copied();
                let found = self
                    .menus
                    .get(open_idx)
                    .and_then(|menu| menu_tree::find_access_key(&menu.items, &parent, ch, current));
                if let Some(path) = found {
                    self.selected_path = path;
                    self.open_path = parent;
                    self.invoke_selected(window, cx);
                    cx.notify();
                }
                return;
            }
        }
        if let Some(idx) = self.top_level_for_key(keystroke) {
            self.open_menu(idx, false);
            cx.notify();
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
            .id(SharedString::from(format!(
                "{}-panel-{:?}",
                "menubar", path_prefix
            )))
            .role(Role::Menu)
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
                    let mut row = menu_tree::row_base(
                        "menu-item",
                        &path,
                        selected,
                        fg,
                        env,
                        Role::MenuItem,
                        label,
                    )
                    .on_mouse_move(cx.listener(
                        move |bar: &mut MenuBar, _, _, cx| {
                            if bar.closing {
                                return;
                            }
                            bar.selected_path = select_path.clone();
                            bar.open_path = hover_parent.clone();
                            cx.notify();
                        },
                    ));
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
                        .child(div().flex_1().child(menu_tree::label_element(label, true)));
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
                    let mut row = menu_tree::row_base(
                        "menu-item",
                        &path,
                        selected,
                        fg,
                        env,
                        Role::MenuItemCheckBox,
                        label,
                    )
                    .on_mouse_move(cx.listener(
                        move |bar: &mut MenuBar, _, _, cx| {
                            if bar.closing {
                                return;
                            }
                            bar.selected_path = select_path.clone();
                            bar.open_path = hover_parent.clone();
                            cx.notify();
                        },
                    ));
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
                        .child(div().flex_1().child(menu_tree::label_element(label, true))),
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
                    let mut row = menu_tree::row_base(
                        "menu-item",
                        &path,
                        selected,
                        fg,
                        env,
                        Role::MenuItemRadio,
                        label,
                    )
                    .on_mouse_move(cx.listener(
                        move |bar: &mut MenuBar, _, _, cx| {
                            if bar.closing {
                                return;
                            }
                            bar.selected_path = select_path.clone();
                            bar.open_path = hover_parent.clone();
                            cx.notify();
                        },
                    ));
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
                        .child(div().flex_1().child(menu_tree::label_element(label, true))),
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
                    let mut row = menu_tree::row_base(
                        "menu-item",
                        &path,
                        selected,
                        fg,
                        env,
                        Role::MenuItem,
                        label,
                    );
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
                            .child(div().flex_1().child(menu_tree::label_element(label, true)))
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
        let rtl = cx.global::<LayoutDirection>().is_rtl();

        let self_weak = cx.entity().downgrade();
        let open_idx = self.open_index;
        let active_idx = self.active_index.min(self.menus.len().saturating_sub(1));
        self.window_id = Some(window.window_handle().window_id());
        let show_access_keys = open_idx.is_some() || self.focus_handle.is_focused(window);

        let bar_bg = colors.surface_dim;
        let trigger_hover_bg = colors.subtle_hover;
        let trigger_active_bg = colors.neutral;
        let trigger_text = colors.on_neutral;

        // Build the trigger buttons row.
        let mut triggers_row = div()
            .flex()
            .when(rtl, |row| row.flex_row_reverse())
            .when(!rtl, |row| row.flex_row())
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
                .role(Role::MenuItem)
                .aria_label(label.clone())
                .aria_expanded(is_open)
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
                    win.focus(&focus_handle, cx);
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
                .child(menu_tree::label_element(&label, show_access_keys))
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
            .id("menu-bar")
            .role(Role::MenuBar)
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

    use fluent_test_utils::{init_fluent_test, open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Entity, TestAppContext};

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

    #[gpui::test]
    fn accessibility_snapshot_reports_menubar_menu_and_items(cx: &mut TestAppContext) {
        use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
        let (bar, vcx) = open_fluent_test_window(cx, |_, cx| {
            MenuBar::new(cx).menu(
                "File",
                vec![
                    ContextMenuItem::action("Open", |_, _, _| {}),
                    ContextMenuItem::checkbox("Wrap", true, |_, _, _, _| {}),
                ],
            )
        });
        assert_eq!(
            vcx.accessibility_snapshot(),
            vec![
                "Window".to_string(),
                "MenuBar".to_string(),
                "MenuItem \"File\" [expanded=false]".to_string(),
            ]
        );
        bar.update(vcx, |bar, cx| {
            bar.open_menu(0, false);
            cx.notify();
        });
        assert_eq!(
            vcx.accessibility_snapshot(),
            vec![
                "Window".to_string(),
                "MenuBar".to_string(),
                "MenuItem \"File\" [expanded=true]".to_string(),
                "Menu".to_string(),
                "MenuItem \"Open\"".to_string(),
                "MenuItemCheckBox \"Wrap\"".to_string(),
            ]
        );
    }

    fn keyboard_bar<'a>(
        cx: &'a mut TestAppContext,
        hits: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>>,
    ) -> (Entity<MenuBar>, &'a mut gpui::VisualTestContext) {
        let h = hits.clone();
        let h2 = hits.clone();
        open_fluent_test_window(cx, move |_, cx| {
            MenuBar::new(cx)
                .menu(
                    "&File",
                    vec![
                        ContextMenuItem::submenu(
                            "&Recent",
                            vec![ContextMenuItem::action("Doc", move |_, _, _| {
                                h.borrow_mut().push("doc")
                            })],
                        ),
                        ContextMenuItem::action("E&xit", move |_, _, _| {
                            h2.borrow_mut().push("exit")
                        }),
                    ],
                )
                .menu(
                    "&Edit",
                    vec![ContextMenuItem::action("&Undo", move |_, _, _| {
                        hits.borrow_mut().push("undo")
                    })],
                )
        })
    }

    #[gpui::test]
    fn f10_focuses_bar_and_arrows_traverse_menus(cx: &mut TestAppContext) {
        let hits = Default::default();
        let (bar, cx) = keyboard_bar(cx, hits);
        let handle = bar.read_with(cx, |bar, _| bar.focus_handle.clone());
        cx.assert_nothing_focused();

        cx.type_keys("f10");
        cx.assert_focused(&handle);
        bar.read_with(cx, |bar, _| assert_eq!(bar.open_index, None));

        cx.type_keys("down");
        bar.read_with(cx, |bar, _| {
            assert_eq!(bar.open_index, Some(0));
            assert_eq!(bar.selected_path, vec![0]);
        });

        cx.type_keys("right");
        bar.read_with(cx, |bar, _| {
            assert_eq!(bar.open_path, vec![0]);
            assert_eq!(bar.selected_path, vec![0, 0]);
        });

        cx.type_keys("left");
        bar.read_with(cx, |bar, _| assert_eq!(bar.selected_path, vec![0]));

        cx.type_keys("down right");
        bar.read_with(cx, |bar, _| {
            assert_eq!(
                bar.open_index,
                Some(1),
                "right on a leaf moves to the next menu"
            );
        });
    }

    #[gpui::test]
    fn access_keys_open_menus_and_invoke_items(cx: &mut TestAppContext) {
        let hits: std::rc::Rc<std::cell::RefCell<Vec<&'static str>>> = Default::default();
        let (bar, cx) = keyboard_bar(cx, hits.clone());

        cx.type_keys("f10 e");
        bar.read_with(cx, |bar, _| assert_eq!(bar.open_index, Some(1)));
        cx.type_keys("u");
        assert_eq!(*hits.borrow(), vec!["undo"]);

        cx.executor().advance_clock(Duration::from_millis(200));
        cx.type_keys("f f");
        bar.read_with(cx, |bar, _| assert_eq!(bar.open_index, Some(0)));
        cx.type_keys("r");
        bar.read_with(cx, |bar, _| assert_eq!(bar.selected_path, vec![0, 0]));
        cx.type_keys("left x");
        assert_eq!(*hits.borrow(), vec!["undo", "exit"]);
    }

    #[gpui::test]
    fn alt_access_key_opens_menu_from_anywhere(cx: &mut TestAppContext) {
        let (bar, cx) = keyboard_bar(cx, Default::default());
        let handle = bar.read_with(cx, |bar, _| bar.focus_handle.clone());

        cx.type_keys("alt-e");
        cx.assert_focused(&handle);
        bar.read_with(cx, |bar, _| {
            assert_eq!(bar.open_index, Some(1));
            assert_eq!(bar.selected_path, vec![0]);
        });
    }

    #[gpui::test]
    fn tapping_alt_focuses_bar(cx: &mut TestAppContext) {
        let (bar, cx) = keyboard_bar(cx, Default::default());
        let handle = bar.read_with(cx, |bar, _| bar.focus_handle.clone());

        cx.type_keys("alt");
        cx.assert_focused(&handle);
    }

    #[gpui::test]
    fn escape_closes_menu_then_f10_dismisses_bar(cx: &mut TestAppContext) {
        let (bar, cx) = keyboard_bar(cx, Default::default());
        let handle = bar.read_with(cx, |bar, _| bar.focus_handle.clone());

        cx.type_keys("f10 down escape");
        cx.executor().advance_clock(Duration::from_millis(200));
        bar.read_with(cx, |bar, _| assert_eq!(bar.open_index, None));
        cx.assert_focused(&handle);

        cx.type_keys("f10");
        cx.assert_not_focused(&handle);
    }
}
