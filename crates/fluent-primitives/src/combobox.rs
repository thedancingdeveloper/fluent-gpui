use std::sync::{Arc, Mutex};

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    anchored, canvas, deferred, div, point, prelude::*, px, svg, App, Bounds, ClickEvent, Context,
    Corner, CursorStyle, FocusHandle, Focusable, IntoElement, KeyDownEvent, MouseButton,
    MouseDownEvent, Pixels, Point, Render, SharedString, Window,
};

#[derive(Clone, Debug)]
pub struct ComboboxOption {
    pub value: SharedString,
    pub label: SharedString,
}

impl ComboboxOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

/// A searchable dropdown that supports typed filtering and freeform text.
#[allow(clippy::type_complexity)]
pub struct Combobox {
    options: Vec<ComboboxOption>,
    value: Option<SharedString>,
    query: String,
    placeholder: SharedString,
    disabled: bool,
    freeform: bool,
    open: bool,
    closing: bool,
    motion_epoch: u64,
    highlighted: usize,
    focus_handle: Option<FocusHandle>,
    drop_origin: Point<Pixels>,
    trigger_bounds: Arc<Mutex<Option<Bounds<Pixels>>>>,
    observing_theme: bool,
    on_select: Option<Box<dyn Fn(SharedString, &mut App) + 'static>>,
}

impl Default for Combobox {
    fn default() -> Self {
        Self {
            options: Vec::new(),
            value: None,
            query: String::new(),
            placeholder: "Select or type".into(),
            disabled: false,
            freeform: false,
            open: false,
            closing: false,
            motion_epoch: 0,
            highlighted: 0,
            focus_handle: None,
            drop_origin: point(px(0.0), px(0.0)),
            trigger_bounds: Arc::new(Mutex::new(None)),
            observing_theme: false,
            on_select: None,
        }
    }
}

impl Combobox {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn options(mut self, options: impl IntoIterator<Item = ComboboxOption>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    pub fn option(
        mut self,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.options.push(ComboboxOption::new(value, label));
        self
    }

    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        let value = value.into();
        self.query = self
            .options
            .iter()
            .find(|option| option.value == value)
            .map(|option| option.label.to_string())
            .unwrap_or_else(|| value.to_string());
        self.value = Some(value);
        self
    }

    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn freeform(mut self, freeform: bool) -> Self {
        self.freeform = freeform;
        self
    }

    pub fn query(mut self, query: impl Into<SharedString>) -> Self {
        self.query = query.into().to_string();
        self
    }

