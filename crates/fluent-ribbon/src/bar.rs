use fluent_core::TextRampExt;
use fluent_core::{
    ColorScheme, ComponentTokens, RadiiTokens, SpacingTokens, Theme, ThemeProvider as _,
    TypographyTokens,
};
use gpui::{
    div, prelude::*, px, svg, App, ClickEvent, Context, FocusHandle, FontWeight, IntoElement,
    Render, Role, SharedString, Toggled, WeakEntity, Window,
};

use crate::defs::{
    ContextualTabDef, RibbonGroupDef, RibbonItemDef, RibbonTabBuilder, RibbonTabDef,
    RibbonToggleCallback,
};

struct RibbonRenderTokens<'a> {
    colors: &'a ColorScheme,
    spacing: SpacingTokens,
    radii: RadiiTokens,
    typography: TypographyTokens,
    components: ComponentTokens,
    bar: WeakEntity<RibbonBar>,
}

/// Run a ribbon command and record it as the bar's last invoked command.
fn invoke(
    bar: &WeakEntity<RibbonBar>,
    label: &SharedString,
    app: &mut App,
    run: impl FnOnce(&mut App),
) {
    run(app);
    let label = label.clone();
    bar.update(app, |bar, cx| {
        bar.last_command = Some(label);
        cx.notify();
    })
    .ok();
}

/// True for the keys that activate a focused button.
fn is_activation_key(key: &str) -> bool {
    matches!(key, "enter" | "space")
}

/// The tab index a navigation key moves to, or `None` for any other key.
fn tab_nav_target(key: &str, current: usize, count: usize) -> Option<usize> {
    if count == 0 {
        return None;
    }
    match key {
        "left" => Some((current + count - 1) % count),
        "right" => Some((current + 1) % count),
        "home" => Some(0),
        "end" => Some(count - 1),
        _ => None,
    }
}

/// The ribbon bar — the primary command surface of a FluentGUI application.
///
/// `RibbonBar` is an Entity; create it with `cx.new(|_| RibbonBar::new().tab(...))`.
///
/// ```ignore
/// let ribbon = cx.new(|_| {
///     RibbonBar::new()
///         .tab("Home", |t| {
///             t.group("Actions", |g| {
///                 g.large_button("Connect", "icons/connect.svg", |_, _, cx| {
///                     cx.dispatch_action(Connect.boxed_clone())
///                 })
///             })
///         })
///         .contextual_tab("Session", false, |t| {
///             t.group("Terminal", |g| { g.button("Clear", "icons/clear.svg", |_, _, _| {}) })
///         })
/// });
/// ```
pub struct RibbonBar {
    pub(crate) tabs: Vec<RibbonTabDef>,
    pub(crate) contextual_tabs: Vec<ContextualTabDef>,
    /// Index into the combined visible-tab list (normal tabs first, then visible contextual).
    pub active_tab: usize,
    /// When true, only the tab strip is shown; the content row is hidden.
    pub collapsed: bool,
    overflow_open: bool,
    /// Label of the most recently invoked command; shown as status feedback.
    last_command: Option<SharedString>,
    tab_handles: Vec<FocusHandle>,
}

