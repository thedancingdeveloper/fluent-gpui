//! Typed drag-and-drop contracts for actions that cross pane boundaries.
//!
//! The contract deliberately contains no application-specific payloads.  A
//! caller chooses its payload type and supplies the policy for accepting it;
//! this keeps mail, file-browser, and other clients independent while still
//! giving them the same pointer and keyboard affordances.

use std::{any::Any, fmt, sync::Arc};

use fluent_core::ThemeProvider;
use gpui::{
    prelude::*, App, Div, ElementId, Entity, FocusHandle, KeyDownEvent, Pixels, Point, Render,
    Stateful, Window,
};

/// The operation requested by a drag gesture.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum DragOperation {
    /// Move the source (the default operation).
    #[default]
    Move,
    /// Copy the source. On Windows and Linux this is requested with Ctrl.
    Copy,
}

impl DragOperation {
    /// Derive the operation from the current platform modifiers.
    pub fn from_modifiers(modifiers: gpui::Modifiers) -> Self {
        if modifiers.control || modifiers.platform {
            Self::Copy
        } else {
            Self::Move
        }
    }
}

/// A typed value offered by a [`drag_source`].
#[derive(Clone)]
pub struct DragPayload<T> {
    /// The application-owned value being dragged.
    pub value: T,
    on_cancel: Option<Arc<CancelHandler<T>>>,
}

type CancelHandler<T> = dyn Fn(&T, &mut Window, &mut App) + 'static;

impl<T> DragPayload<T> {
    /// Create a payload from an application value.
    pub fn new(value: T) -> Self {
        Self {
            value,
            on_cancel: None,
        }
    }

    fn with_cancel_handler(
        value: T,
        on_cancel: impl Fn(&T, &mut Window, &mut App) + 'static,
    ) -> Self {
        Self {
            value,
            on_cancel: Some(Arc::new(on_cancel)),
        }
    }

    fn notify_cancelled(&self, window: &mut Window, cx: &mut App) {
        if let Some(on_cancel) = &self.on_cancel {
            on_cancel(&self.value, window, cx);
        }
    }
}

impl<T: fmt::Debug> fmt::Debug for DragPayload<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("DragPayload")
            .field("value", &self.value)
            .finish_non_exhaustive()
    }
}

impl<T: PartialEq> PartialEq for DragPayload<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for DragPayload<T> {}

/// A reusable policy for deciding whether a payload may be dropped.
pub struct DropPolicy<T> {
    accepts: Arc<dyn Fn(&T) -> bool + 'static>,
}

impl<T> DropPolicy<T> {
    /// Construct a policy. Returning `false` rejects the drop and its feedback.
    pub fn new(accepts: impl Fn(&T) -> bool + 'static) -> Self {
        Self {
            accepts: Arc::new(accepts),
        }
    }

    /// Test a typed value without involving a window or a drag gesture.
    pub fn accepts(&self, value: &T) -> bool {
        (self.accepts)(value)
    }
}

/// Add a typed drag source to an existing GPUI element.
///
/// `ghost` creates the view shown below the pointer while dragging. The
/// payload remains type checked by GPUI: a target registered for another type
/// cannot accept it. `on_cancel` is called when a target accepts the payload
/// but its drop handler declines the operation.
pub fn drag_source<T, W>(
    element: Div,
    id: impl Into<ElementId>,
    payload: T,
    ghost: impl Fn(&T, Point<Pixels>, &mut Window, &mut App) -> Entity<W> + 'static,
) -> Stateful<Div>
where
    T: 'static,
    W: Render + 'static,
{
    drag_source_with_cancel(element, id, payload, ghost, |_value, _window, _cx| {})
}

/// Add a typed drag source and receive a callback when an accepted target
/// declines the drop. The callback is the place for an application to restore
/// any pending source state; the primitive never mutates application data.
pub fn drag_source_with_cancel<T, W>(
    element: Div,
    id: impl Into<ElementId>,
    payload: T,
    ghost: impl Fn(&T, Point<Pixels>, &mut Window, &mut App) -> Entity<W> + 'static,
    on_cancel: impl Fn(&T, &mut Window, &mut App) + 'static,
) -> Stateful<Div>
where
    T: 'static,
    W: Render + 'static,
{
    element
        .id(id)
        .on_drag(
            DragPayload::with_cancel_handler(payload, on_cancel),
            move |payload, offset, window, cx| ghost(&payload.value, offset, window, cx),
        )
        .aria_description("Drag to move; hold Ctrl while dragging to copy")
}

