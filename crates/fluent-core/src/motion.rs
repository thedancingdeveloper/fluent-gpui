use std::time::Duration;

use gpui::{
    div, ease_out_quint, prelude::*, px, Animation, AnimationExt as _, AnyElement, ElementId,
    IntoElement,
};

/// Motion timing defaults for lightweight Fluent-style UI transitions.
///
/// These values are intentionally modest: popup surfaces should feel responsive
/// and should not draw attention away from the command being performed.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTokens {
    pub popup_enter_duration_ms: u64,
    pub popup_exit_duration_ms: u64,
    pub popup_offset: f32,
    pub popup_scale_delta: f32,
}

impl MotionTokens {
    pub fn popup_enter_duration(self) -> Duration {
        Duration::from_millis(self.popup_enter_duration_ms)
    }

    pub fn popup_exit_duration(self) -> Duration {
        Duration::from_millis(self.popup_exit_duration_ms)
    }
}

impl Default for MotionTokens {
    fn default() -> Self {
        Self {
            popup_enter_duration_ms: 160,
            popup_exit_duration_ms: 160,
            popup_offset: 2.0,
            popup_scale_delta: 0.02,
        }
    }
}

pub fn popup_motion_surface(
    id: impl Into<ElementId>,
    exiting: bool,
    motion: MotionTokens,
    child: impl IntoElement + 'static,
) -> AnyElement {
    let duration = if exiting {
        motion.popup_exit_duration()
    } else {
        motion.popup_enter_duration()
    };
    let offset = motion.popup_offset;

    div()
        .child(child)
        .with_animation(
            id,
            Animation::new(duration).with_easing(ease_out_quint()),
            move |surface, delta| {
                let opacity = if exiting { 1.0 - delta } else { delta }.clamp(0.0, 1.0);
                let y_offset = if exiting {
                    offset * delta
                } else {
                    offset * (1.0 - delta)
                };
                surface.opacity(opacity).mt(px(y_offset))
            },
        )
        .into_any_element()
}
