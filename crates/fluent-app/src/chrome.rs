use gpui::{
    canvas, div, point, prelude::*, px, App, Bounds, Context, CursorStyle, Decorations, Entity,
    HitboxBehavior, IntoElement, MouseButton, Pixels, Point, Render, ResizeEdge, Size, Window,
};

/// Width of the invisible edge zone that captures resize gestures when the
/// window is rendered with client-side decorations.
const RESIZE_INSET: Pixels = px(4.0);

/// Wraps a user's window content with client-decoration helpers:
///
/// - 8 resize edges/corners around the perimeter that change the cursor and
///   trigger `start_window_resize` on press.
/// - The wrapped content is rendered untouched inside — interactive children
///   still receive their own mouse events as normal.
///
/// `FluentApp::run` installs this automatically when
/// `WindowDecorations::Client` is in use, so consumers don't need to opt in.
pub struct WindowChrome<V: Render + 'static> {
    content: Entity<V>,
}

impl<V: Render + 'static> WindowChrome<V> {
    pub fn new(content: Entity<V>) -> Self {
        Self { content }
    }
}

impl<V: Render + 'static> Render for WindowChrome<V> {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let decorations = window.window_decorations();
        let content = self.content.clone();

        div().size_full().child(content).map(|d| match decorations {
            Decorations::Server => d,
            Decorations::Client { .. } => d
                // Edge hitbox: only sets the cursor style when hovering an edge.
                // The on_mouse_down handler below is what actually starts the resize.
                .child(
                    canvas(
                        |_bounds, window, _cx| {
                            window.insert_hitbox(
                                Bounds::new(
                                    point(px(0.0), px(0.0)),
                                    window.window_bounds().get_bounds().size,
                                ),
                                HitboxBehavior::Normal,
                            )
                        },
                        move |_bounds, hitbox, window, _cx| {
                            let mouse = window.mouse_position();
                            let size = window.window_bounds().get_bounds().size;
                            let Some(edge) = resize_edge_at(mouse, RESIZE_INSET, size) else {
                                return;
                            };
                            window.set_cursor_style(cursor_for_edge(edge), &hitbox);
                        },
                    )
                    .size_full()
                    .absolute(),
                )
                .on_mouse_move(|_, window, _| window.refresh())
                .on_mouse_down(MouseButton::Left, move |e, window, _| {
                    let size = window.window_bounds().get_bounds().size;
                    if let Some(edge) = resize_edge_at(e.position, RESIZE_INSET, size) {
                        window.start_window_resize(edge);
                    }
                }),
        })
    }
}

/// Construct a chrome wrapper entity for `content`. Used by `FluentApp::run`.
pub fn wrap_with_chrome<V: Render + 'static>(
    content: Entity<V>,
    cx: &mut App,
) -> Entity<WindowChrome<V>> {
    cx.new(|_| WindowChrome::new(content))
}

fn resize_edge_at(pos: Point<Pixels>, inset: Pixels, size: Size<Pixels>) -> Option<ResizeEdge> {
    let top = pos.y < inset;
    let bottom = pos.y > size.height - inset;
    let left = pos.x < inset;
    let right = pos.x > size.width - inset;
    Some(match (top, bottom, left, right) {
        (true, _, true, _) => ResizeEdge::TopLeft,
        (true, _, _, true) => ResizeEdge::TopRight,
        (_, true, true, _) => ResizeEdge::BottomLeft,
        (_, true, _, true) => ResizeEdge::BottomRight,
        (true, _, _, _) => ResizeEdge::Top,
        (_, true, _, _) => ResizeEdge::Bottom,
        (_, _, true, _) => ResizeEdge::Left,
        (_, _, _, true) => ResizeEdge::Right,
        _ => return None,
    })
}

fn cursor_for_edge(edge: ResizeEdge) -> CursorStyle {
    match edge {
        ResizeEdge::Top | ResizeEdge::Bottom => CursorStyle::ResizeUpDown,
        ResizeEdge::Left | ResizeEdge::Right => CursorStyle::ResizeLeftRight,
        ResizeEdge::TopLeft | ResizeEdge::BottomRight => CursorStyle::ResizeUpLeftDownRight,
        ResizeEdge::TopRight | ResizeEdge::BottomLeft => CursorStyle::ResizeUpRightDownLeft,
    }
}
