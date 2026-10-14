use std::sync::Arc;

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    anchored, deferred, div, prelude::*, px, svg, App, ClickEvent, Context, FocusHandle,
    IntoElement, KeyDownEvent, MouseButton, Point, Render, SharedString, WeakEntity, Window,
};

use crate::menu_tree::{self, MenuRenderEnv};

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub enum ContextMenuItem {
    Action {
        label: SharedString,
        shortcut: Option<SharedString>,
        icon: Option<SharedString>,
        disabled: bool,
        on_click: Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
    },
    Checkbox {
        label: SharedString,
        checked: bool,
        disabled: bool,
        on_click: Arc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>,
    },
    Radio {
        label: SharedString,
        selected: bool,
        disabled: bool,
        on_click: Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
    },
    Submenu {
        label: SharedString,
        icon: Option<SharedString>,
        disabled: bool,
        items: Vec<ContextMenuItem>,
    },
    Separator,
}

impl ContextMenuItem {
    pub fn action(
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Action {
            label: label.into(),
            shortcut: None,
            icon: None,
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    pub fn action_with_icon(
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Action {
            label: label.into(),
            shortcut: None,
            icon: Some(icon.into()),
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    pub fn checkbox(
        label: impl Into<SharedString>,
        checked: bool,
        on_click: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Checkbox {
            label: label.into(),
            checked,
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    pub fn radio(
        label: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self::Radio {
            label: label.into(),
            selected,
            disabled: false,
            on_click: Arc::new(on_click),
        }
    }

    pub fn submenu(label: impl Into<SharedString>, items: Vec<ContextMenuItem>) -> Self {
        Self::Submenu {
            label: label.into(),
            icon: None,
            disabled: false,
            items,
        }
    }

    pub fn shortcut(self, shortcut: impl Into<SharedString>) -> Self {
        match self {
            Self::Action {
                label,
                icon,
                disabled,
                on_click,
                ..
            } => Self::Action {
                label,
                shortcut: Some(shortcut.into()),
                icon,
                disabled,
                on_click,
            },
            other => other,
        }
    }

    pub fn disabled(self, disabled: bool) -> Self {
        match self {
            Self::Action {
                label,
                shortcut,
                icon,
                on_click,
                ..
            } => Self::Action {
                label,
                shortcut,
                icon,
                disabled,
                on_click,
            },
            Self::Checkbox {
                label,
                checked,
                on_click,
                ..
            } => Self::Checkbox {
                label,
                checked,
                disabled,
                on_click,
            },
            Self::Radio {
                label,
                selected,
                on_click,
                ..
            } => Self::Radio {
                label,
                selected,
                disabled,
                on_click,
            },
            Self::Submenu {
                label, icon, items, ..
            } => Self::Submenu {
                label,
                icon,
                disabled,
                items,
            },
            Self::Separator => Self::Separator,
        }
    }

    pub fn separator() -> Self {
        Self::Separator
    }

    pub(crate) fn enabled(&self) -> bool {
        match self {
            Self::Action { disabled, .. }
            | Self::Checkbox { disabled, .. }
            | Self::Radio { disabled, .. }
            | Self::Submenu { disabled, .. } => !disabled,
            Self::Separator => false,
        }
    }
}

pub struct ContextMenu {
    pub items: Vec<ContextMenuItem>,
    pub position: Point<gpui::Pixels>,
    pub open: bool,
    closing: bool,
    motion_epoch: u64,
    observing_theme: bool,
    focus_handle: Option<FocusHandle>,
    open_path: Vec<usize>,
    selected_path: Vec<usize>,
}

impl ContextMenu {
    pub fn build(position: Point<gpui::Pixels>) -> Self {
        Self {
            items: vec![],
            position,
            open: true,
            closing: false,
            motion_epoch: 0,
            observing_theme: false,
            focus_handle: None,
            open_path: Vec::new(),
            selected_path: Vec::new(),
        }
    }

    pub fn item(mut self, item: ContextMenuItem) -> Self {
        self.items.push(item);
        self
    }

    pub fn separator(mut self) -> Self {
        self.items.push(ContextMenuItem::Separator);
        self
    }

    pub fn action(
        self,
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(ContextMenuItem::action(label, on_click))
    }

    pub fn action_with_icon(
        self,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(ContextMenuItem::action_with_icon(label, icon, on_click))
    }

    pub fn checkbox(
        self,
        label: impl Into<SharedString>,
        checked: bool,
        on_click: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(ContextMenuItem::checkbox(label, checked, on_click))
    }

    pub fn radio(
        self,
        label: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(ContextMenuItem::radio(label, selected, on_click))
    }

    pub fn submenu(self, label: impl Into<SharedString>, items: Vec<ContextMenuItem>) -> Self {
        self.item(ContextMenuItem::submenu(label, items))
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        if !self.open || self.closing {
            return;
        }

        self.closing = true;
        self.motion_epoch = self.motion_epoch.wrapping_add(1);
        let epoch = self.motion_epoch;
        let duration = cx.theme().motion().popup_exit_duration();

        cx.spawn(
            move |menu: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    cx.background_executor().timer(duration).await;
                    menu.update(&mut cx, move |menu, cx| {
                        if menu.motion_epoch == epoch && menu.closing {
                            menu.open = false;
                            menu.closing = false;
                            menu.open_path.clear();
                            menu.selected_path.clear();
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

    fn focus_handle(&mut self, cx: &mut Context<Self>) -> FocusHandle {
        self.focus_handle
            .get_or_insert_with(|| cx.focus_handle().tab_stop(true))
            .clone()
    }

    fn ensure_selection(&mut self) {
        menu_tree::ensure_selection(&self.items, &mut self.selected_path);
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
            "up" => self.move_selection(-1, cx),
            "down" => self.move_selection(1, cx),
            "right" => self.enter_submenu(cx),
            "left" => self.leave_submenu(cx),
            "return" | "space" => self.invoke_selected(window, cx),
            _ => {}
        }
    }

    fn render_menu(
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
            .min_w(px(env.components.dropdown_min_width))
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
                    let mut row = menu_tree::row_base("ctx-item", &path, selected, fg, env);

                    if !disabled && !self.closing {
                        row = row
                            .on_mouse_move(cx.listener(move |menu: &mut ContextMenu, _, _, cx| {
                                if menu.closing {
                                    return;
                                }
                                menu.selected_path = select_path.clone();
                                menu.open_path = hover_path.clone();
                                cx.notify();
                            }))
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_mouse_down(MouseButton::Left, move |_, win, cx| {
                                if closer.update(cx, |menu, _| menu.closing).unwrap_or(true) {
                                    return;
                                }
                                cx.stop_propagation();
                                let ev = ClickEvent::default();
                                cb(&ev, win, cx);
                                closer.update(cx, |menu, cx| menu.close(cx)).ok();
                            });
                    }

                    row = row.child(menu_tree::icon_slot(icon.clone(), fg, env));
                    row = row.child(div().flex_1().child(label.clone()));
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
                    let mut row = menu_tree::row_base("ctx-item", &path, selected, fg, env);
                    if !disabled && !self.closing {
                        row = row
                            .on_mouse_move(cx.listener(move |menu: &mut ContextMenu, _, _, cx| {
                                if menu.closing {
                                    return;
                                }
                                menu.selected_path = select_path.clone();
                                menu.open_path = hover_path.clone();
                                cx.notify();
                            }))
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_mouse_down(MouseButton::Left, move |_, win, cx| {
                                if closer.update(cx, |menu, _| menu.closing).unwrap_or(true) {
                                    return;
                                }
                                cx.stop_propagation();
                                let ev = ClickEvent::default();
                                cb(next, &ev, win, cx);
                                closer.update(cx, |menu, cx| menu.close(cx)).ok();
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
                    let mut row = menu_tree::row_base("ctx-item", &path, selected, fg, env);
                    if !disabled && !self.closing {
                        row = row
                            .on_mouse_move(cx.listener(move |menu: &mut ContextMenu, _, _, cx| {
                                if menu.closing {
                                    return;
                                }
                                menu.selected_path = select_path.clone();
                                menu.open_path = hover_path.clone();
                                cx.notify();
                            }))
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_mouse_down(MouseButton::Left, move |_, win, cx| {
                                if closer.update(cx, |menu, _| menu.closing).unwrap_or(true) {
                                    return;
                                }
                                cx.stop_propagation();
                                let ev = ClickEvent::default();
                                cb(&ev, win, cx);
                                closer.update(cx, |menu, cx| menu.close(cx)).ok();
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
                    let mut row = menu_tree::row_base("ctx-item", &path, selected, fg, env);
                    if !disabled && !self.closing {
                        row = row
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .on_mouse_move(cx.listener(move |menu: &mut ContextMenu, _, _, cx| {
                                if menu.closing {
                                    return;
                                }
                                menu.selected_path = select_path.clone();
                                menu.open_path = select_path.clone();
                                cx.notify();
                            }));
                    }

                    row = row.child(menu_tree::icon_slot(icon.clone(), fg, env));
                    row = row.child(div().flex_1().child(label.clone())).child(
                        svg()
                            .path("icons/chevron_right.svg")
                            .size(px(12.0))
                            .text_color(fg),
                    );
                    menu = menu.child(row);

                    if !disabled && menu_tree::path_starts_with(&self.open_path, &path) {
                        let submenu =
                            self.render_menu(items, path.clone(), self_weak.clone(), env, cx);
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

impl Render for ContextMenu {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing_theme {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            self.observing_theme = true;
        }

        if !self.open {
            return div();
        }

        let focus_handle = self.focus_handle(cx);
        if !self.closing {
            self.ensure_selection();
        }
        if !self.closing && !focus_handle.is_focused(window) {
            window.focus(&focus_handle, cx);
        }

        let theme = cx.theme();
        let motion = theme.motion();
        let env = MenuRenderEnv::from_theme(theme);
        let pos = self.position;
        let self_weak = cx.entity().downgrade();

        let menu = self.render_menu(&self.items, Vec::new(), self_weak, &env, cx);
        let menu = fluent_core::popup_motion_surface(
            ("context-menu-motion", self.motion_epoch),
            self.closing,
            motion,
            menu,
        );

        let mut overlay = div().absolute().inset_0();
        if !self.closing {
            let backdrop_closer = cx.entity().downgrade();
            overlay = overlay.child(div().id("ctx-backdrop").absolute().inset_0().on_mouse_down(
                gpui::MouseButton::Left,
                move |_, _, cx| {
                    cx.stop_propagation();
                    backdrop_closer.update(cx, |menu, cx| menu.close(cx)).ok();
                },
            ));
        }

        div()
            .key_context("ContextMenu")
            .track_focus(&focus_handle)
            .on_key_down(
                cx.listener(|menu: &mut ContextMenu, ev: &KeyDownEvent, window, cx| {
                    menu.handle_key(ev, window, cx);
                }),
            )
            .child(deferred(
                overlay.child(anchored().position(pos).child(menu)),
            ))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use fluent_test_utils::init_fluent_test;
    use gpui::{point, px, TestAppContext};

    use super::*;

    #[gpui::test]
    fn close_keeps_context_menu_mounted_until_exit_duration(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let menu =
            cx.new(|_| ContextMenu::build(point(px(0.0), px(0.0))).action("Open", |_, _, _| {}));
        menu.update(cx, |menu, cx| {
            menu.selected_path = vec![0];
            menu.open_path = vec![0];
            menu.close(cx);
            assert!(menu.open);
            assert!(menu.closing);
            assert_eq!(menu.selected_path, vec![0]);
            assert_eq!(menu.open_path, vec![0]);
        });

        cx.executor().advance_clock(Duration::from_millis(159));
        menu.update(cx, |menu, _| {
            assert!(menu.open);
            assert!(menu.closing);
            assert_eq!(menu.selected_path, vec![0]);
            assert_eq!(menu.open_path, vec![0]);
        });

        cx.executor().advance_clock(Duration::from_millis(1));
        menu.update(cx, |menu, _| {
            assert!(!menu.open);
            assert!(!menu.closing);
            assert!(menu.selected_path.is_empty());
            assert!(menu.open_path.is_empty());
        });
    }

    #[gpui::test]
    fn keyboard_move_closes_stale_context_submenu(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let menu = cx.new(|_| {
            ContextMenu::build(point(px(0.0), px(0.0)))
                .submenu("More", vec![ContextMenuItem::action("Child", |_, _, _| {})])
                .action("Next", |_, _, _| {})
        });
        menu.update(cx, |menu, cx| {
            menu.selected_path = vec![0];
            menu.open_path = vec![0];
            menu.move_selection(1, cx);

            assert_eq!(menu.selected_path, vec![1]);
            assert!(menu.open_path.is_empty());
        });
    }
}