impl RibbonBar {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self {
            tabs: vec![],
            contextual_tabs: vec![],
            active_tab: 0,
            collapsed: false,
            overflow_open: false,
            last_command: None,
            tab_handles: vec![],
        }
    }

    /// Add a normal tab.
    pub fn tab(
        mut self,
        name: impl Into<gpui::SharedString>,
        build: impl FnOnce(RibbonTabBuilder) -> RibbonTabBuilder,
    ) -> Self {
        self.tabs.push(build(RibbonTabBuilder::new(name)).build());
        self
    }

    /// Add a contextual tab (initially shown or hidden based on `visible`).
    pub fn contextual_tab(
        mut self,
        name: impl Into<gpui::SharedString>,
        visible: bool,
        build: impl FnOnce(RibbonTabBuilder) -> RibbonTabBuilder,
    ) -> Self {
        self.contextual_tabs.push(ContextualTabDef {
            name: name.into(),
            visible,
            tab: build(RibbonTabBuilder::new("")).build(),
        });
        self
    }

    /// Show or hide a contextual tab by name. Call `cx.notify()` after.
    pub fn set_contextual_visible(&mut self, name: &str, visible: bool) {
        if let Some(ct) = self
            .contextual_tabs
            .iter_mut()
            .find(|ct| ct.name.as_ref() == name)
        {
            ct.visible = visible;
            // If we're hiding the currently active contextual tab, fall back to tab 0.
            let n_normal = self.tabs.len();
            if !visible && self.active_tab >= n_normal {
                self.active_tab = 0;
            }
        }
    }

    /// Label of the most recently invoked command, if any.
    pub fn last_command(&self) -> Option<&SharedString> {
        self.last_command.as_ref()
    }

    /// Update the `selected` state of a toggle button by path.
    pub fn set_toggle_selected(
        &mut self,
        tab_idx: usize,
        group_idx: usize,
        item_idx: usize,
        selected: bool,
    ) {
        let tab = match self.tabs.get_mut(tab_idx) {
            Some(t) => t,
            None => return,
        };
        let group = match tab.groups.get_mut(group_idx) {
            Some(g) => g,
            None => return,
        };
        if let Some(RibbonItemDef::ToggleButton { selected: s, .. }) = group.items.get_mut(item_idx)
        {
            *s = selected;
        }
    }

    // -----------------------------------------------------------------------
    // Rendering helpers (free functions to avoid self-borrow issues)
    // -----------------------------------------------------------------------

    fn render_item(
        item: &RibbonItemDef,
        gidx: usize,
        iidx: usize,
        tokens: &RibbonRenderTokens<'_>,
    ) -> gpui::AnyElement {
        let colors = tokens.colors;
        let spacing = tokens.spacing;
        let radii = tokens.radii;
        let typography = tokens.typography;
        let components = tokens.components;
        let btn_hover_bg = colors.subtle_hover;
        let btn_pressed_bg = colors.subtle_selected;
        let fg = colors.on_neutral;
        let selected_bg = colors.neutral_selected;
        let hover_bg = colors.neutral_hover;
        let pressed_bg = colors.neutral_selected;
        let sep_color = colors.ribbon_group_separator;

        match item {
            RibbonItemDef::LargeButton {
                label,
                icon,
                on_click,
            } => {
                let (cb, key_cb) = (on_click.clone(), on_click.clone());
                let (bar, key_bar, label_c, key_label) = (
                    tokens.bar.clone(),
                    tokens.bar.clone(),
                    label.clone(),
                    label.clone(),
                );
                div()
                    .id(("rli", (gidx * 100 + iidx) as u64))
                    .debug_selector(move || format!("ribbon-item-{gidx}-{iidx}"))
                    .role(Role::Button)
                    .aria_label(label.clone())
                    .tab_index(0)
                    .on_key_down(move |ev, win, app| {
                        if is_activation_key(ev.keystroke.key.as_str()) {
                            invoke(&key_bar, &key_label, app, |app| {
                                key_cb(&ClickEvent::default(), win, app)
                            });
                        }
                    })
                    .h_full()
                    .min_w(px(48.0))
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(spacing.xs))
                    .px(px(spacing.sm))
                    .cursor_pointer()
                    .rounded(px(radii.sm))
                    .hover(move |s| s.bg(btn_hover_bg))
                    .active(move |s| s.bg(btn_pressed_bg))
                    .on_click(move |ev, win, app| {
                        invoke(&bar, &label_c, app, |app| cb(ev, win, app))
                    })
                    .child(
                        svg()
                            .path(icon.clone())
                            .size(px(components.ribbon_large_icon_size))
                            .text_color(fg),
                    )
                    .child(
                        div()
                            .text_ramp(&typography.caption)
                            .text_color(fg)
                            .child(label.clone()),
                    )
                    .into_any_element()
            }

            RibbonItemDef::Button {
                label,
                icon,
                on_click,
            } => {
                let (cb, key_cb) = (on_click.clone(), on_click.clone());
                let (bar, key_bar, label_c, key_label) = (
                    tokens.bar.clone(),
                    tokens.bar.clone(),
                    label.clone(),
                    label.clone(),
                );
                let icon_empty = icon.is_empty();
                div()
                    .id(("rbi", (gidx * 100 + iidx) as u64))
                    .role(Role::Button)
                    .aria_label(label.clone())
                    .tab_index(0)
                    .on_key_down(move |ev, win, app| {
                        if is_activation_key(ev.keystroke.key.as_str()) {
                            invoke(&key_bar, &key_label, app, |app| {
                                key_cb(&ClickEvent::default(), win, app)
                            });
                        }
                    })
                    .flex()
                    .flex_row()
                    .items_center()
                    .when(icon_empty, |d| d.justify_center())
                    .gap(px(spacing.xs))
                    .px(px(spacing.sm))
                    .py(px(spacing.xs))
                    .cursor_pointer()
                    .rounded(px(radii.sm))
                    .hover(move |s| s.bg(btn_hover_bg))
                    .active(move |s| s.bg(btn_pressed_bg))
                    .on_click(move |ev, win, app| {
                        invoke(&bar, &label_c, app, |app| cb(ev, win, app))
                    })
                    .when(!icon_empty, |d| {
                        d.child(
                            svg()
                                .path(icon.clone())
                                .size(px(components.ribbon_small_icon_size))
                                .text_color(fg),
                        )
                    })
                    .child(
                        div()
                            .text_ramp(&typography.caption)
                            .text_color(fg)
                            .child(label.clone()),
                    )
                    .into_any_element()
            }

            RibbonItemDef::IconButton {
                icon,
                tooltip,
                on_click,
            } => {
                let (cb, key_cb) = (on_click.clone(), on_click.clone());
                let (bar, key_bar, label_c, key_label) = (
                    tokens.bar.clone(),
                    tokens.bar.clone(),
                    tooltip.clone(),
                    tooltip.clone(),
                );
                div()
                    .id(("rib", (gidx * 100 + iidx) as u64))
                    .role(Role::Button)
                    .aria_label(tooltip.clone())
                    .tab_index(0)
                    .on_key_down(move |ev, win, app| {
                        if is_activation_key(ev.keystroke.key.as_str()) {
                            invoke(&key_bar, &key_label, app, |app| {
                                key_cb(&ClickEvent::default(), win, app)
                            });
                        }
                    })
                    .size(px(components.ribbon_small_icon_size + spacing.md))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .rounded(px(radii.sm))
                    .hover(move |s| s.bg(btn_hover_bg))
                    .active(move |s| s.bg(btn_pressed_bg))
                    .on_click(move |ev, win, app| {
                        invoke(&bar, &label_c, app, |app| cb(ev, win, app))
                    })
                    .child(
                        svg()
                            .path(icon.clone())
                            .size(px(components.ribbon_small_icon_size))
                            .text_color(fg),
                    )
                    .into_any_element()
            }

            RibbonItemDef::ToggleButton {
                label,
                icon,
                selected,
                on_click,
            } => {
                let is_sel = *selected;
                let new_state = !is_sel;
                let bg = if is_sel { selected_bg } else { colors.subtle };
                let (cb, key_cb): (RibbonToggleCallback, RibbonToggleCallback) =
                    (on_click.clone(), on_click.clone());
                let (bar, key_bar, label_c, key_label) = (
                    tokens.bar.clone(),
                    tokens.bar.clone(),
                    label.clone(),
                    label.clone(),
                );
                div()
                    .id(("rtb", (gidx * 100 + iidx) as u64))
                    .role(Role::Button)
                    .aria_label(label.clone())
                    .aria_toggled(if is_sel {
                        Toggled::True
                    } else {
                        Toggled::False
                    })
                    .tab_index(0)
                    .on_key_down(move |ev, win, app| {
                        if is_activation_key(ev.keystroke.key.as_str()) {
                            invoke(&key_bar, &key_label, app, |app| {
                                key_cb(new_state, &ClickEvent::default(), win, app)
                            });
                        }
                    })
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.xs))
                    .px(px(spacing.sm))
                    .py(px(spacing.xs))
                    .bg(bg)
                    .cursor_pointer()
                    .rounded(px(radii.sm))
                    .hover(move |s| s.bg(hover_bg))
                    .active(move |s| s.bg(pressed_bg))
                    .on_click(move |ev, win, app| {
                        invoke(&bar, &label_c, app, |app| cb(new_state, ev, win, app))
                    })
                    .child(
                        svg()
                            .path(icon.clone())
                            .size(px(components.ribbon_small_icon_size))
                            .text_color(fg),
                    )
                    .child(
                        div()
                            .text_ramp(&typography.caption)
                            .text_color(fg)
                            .child(label.clone()),
                    )
                    .into_any_element()
            }

            RibbonItemDef::Stack(items) => {
                const MAX_STACK_ROWS: usize = 3;
                let mut stack = div()
                    .flex()
                    .flex_row()
                    .gap(px(spacing.xs))
                    .h_full()
                    .py(px(spacing.xs));

                for (col_idx, column_items) in items.chunks(MAX_STACK_ROWS).enumerate() {
                    let mut column = div().flex().flex_col().justify_around().h_full();
                    for (row_idx, sub_item) in column_items.iter().enumerate() {
                        let sub_idx = col_idx * MAX_STACK_ROWS + row_idx;
                        let el = Self::render_item(sub_item, gidx, iidx * 10 + sub_idx, tokens);
                        column = column.child(el);
                    }
                    stack = stack.child(column);
                }
                stack.into_any_element()
            }

            RibbonItemDef::Separator => div()
                .w(px(1.0))
                .h_full()
                .mx(px(spacing.xs))
                .bg(sep_color)
                .into_any_element(),
        }
    }

    fn render_group(
        group: &RibbonGroupDef,
        gidx: usize,
        tokens: &RibbonRenderTokens<'_>,
    ) -> gpui::AnyElement {
        let colors = tokens.colors;
        let spacing = tokens.spacing;
        let typography = tokens.typography;
        let components = tokens.components;
        let label_fg = colors.on_subtle_disabled;

        // Button area
        // No items_center here — children stretch to full height by default.
        // Large buttons use h_full() to fill; compact buttons center themselves internally.
        let mut btn_row = div()
            .flex()
            .flex_row()
            .h(px(components.ribbon_content_button_height));
        for (iidx, item) in group.items.iter().enumerate() {
            let el = Self::render_item(item, gidx, iidx, tokens);
            btn_row = btn_row.child(el);
        }

        // Group label row
        let label_row = div()
            .h(px(components.ribbon_group_label_height))
            .flex()
            .items_center()
            .justify_center()
            .text_ramp(&typography.caption)
            .text_color(label_fg)
            .child(group.name.clone());

        div()
            .id(("rib-group", gidx as u64))
            .role(Role::Group)
            .aria_label(group.name.clone())
            .flex()
            .flex_col()
            .px(px(spacing.xs))
            .child(btn_row)
            .child(label_row)
            .into_any_element()
    }

    fn estimate_item_width(item: &RibbonItemDef, components: ComponentTokens) -> f32 {
        match item {
            RibbonItemDef::LargeButton { label, .. } => (label.len() as f32 * 6.5).max(56.0) + 16.0,
            RibbonItemDef::Button { label, icon, .. }
            | RibbonItemDef::ToggleButton { label, icon, .. } => {
                let icon_width = if icon.is_empty() {
                    0.0
                } else {
                    components.ribbon_small_icon_size + 8.0
                };
                (label.len() as f32 * 6.5) + icon_width + 20.0
            }
            RibbonItemDef::IconButton { .. } => components.ribbon_small_icon_size + 24.0,
            RibbonItemDef::Stack(items) => {
                const MAX_STACK_ROWS: usize = 3;
                items
                    .chunks(MAX_STACK_ROWS)
                    .map(|column| {
                        column
                            .iter()
                            .map(|item| Self::estimate_item_width(item, components))
                            .fold(72.0, f32::max)
                    })
                    .sum()
            }
            RibbonItemDef::Separator => 8.0,
        }
    }

    fn estimate_group_width(group: &RibbonGroupDef, components: ComponentTokens) -> f32 {
        let content_width: f32 = group
            .items
            .iter()
            .map(|item| Self::estimate_item_width(item, components))
            .sum();
        content_width.max(group.name.len() as f32 * 6.5 + 24.0) + 12.0
    }
}