/// Add a typed drop target to an existing GPUI element.
///
/// The target highlights only while an acceptable payload is over it. A drop
/// handler returning `false` cancels the operation, allowing asynchronous or
/// otherwise unavailable destinations to reject it at the contract boundary.
/// The same handler is invoked by Enter or Space when `keyboard_payload` is
/// provided, making the pointer action available without a mouse.
pub fn drop_target<T>(
    element: Div,
    id: impl Into<ElementId>,
    focus_handle: FocusHandle,
    focus_ring: gpui::Hsla,
    policy: DropPolicy<T>,
    on_drop: impl Fn(&T, DragOperation, &mut Window, &mut App) -> bool + 'static,
    keyboard_payload: Option<T>,
) -> Stateful<Div>
where
    T: Clone + 'static,
{
    let policy_for_drop = Arc::clone(&policy.accepts);
    let policy_for_over = Arc::clone(&policy.accepts);
    let policy_for_key = Arc::clone(&policy.accepts);
    let on_drop = Arc::new(on_drop);
    let keyboard_payload = Arc::new(keyboard_payload);
    let on_drop_for_key = Arc::clone(&on_drop);
    let payload_for_key = Arc::clone(&keyboard_payload);

    element
        .id(id)
        // Keep the border in the normal style as well as in the drag-over
        // style.  Refining it only while dragging changes the measured bounds
        // for border-box layouts.
        .border_2()
        .border_color(gpui::transparent_black())
        .tab_index(0)
        .track_focus(&focus_handle)
        .focus_visible(move |style| style.border_color(focus_ring))
        .aria_label("Drop target")
        .aria_description("Press Enter or Space to perform the move")
        .aria_keyshortcuts("Enter Space")
        .drag_over::<DragPayload<T>>(move |style, payload, _, cx| {
            if policy_for_over(&payload.value) {
                style.border_color(cx.theme().colors.accent)
            } else {
                style
            }
        })
        .can_drop({
            let policy_for_can_drop = Arc::clone(&policy_for_drop);
            move |value, _, _| {
                value
                    .downcast_ref::<DragPayload<T>>()
                    .is_some_and(|payload| policy_for_can_drop(&payload.value))
            }
        })
        .on_drop(move |payload: &DragPayload<T>, window, cx| {
            if policy_for_drop(&payload.value) {
                let completed = on_drop(
                    &payload.value,
                    DragOperation::from_modifiers(window.modifiers()),
                    window,
                    cx,
                );
                if !completed {
                    payload.notify_cancelled(window, cx);
                }
            }
        })
        .on_key_down(move |event: &KeyDownEvent, window, cx| {
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                if let Some(payload) = payload_for_key.as_ref() {
                    if policy_for_key(payload) {
                        on_drop_for_key(
                            payload,
                            DragOperation::from_modifiers(window.modifiers()),
                            window,
                            cx,
                        );
                    }
                }
            }
        })
}

