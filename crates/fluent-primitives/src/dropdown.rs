use std::sync::{Arc, Mutex};

use crate::focus::FluentFocus as _;
use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    anchored, canvas, deferred, div, point, prelude::*, px, svg, Anchor, App, Bounds, ClickEvent,
    Context, ElementId, IntoElement, KeyDownEvent, Pixels, Point, Render, SharedString, Window,
};

#[derive(Clone, Debug)]
pub struct DropdownOption {
    pub value: SharedString,
    pub label: SharedString,
}

impl DropdownOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// A selectable dropdown with an anchored popover and controlled selection callback.
#[allow(clippy::type_complexity)]
pub struct Dropdown {
    id: ElementId,
    selected: Option<SharedString>,
    placeholder: SharedString,
    disabled: bool,
    tab_index: Option<isize>,
    options: Vec<DropdownOption>,
    open: bool,
    closing: bool,
    highlighted: usize,
    motion_epoch: u64,
    drop_origin: Point<Pixels>,
    trigger_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    observing_theme: bool,
    on_select: Option<Box<dyn Fn(SharedString, &mut App) + 'static>>,
}

impl Dropdown {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            selected: None,
            placeholder: "Select...".into(),
            disabled: false,
            tab_index: None,
            options: vec![],
            open: false,
            closing: false,
            highlighted: 0,
            motion_epoch: 0,
            drop_origin: point(px(0.0), px(0.0)),
            trigger_bounds: Arc::new(Mutex::new(None)),
            observing_theme: false,
            on_select: None,
        }
    }

    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn placeholder(mut self, text: impl Into<SharedString>) -> Self {
        self.placeholder = text.into();
        self
    }

    /// Explicit position in the Tab order (lower first). Defaults to document order.
    pub fn tab_index(mut self, index: isize) -> Self {
        self.tab_index = Some(index);
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn option(
        mut self,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.options.push(DropdownOption::new(value, label));
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = DropdownOption>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    pub fn on_select(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn set_selected(&mut self, value: impl Into<SharedString>, cx: &mut Context<Self>) {
        self.selected = Some(value.into());
        cx.notify();
    }

    pub fn selected_value(&self) -> Option<&SharedString> {
        self.selected.as_ref()
    }

    pub fn set_options(
        &mut self,
        options: impl IntoIterator<Item = DropdownOption>,
        cx: &mut Context<Self>,
    ) {
        self.options = options.into_iter().collect();
        cx.notify();
    }

    fn selected_label(&self) -> SharedString {
        let Some(selected) = &self.selected else {
            return self.placeholder.clone();
        };
        self.options
            .iter()
            .find(|option| option.value == *selected)
            .map(|option| option.label.clone())
            .unwrap_or_else(|| selected.clone())
    }

    fn open_popup(&mut self, cx: &mut Context<Self>) {
        self.highlighted = self
            .selected
            .as_ref()
            .and_then(|selected| {
                self.options
                    .iter()
                    .position(|option| option.value == *selected)
            })
            .unwrap_or(0);
        self.open = true;
        self.closing = false;
        self.motion_epoch = self.motion_epoch.wrapping_add(1);
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
            move |dropdown: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    cx.background_executor().timer(duration).await;
                    dropdown
                        .update(&mut cx, move |dropdown, cx| {
                            if dropdown.motion_epoch == epoch && dropdown.closing {
                                dropdown.open = false;
                                dropdown.closing = false;
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

    fn handle_key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        if self.disabled || self.closing {
            return;
        }

        match ev.keystroke.key.as_str() {
            "escape" => self.close_popup(cx),
            "down" if ev.keystroke.modifiers.alt => self.open_popup(cx),
            "down" => {
                if self.options.is_empty() {
                    return;
                }
                if self.open {
                    self.highlighted = (self.highlighted + 1).min(self.options.len() - 1);
                } else {
                    self.open_popup(cx);
                }
                cx.notify();
            }
            "up" => {
                if self.options.is_empty() {
                    return;
                }
                if self.open {
                    self.highlighted = self.highlighted.saturating_sub(1);
                } else {
                    self.open_popup(cx);
                }
                cx.notify();
            }
            "home" => {
                if self.open && !self.options.is_empty() {
                    self.highlighted = 0;
                    cx.notify();
                }
            }
            "end" => {
                if self.open && !self.options.is_empty() {
                    self.highlighted = self.options.len() - 1;
                    cx.notify();
                }
            }
            "pageup" => {
                if self.open && !self.options.is_empty() {
                    self.highlighted = self.highlighted.saturating_sub(5);
                    cx.notify();
                }
            }
            "pagedown" => {
                if self.open && !self.options.is_empty() {
                    self.highlighted = (self.highlighted + 5).min(self.options.len() - 1);
                    cx.notify();
                }
            }
            "enter" | "space" if self.open => {
                if let Some(value) = self
                    .options
                    .get(self.highlighted)
                    .map(|option| option.value.clone())
                {
                    self.selected = Some(value.clone());
                    if let Some(f) = &self.on_select {
                        f(value, cx);
                    }
                    self.close_popup(cx);
                }
            }
            "enter" | "space" => self.open_popup(cx),
            _ => {}
        }
    }
}

impl Render for Dropdown {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing_theme {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            self.observing_theme = true;
        }

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let tab_index = self.tab_index;
        let focus_ring = colors.focus_ring;
        let radii = theme.radii;
        let spacing = theme.spacing;
        let typography = theme.typography;
        let components = theme.components;

        let label = self.selected_label();
        let fg = if self.disabled {
            colors.on_neutral_disabled
        } else if self.selected.is_none() {
            colors.on_subtle_disabled
        } else {
            colors.on_neutral
        };

        let trigger_bounds_arc = Arc::clone(&self.trigger_bounds);
        let measure_trigger = canvas(
            move |bounds, _, _| {
                if let Ok(mut slot) = trigger_bounds_arc.lock() {
                    *slot = Some(bounds);
                }
            },
            |_, _, _, _| {},
        )
        .absolute()
        .inset_0();

        let mut el = div()
            .id(self.id.clone())
            .debug_selector(|| "dropdown-trigger".into())
            .focusable()
            .tab_stop(true)
            .fluent_focus(tab_index, focus_ring)
            .on_key_down(
                cx.listener(|dropdown: &mut Dropdown, ev: &KeyDownEvent, _, cx| {
                    dropdown.handle_key(ev, cx);
                }),
            )
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .justify_between()
            .px(px(spacing.md))
            .h(px(components.dropdown_height))
            .rounded(px(radii.md))
            .bg(if self.disabled {
                colors.neutral_disabled
            } else {
                colors.neutral
            })
            .border_1()
            .border_color(colors.stroke_neutral)
            .text_size(px(typography.body.size))
            .text_color(fg)
            .child(div().flex_1().child(label))
            .child(
                div()
                    .flex()
                    .items_center()
                    .justify_center()
                    .flex_none()
                    .size(px(components.popup_icon_slot))
                    .child(
                        svg()
                            .path("icons/chevron_down.svg")
                            .size(px(components.popup_icon_slot))
                            .text_color(fg),
                    ),
            )
            .child(measure_trigger);

        if !self.disabled {
            let trigger_bounds = Arc::clone(&self.trigger_bounds);
            el = el.cursor_pointer().on_click(cx.listener(
                move |dropdown: &mut Dropdown, ev: &ClickEvent, _window, cx| {
                    let fallback = point(
                        ev.position().x,
                        ev.position().y + px(components.dropdown_height),
                    );
                    dropdown.drop_origin = trigger_bounds
                        .lock()
                        .ok()
                        .and_then(|slot| *slot)
                        .map(|bounds| point(bounds.origin.x, bounds.origin.y + bounds.size.height))
                        .unwrap_or(fallback);
                    if dropdown.open && !dropdown.closing {
                        dropdown.close_popup(cx);
                    } else if !dropdown.open {
                        dropdown.open_popup(cx);
                    }
                },
            ));
        }

        if self.open && !self.disabled {
            let close_listener = cx.listener(|dropdown: &mut Dropdown, _, _, cx| {
                dropdown.close_popup(cx);
            });

            let popup_min_width = self
                .trigger_bounds
                .lock()
                .ok()
                .and_then(|slot| *slot)
                .map(|bounds| f32::from(bounds.size.width).max(components.dropdown_min_width))
                .unwrap_or(components.dropdown_min_width);

            let mut list = div()
                .id("dropdown-list")
                .flex()
                .flex_col()
                .min_w(px(popup_min_width))
                .max_h(px(components.dropdown_max_height))
                .overflow_y_scroll()
                .bg(colors.surface)
                .border_1()
                .border_color(colors.stroke_neutral)
                .rounded(px(radii.md))
                .shadow_md()
                .py(px(spacing.xs))
                .on_mouse_down_out(close_listener);

            for (idx, option) in self.options.iter().enumerate() {
                let value = option.value.clone();
                let label = option.label.clone();
                let selected = self.selected.as_ref() == Some(&value);
                let row_bg = if selected {
                    colors.neutral_selected
                } else {
                    colors.surface
                };
                let hover_bg = colors.neutral_hover;

                let mut row = div()
                    .id(("dropdown-option", idx as u64))
                    .debug_selector(move || format!("dropdown-option-{idx}"))
                    .flex()
                    .items_center()
                    .h(px(components.dropdown_option_height))
                    .px(px(spacing.md))
                    .bg(row_bg)
                    .text_size(px(typography.body.size))
                    .text_color(colors.on_neutral);

                if !self.closing {
                    row = row
                        .cursor_pointer()
                        .hover(move |s| s.bg(hover_bg))
                        .on_click(cx.listener(move |dropdown: &mut Dropdown, _, _, cx| {
                            if dropdown.closing {
                                return;
                            }
                            dropdown.selected = Some(value.clone());
                            if let Some(f) = &dropdown.on_select {
                                f(value.clone(), cx);
                            }
                            dropdown.close_popup(cx);
                        }));
                }

                list = list.child(row.child(label));
            }

            if self.options.is_empty() {
                list = list.child(
                    div()
                        .h(px(components.dropdown_option_height))
                        .px(px(spacing.md))
                        .flex()
                        .items_center()
                        .text_size(px(typography.body.size))
                        .text_color(colors.on_subtle_disabled)
                        .child("No options"),
                );
            }

            let exiting = self.closing;
            let motion = theme.motion();
            let motion_id = ("dropdown-list-motion", self.motion_epoch);
            let list = fluent_core::popup_motion_surface(motion_id, exiting, motion, list);

            el = el.child(deferred(
                anchored()
                    .anchor(Anchor::TopLeft)
                    .position(self.drop_origin)
                    .snap_to_window()
                    .child(list),
            ));
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;
    use std::{cell::Cell, rc::Rc};

    use fluent_test_utils::{init_fluent_test, open_fluent_test_window, FluentInteractionExt as _};
    use gpui::TestAppContext;

    use super::*;

    fn open(
        cx: &mut TestAppContext,
        changes: Rc<Cell<usize>>,
    ) -> (gpui::Entity<Dropdown>, &mut gpui::VisualTestContext) {
        open_fluent_test_window(cx, move |_, _| {
            Dropdown::new("dropdown")
                .option("one", "One")
                .option("two", "Two")
                .on_select(move |_, _| changes.set(changes.get() + 1))
        })
    }

    #[gpui::test]
    fn click_and_keyboard_selection_fire_once(cx: &mut TestAppContext) {
        let changes = Rc::new(Cell::new(0));
        let (dropdown, cx) = open(cx, changes.clone());
        cx.click_selector("dropdown-trigger");
        cx.click_selector("dropdown-option-1");
        assert_eq!(
            dropdown.read_with(cx, |dropdown, _| dropdown.selected_value().cloned()),
            Some("two".into())
        );
        assert_eq!(changes.get(), 1);

        cx.executor().advance_clock(Duration::from_millis(160));
        cx.update(|window, app| window.draw(app).clear(app));
        cx.click_selector("dropdown-trigger");
        cx.type_keys("up enter");
        assert_eq!(
            dropdown.read_with(cx, |dropdown, _| dropdown.selected_value().cloned()),
            Some("one".into())
        );
        assert_eq!(changes.get(), 2);
    }

    #[gpui::test]
    fn escape_closes_open_dropdown(cx: &mut TestAppContext) {
        let (dropdown, cx) = open(cx, Rc::new(Cell::new(0)));
        cx.click_selector("dropdown-trigger");
        assert!(dropdown.read_with(cx, |dropdown, _| dropdown.open));
        cx.type_keys("escape");
        assert!(dropdown.read_with(cx, |dropdown, _| dropdown.closing));
    }

    #[gpui::test]
    fn close_popup_keeps_dropdown_mounted_until_exit_duration(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let dropdown = cx.new(|_| Dropdown::new("test-dropdown").option("one", "One"));
        dropdown.update(cx, |dropdown, cx| {
            dropdown.open_popup(cx);
            dropdown.close_popup(cx);
            assert!(dropdown.open);
            assert!(dropdown.closing);
        });

        cx.executor().advance_clock(Duration::from_millis(159));
        dropdown.update(cx, |dropdown, _| {
            assert!(dropdown.open);
            assert!(dropdown.closing);
        });

        cx.executor().advance_clock(Duration::from_millis(1));
        dropdown.update(cx, |dropdown, _| {
            assert!(!dropdown.open);
            assert!(!dropdown.closing);
        });
    }

    #[gpui::test]
    fn reopening_dropdown_cancels_stale_exit_timer(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let dropdown = cx.new(|_| Dropdown::new("test-dropdown").option("one", "One"));
        dropdown.update(cx, |dropdown, cx| {
            dropdown.open_popup(cx);
            dropdown.close_popup(cx);
            dropdown.open_popup(cx);
            assert!(dropdown.open);
            assert!(!dropdown.closing);
        });

        cx.executor().advance_clock(Duration::from_millis(160));
        dropdown.update(cx, |dropdown, _| {
            assert!(dropdown.open);
            assert!(!dropdown.closing);
        });
    }
}
