use std::sync::Arc;

use crate::focus::FluentFocus as _;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, Hsla, IntoElement, KeyDownEvent, RenderOnce,
    Role, SharedString, Toggled, Window,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadioGroupOrientation {
    #[default]
    Vertical,
    Horizontal,
}

#[derive(Clone, Debug)]
pub struct RadioOption {
    pub value: SharedString,
    pub label: SharedString,
    pub disabled: bool,
}

impl RadioOption {
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            disabled: false,
        }
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }
}

/// A controlled Fluent radio group.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct RadioGroup {
    id: ElementId,
    options: Vec<RadioOption>,
    label: Option<SharedString>,
    selected: Option<SharedString>,
    orientation: RadioGroupOrientation,
    disabled: bool,
    tab_index: Option<isize>,
    on_select: Option<Arc<dyn Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl RadioGroup {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            options: Vec::new(),
            label: None,
            selected: None,
            orientation: RadioGroupOrientation::Vertical,
            disabled: false,
            tab_index: None,
            on_select: None,
        }
    }

    /// The accessible name of the group (the legend of the choice).
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn option(
        mut self,
        value: impl Into<SharedString>,
        label: impl Into<SharedString>,
    ) -> Self {
        self.options.push(RadioOption::new(value, label));
        self
    }

    pub fn options(mut self, options: impl IntoIterator<Item = RadioOption>) -> Self {
        self.options = options.into_iter().collect();
        self
    }

    pub fn selected(mut self, value: impl Into<SharedString>) -> Self {
        self.selected = Some(value.into());
        self
    }

    pub fn orientation(mut self, orientation: RadioGroupOrientation) -> Self {
        self.orientation = orientation;
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

    pub fn on_select(
        mut self,
        f: impl Fn(SharedString, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Arc::new(f));
        self
    }
}

impl RenderOnce for RadioGroup {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let enabled: Vec<bool> = self
            .options
            .iter()
            .map(|o| !(self.disabled || o.disabled))
            .collect();
        let handles = crate::roving::handles(window, cx, &self.id, self.options.len());
        let default = self
            .options
            .iter()
            .position(|o| self.selected.as_ref() == Some(&o.value));
        let stop = crate::roving::stop_index(window, &handles, &enabled, default);
        let values: Vec<SharedString> = self.options.iter().map(|o| o.value.clone()).collect();

        let theme = cx.theme();
        let colors = &theme.colors;
        let spacing = &theme.spacing;
        let typography = &theme.typography;

        let mut root = div()
            .id(self.id)
            .role(Role::RadioGroup)
            .when_some(self.label.clone(), |d, name| d.aria_label(name))
            .flex()
            .gap(px(spacing.md))
            .when(self.orientation == RadioGroupOrientation::Horizontal, |d| {
                d.flex_row()
            })
            .when(self.orientation == RadioGroupOrientation::Vertical, |d| {
                d.flex_col()
            });

        for (idx, option) in self.options.into_iter().enumerate() {
            let checked = self.selected.as_ref() == Some(&option.value);
            let tab_index = self.tab_index;
            let focus_ring = colors.stroke_accent;
            let disabled = self.disabled || option.disabled;
            let on_select = self.on_select.clone();
            let value = option.value.clone();
            let fg = if disabled {
                colors.on_neutral_disabled
            } else {
                colors.on_neutral
            };
            let outer_border: Hsla = if checked {
                colors.accent
            } else {
                colors.stroke_neutral
            };

            let mut row = div()
                .id(("radio-option", idx as u64))
                .debug_selector(move || format!("radio-option-{idx}"))
                .role(Role::RadioButton)
                .aria_label(option.label.clone())
                .aria_toggled(if checked {
                    Toggled::True
                } else {
                    Toggled::False
                })
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing.sm))
                .text_size(px(typography.body.size))
                .line_height(px(typography.body.line_height))
                .text_color(fg);

            if !disabled {
                let nav_handles = handles.clone();
                let nav_enabled = enabled.clone();
                let nav_values = values.clone();
                let nav_handler = on_select.clone();
                row = row
                    .track_focus(&handles[idx])
                    .fluent_focus(tab_index, focus_ring)
                    .tab_stop(stop == Some(idx))
                    .on_key_down(move |ev: &KeyDownEvent, win, app| {
                        if let Some(next) =
                            crate::roving::target(ev.keystroke.key.as_str(), idx, &nav_enabled)
                        {
                            nav_handles[next].focus(win, app);
                            if let Some(h) = &nav_handler {
                                h(nav_values[next].clone(), &ClickEvent::default(), win, app);
                            }
                            app.stop_propagation();
                        }
                    })
                    .cursor_pointer()
                    .hover({
                        let hover = colors.subtle_hover;
                        move |s| s.bg(hover)
                    });
                if let Some(handler) = on_select {
                    let key_handler = handler.clone();
                    let key_value = value.clone();
                    row = row
                        .on_key_down(move |ev: &KeyDownEvent, win, app| {
                            if matches!(ev.keystroke.key.as_str(), "enter" | "space") {
                                key_handler(key_value.clone(), &ClickEvent::default(), win, app);
                            }
                        })
                        .on_click(move |ev, win, app| handler(value.clone(), ev, win, app));
                }
            }

            row = row
                .child(
                    div()
                        .size(px(16.0))
                        .rounded(px(9999.0))
                        .border_1()
                        .border_color(outer_border)
                        .flex()
                        .items_center()
                        .justify_center()
                        .child(
                            div()
                                .size(px(if checked { 8.0 } else { 0.0 }))
                                .rounded(px(9999.0))
                                .bg(colors.accent),
                        ),
                )
                .child(option.label);

            root = root.child(row);
        }

        root
    }
}