impl Render for RibbonBar {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let radii = theme.radii;
        let typography = theme.typography;
        let components = theme.components;
        let render_tokens = RibbonRenderTokens {
            colors: &colors,
            spacing,
            radii,
            typography,
            components,
            bar: cx.entity().downgrade(),
        };

        let active = self.active_tab;
        let collapsed = self.collapsed;
        let n_normal = self.tabs.len();

        // ---- Collect tab strip data ----
        let normal_tab_names: Vec<gpui::SharedString> =
            self.tabs.iter().map(|t| t.name.clone()).collect();

        // Visible contextual tabs: (name, original index in contextual_tabs)
        let visible_ctx: Vec<(gpui::SharedString, usize)> = self
            .contextual_tabs
            .iter()
            .enumerate()
            .filter(|(_, ct)| ct.visible)
            .map(|(i, ct)| (ct.name.clone(), i))
            .collect();

        // ---- Determine active tab groups ----
        let active_groups: Vec<RibbonGroupDef> = if active < n_normal {
            self.tabs[active].groups.clone()
        } else {
            let ctx_idx = active - n_normal;
            visible_ctx
                .get(ctx_idx)
                .and_then(|(_, orig_idx)| self.contextual_tabs.get(*orig_idx))
                .map(|ct| ct.tab.groups.clone())
                .unwrap_or_default()
        };