/// Return whether a dynamically typed GPUI drag contains the requested type.
pub fn is_payload<T: 'static>(value: &dyn Any) -> bool {
    value.downcast_ref::<DragPayload<T>>().is_some()
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::{AtomicUsize, Ordering};

    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};
    use gpui::{div, px, Context, MouseButton, TestAppContext};

    use super::*;

    struct TestGhost;

    impl Render for TestGhost {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().size(px(8.0))
        }
    }

    struct DragDropHarness {
        first_focus: FocusHandle,
        second_focus: FocusHandle,
        first_drops: Arc<AtomicUsize>,
        second_drops: Arc<AtomicUsize>,
        cancelled: Arc<AtomicUsize>,
    }

    impl Render for DragDropHarness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let first_drops = Arc::clone(&self.first_drops);
            let second_drops = Arc::clone(&self.second_drops);
            let cancelled = Arc::clone(&self.cancelled);
            let colors = &cx.theme().colors;

            div()
                .flex()
                .gap_2()
                .child(
                    drag_source_with_cancel(
                        div().size(px(40.0)).debug_selector(|| "drag-source".into()),
                        "test-drag-source",
                        7_u32,
                        |_, _, _, cx| cx.new(|_| TestGhost),
                        move |_, _, _| {
                            cancelled.fetch_add(1, Ordering::SeqCst);
                        },
                    )
                    .child("source"),
                )
                .child(
                    drop_target(
                        div()
                            .size(px(64.0))
                            .debug_selector(|| "first-target".into()),
                        "test-drop-target-one",
                        self.first_focus.clone(),
                        colors.focus_ring,
                        DropPolicy::new(|value: &u32| *value == 7),
                        move |_, _, _, _| {
                            first_drops.fetch_add(1, Ordering::SeqCst);
                            false
                        },
                        Some(7),
                    )
                    .child("first"),
                )
                .child(
                    drop_target(
                        div()
                            .size(px(64.0))
                            .debug_selector(|| "second-target".into()),
                        "test-drop-target-two",
                        self.second_focus.clone(),
                        colors.focus_ring,
                        DropPolicy::new(|value: &u32| *value == 7),
                        move |_, _, _, _| {
                            second_drops.fetch_add(1, Ordering::SeqCst);
                            true
                        },
                        Some(7),
                    )
                    .child("second"),
                )
        }
    }

    fn harness(cx: &mut Context<DragDropHarness>) -> DragDropHarness {
        DragDropHarness {
            first_focus: cx.focus_handle().tab_stop(true),
            second_focus: cx.focus_handle().tab_stop(true),
            first_drops: Arc::default(),
            second_drops: Arc::default(),
            cancelled: Arc::default(),
        }
    }

    #[test]
    fn payloads_are_typed() {
        let payload = DragPayload::new(42_u32);
        assert!(is_payload::<u32>(&payload));
        assert!(!is_payload::<String>(&payload));
    }

    #[test]
    fn policy_accepts_and_rejects_values() {
        let policy = DropPolicy::new(|value: &u32| *value > 10);
        assert!(policy.accepts(&11));
        assert!(!policy.accepts(&10));
    }

    #[test]
    fn modifiers_choose_copy_or_move() {
        assert_eq!(
            DragOperation::from_modifiers(gpui::Modifiers::default()),
            DragOperation::Move
        );
        assert_eq!(
            DragOperation::from_modifiers(gpui::Modifiers {
                control: true,
                ..Default::default()
            }),
            DragOperation::Copy
        );
    }

    #[gpui::test]
    fn distinct_targets_receive_keyboard_drops(cx: &mut TestAppContext) {
        let (view, cx) = open_fluent_test_window(cx, |_, cx| harness(cx));
        let (first, second, first_focus, second_focus) = view.read_with(cx, |view, _| {
            (
                Arc::clone(&view.first_drops),
                Arc::clone(&view.second_drops),
                view.first_focus.clone(),
                view.second_focus.clone(),
            )
        });

        cx.update(|window, app| window.focus(&first_focus, app));
        cx.type_keys("enter");
        cx.update(|window, app| window.focus(&second_focus, app));
        cx.type_keys("enter");

        assert_eq!(first.load(Ordering::SeqCst), 1);
        assert_eq!(second.load(Ordering::SeqCst), 1);
    }

    #[gpui::test]
    fn declined_pointer_drop_notifies_source_and_keeps_bounds(cx: &mut TestAppContext) {
        let (view, cx) = open_fluent_test_window(cx, |_, cx| harness(cx));
        let (drops, cancelled) = view.read_with(cx, |view, _| {
            (Arc::clone(&view.first_drops), Arc::clone(&view.cancelled))
        });
        let source = cx.selector_bounds("drag-source").unwrap();
        let before = cx.selector_bounds("first-target").unwrap();

        cx.simulate_mouse_down(source.center(), MouseButton::Left, gpui::Modifiers::none());
        cx.simulate_mouse_move(
            before.center(),
            Some(MouseButton::Left),
            gpui::Modifiers::none(),
        );
        cx.run_until_parked();
        let during = cx.selector_bounds("first-target").unwrap();
        cx.simulate_mouse_up(before.center(), MouseButton::Left, gpui::Modifiers::none());
        cx.run_until_parked();

        assert_eq!(during, before, "drag-over feedback must not shift layout");
        assert_eq!(drops.load(Ordering::SeqCst), 1);
        assert_eq!(cancelled.load(Ordering::SeqCst), 1);
    }
}
