use std::sync::{Arc, Mutex};

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    anchored, canvas, deferred, div, point, prelude::*, px, svg, App, Bounds, ClickEvent, Context,
    Corner, ElementId, FocusHandle, IntoElement, KeyDownEvent, MouseButton, Pixels, Point, Render,
    SharedString, WeakEntity, Window,
};

use crate::context_menu::ContextMenuItem;
use crate::menu_tree::{self, MenuRenderEnv};

/// A Fluent menu button: compact command trigger plus anchored dropdown menu.
///
/// `MenuButton` uses `ContextMenuItem` so application menus, context menus, and
/// menu-button dropdowns can share the same command definitions.
pub struct MenuButton {
    id: ElementId,
    label: SharedString,
    icon: Option<SharedString>,
    disabled: bool,
    items: Vec<ContextMenuItem>,
    open: bool,
    closing: bool,
    motion_epoch: u64,
    popup_origin: Point<Pixels>,
    trigger_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    focus_handle: Option<FocusHandle>,
    observing_theme: bool,
    open_path: Vec<usize>,
    selected_path: Vec<usize>,
}

impl MenuButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: None,
            disabled: false,
            items: Vec::new(),
            open: false,
            closing: false,
            motion_epoch: 0,
            popup_origin: point(px(0.0), px(0.0)),
            trigger_bounds: Arc::new(Mutex::new(None)),
            focus_handle: None,
            observing_theme: false,
            open_path: Vec::new(),
            selected_path: Vec::new(),
        }
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn item(mut self, item: ContextMenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = ContextMenuItem>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn separator(self) -> Self {
        self.item(ContextMenuItem::Separator)
    }

    pub fn action(
        self,
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(ContextMenuItem::action(label, on_click))
    }

    fn focus_handle(&mut self, cx: &mut Context<Self>) -> FocusHandle {
        self.focus_handle
            .get_or_insert_with(|| cx.focus_handle().tab_stop(true))
            .clone()
    }

    fn update_origin(&mut self, fallback: Point<Pixels>) {
        self.popup_origin = self
            .trigger_bounds
            .lock()
            .ok()
            .and_then(|slot| *slot)
            .map(|bounds| point(bounds.origin.x, bounds.origin.y + bounds.size.height))
            .unwrap_or(fallback);
    }

    fn open_popup(&mut self, cx: &mut Context<Self>) {
        self.open_popup_selecting(false, cx);
    }

    fn open_popup_selecting(&mut self, select_last: bool, cx: &mut Context<Self>) {
        if self.items.is_empty() {
            return;
        }

        self.open = true;
        self.closing = false;
        self.motion_epoch = self.motion_epoch.wrapping_add(1);
        if let Some(bounds) = self.trigger_bounds.lock().ok().and_then(|slot| *slot) {
            self.popup_origin = point(bounds.origin.x, bounds.origin.y + bounds.size.height);
        }
        self.open_path.clear();
        self.selected_path = if select_last {
            menu_tree::last_enabled_path(&self.items, &[]).unwrap_or_default()
        } else {
            menu_tree::first_enabled_path(&self.items, &[]).unwrap_or_default()
        };
        cx.notify();
    }

    fn close_popup(&mut self, cx: &mut Context<Self>) {
        if !self.open || self.closing {
            return;
        }

        self.closing = true;
        self.motion_epoch = self.motion_epoch.wrapping_add(1);
        let epoch = self.motion_epoch;
        let duration = cx.theme().motion().popup_exit_duration();

        cx.spawn(
            move |button: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    cx.background_executor().timer(duration).await;
                    button
                        .update(&mut cx, move |button, cx| {
                            if button.motion_epoch == epoch && button.closing {
                                button.open = false;
                                button.closing = false;
                                button.open_path.clear();
                                button.selected_path.clear();
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

    fn move_selection(&mut self, delta: isize, cx: &mut Context<Self>) {
        if menu_tree::move_selection(&self.items, &mut self.selected_path, delta) {
            self.open_path = menu_tree::parent_path(&self.selected_path);
            cx.notify();
        }
    }

    fn enter_submenu(&mut self, cx: &mut Context<Self>) {
        if menu_tree::enter_submenu(&self.items, &mut self.selected_path, &mut self.open_path) {
            cx.notify();
        }
    }

    fn leave_submenu(&mut self, cx: &mut Context<Self>) {
        menu_tree::leave_submenu(&mut self.selected_path, &mut self.open_path);
        cx.notify();
    }

    fn invoke_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(item) = menu_tree::enabled_item_at_path(&self.items, &self.selected_path).cloned()
        else {
            return;
        };

        match item {
            ContextMenuItem::Action {
                disabled, on_click, ..
            } if !disabled => {
                let ev = ClickEvent::default();
                on_click(&ev, window, cx);
                self.close_popup(cx);
            }
            ContextMenuItem::Checkbox {
                checked,
                disabled,
                on_click,
                ..
            } if !disabled => {
                let ev = ClickEvent::default();
                on_click(!checked, &ev, window, cx);
                self.close_popup(cx);
            }
            ContextMenuItem::Radio {
                disabled, on_click, ..
            } if !disabled => {
                let ev = ClickEvent::default();
                on_click(&ev, window, cx);
                self.close_popup(cx);
            }
            ContextMenuItem::Submenu { disabled, .. } if !disabled => self.enter_submenu(cx),
            _ => {}
        }
    }

    fn handle_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.closing {
            return;
        }

        match ev.keystroke.key.as_str() {
            "escape" => self.close_popup(cx),
            "down" => {
                if self.open {
                    self.move_selection(1, cx);
                } else {
                    self.open_popup(cx);
                }
            }
            "up" => {
                if self.open {
                    self.move_selection(-1, cx);
                } else {
                    self.open_popup_selecting(true, cx);
                }
            }
            "right" => self.enter_submenu(cx),
            "left" => self.leave_submenu(cx),
            "return" | "space" => {
                if self.open {
                    self.invoke_selected(window, cx);
                } else {
                    self.open_popup(cx);
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
                    let hover_path = menu_tree::parent_path(&path);
                    let select_path = path.clone();
                    let fg = if *disabled {
                        env.colors.on_neutral_disabled
                    } else {
                        env.colors.on_neutral
                    };
                    let mut row = menu_tree::row_base("menu-button-item", &path, selected, fg, env)
                        .on_mouse_move(cx.listener(move |button: &mut MenuButton, _, _, cx| {
                            if button.closing {
                                return;
                            }
                            button.selected_path = select_path.clone();
                            button.open_path = hover_path.clone();
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
                                if closer
                                    .update(cx, |button, _| button.closing)
                                    .unwrap_or(true)
                                {
                                    return;
                                }
                                cx.stop_propagation();
                                let ev = ClickEvent::default();
                                cb(&ev, win, cx);
                                closer.update(cx, |button, cx| button.close_popup(cx)).ok();
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
                    let hover_path = menu_tree::parent_path(&path);
                    let select_path = path.clone();
                    let next = !checked;
                    let fg = if *disabled {
                        env.colors.on_neutral_disabled
                    } else {
                        env.colors.on_neutral
                    };
                    let mut row = menu_tree::row_base("menu-button-item", &path, selected, fg, env)
                        .on_mouse_move(cx.listener(move |button: &mut MenuButton, _, _, cx| {
                            if button.closing {
                                return;
                            }
                            button.selected_path = select_path.clone();
                            button.open_path = hover_path.clone();
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
                                if closer
                                    .update(cx, |button, _| button.closing)
                                    .unwrap_or(true)
                                {
                                    return;
                                }
                                cx.stop_propagation();
                                let ev = ClickEvent::default();
                                cb(next, &ev, win, cx);
                                closer.update(cx, |button, cx| button.close_popup(cx)).ok();
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
                    let hover_path = menu_tree::parent_path(&path);
                    let select_path = path.clone();
                    let fg = if *disabled {
                        env.colors.on_neutral_disabled
                    } else {
                        env.colors.on_neutral
                    };
                    let mut row = menu_tree::row_base("menu-button-item", &path, selected, fg, env)
                        .on_mouse_move(cx.listener(move |button: &mut MenuButton, _, _, cx| {
                            if button.closing {
                                return;
                            }
                            button.selected_path = select_path.clone();
                            button.open_path = hover_path.clone();
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
                                if closer
                                    .update(cx, |button, _| button.closing)
                                    .unwrap_or(true)
                                {
                                    return;
                                }
                                cx.stop_propagation();
                                let ev = ClickEvent::default();
                                cb(&ev, win, cx);
                                closer.update(cx, |button, cx| button.close_popup(cx)).ok();
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
                    let mut row = menu_tree::row_base("menu-button-item", &path, selected, fg, env);

                    if !disabled && !self.closing {
                        row = row
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_mouse_move(cx.listener(
                                move |button: &mut MenuButton, _, _, cx| {
                                    if button.closing {
                                        return;
                                    }
                                    button.selected_path = select_path.clone();
                                    button.open_path = select_path.clone();
                                    cx.notify();
                                },
                            ));
                    }

                    row = row
                        .child(menu_tree::icon_slot(icon.clone(), fg, env))
                        .child(div().flex_1().child(label.clone()))
                        .child(
                            svg()
                                .path("icons/chevron_right.svg")
                                .size(px(12.0))
                                .text_color(fg),
                        );
                    menu = menu.child(row);

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

impl Render for MenuButton {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing_theme {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            self.observing_theme = true;
        }

        let focus_handle = self.focus_handle(cx);
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let components = theme.components;
        let motion = theme.motion();
        let focused = focus_handle.is_focused(window);
        let fg = if self.disabled {
            colors.on_neutral_disabled
        } else {
            colors.on_neutral
        };

        let trigger_bounds = Arc::clone(&self.trigger_bounds);
        let measure_trigger = canvas(
            move |bounds, _, _| {
                if let Ok(mut slot) = trigger_bounds.lock() {
                    *slot = Some(bounds);
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0();

        let env = MenuRenderEnv::from_theme(theme);
        let self_weak = cx.entity().downgrade();

        let mut trigger = div()
            .id(self.id.clone())
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.sm))
            .px(px(spacing.md))
            .h(px(components.dropdown_height))
            .rounded(px(radii.md))
            .bg(if self.disabled {
                colors.neutral_disabled
            } else if self.open && !self.closing {
                colors.neutral_selected
            } else {
                colors.neutral
            })
            .border_1()
            .border_color(if focused {
                colors.accent
            } else {
                colors.stroke_neutral
            })
            .text_size(px(typography.body.size))
            .text_color(fg)
            .child(measure_trigger);

        if let Some(icon) = self.icon.clone() {
            trigger = trigger.child(
                svg()
                    .path(icon)
                    .size(px(components.popup_icon_slot))
                    .text_color(fg),
            );
        }

        trigger = trigger.child(div().child(self.label.clone())).child(
            svg()
                .path("icons/chevron_down.svg")
                .size(px(components.popup_icon_slot))
                .text_color(fg),
        );

        if !self.disabled {
            let focus_handle = focus_handle.clone();
            trigger = trigger
                .cursor_pointer()
                .hover({
                    let hover = colors.neutral_hover;
                    move |s| s.bg(hover)
                })
                .on_click(cx.listener(
                    move |button: &mut MenuButton, ev: &ClickEvent, window, cx| {
                        window.focus(&focus_handle);
                        button.update_origin(point(
                            ev.position().x,
                            ev.position().y + px(cx.theme().components.dropdown_height),
                        ));
                        if button.open && !button.closing {
                            button.close_popup(cx);
                        } else if !button.open {
                            button.open_popup(cx);
                        }
                    },
                ));
        }

        let mut root = div()
            .key_context("MenuButton")
            .track_focus(&focus_handle)
            .on_key_down(
                cx.listener(|button: &mut MenuButton, ev: &KeyDownEvent, window, cx| {
                    button.handle_key(ev, window, cx);
                }),
            )
            .relative()
            .flex()
            .flex_col()
            .child(trigger);

        if self.open && !self.disabled {
            let menu = self.render_items(&self.items, Vec::new(), self_weak.clone(), &env, cx);
            let menu = fluent_core::popup_motion_surface(
                ("menu-button-popup-motion", self.motion_epoch),
                self.closing,
                motion,
                menu,
            );

            let mut overlay = div().absolute().inset_0();
            if !self.closing {
                let closer = self_weak.clone();
                overlay = overlay.child(
                    div()
                        .id("menu-button-backdrop")
                        .absolute()
                        .inset_0()
                        .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                            cx.stop_propagation();
                            closer.update(cx, |button, cx| button.close_popup(cx)).ok();
                        }),
                );
            }

            root = root.child(deferred(
                overlay.child(
                    anchored()
                        .anchor(Corner::TopLeft)
                        .position(self.popup_origin)
                        .snap_to_window()
                        .child(menu),
                ),
            ));
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use fluent_test_utils::init_fluent_test;
    use gpui::{AppContext as _, ClickEvent, TestAppContext};

    use super::*;

    fn item(label: &'static str) -> ContextMenuItem {
        ContextMenuItem::action(label, |_: &ClickEvent, _, _| {})
    }

    #[gpui::test]
    fn close_popup_keeps_menu_button_mounted_until_exit_duration(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let button = cx.new(|_| MenuButton::new("test-menu-button", "Actions").item(item("One")));
        button.update(cx, |button, cx| {
            button.open_popup(cx);
            assert_eq!(button.selected_path, vec![0]);
            button.close_popup(cx);
            assert!(button.open);
            assert!(button.closing);
            assert_eq!(button.selected_path, vec![0]);
        });

        cx.executor().advance_clock(Duration::from_millis(159));
        button.update(cx, |button, _| {
            assert!(button.open);
            assert!(button.closing);
            assert_eq!(button.selected_path, vec![0]);
        });

        cx.executor().advance_clock(Duration::from_millis(1));
        button.update(cx, |button, _| {
            assert!(!button.open);
            assert!(!button.closing);
            assert!(button.selected_path.is_empty());
        });
    }

    #[gpui::test]
    fn reopening_menu_button_cancels_stale_exit_timer(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let button = cx.new(|_| MenuButton::new("test-menu-button", "Actions").item(item("One")));
        button.update(cx, |button, cx| {
            button.open_popup(cx);
            button.close_popup(cx);
            button.open_popup(cx);
            assert!(button.open);
            assert!(!button.closing);
        });

        cx.executor().advance_clock(Duration::from_millis(160));
        button.update(cx, |button, _| {
            assert!(button.open);
            assert!(!button.closing);
        });
    }

    #[gpui::test]
    fn opening_menu_button_from_up_selects_last_enabled_item(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let button = cx.new(|_| {
            MenuButton::new("test-menu-button", "Actions")
                .item(item("One"))
                .separator()
                .item(item("Two"))
                .item(item("Disabled").disabled(true))
        });
        button.update(cx, |button, cx| {
            button.open_popup_selecting(true, cx);
            assert_eq!(button.selected_path, vec![2]);
        });
    }

    #[gpui::test]
    fn keyboard_move_closes_stale_menu_button_submenu(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let button = cx.new(|_| {
            MenuButton::new("test-menu-button", "Actions")
                .item(ContextMenuItem::submenu(
                    "More",
                    vec![ContextMenuItem::action("Child", |_, _, _| {})],
                ))
                .item(item("Next"))
        });
        button.update(cx, |button, cx| {
            button.selected_path = vec![0];
            button.open_path = vec![0];
            button.move_selection(1, cx);

            assert_eq!(button.selected_path, vec![1]);
            assert!(button.open_path.is_empty());
        });
    }
}