        // ---- Tab strip ----
        let tab_strip_bg = colors.tab_strip_bg;
        let tab_active_bg = colors.tab_active_bg;
        let tab_hover_bg = colors.tab_hover_bg;
        let tab_pressed_bg = colors.neutral_selected;
        let tab_indicator = colors.ribbon_tab_indicator;
        let ctx_accent = colors.accent;
        let on_neutral = colors.on_neutral;
        let on_accent = colors.on_accent;

        // Visible tabs in strip order: normal first, then visible contextual.
        let tab_names: Vec<SharedString> = normal_tab_names
            .iter()
            .cloned()
            .chain(visible_ctx.iter().map(|(n, _)| n.clone()))
            .collect();
        let tab_count = tab_names.len();
        while self.tab_handles.len() < tab_count {
            self.tab_handles.push(cx.focus_handle());
        }
        self.tab_handles.truncate(tab_count);
        let tab_handles = self.tab_handles.clone();
        let tab_keys = {
            let handles = tab_handles.clone();
            let bar = cx.entity().downgrade();
            move |idx: usize| {
                let (handles, bar) = (handles.clone(), bar.clone());
                move |ev: &gpui::KeyDownEvent, win: &mut Window, app: &mut App| {
                    if let Some(next) = tab_nav_target(ev.keystroke.key.as_str(), idx, tab_count) {
                        bar.update(app, |bar, cx| {
                            bar.active_tab = next;
                            bar.overflow_open = false;
                            cx.notify();
                        })
                        .ok();
                        handles[next].focus(win, app);
                        app.stop_propagation();
                    }
                }
            }
        };

