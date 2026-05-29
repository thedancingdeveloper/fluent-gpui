use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, svg, AnyView, ClickEvent, Context, CursorStyle, DragMoveEvent, FontWeight,
    IntoElement, MouseButton, MouseDownEvent, Render, SharedString, Window,
};

/// Which edge a dock panel is attached to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DockPosition {
    Left,
    Right,
    Bottom,
    Top,
}

/// Phantom drag type used for the resize-handle drag gesture.
///
/// Using a named unit struct avoids conflicts with other `on_drag` usages.
pub(crate) struct DockResizeDrag;

impl Render for DockResizeDrag {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        // Invisible ghost entity — only the drag gesture matters, not the visual.
        div()
    }
}

/// A resizable edge panel (sidebar / output / status bar).
///
/// The panel edge is draggable: grab the 4px resize handle to change the width
/// (left/right panels) or height (top/bottom panels).
pub struct DockPanel {
    pub position: DockPosition,
    pub title: SharedString,
    pub content: Option<AnyView>,
    pub size: f32,
    pub collapsed: bool,
    pub pinned: bool,
    /// Captured at drag-start: (initial_size, initial_cursor_pos).
    resize_start: Option<(f32, f32)>,
}

impl DockPanel {
    pub fn new(
        cx: &mut Context<Self>,
        position: DockPosition,
        title: impl Into<SharedString>,
    ) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        let size = match position {
            DockPosition::Left | DockPosition::Right => 240.0,
            DockPosition::Top | DockPosition::Bottom => 200.0,
        };
        Self {
            position,
            title: title.into(),
            content: None,
            size,
            collapsed: false,
            pinned: true,
            resize_start: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn content(mut self, view: AnyView) -> Self {
        self.content = Some(view);
        self
    }

    pub fn set_content(&mut self, view: AnyView, cx: &mut Context<Self>) {
        self.content = Some(view);
        cx.notify();
    }

    pub fn toggle_collapsed(&mut self, cx: &mut Context<Self>) {
        self.collapsed = !self.collapsed;
        cx.notify();
    }
}

impl Render for DockPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let components = theme.components;

        let panel_bg = colors.panel_bg;
        let panel_border = colors.panel_border;
        let on_neutral = colors.on_neutral;
        let hover_bg = colors.subtle_hover;
        let handle_hover = colors.stroke_neutral_dim;

        let title = self.title.clone();
        let collapsed = self.collapsed;
        let pos = self.position;

        // ---- Collapse toggle ----
        let collapse_handler = cx.listener(|panel: &mut DockPanel, _: &ClickEvent, _, cx| {
            panel.collapsed = !panel.collapsed;
            cx.notify();
        });

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .px(px(spacing.sm))
            .h(px(components.dock_header_height))
            .bg(panel_bg)
            .border_b_1()
            .border_color(panel_border)
            .child(
                div()
                    .flex_1()
                    .text_size(px(typography.caption.size))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(on_neutral)
                    .child(title),
            )
            .child(
                div()
                    .id("dock-close")
                    .size(px(components.dock_action_size))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .rounded(px(2.0))
                    .hover(move |s| s.bg(hover_bg))
                    .on_click(collapse_handler)
                    .child(
                        svg()
                            .path(if collapsed {
                                "icons/panel_open.svg"
                            } else {
                                "icons/dismiss.svg"
                            })
                            .size(px(12.0))
                            .text_color(on_neutral),
                    ),
            );

        if collapsed {
            return div()
                .id("dock-panel-collapsed")
                .flex()
                .flex_col()
                .bg(panel_bg)
                .border_color(panel_border)
                .child(header)
                .on_drag_move::<DockResizeDrag>(cx.listener(|_, _, _, _| {}));
        }

        // ---- Resize handle ----
        //
        // The handle is a 4px strip at the "open" edge of the panel.
        // `on_drag` starts the drag gesture; `on_drag_move` (on the root) updates size.

        let resize_cursor = match pos {
            DockPosition::Left | DockPosition::Right => CursorStyle::ResizeLeftRight,
            DockPosition::Top | DockPosition::Bottom => CursorStyle::ResizeUpDown,
        };

