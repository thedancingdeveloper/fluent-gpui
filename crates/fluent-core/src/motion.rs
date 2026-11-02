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
pub struct MotionCurve {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl MotionCurve {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MotionTokens {
    pub accelerate: MotionCurve,
    pub decelerate: MotionCurve,
    pub easy_ease: MotionCurve,
    pub ultra_fast_duration_ms: u64,
    pub fast_duration_ms: u64,
    pub normal_duration_ms: u64,
    pub slow_duration_ms: u64,
    pub ultra_slow_duration_ms: u64,
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

    pub fn ultra_fast_duration(self) -> Duration {
        Duration::from_millis(self.ultra_fast_duration_ms)
    }

    pub fn fast_duration(self) -> Duration {
        Duration::from_millis(self.fast_duration_ms)
    }

    pub fn normal_duration(self) -> Duration {
        Duration::from_millis(self.normal_duration_ms)
    }

    pub fn slow_duration(self) -> Duration {
        Duration::from_millis(self.slow_duration_ms)
    }

    pub fn ultra_slow_duration(self) -> Duration {
        Duration::from_millis(self.ultra_slow_duration_ms)
    }
}

impl Default for MotionTokens {
    fn default() -> Self {
        Self {
            accelerate: MotionCurve::new(0.9, 0.1, 1.0, 0.2),
            decelerate: MotionCurve::new(0.1, 0.9, 0.2, 1.0),
            easy_ease: MotionCurve::new(0.33, 0.0, 0.67, 1.0),
            ultra_fast_duration_ms: 50,
            fast_duration_ms: 100,
            normal_duration_ms: 200,
            slow_duration_ms: 300,
            ultra_slow_duration_ms: 500,
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