    pub fn on_select(mut self, f: impl Fn(SharedString, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn selected_value(&self) -> Option<&SharedString> {
        self.value.as_ref()
    }

    fn focus_handle(&mut self, cx: &mut Context<Self>) -> FocusHandle {
        self.focus_handle
            .get_or_insert_with(|| cx.focus_handle().tab_stop(true))
            .clone()
    }

    fn filtered_options(&self) -> Vec<ComboboxOption> {
        if self.query.is_empty() {
            return self.options.clone();
        }
        let needle = self.query.to_lowercase();
        self.options
            .iter()
            .filter(|option| option.label.to_lowercase().contains(&needle))
            .cloned()
            .collect()
    }

    fn select_option(&mut self, option: ComboboxOption, cx: &mut Context<Self>) {
        self.value = Some(option.value.clone());
        self.query = option.label.to_string();
        self.highlighted = 0;
        if let Some(f) = &self.on_select {
            f(option.value, cx);
        }
        self.close_popup(cx);
    }

    fn commit_freeform(&mut self, cx: &mut Context<Self>) {
        if !self.freeform || self.query.is_empty() {
            self.close_popup(cx);
            return;
        }

        let value = SharedString::from(self.query.clone());
        self.value = Some(value.clone());
        self.highlighted = 0;
        if let Some(f) = &self.on_select {
            f(value, cx);
        }
        self.close_popup(cx);
    }

    fn open_popup(&mut self, cx: &mut Context<Self>) {
        if !self.open || self.closing {
            self.motion_epoch = self.motion_epoch.wrapping_add(1);
        }
        self.open = true;
        self.closing = false;
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
            move |combo: gpui::WeakEntity<Self>, cx: &mut gpui::AsyncApp| {
                let mut cx = cx.clone();
                async move {
                    cx.background_executor().timer(duration).await;
                    combo
                        .update(&mut cx, move |combo, cx| {
                            if combo.motion_epoch == epoch && combo.closing {
                                combo.open = false;
                                combo.closing = false;
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

    fn update_origin(&mut self, fallback: Point<Pixels>) {
        self.drop_origin = self
            .trigger_bounds
            .lock()
            .ok()
            .and_then(|slot| *slot)
            .map(|bounds| point(bounds.origin.x, bounds.origin.y + bounds.size.height))
            .unwrap_or(fallback);
    }

    fn handle_key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if crate::focus::handle_tab_navigation(ev, window) {
            cx.notify();
            return;
        }

        if self.disabled || self.closing {
            return;
        }

        match ev.keystroke.key.as_str() {
            "backspace" => {
                self.query.pop();
                self.open_popup(cx);
                self.highlighted = 0;
            }
            "escape" => {
                self.close_popup(cx);
            }
            "down" => {
                let len = self.filtered_options().len();
                if len > 0 {
                    self.open_popup(cx);
                    self.highlighted = (self.highlighted + 1).min(len - 1);
                }
            }
            "up" => {
                if self.filtered_options().is_empty() {
                    return;
                }
                self.open_popup(cx);
                self.highlighted = self.highlighted.saturating_sub(1);
            }
            "enter" => {
                let filtered = self.filtered_options();
                if let Some(option) = filtered.get(self.highlighted).cloned() {
                    self.select_option(option, cx);
                    return;
                }
                self.commit_freeform(cx);
                return;
            }
            key if key.chars().count() == 1 => {
                self.query.push_str(key);
                self.open_popup(cx);
                self.highlighted = 0;
            }
            _ => return,
        }
        cx.notify();
    }

    fn on_mouse_down(&mut self, ev: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if self.disabled || self.closing {
            return;
        }
        let handle = self.focus_handle(cx);
        window.focus(&handle);
        self.update_origin(point(
            ev.position.x,
            ev.position.y + px(cx.theme().components.dropdown_height),
        ));
        self.open_popup(cx);
    }
}

impl Render for Combobox {
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
        let focused = focus_handle.is_focused(window);
        let fg = if self.disabled {
            colors.on_neutral_disabled
        } else if self.query.is_empty() {
            colors.on_subtle_disabled
        } else {
            colors.on_neutral
        };
        let text = if self.query.is_empty() {
            self.placeholder.clone()
        } else {
            SharedString::from(self.query.clone())
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

        let mut root = div()
            .key_context("Combobox")
            .track_focus(&focus_handle)
            .on_key_down(
                cx.listener(|combo: &mut Combobox, ev: &KeyDownEvent, window, cx| {
                    combo.handle_key(ev, window, cx);
                }),
            )
            .on_action(crate::focus::focus_next)
            .on_action(crate::focus::focus_previous)
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .relative()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.sm))
            .w_full()
            .h(px(components.dropdown_height))
            .px(px(spacing.md))
            .rounded(px(radii.md))
            .bg(if self.disabled {
                colors.neutral_disabled
            } else {
                colors.neutral
            })
            .border_1()
            .border_color(if focused {
                colors.accent
            } else {
                colors.stroke_neutral
            })
            .cursor(if self.disabled {
                CursorStyle::Arrow
            } else {
                CursorStyle::IBeam
            })
            .child(
                div()
                    .flex_1()
                    .text_size(px(typography.body.size))
                    .text_color(fg)
                    .child(text),
            )
            .child(
                svg()
                    .path("icons/chevron_down.svg")
                    .size(px(components.popup_icon_slot))
                    .text_color(fg),
            )
            .child(measure_trigger);

        if self.open && !self.disabled {
            let close_listener = cx.listener(|combo: &mut Combobox, _, _, cx| {
                combo.close_popup(cx);
            });
            let popup_min_width = self
                .trigger_bounds
                .lock()
                .ok()
                .and_then(|slot| *slot)
                .map(|bounds| f32::from(bounds.size.width).max(components.dropdown_min_width))
                .unwrap_or(components.dropdown_min_width);
            let filtered = self.filtered_options();
            let is_empty = filtered.is_empty();
            if self.highlighted >= filtered.len().max(1) {
                self.highlighted = 0;
            }
            let mut list = div()
                .id("combobox-list")
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

            for (idx, option) in filtered.into_iter().enumerate() {
                let value = option.value.clone();
                let label = option.label.clone();
                let highlighted = idx == self.highlighted;
                let row_bg = if highlighted {
                    colors.neutral_selected
                } else {
                    colors.surface
                };
                let mut row = div()
                    .id(("combobox-option", idx as u64))
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
                        .on_mouse_move(cx.listener(move |combo: &mut Combobox, _, _, cx| {
                            if combo.closing {
                                return;
                            }
                            if combo.highlighted != idx {
                                combo.highlighted = idx;
                                cx.notify();
                            }
                        }))
                        .hover({
                            let hover = colors.neutral_hover;
                            move |s| s.bg(hover)
                        })
                        .on_click(cx.listener(
                            move |combo: &mut Combobox, _: &ClickEvent, _, cx| {
                                if combo.closing {
                                    return;
                                }
                                combo.select_option(
                                    ComboboxOption {
                                        value: value.clone(),
                                        label: label.clone(),
                                    },
                                    cx,
                                );
                            },
                        ));
                }

                list = list.child(row.child(option.label));
            }

            if is_empty {
                list = list.child(
                    div()
                        .h(px(components.dropdown_option_height))
                        .px(px(spacing.md))
                        .flex()
                        .items_center()
                        .text_size(px(typography.body.size))
                        .text_color(colors.on_subtle_disabled)
                        .child("No matches"),
                );
            }

            let exiting = self.closing;
            let motion = theme.motion();
            let motion_id = ("combobox-list-motion", self.motion_epoch);
            let list = fluent_core::popup_motion_surface(motion_id, exiting, motion, list);

            root = root.child(deferred(
                anchored()
                    .anchor(Corner::TopLeft)
                    .position(self.drop_origin)
                    .snap_to_window()
                    .child(list),
            ));
        }

        root
    }
}

impl Focusable for Combobox {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle
            .clone()
            .unwrap_or_else(|| cx.focus_handle().tab_stop(true))
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use fluent_test_utils::init_fluent_test;
    use gpui::TestAppContext;

    use super::*;

    #[gpui::test]
    fn close_popup_keeps_combobox_mounted_until_exit_duration(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let combobox = cx.new(|_| Combobox::new().option("one", "One"));
        combobox.update(cx, |combobox, cx| {
            combobox.open_popup(cx);
            combobox.close_popup(cx);
            assert!(combobox.open);
            assert!(combobox.closing);
        });

        cx.executor().advance_clock(Duration::from_millis(159));
        combobox.update(cx, |combobox, _| {
            assert!(combobox.open);
            assert!(combobox.closing);
        });

        cx.executor().advance_clock(Duration::from_millis(1));
        combobox.update(cx, |combobox, _| {
            assert!(!combobox.open);
            assert!(!combobox.closing);
        });
    }

    #[gpui::test]
    fn reopening_combobox_cancels_stale_exit_timer(cx: &mut TestAppContext) {
        init_fluent_test(cx);

        let combobox = cx.new(|_| Combobox::new().option("one", "One"));
        combobox.update(cx, |combobox, cx| {
            combobox.open_popup(cx);
            combobox.close_popup(cx);
            combobox.open_popup(cx);
            assert!(combobox.open);
            assert!(!combobox.closing);
        });

        cx.executor().advance_clock(Duration::from_millis(160));
        combobox.update(cx, |combobox, _| {
            assert!(combobox.open);
            assert!(!combobox.closing);
        });
    }
}