        // Capture starting position when the drag begins.
        let start_handler =
            cx.listener(move |panel: &mut DockPanel, ev: &MouseDownEvent, _, _cx| {
                let coord = match pos {
                    DockPosition::Left | DockPosition::Right => f32::from(ev.position.x),
                    DockPosition::Top | DockPosition::Bottom => f32::from(ev.position.y),
                };
                panel.resize_start = Some((panel.size, coord));
            });

        // Update size as the drag moves.
        let move_handler = cx.listener(
            move |panel: &mut DockPanel, ev: &DragMoveEvent<DockResizeDrag>, _, cx| {
                if let Some((initial, start_coord)) = panel.resize_start {
                    let current = match pos {
                        DockPosition::Left | DockPosition::Right => f32::from(ev.event.position.x),
                        DockPosition::Top | DockPosition::Bottom => f32::from(ev.event.position.y),
                    };
                    let delta = match pos {
                        // Right/bottom edges: grow as mouse moves right/down
                        DockPosition::Left | DockPosition::Bottom => current - start_coord,
                        // Left/top edges: grow as mouse moves left/up (inverted)
                        DockPosition::Right | DockPosition::Top => start_coord - current,
                    };
                    panel.size = (initial + delta).clamp(80.0, 700.0);
                    cx.notify();
                }
            },
        );

        let handle = div()
            .id("dock-resize-handle")
            .cursor(resize_cursor)
            .hover(move |s| s.bg(handle_hover))
            .on_mouse_down(MouseButton::Left, start_handler)
            .on_drag(DockResizeDrag, |_, _, _, cx| cx.new(|_| DockResizeDrag));

        let (handle_w, handle_h) = match pos {
            DockPosition::Left | DockPosition::Right => (px(5.0), Default::default()),
            DockPosition::Top | DockPosition::Bottom => (Default::default(), px(5.0)),
        };
        // flex_none prevents the handle from being shrunk to 0 by flex layout
        let handle = handle.w(handle_w).h(handle_h).flex_none();

        // ---- Content ----
        let content_div = div()
            .flex()
            .flex_col()
            .flex_1()
            .min_h_0()
            .min_w_0()
            .h_full()
            .w_full()
            .overflow_hidden();
        let content_div = if let Some(v) = &self.content {
            content_div.child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .h_full()
                    .w_full()
                    .child(v.clone()),
            )
        } else {
            content_div
        };

        // ---- Panel size ----
        let (panel_w, panel_h) = match pos {
            DockPosition::Left | DockPosition::Right => (Some(px(self.size)), None),
            DockPosition::Top | DockPosition::Bottom => (None, Some(px(self.size))),
        };

        // ---- Root ----
        // Root has on_drag_move to receive move events globally during drag.
        let mut root = div()
            .id("dock-panel-root")
            .flex()
            .flex_none()
            .overflow_hidden()
            .bg(panel_bg)
            .border_color(panel_border)
            .on_drag_move::<DockResizeDrag>(move_handler);

        // Apply size
        if let Some(w) = panel_w {
            root = root.w(w);
        }
        if let Some(h) = panel_h {
            root = root.h(h);
        }

        // Arrange header, content, and resize handle based on position
        match pos {
            DockPosition::Left => root
                .flex_row()
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .min_w_0()
                        .border_r_1()
                        .border_color(panel_border)
                        .child(header)
                        .child(content_div),
                )
                .child(handle.h_full()),

            DockPosition::Right => root.flex_row().child(handle.h_full()).child(
                div()
                    .flex()
                    .flex_col()
                    .flex_1()
                    .min_w_0()
                    .child(header)
                    .child(content_div),
            ),

            DockPosition::Bottom => root
                .flex_col()
                .child(handle.w_full())
                .child(header)
                .child(content_div),

            DockPosition::Top => root
                .flex_col()
                .child(header)
                .child(content_div)
                .child(handle.w_full()),
        }
    }
}