        let mut tab_strip = div()
            .id("rib-tab-strip")
            .role(Role::TabList)
            .aria_orientation(gpui::accesskit::Orientation::Horizontal)
            .flex()
            .flex_row()
            .items_end() // align to bottom so underlines sit at the same baseline
            .h(px(components.ribbon_tab_height))
            .bg(tab_strip_bg);

        // Normal tabs
        for (i, name) in normal_tab_names.iter().enumerate() {
            let is_active = i == active;
            let name = name.clone();
            let handler = cx.listener(move |bar: &mut RibbonBar, _: &ClickEvent, _, cx| {
                bar.active_tab = i;
                bar.overflow_open = false;
                cx.notify();
            });

            // The active-tab underline: a 2px accent-coloured bar at the very bottom
            // implemented as an absolute-positioned child inside a relative wrapper.
            let underline_color = if is_active {
                tab_indicator
            } else {
                tab_strip_bg
            };
            let tab_bg = if is_active {
                tab_active_bg
            } else {
                tab_strip_bg
            };

            tab_strip = tab_strip.child(
                div()
                    .id(("rib-tab", i as u64))
                    .debug_selector(move || format!("ribbon-tab-{i}"))
                    .role(Role::Tab)
                    .aria_label(name.clone())
                    .aria_selected(is_active)
                    .aria_position_in_set(i + 1)
                    .aria_size_of_set(tab_count)
                    .track_focus(&tab_handles[i])
                    .tab_index(0)
                    .tab_stop(is_active)
                    .on_key_down(tab_keys(i))
                    .relative()
                    .px(px(spacing.lg))
                    .py(px(4.0))
                    .bg(tab_bg)
                    .cursor_pointer()
                    .hover(move |s| s.bg(tab_hover_bg))
                    .active(move |s| s.bg(tab_pressed_bg))
                    .font_weight(if is_active {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_ramp(&typography.body)
                    .text_color(on_neutral)
                    .on_click(handler)
                    .child(name)
                    // 2px underline at the very bottom
                    .child(
                        div()
                            .absolute()
                            .bottom_0()
                            .left_0()
                            .right_0()
                            .h(px(2.0))
                            .bg(underline_color),
                    ),
            );
        }

        // Contextual tabs (with accent header band)
        if !visible_ctx.is_empty() {
            let mut ctx_strip = div().flex().flex_row().items_end().relative(); // so header band can be absolute

            // Contextual header band at the very top of this section
            ctx_strip = ctx_strip.child(
                div()
                    .absolute()
                    .top_0()
                    .left_0()
                    .right_0()
                    .h(px(components.ribbon_contextual_band_height))
                    .bg(ctx_accent),
            );

            for (ctx_pos, (name, _orig_idx)) in visible_ctx.iter().enumerate() {
                let global_idx = n_normal + ctx_pos;
                let is_active = global_idx == active;
                let name = name.clone();
                let handler = cx.listener(move |bar: &mut RibbonBar, _: &ClickEvent, _, cx| {
                    bar.active_tab = global_idx;
                    bar.overflow_open = false;
                    cx.notify();
                });

                // Contextual tabs: solid accent background, white text
                let fg = on_accent;
                // Slightly lighter bg when active to show selection
                let ctx_tab_bg = if is_active {
                    colors.accent_hover
                } else {
                    ctx_accent
                };
                let underline_color = if is_active { on_accent } else { ctx_tab_bg };

                ctx_strip = ctx_strip.child(
                    div()
                        .id(("rib-ctx-tab", global_idx as u64))
                        .role(Role::Tab)
                        .aria_label(name.clone())
                        .aria_selected(is_active)
                        .aria_position_in_set(global_idx + 1)
                        .aria_size_of_set(tab_count)
                        .track_focus(&tab_handles[global_idx])
                        .tab_index(0)
                        .tab_stop(is_active)
                        .on_key_down(tab_keys(global_idx))
                        .relative()
                        .px(px(spacing.lg))
                        .py(px(4.0))
                        .pt(px(components.ribbon_contextual_band_height + 4.0)) // make room for the header band
                        .bg(ctx_tab_bg)
                        .cursor_pointer()
                        .font_weight(if is_active {
                            FontWeight::SEMIBOLD
                        } else {
                            FontWeight::NORMAL
                        })
                        .text_ramp(&typography.body)
                        .text_color(fg)
                        .on_click(handler)
                        .child(name)
                        .child(
                            div()
                                .absolute()
                                .bottom_0()
                                .left_0()
                                .right_0()
                                .h(px(2.0))
                                .bg(underline_color),
                        ),
                );
            }

            tab_strip = tab_strip.child(ctx_strip);
        }

        // Collapse toggle (∧/∨) at the right edge of the tab strip
        let collapse_handler = cx.listener(|bar: &mut RibbonBar, _: &ClickEvent, _, cx| {
            bar.collapsed = !bar.collapsed;
            bar.overflow_open = false;
            cx.notify();
        });
        let collapse_icon = if collapsed {
            "icons/chevron_down.svg"
        } else {
            "icons/chevron_up.svg"
        };
        let status = self.last_command.clone();
        tab_strip = tab_strip
            .child(div().flex_1()) // spacer
            .when_some(status, |d, label| {
                d.child(
                    div()
                        .id("rib-status")
                        .role(Role::Status)
                        .aria_label(SharedString::from(format!("Last command: {label}")))
                        .self_center()
                        .px(px(spacing.sm))
                        .text_ramp(&typography.caption)
                        .text_color(colors.on_subtle_disabled)
                        .child(label),
                )
            })
            .child(
                div()
                    .id("rib-collapse")
                    .role(Role::Button)
                    .aria_label(if collapsed {
                        "Expand the ribbon"
                    } else {
                        "Collapse the ribbon"
                    })
                    .aria_expanded(!collapsed)
                    .tab_index(0)
                    .on_key_down({
                        let bar = cx.entity().downgrade();
                        move |ev, _, app| {
                            if is_activation_key(ev.keystroke.key.as_str()) {
                                bar.update(app, |bar, cx| {
                                    bar.collapsed = !bar.collapsed;
                                    bar.overflow_open = false;
                                    cx.notify();
                                })
                                .ok();
                            }
                        }
                    })
                    .size(px(components.ribbon_tab_height))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(move |s| s.bg(tab_hover_bg))
                    .active(move |s| s.bg(tab_pressed_bg))
                    .on_click(collapse_handler)
                    .child(
                        svg()
                            .path(collapse_icon)
                            .size(px((components.ribbon_small_icon_size - 4.0).max(10.0)))
                            .text_color(on_neutral),
                    ),
            );

        // ---- Content row ----
        let ribbon_bg = colors.ribbon_bg;
        let group_sep_color = colors.ribbon_group_separator;

        let content = if collapsed {
            None
        } else {
            let viewport_width = f32::from(window.viewport_size().width);
            let overflow_button_width = 72.0;
            let available_width = (viewport_width - overflow_button_width).max(160.0);
            let mut used_width = 0.0;
            let mut visible_count = 0;
            for group in &active_groups {
                let estimated = Self::estimate_group_width(group, components);
                if visible_count == 0 || used_width + estimated <= available_width {
                    visible_count += 1;
                    used_width += estimated;
                } else {
                    break;
                }
            }
            let visible_count = visible_count.min(active_groups.len());
            let hidden_groups = active_groups[visible_count..].to_vec();

            let mut content_row = div()
                .id("rib-content")
                .role(Role::TabPanel)
                .relative()
                .flex()
                .flex_row()
                .bg(ribbon_bg)
                .border_b_1()
                .border_color(group_sep_color);

            for (gidx, group) in active_groups.iter().take(visible_count).enumerate() {
                // Group separator before each group (except the first)
                if gidx > 0 {
                    content_row =
                        content_row.child(div().w(px(1.0)).bg(group_sep_color).my(px(spacing.sm)));
                }

                let el = Self::render_group(group, gidx, &render_tokens);
                content_row = content_row.child(el);
            }

            // Right-side spacer so groups don't stretch to fill
            content_row = content_row.child(div().flex_1());

            if !hidden_groups.is_empty() {
                let overflow_handler = cx.listener(|bar: &mut RibbonBar, _: &ClickEvent, _, cx| {
                    bar.overflow_open = !bar.overflow_open;
                    cx.notify();
                });
                content_row = content_row.child(
                    div()
                        .id("rib-overflow")
                        .role(Role::Button)
                        .aria_label("More commands")
                        .aria_expanded(self.overflow_open)
                        .tab_index(0)
                        .on_key_down({
                            let bar = cx.entity().downgrade();
                            move |ev, _, app| {
                                if is_activation_key(ev.keystroke.key.as_str()) {
                                    bar.update(app, |bar, cx| {
                                        bar.overflow_open = !bar.overflow_open;
                                        cx.notify();
                                    })
                                    .ok();
                                } else if ev.keystroke.key == "escape" {
                                    bar.update(app, |bar, cx| {
                                        bar.overflow_open = false;
                                        cx.notify();
                                    })
                                    .ok();
                                }
                            }
                        })
                        .h(px(components.ribbon_content_button_height))
                        .min_w(px(overflow_button_width))
                        .flex()
                        .flex_col()
                        .items_center()
                        .justify_center()
                        .gap(px(spacing.xs))
                        .px(px(spacing.sm))
                        .cursor_pointer()
                        .rounded(px(radii.sm))
                        .hover({
                            let hover = colors.subtle_hover;
                            move |s| s.bg(hover)
                        })
                        .active(move |s| s.bg(colors.subtle_selected))
                        .on_click(overflow_handler)
                        .child(
                            svg()
                                .path("icons/chevron_down.svg")
                                .size(px(components.ribbon_small_icon_size))
                                .text_color(on_neutral),
                        )
                        .child(
                            div()
                                .text_ramp(&typography.caption)
                                .text_color(on_neutral)
                                .child("More"),
                        ),
                );

                if self.overflow_open {
                    let close_listener = cx.listener(|bar: &mut RibbonBar, _, _, cx| {
                        bar.overflow_open = false;
                        cx.notify();
                    });
                    let mut overflow_menu = div()
                        .id("rib-overflow-menu")
                        .role(Role::List)
                        .aria_label("More commands")
                        .absolute()
                        .top(px(components.ribbon_content_button_height
                            + components.ribbon_group_label_height))
                        .right(px(spacing.sm))
                        .min_w(px(240.0))
                        .flex()
                        .flex_col()
                        .bg(colors.surface)
                        .border_1()
                        .border_color(colors.stroke_neutral)
                        .rounded(px(radii.md))
                        .shadow_md()
                        .py(px(spacing.sm))
                        .on_mouse_down_out(close_listener);

                    for (idx, group) in hidden_groups.iter().enumerate() {
                        if idx > 0 {
                            overflow_menu = overflow_menu.child(
                                div()
                                    .h(px(components.popup_separator_height))
                                    .mx(px(spacing.sm))
                                    .my(px(spacing.xs))
                                    .bg(colors.stroke_neutral_subtle),
                            );
                        }
                        overflow_menu =
                            overflow_menu.child(Self::render_group(group, idx, &render_tokens));
                    }
                    content_row = content_row.child(overflow_menu);
                }
            } else if self.overflow_open {
                self.overflow_open = false;
            }

            Some(content_row)
        };

        // ---- Assemble ----
        let mut bar = div().flex().flex_col().bg(tab_strip_bg).child(tab_strip);

        if let Some(content) = content {
            bar = bar.child(content);
        }

        bar
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::Cell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::TestAppContext;

    use super::*;

    #[test]
    fn tab_navigation_wraps_and_jumps() {
        assert_eq!(tab_nav_target("right", 2, 3), Some(0));
        assert_eq!(tab_nav_target("left", 0, 3), Some(2));
        assert_eq!(tab_nav_target("home", 2, 3), Some(0));
        assert_eq!(tab_nav_target("end", 0, 3), Some(2));
        assert_eq!(tab_nav_target("a", 0, 3), None);
        assert_eq!(tab_nav_target("right", 0, 0), None);
    }

    fn build(cx: &mut Context<RibbonBar>, hits: Rc<Cell<usize>>) -> RibbonBar {
        RibbonBar::new(cx)
            .tab("Home", move |t| {
                t.group("Actions", move |g| {
                    let h = hits.clone();
                    g.large_button("Run", "", move |_, _, _| h.set(h.get() + 1))
                        .toggle_button("Wrap", "", true, |_, _, _, _| {})
                })
            })
            .tab("View", |t| {
                t.group("Zoom", |g| g.button("In", "", |_, _, _| {}))
            })
            .contextual_tab("Session", false, |t| {
                t.group("Terminal", |g| g.button("Clear", "", |_, _, _| {}))
            })
    }

    #[gpui::test]
    fn exposes_tablist_tab_group_and_button_semantics(cx: &mut TestAppContext) {
        let (_bar, vcx) = open_fluent_test_window(cx, |_, cx| build(cx, Rc::default()));
        let joined = vcx.accessibility_snapshot().join("\n");
        assert!(joined.contains("TabList"), "{joined}");
        assert!(joined.contains("Tab \"Home\""), "{joined}");
        assert!(joined.contains("Tab \"View\""), "{joined}");
        assert!(!joined.contains("Session"), "{joined}");
        assert!(joined.contains("TabPanel"), "{joined}");
        assert!(joined.contains("Group \"Actions\""), "{joined}");
        assert!(joined.contains("Button \"Run\""), "{joined}");
        assert!(joined.contains("Button \"Wrap\""), "{joined}");
    }

    #[gpui::test]
    fn arrow_keys_move_between_tabs(cx: &mut TestAppContext) {
        let (bar, vcx) = open_fluent_test_window(cx, |_, cx| build(cx, Rc::default()));
        vcx.click_selector("ribbon-tab-0");
        vcx.type_keys("right");
        assert_eq!(bar.read_with(vcx, |b, _| b.active_tab), 1);
        vcx.type_keys("right");
        assert_eq!(bar.read_with(vcx, |b, _| b.active_tab), 0);
        vcx.type_keys("end");
        assert_eq!(bar.read_with(vcx, |b, _| b.active_tab), 1);
    }

    #[gpui::test]
    fn contextual_tab_appears_when_made_visible(cx: &mut TestAppContext) {
        let (bar, vcx) = open_fluent_test_window(cx, |_, cx| build(cx, Rc::default()));
        bar.update(vcx, |b, cx| {
            b.set_contextual_visible("Session", true);
            cx.notify();
        });
        let joined = vcx.accessibility_snapshot().join("\n");
        assert!(joined.contains("Tab \"Session\""), "{joined}");
    }

    #[gpui::test]
    fn keyboard_invocation_runs_command_and_records_feedback(cx: &mut TestAppContext) {
        let hits = Rc::new(Cell::new(0));
        let (bar, vcx) = open_fluent_test_window(cx, |_, cx| build(cx, hits.clone()));
        vcx.click_selector("ribbon-item-0-0"); // click focuses the button
        assert_eq!(hits.get(), 1);
        vcx.type_keys("space enter");
        assert_eq!(hits.get(), 3);
        assert_eq!(
            bar.read_with(vcx, |b, _| b.last_command().cloned()),
            Some("Run".into())
        );
        let joined = vcx.accessibility_snapshot().join("\n");
        assert!(joined.contains("Last command: Run"), "{joined}");
    }
}
