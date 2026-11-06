use fluent_core::TextRampExt;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::sync::Arc;

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    anchored, deferred, div, prelude::*, px, svg, App, ClickEvent, Context, FocusHandle,
    IntoElement, KeyDownEvent, MouseButton, Pixels, Point, Render, Role, ScrollHandle,
    SharedString, WeakEntity, Window,
};

use crate::menu_tree::{self, MenuRenderEnv};

pub use crate::menu_tree::is_context_menu_key;

#[allow(clippy::type_complexity)]
#[derive(Clone)]
/// Public API item.
pub enum ContextMenuItem {
    /// Public API item.
    Action {
        /// Public API item.
        label: SharedString,
        /// Public API item.
        shortcut: Option<SharedString>,
        /// Public API item.
        icon: Option<SharedString>,
        /// Public API item.
        disabled: bool,
        /// Public API item.
        on_click: Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
    },
    /// Public API item.
    Checkbox {
        /// Public API item.
        label: SharedString,
        /// Public API item.
        checked: bool,
        /// Public API item.
        disabled: bool,
        /// Public API item.
        on_click: Arc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>,
    },
    /// Public API item.
    Radio {
        /// Public API item.
        label: SharedString,
        /// Public API item.
        selected: bool,
        /// Public API item.
        disabled: bool,
        /// Public API item.
        on_click: Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>,
    },
    /// Public API item.
    Submenu {
        /// Public API item.
        label: SharedString,
        /// Public API item.
        icon: Option<SharedString>,
        /// Public API item.
        disabled: bool,
        /// Public API item.
        items: Vec<ContextMenuItem>,
    },
    /// Public API item.
    Separator,
}

impl ContextMenuItem {
    /// Public API item.
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

    /// Public API item.
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

    /// Public API item.
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

    /// Public API item.
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

    /// Public API item.
    pub fn submenu(label: impl Into<SharedString>, items: Vec<ContextMenuItem>) -> Self {
        Self::Submenu {
            label: label.into(),
            icon: None,
            disabled: false,
            items,
        }
    }

    /// Public API item.
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

    /// Public API item.
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

