use fluent_core::{
    ColorScheme, ComponentTokens, RadiiTokens, SpacingTokens, Theme, ThemeProvider as _,
    TypographyTokens,
};
use gpui::{
    div, prelude::*, px, svg, ClickEvent, Context, FontWeight, IntoElement, Render, Window,
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
        let fg = colors.on_neutral;
        let selected_bg = colors.neutral_selected;
        let hover_bg = colors.neutral_hover;
        let sep_color = colors.ribbon_group_separator;

        match item {
            RibbonItemDef::LargeButton {
                label,
                icon,
                on_click,
            } => {
                let cb = on_click.clone();
                div()
                    .id(("rli", (gidx * 100 + iidx) as u64))
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
                    .on_click(move |ev, win, app| cb(ev, win, app))
                    .child(
                        svg()
                            .path(icon.clone())
                            .size(px(components.ribbon_large_icon_size))
                            .text_color(fg),
                    )
                    .child(
                        div()
                            .text_size(px(typography.caption.size))
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
                let cb = on_click.clone();
                let icon_empty = icon.is_empty();
                div()
                    .id(("rbi", (gidx * 100 + iidx) as u64))
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
                    .on_click(move |ev, win, app| cb(ev, win, app))
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
                            .text_size(px(typography.caption.size))
                            .text_color(fg)
                            .child(label.clone()),
                    )
                    .into_any_element()
            }

            RibbonItemDef::IconButton { icon, on_click, .. } => {
                let cb = on_click.clone();
                div()
                    .id(("rib", (gidx * 100 + iidx) as u64))
                    .size(px(components.ribbon_small_icon_size + spacing.md))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .rounded(px(radii.sm))
                    .hover(move |s| s.bg(btn_hover_bg))
                    .on_click(move |ev, win, app| cb(ev, win, app))
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
                let cb: RibbonToggleCallback = on_click.clone();
                div()
                    .id(("rtb", (gidx * 100 + iidx) as u64))
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
                    .on_click(move |ev, win, app| cb(new_state, ev, win, app))
                    .child(
                        svg()
                            .path(icon.clone())
                            .size(px(components.ribbon_small_icon_size))
                            .text_color(fg),
                    )
                    .child(
                        div()
                            .text_size(px(typography.caption.size))
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
            .text_size(px(typography.caption.size))
            .text_color(label_fg)
            .child(group.name.clone());

        div()
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
        let tab_indicator = colors.ribbon_tab_indicator;
        let ctx_accent = colors.accent;
        let on_neutral = colors.on_neutral;
        let on_accent = colors.on_accent;

        let mut tab_strip = div()
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
                    .relative()
                    .px(px(spacing.lg))
                    .py(px(4.0))
                    .bg(tab_bg)
                    .cursor_pointer()
                    .hover(move |s| s.bg(tab_hover_bg))
                    .font_weight(if is_active {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .text_size(px(typography.body.size))
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
                        .text_size(px(typography.body.size))
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
        tab_strip = tab_strip
            .child(div().flex_1()) // spacer
            .child(
                div()
                    .id("rib-collapse")
                    .size(px(components.ribbon_tab_height))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(move |s| s.bg(tab_hover_bg))
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
                        .on_click(overflow_handler)
                        .child(
                            svg()
                                .path("icons/chevron_down.svg")
                                .size(px(components.ribbon_small_icon_size))
                                .text_color(on_neutral),
                        )
                        .child(
                            div()
                                .text_size(px(typography.caption.size))
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