#[cfg(test)]
mod tests {
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{Context, Entity, Render, TestAppContext};

    use super::*;

    struct Host {
        selected: SharedString,
        changes: usize,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.entity();
            RadioGroup::new("radio-group")
                .option("one", "One")
                .option("two", "Two")
                .selected(self.selected.clone())
                .on_select(move |value, _, _, cx| {
                    entity.update(cx, |host, cx| {
                        host.selected = value;
                        host.changes += 1;
                        cx.notify();
                    });
                })
        }
    }

    fn open(cx: &mut TestAppContext) -> (Entity<Host>, &mut gpui::VisualTestContext) {
        open_fluent_test_window(cx, |_, _| Host {
            selected: "one".into(),
            changes: 0,
        })
    }

    #[gpui::test]
    fn click_selects_once(cx: &mut TestAppContext) {
        let (host, cx) = open(cx);
        cx.click_selector("radio-option-1");

        host.read_with(cx, |host, _| {
            assert_eq!(host.selected.as_ref(), "two");
            assert_eq!(host.changes, 1);
        });
    }

    #[gpui::test]
    fn arrows_move_and_select_and_home_end_jump(cx: &mut TestAppContext) {
        let (host, cx) = open(cx);
        cx.click_selector("radio-option-0");
        cx.type_keys("right");
        host.read_with(cx, |h, _| assert_eq!(h.selected.as_ref(), "two"));
        cx.type_keys("right");
        host.read_with(cx, |h, _| assert_eq!(h.selected.as_ref(), "one"));
        cx.type_keys("end");
        host.read_with(cx, |h, _| assert_eq!(h.selected.as_ref(), "two"));
        cx.type_keys("home");
        host.read_with(cx, |h, _| assert_eq!(h.selected.as_ref(), "one"));
    }

    #[gpui::test]
    fn group_is_one_tab_stop(cx: &mut TestAppContext) {
        let (host, cx) = open(cx);
        cx.type_keys("tab");
        host.read_with(cx, |h, _| assert_eq!(h.selected.as_ref(), "one"));
        cx.type_keys("tab");
        // Tab left the group instead of visiting the second option.
        host.read_with(cx, |h, _| assert_eq!(h.changes, 0));
        cx.type_keys("space");
        host.read_with(cx, |h, _| assert_eq!(h.changes, 0));
    }

    #[gpui::test]
    fn keyboard_selects_once(cx: &mut TestAppContext) {
        let (host, cx) = open(cx);
        cx.click_selector("radio-option-1");
        host.update(cx, |host, cx| {
            host.selected = "one".into();
            host.changes = 0;
            cx.notify();
        });
        cx.type_keys("space");

        host.read_with(cx, |host, _| {
            assert_eq!(host.selected.as_ref(), "two");
            assert_eq!(host.changes, 1);
        });
    }

    struct A11yHost(Box<dyn Fn() -> gpui::AnyElement>);

    impl Render for A11yHost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            (self.0)()
        }
    }

    fn snapshot(
        cx: &mut TestAppContext,
        build: impl Fn() -> gpui::AnyElement + 'static,
    ) -> Vec<String> {
        let (_, cx) = open_fluent_test_window(cx, move |_, _| A11yHost(Box::new(build)));
        cx.accessibility_snapshot()
    }

    #[gpui::test]
    fn accessibility_snapshot_reports_radio_group(cx: &mut TestAppContext) {
        let snap = snapshot(cx, || {
            RadioGroup::new("r")
                .label("Size")
                .option("s", "Small")
                .option("l", "Large")
                .selected("l")
                .into_any_element()
        });
        assert_eq!(
            snap,
            vec![
                "Window".to_string(),
                "RadioGroup \"Size\"".to_string(),
                "RadioButton \"Small\" [toggled=False]".to_string(),
                "RadioButton \"Large\" [toggled=True]".to_string(),
            ]
        );
    }
}
