//! Typed drag-and-drop contracts for actions that cross pane boundaries.
//!
//! The contract deliberately contains no application-specific payloads.  A
//! caller chooses its payload type and supplies the policy for accepting it;
//! this keeps mail, file-browser, and other clients independent while still
//! giving them the same pointer and keyboard affordances.

use std::{any::Any, sync::Arc};

use gpui::{prelude::*, App, Div, Entity, KeyDownEvent, Pixels, Point, Render, Stateful, Window};

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
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DragPayload<T> {
    /// The application-owned value being dragged.
    pub value: T,
}

impl<T> DragPayload<T> {
    /// Create a payload from an application value.
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

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
/// cannot accept it.
pub fn drag_source<T, W>(
    element: Div,
    payload: T,
    ghost: impl Fn(&T, Point<Pixels>, &mut Window, &mut App) -> Entity<W> + 'static,
) -> Stateful<Div>
where
    T: 'static,
    W: Render + 'static,
{
    element
        .id("fluent-drag-source")
        .on_drag(
            DragPayload::new(payload),
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
        .id("fluent-drop-target")
        .aria_label("Drop target")
        .aria_description("Press Enter or Space to perform the move")
        .aria_keyshortcuts("Enter Space")
        .drag_over::<DragPayload<T>>(move |style, payload, _, _| {
            if policy_for_over(&payload.value) {
                style.border_2().border_color(gpui::rgb(0x0078d4))
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
                let _ = on_drop(
                    &payload.value,
                    DragOperation::from_modifiers(window.modifiers()),
                    window,
                    cx,
                );
            }
        })
        .on_key_down(move |event: &KeyDownEvent, window, cx| {
            if matches!(event.keystroke.key.as_str(), "enter" | "space") {
                if let Some(payload) = payload_for_key.as_ref() {
                    if policy_for_key(payload) {
                        let _ = on_drop_for_key(
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
    use super::*;

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
}