    /// Public API item.
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

/// Wraps `child` so Shift+F10 or the Menu key, pressed while focus is inside
/// it, calls `on_open` with the point just below the wrapper's bottom-left
/// corner, where a keyboard-opened [`ContextMenu`] should appear.
///
/// Wrap the focusable element itself so the point sits at the focused element.
pub fn context_menu_key_target(
    id: impl Into<gpui::ElementId>,
    child: impl IntoElement,
    on_open: impl Fn(Point<gpui::Pixels>, &mut Window, &mut App) + 'static,
) -> impl IntoElement {
    let bounds: Arc<std::sync::Mutex<Option<gpui::Bounds<gpui::Pixels>>>> = Arc::default();
    let tracker = bounds.clone();
    div()
        .id(id)
        .relative()
        .on_key_down(move |ev: &KeyDownEvent, window, cx| {
            if !is_context_menu_key(&ev.keystroke) {
                return;
            }
            let Some(b) = bounds.lock().ok().and_then(|g| *g) else {
                return;
            };
            cx.stop_propagation();
            on_open(
                gpui::point(b.origin.x, b.origin.y + b.size.height),
                window,
                cx,
            );
        })
        .child(child)
        .child(
            gpui::canvas(
                move |b, _, _| {
                    if let Ok(mut slot) = tracker.lock() {
                        *slot = Some(b);
                    }
                },
                |_, _, _, _| {},
            )
            .absolute()
            .inset_0(),
        )
}

/// Public API item.
pub struct ContextMenu {
    /// Public API item.
    pub items: Vec<ContextMenuItem>,
    /// Public API item.
    pub position: Point<gpui::Pixels>,
    /// Public API item.
    pub open: bool,
    closing: bool,
    motion_epoch: u64,
    observing_theme: bool,
    focus_handle: Option<FocusHandle>,
    open_path: Vec<usize>,
    selected_path: Vec<usize>,
    hover_intent: menu_tree::HoverIntent,
    /// One scroll handle per open menu level, keyed by the level's path prefix.
    scroll_handles: RefCell<HashMap<Vec<usize>, ScrollHandle>>,
    /// Where each submenu-trigger row was last painted, keyed by its path.
    row_anchors: RefCell<HashMap<Vec<usize>, menu_tree::RowAnchor>>,
    /// Set by keyboard navigation; the next render scrolls the selected row
    /// into view, then clears it so the wheel is not fought on every frame.
    reveal_selected: Cell<bool>,
}

impl ContextMenu {
    /// Public API item.
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
            hover_intent: menu_tree::HoverIntent::default(),
            scroll_handles: RefCell::default(),
            row_anchors: RefCell::default(),
            reveal_selected: Cell::new(false),
        }
    }

    /// Public API item.
    pub fn item(mut self, item: ContextMenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Public API item.
    pub fn separator(mut self) -> Self {
        self.items.push(ContextMenuItem::Separator);
        self
    }

    /// Public API item.
    pub fn action(
        self,
        label: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(ContextMenuItem::action(label, on_click))
    }

    /// Public API item.
    pub fn action_with_icon(
        self,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(ContextMenuItem::action_with_icon(label, icon, on_click))
    }

    /// Public API item.
    pub fn checkbox(
        self,
        label: impl Into<SharedString>,
        checked: bool,
        on_click: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(ContextMenuItem::checkbox(label, checked, on_click))
    }

    /// Public API item.
    pub fn radio(
        self,
        label: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.item(ContextMenuItem::radio(label, selected, on_click))
    }

    /// Public API item.
    pub fn submenu(self, label: impl Into<SharedString>, items: Vec<ContextMenuItem>) -> Self {
        self.item(ContextMenuItem::submenu(label, items))
    }

    /// Public API item.
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
            self.reveal_selected.set(true);
            cx.notify();
        }
    }

    fn enter_submenu(&mut self, cx: &mut Context<Self>) {
        if menu_tree::enter_submenu(&self.items, &mut self.selected_path, &mut self.open_path) {
            self.reveal_selected.set(true);
            cx.notify();
        }
    }

    fn leave_submenu(&mut self, cx: &mut Context<Self>) {
        menu_tree::leave_submenu(&mut self.selected_path, &mut self.open_path);
        self.reveal_selected.set(true);
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
            _ => {
                if let Some(ch) = menu_tree::typed_access_char(&ev.keystroke) {
                    self.press_access_key(ch, window, cx);
                }
            }
        }
    }

    /// Selects and activates the item in the open level whose access key is `ch`.
    fn press_access_key(&mut self, ch: char, window: &mut Window, cx: &mut Context<Self>) {
        let parent = menu_tree::parent_path(&self.selected_path);
        let current = self.selected_path.last().copied();
        let Some(path) = menu_tree::find_access_key(&self.items, &parent, ch, current) else {
            return;
        };
        self.selected_path = path;
        self.open_path = parent;
        self.reveal_selected.set(true);
        self.invoke_selected(window, cx);
        cx.notify();
    }

    fn render_menu(
        &self,
        items: &[ContextMenuItem],
        path_prefix: Vec<usize>,
        self_weak: WeakEntity<Self>,
        env: &MenuRenderEnv,
        max_height: Pixels,
        cx: &mut Context<Self>,
    ) -> gpui::AnyElement {
        let scroll = self
            .scroll_handles
            .borrow_mut()
            .entry(path_prefix.clone())
            .or_default()
            .clone();
        if self.reveal_selected.get() && menu_tree::parent_path(&self.selected_path) == path_prefix
        {
            if let Some(&idx) = self.selected_path.last() {
                // Every item, separators included, is one child of the panel.
                scroll.scroll_to_item(idx);
            }
        }
        let mut menu = div()
            .id(SharedString::from(format!(
                "{}-panel-{:?}",
                "ctx", path_prefix
            )))
            .debug_selector({
                let path_prefix = path_prefix.clone();
                move || format!("ctx-panel-{path_prefix:?}")
            })
            .role(Role::Menu)
            .relative()
            .flex()
            .flex_col()
            .bg(env.colors.surface)
            .border_1()
            .border_color(env.colors.stroke_neutral)
            .rounded(px(env.radii.md))
            .py(px(env.spacing.xs))
            .min_w(px(env.components.dropdown_min_width))
            .max_h(max_height)
            .overflow_y_scroll()
            .track_scroll(&scroll)
            .shadow_md();

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
                    let mut row = menu_tree::row_base(
                        "ctx-item",
                        &path,
                        selected,
                        fg,
                        env,
                        Role::MenuItem,
                        label,
                    );

                    if !disabled && !self.closing {
                        row = row
                            .on_mouse_move(cx.listener(move |menu: &mut ContextMenu, _, _, cx| {
                                menu_tree::hover_row(
                                    menu,
                                    select_path.clone(),
                                    hover_path.clone(),
                                    cx,
                                );
                            }))
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .active({
                                let pressed = env.colors.subtle_selected;
                                move |s| s.bg(pressed)
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
                    row = row.child(div().flex_1().child(menu_tree::label_element(label, true)));
                    if let Some(shortcut) = shortcut.clone() {
                        row = row.child(
                            div()
                                .flex_none()
                                .pl(px(env.spacing.xl))
                                .text_ramp(&env.typography.caption)
                                .text_color(env.colors.on_subtle_disabled)
                                .child(shortcut),
                        );
                    }
                    menu = menu.child(row);
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
                    let mut row = menu_tree::row_base(
                        "ctx-item",
                        &path,
                        selected,
                        fg,
                        env,
                        Role::MenuItemCheckBox,
                        label,
                    );
                    if !disabled && !self.closing {
                        row = row
                            .on_mouse_move(cx.listener(move |menu: &mut ContextMenu, _, _, cx| {
                                menu_tree::hover_row(
                                    menu,
                                    select_path.clone(),
                                    hover_path.clone(),
                                    cx,
                                );
                            }))
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .active({
                                let pressed = env.colors.subtle_selected;
                                move |s| s.bg(pressed)
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
                        .child(div().flex_1().child(menu_tree::label_element(label, true))),
                    );
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
                    let mut row = menu_tree::row_base(
                        "ctx-item",
                        &path,
                        selected,
                        fg,
                        env,
                        Role::MenuItemRadio,
                        label,
                    );
                    if !disabled && !self.closing {
                        row = row
                            .on_mouse_move(cx.listener(move |menu: &mut ContextMenu, _, _, cx| {
                                menu_tree::hover_row(
                                    menu,
                                    select_path.clone(),
                                    hover_path.clone(),
                                    cx,
                                );
                            }))
                            .cursor_pointer()
                            .hover({
                                let hover = env.colors.subtle_hover;
                                move |s| s.bg(hover)
                            })
                            .active({
                                let pressed = env.colors.subtle_selected;
                                move |s| s.bg(pressed)
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
                        .child(div().flex_1().child(menu_tree::label_element(label, true))),
                    );
                }
                ContextMenuItem::Submenu {
                    label,
                    icon,
                    disabled,
                    ..
                } => {
                    let select_path = path.clone();
                    let fg = if *disabled {
                        env.colors.on_neutral_disabled
                    } else {
                        env.colors.on_neutral
                    };
                    let mut row = menu_tree::row_base(
                        "ctx-item",
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
                            .active({
                                let pressed = env.colors.subtle_selected;
                                move |s| s.bg(pressed)
                            })
                            .on_mouse_move(cx.listener(move |menu: &mut ContextMenu, _, _, cx| {
                                menu_tree::hover_row(
                                    menu,
                                    select_path.clone(),
                                    select_path.clone(),
                                    cx,
                                );
                            }));
                    }

                    row = row.child(menu_tree::icon_slot(icon.clone(), fg, env));
                    row = row
                        .child(div().flex_1().child(menu_tree::label_element(label, true)))
                        .child(
                            svg()
                                .path("icons/chevron_right.svg")
                                .size(px(12.0))
                                .text_color(fg),
                        );
                    if !disabled {
                        let anchor = self
                            .row_anchors
                            .borrow_mut()
                            .entry(path.clone())
                            .or_default()
                            .clone();
                        row = row.child(menu_tree::track_row(&anchor));
                    }
                    menu = menu.child(row);
                }
                ContextMenuItem::Separator => {
                    menu = menu.child(
                        div()
                            .h(px(env.components.popup_separator_height))
                            .mx(px(env.spacing.sm))
                            .my(px(env.spacing.xs))
                            .bg(env.colors.stroke_neutral_subtle),
                    );
                }
            }
        }

        menu.into_any_element()
    }

    /// The open submenus, one layer per level beside the root menu, each
    /// anchored to its trigger row. They are siblings of the root panel, not
    /// its children, so the panel can scroll without clipping them.
    fn render_submenu_layers(
        &self,
        self_weak: &WeakEntity<Self>,
        env: &MenuRenderEnv,
        max_height: Pixels,
        cx: &mut Context<Self>,
    ) -> Vec<gpui::AnyElement> {
        // Fade only: the anchor row already carries the root panel's slide.
        let mut motion = cx.theme().motion();
        motion.popup_offset = 0.0;
        let mut layers = Vec::new();
        for depth in 1..=self.open_path.len() {
            let prefix = &self.open_path[..depth];
            let Some(ContextMenuItem::Submenu {
                items,
                disabled: false,
                ..
            }) = menu_tree::item_at_path(&self.items, prefix)
            else {
                break;
            };
            let panel = self.render_menu(
                items,
                prefix.to_vec(),
                self_weak.clone(),
                env,
                max_height,
                cx,
            );
            let panel = fluent_core::popup_motion_surface(
                ("context-submenu-motion", self.motion_epoch),
                self.closing,
                motion,
                panel,
            );
            let anchor = self
                .row_anchors
                .borrow_mut()
                .entry(prefix.to_vec())
                .or_default()
                .clone();
            layers.push(menu_tree::submenu_layer(
                panel,
                env,
                fluent_core::layer::CONTEXT_MENU + depth,
                anchor,
            ));
        }
        layers
    }
}

impl menu_tree::HoverMenu for ContextMenu {
    fn is_closing(&self) -> bool {
        self.closing
    }

    fn hover_intent(&mut self) -> &mut menu_tree::HoverIntent {
        &mut self.hover_intent
    }

    fn paths_mut(&mut self) -> (&mut Vec<usize>, &mut Vec<usize>) {
        (&mut self.selected_path, &mut self.open_path)
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

        let max_height = menu_tree::max_panel_height(window);
        let menu = self.render_menu(
            &self.items,
            Vec::new(),
            self_weak.clone(),
            &env,
            max_height,
            cx,
        );
        let layers = self.render_submenu_layers(&self_weak, &env, max_height, cx);
        self.reveal_selected.set(false);
        let open_levels = menu_tree::open_levels(&self.open_path);
        self.scroll_handles
            .borrow_mut()
            .retain(|prefix, _| open_levels.iter().any(|open| open == prefix));
        let menu = fluent_core::popup_motion_surface(
            ("context-menu-motion", self.motion_epoch),
            self.closing,
            motion,
            menu,
        );

        let mut overlay = div();
        if !self.closing {
            let backdrop_closer = cx.entity().downgrade();
            overlay = overlay.child(menu_tree::window_backdrop(
                "ctx-backdrop",
                window,
                move |_, cx| {
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
            .child(
                deferred(
                    overlay
                        .child(anchored().position(pos).child(menu))
                        .children(layers),
                )
                .with_priority(fluent_core::layer::CONTEXT_MENU),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use fluent_test_utils::{init_fluent_test, open_fluent_test_window, FluentInteractionExt as _};
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

    #[gpui::test]
    fn accessibility_snapshot_reports_menu_and_items(cx: &mut TestAppContext) {
        use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
        let (_, vcx) = open_fluent_test_window(cx, |_, _| {
            ContextMenu::build(point(px(0.0), px(0.0)))
                .action("Open", |_, _, _| {})
                .submenu("More", vec![ContextMenuItem::action("Deep", |_, _, _| {})])
        });
        assert_eq!(
            vcx.accessibility_snapshot(),
            vec![
                "Window".to_string(),
                "Menu".to_string(),
                "MenuItem \"Open\"".to_string(),
                "MenuItem \"More\"".to_string(),
            ]
        );
    }

    #[gpui::test]
    fn access_key_invokes_item_in_open_menu(cx: &mut TestAppContext) {
        let hits = std::rc::Rc::new(std::cell::Cell::new(0));
        let h = hits.clone();
        let (_menu, cx) = open_fluent_test_window(cx, move |_, _| {
            ContextMenu::build(point(px(0.0), px(0.0)))
                .action("&Copy", |_, _, _| {})
                .action("&Paste", move |_, _, _| h.set(h.get() + 1))
        });
        cx.run_until_parked();

        cx.type_keys("p");
        assert_eq!(hits.get(), 1);
    }

    struct KeyTargetView {
        opened: Option<Point<gpui::Pixels>>,
        focus: FocusHandle,
    }

    impl Render for KeyTargetView {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let this = cx.entity().downgrade();
            div().p(px(20.0)).child(context_menu_key_target(
                "target",
                div()
                    .id("field")
                    .debug_selector(|| "field".into())
                    .track_focus(&self.focus)
                    .w(px(100.0))
                    .h(px(30.0)),
                move |pos, _, cx| {
                    this.update(cx, |view, _| view.opened = Some(pos)).ok();
                },
            ))
        }
    }

    #[gpui::test]
    fn shift_f10_and_menu_key_open_at_focused_element(cx: &mut TestAppContext) {
        let (view, cx) = open_fluent_test_window(cx, |_, cx| KeyTargetView {
            opened: None,
            focus: cx.focus_handle(),
        });
        let focus = view.read_with(cx, |view, _| view.focus.clone());
        cx.update(|window, cx| window.focus(&focus, cx));
        cx.run_until_parked();
        let bounds = cx.selector_bounds("field").expect("field painted");

        cx.type_keys("f10");
        view.read_with(cx, |view, _| assert!(view.opened.is_none()));

        cx.type_keys("shift-f10");
        let expected = point(bounds.origin.x, bounds.origin.y + bounds.size.height);
        view.read_with(cx, |view, _| assert_eq!(view.opened, Some(expected)));

        view.update(cx, |view, _| view.opened = None);
        cx.type_keys("menu");
        view.read_with(cx, |view, _| assert_eq!(view.opened, Some(expected)));
    }

    fn bounds_of(
        cx: &mut gpui::VisualTestContext,
        selector: &'static str,
    ) -> gpui::Bounds<gpui::Pixels> {
        cx.selector_bounds(selector)
            .unwrap_or_else(|| panic!("no bounds for {selector}"))
    }

    fn submenu_menu(at: gpui::Point<gpui::Pixels>) -> ContextMenu {
        let mut menu = ContextMenu::build(at)
            .submenu("More", vec![ContextMenuItem::action("Inner", |_, _, _| {})]);
        menu.open_path = vec![0];
        menu.selected_path = vec![0];
        menu
    }

    #[gpui::test]
    fn submenu_first_row_lines_up_with_parent_row(cx: &mut TestAppContext) {
        let (_menu, cx) = open_fluent_test_window(cx, |_, _| submenu_menu(point(px(40.), px(30.))));
        cx.simulate_resize(gpui::size(px(800.), px(600.)));
        cx.run_until_parked();
        cx.accessibility_snapshot();

        let parent = bounds_of(cx, "ctx-item-row-[0]");
        let child = bounds_of(cx, "ctx-item-row-[0, 0]");
        assert_eq!(
            child.top(),
            parent.top(),
            "first submenu row level with parent row"
        );
        assert_eq!(
            child.left(),
            parent.right() - px(1.) + px(1.),
            "submenu opens one pixel over the parent border, its own border inside it"
        );
    }

    #[gpui::test]
    fn submenu_flips_left_at_the_right_window_edge(cx: &mut TestAppContext) {
        let (_menu, cx) =
            open_fluent_test_window(cx, |_, _| submenu_menu(point(px(600.), px(30.))));
        cx.simulate_resize(gpui::size(px(800.), px(600.)));
        cx.run_until_parked();
        cx.accessibility_snapshot();

        let parent = bounds_of(cx, "ctx-item-row-[0]");
        let child = bounds_of(cx, "ctx-item-row-[0, 0]");
        assert!(
            child.right() <= parent.left() + px(2.),
            "{child:?} vs {parent:?}"
        );
        assert!(child.left() >= px(8.));
        assert_eq!(child.top(), parent.top());
    }

    #[gpui::test]
    fn submenu_snaps_inside_the_bottom_window_edge(cx: &mut TestAppContext) {
        let (_menu, cx) = open_fluent_test_window(cx, |_, _| submenu_menu(point(px(40.), px(30.))));
        cx.simulate_resize(gpui::size(px(800.), px(80.)));
        cx.run_until_parked();
        cx.accessibility_snapshot();

        let child = bounds_of(cx, "ctx-item-row-[0, 0]");
        assert!(child.bottom() <= px(80. - 8. + 1.), "{child:?}");
    }

    #[gpui::test]
    fn clicking_anywhere_outside_closes_the_menu(cx: &mut TestAppContext) {
        let (menu, cx) = open_fluent_test_window(cx, |_, _| {
            ContextMenu::build(point(px(20.), px(20.))).action("Open", |_, _, _| {})
        });
        cx.simulate_resize(gpui::size(px(800.), px(600.)));
        cx.run_until_parked();
        cx.accessibility_snapshot();

        cx.click_at(
            point(px(700.), px(500.)),
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );
        menu.read_with(cx, |menu, _| assert!(menu.closing));
        cx.executor().advance_clock(Duration::from_millis(200));
        menu.read_with(cx, |menu, _| assert!(!menu.open));
    }

    fn long_menu(at: gpui::Point<gpui::Pixels>, rows: usize) -> ContextMenu {
        let mut menu = ContextMenu::build(at);
        for i in 0..rows {
            menu = menu.action(format!("Item {i}"), |_, _, _| {});
        }
        menu
    }

    fn long_menu_with_submenu(at: gpui::Point<gpui::Pixels>, children: usize) -> ContextMenu {
        let children = (0..children)
            .map(|i| ContextMenuItem::action(format!("Child {i}"), |_, _, _| {}))
            .collect();
        long_menu(at, 3)
            .submenu("More", children)
            .action("Last", |_, _, _| {})
    }

    #[gpui::test]
    fn long_menu_is_capped_to_the_window_and_scrolls_selection_into_view(cx: &mut TestAppContext) {
        let (menu, cx) = open_fluent_test_window(cx, |_, _| long_menu(point(px(40.), px(20.)), 60));
        cx.simulate_resize(gpui::size(px(800.), px(300.)));
        cx.run_until_parked();
        cx.accessibility_snapshot();

        let panel = bounds_of(cx, "ctx-panel-[]");
        // The enter animation still slides the panel by `popup_offset`.
        assert!(panel.bottom() <= px(300. + 2.), "{panel:?}");
        assert!(panel.top() >= px(0.), "{panel:?}");

        for _ in 0..40 {
            cx.type_keys("down");
        }
        cx.run_until_parked();
        cx.accessibility_snapshot();
        menu.read_with(cx, |menu, _| assert_eq!(menu.selected_path, vec![40]));

        let panel = bounds_of(cx, "ctx-panel-[]");
        let row = bounds_of(cx, "ctx-item-row-[40]");
        assert!(
            row.top() >= panel.top() && row.bottom() <= panel.bottom(),
            "selected row {row:?} not inside panel {panel:?}"
        );
        menu.read_with(cx, |menu, _| {
            let handles = menu.scroll_handles.borrow();
            assert!(handles[&Vec::<usize>::new()].offset().y < px(0.));
        });

        // Wrapping past the end scrolls back to the top.
        for _ in 0..20 {
            cx.type_keys("down");
        }
        cx.run_until_parked();
        cx.accessibility_snapshot();
        menu.read_with(cx, |menu, _| assert_eq!(menu.selected_path, vec![0]));
        let panel = bounds_of(cx, "ctx-panel-[]");
        let row = bounds_of(cx, "ctx-item-row-[0]");
        assert!(row.top() >= panel.top() && row.bottom() <= panel.bottom());
    }

    #[gpui::test]
    fn scrolling_root_menu_does_not_clip_its_submenu(cx: &mut TestAppContext) {
        let (_menu, cx) = open_fluent_test_window(cx, |_, _| {
            let mut menu = long_menu_with_submenu(point(px(40.), px(20.)), 3);
            menu.open_path = vec![3];
            menu.selected_path = vec![3];
            menu
        });
        cx.simulate_resize(gpui::size(px(800.), px(600.)));
        cx.run_until_parked();
        cx.accessibility_snapshot();

        let root = bounds_of(cx, "ctx-panel-[]");
        let sub = bounds_of(cx, "ctx-panel-[3]");
        let parent = bounds_of(cx, "ctx-item-row-[3]");
        assert!(sub.left() >= root.right() - px(2.), "{sub:?} vs {root:?}");
        assert_eq!(bounds_of(cx, "ctx-item-row-[3, 0]").top(), parent.top());
        assert!(sub.size.height > px(0.));
    }

    #[gpui::test]
    fn tall_submenu_is_capped_inside_the_window_and_keyboard_scrolls_it(cx: &mut TestAppContext) {
        let (menu, cx) = open_fluent_test_window(cx, |_, _| {
            let mut menu = long_menu_with_submenu(point(px(40.), px(20.)), 40);
            menu.open_path = vec![3];
            menu.selected_path = vec![3];
            menu
        });
        cx.simulate_resize(gpui::size(px(800.), px(300.)));
        cx.run_until_parked();
        cx.accessibility_snapshot();

        let sub = bounds_of(cx, "ctx-panel-[3]");
        assert!(sub.top() >= px(0.) && sub.bottom() <= px(300.), "{sub:?}");

        cx.type_keys("right");
        menu.read_with(cx, |menu, _| assert_eq!(menu.selected_path, vec![3, 0]));
        for _ in 0..30 {
            cx.type_keys("down");
        }
        cx.run_until_parked();
        cx.accessibility_snapshot();
        menu.read_with(cx, |menu, _| assert_eq!(menu.selected_path, vec![3, 30]));
        let sub = bounds_of(cx, "ctx-panel-[3]");
        let row = bounds_of(cx, "ctx-item-row-[3, 30]");
        assert!(
            row.top() >= sub.top() && row.bottom() <= sub.bottom(),
            "selected row {row:?} not inside submenu {sub:?}"
        );
    }

    #[gpui::test]
    fn keyboard_enters_leaves_and_dismisses_submenu_layers(cx: &mut TestAppContext) {
        let (menu, cx) = open_fluent_test_window(cx, |_, _| {
            long_menu_with_submenu(point(px(40.), px(20.)), 3)
        });
        cx.simulate_resize(gpui::size(px(800.), px(600.)));
        cx.run_until_parked();

        for _ in 0..3 {
            cx.type_keys("down");
        }
        cx.type_keys("right");
        menu.read_with(cx, |menu, _| {
            assert_eq!(menu.selected_path, vec![3, 0]);
            assert_eq!(menu.open_path, vec![3]);
        });
        cx.run_until_parked();
        cx.accessibility_snapshot();
        assert!(cx.selector_bounds("ctx-panel-[3]").is_some());

        cx.type_keys("left");
        menu.read_with(cx, |menu, _| {
            assert_eq!(menu.selected_path, vec![3]);
            assert!(menu.open_path.is_empty());
        });
        cx.run_until_parked();
        cx.accessibility_snapshot();
        assert!(cx.selector_bounds("ctx-panel-[3]").is_none());

        cx.type_keys("escape");
        menu.read_with(cx, |menu, _| assert!(menu.closing));
    }

    #[gpui::test]
    fn clicking_outside_closes_a_menu_with_an_open_submenu_layer(cx: &mut TestAppContext) {
        let (menu, cx) = open_fluent_test_window(cx, |_, _| {
            let mut menu = long_menu_with_submenu(point(px(20.), px(20.)), 3);
            menu.open_path = vec![3];
            menu.selected_path = vec![3];
            menu
        });
        cx.simulate_resize(gpui::size(px(800.), px(600.)));
        cx.run_until_parked();
        cx.accessibility_snapshot();

        cx.click_at(
            point(px(780.), px(580.)),
            gpui::MouseButton::Left,
            gpui::Modifiers::none(),
        );
        menu.read_with(cx, |menu, _| assert!(menu.closing));
    }
}
