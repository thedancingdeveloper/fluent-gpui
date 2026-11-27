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
    /// Public API item.
    pub x1: f32,
    /// Public API item.
    pub y1: f32,
    /// Public API item.
    pub x2: f32,
    /// Public API item.
    pub y2: f32,
}

impl MotionCurve {
    /// Public API item.
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
/// Public API item.
pub struct MotionTokens {
    /// Public API item.
    pub accelerate: MotionCurve,
    /// Public API item.
    pub decelerate: MotionCurve,
    /// Public API item.
    pub easy_ease: MotionCurve,
    /// Public API item.
    pub ultra_fast_duration_ms: u64,
    /// Public API item.
    pub fast_duration_ms: u64,
    /// Public API item.
    pub normal_duration_ms: u64,
    /// Public API item.
    pub slow_duration_ms: u64,
    /// Public API item.
    pub ultra_slow_duration_ms: u64,
    /// Public API item.
    pub popup_enter_duration_ms: u64,
    /// Public API item.
    pub popup_exit_duration_ms: u64,
    /// Public API item.
    pub spinner_loop_ms: u64,
    /// Public API item.
    pub pulse_loop_ms: u64,
    /// Public API item.
    pub skeleton_loop_ms: u64,
    /// Public API item.
    pub progress_loop_ms: u64,
    /// Public API item.
    pub popup_offset: f32,
    /// Public API item.
    pub toast_offset: f32,
    /// Public API item.
    pub popup_scale_delta: f32,
}

impl MotionTokens {
    /// Public API item.
    pub fn popup_enter_duration(self) -> Duration {
        Duration::from_millis(self.popup_enter_duration_ms)
    }

    /// Public API item.
    pub fn popup_exit_duration(self) -> Duration {
        Duration::from_millis(self.popup_exit_duration_ms)
    }

    /// Duration of one spinner revolution.
    pub fn spinner_loop(self) -> Duration {
        Duration::from_millis(self.spinner_loop_ms)
    }

    /// Duration of one status-dot pulse.
    pub fn pulse_loop(self) -> Duration {
        Duration::from_millis(self.pulse_loop_ms)
    }

    /// Duration of one skeleton shimmer cycle.
    pub fn skeleton_loop(self) -> Duration {
        Duration::from_millis(self.skeleton_loop_ms)
    }

    /// Duration of one indeterminate progress sweep.
    pub fn progress_loop(self) -> Duration {
        Duration::from_millis(self.progress_loop_ms)
    }

    /// Public API item.
    pub fn ultra_fast_duration(self) -> Duration {
        Duration::from_millis(self.ultra_fast_duration_ms)
    }

    /// Public API item.
    pub fn fast_duration(self) -> Duration {
        Duration::from_millis(self.fast_duration_ms)
    }

    /// Public API item.
    pub fn normal_duration(self) -> Duration {
        Duration::from_millis(self.normal_duration_ms)
    }

    /// Public API item.
    pub fn slow_duration(self) -> Duration {
        Duration::from_millis(self.slow_duration_ms)
    }

    /// Public API item.
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
            normal_duration_ms: 167,
            slow_duration_ms: 300,
            ultra_slow_duration_ms: 500,
            popup_enter_duration_ms: 160,
            popup_exit_duration_ms: 160,
            spinner_loop_ms: 900,
            pulse_loop_ms: 1600,
            skeleton_loop_ms: 1500,
            progress_loop_ms: 1280,
            popup_offset: 2.0,
            toast_offset: 12.0,
            popup_scale_delta: 0.02,
        }
    }
}

/// Public API item.
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
                // Offset with `top`, not a margin: `anchored` ignores margins when it
                // measures and snaps its child, so a margin would paint outside the
                // snapped bounds for the length of the animation.
                surface.opacity(opacity).relative().top(px(y_offset))
            },
        )
        .into_any_element()
}

/// Slides a toast in from `offset` px below its resting place while fading it in.
pub fn toast_motion_surface(
    id: impl Into<ElementId>,
    motion: MotionTokens,
    child: impl IntoElement + 'static,
) -> AnyElement {
    let offset = motion.toast_offset;
    div()
        .child(child)
        .with_animation(
            id,
            Animation::new(motion.normal_duration()).with_easing(ease_out_quint()),
            move |surface, delta| {
                surface
                    .opacity(delta.clamp(0.0, 1.0))
                    .relative()
                    .top(px(offset * (1.0 - delta)))
            },
        )
        .into_any_element()
}

/// What [`track_value_change`] found for one render.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ValueChange {
    /// Where the value was when the change began (the on-screen value if a
    /// previous tween was still running).
    pub from: f32,
    /// The value now.
    pub to: f32,
    /// Zero until the value has changed after first render; then counts the
    /// changes. Put it in the animation id so each change starts a fresh tween.
    pub generation: u64,
}

struct TrackedValue {
    from: f32,
    to: f32,
    generation: u64,
    started: std::time::Instant,
    duration: Duration,
}

/// Remembers `value` under the stable `id` and reports real changes.
///
/// The first render reports `generation == 0` with `from == to`, so callers draw
/// the value with no animation. A later render with a different value starts a
/// new generation whose `from` is the value on screen at that moment.
pub fn track_value_change(
    id: impl Into<ElementId>,
    value: f32,
    duration: Duration,
    window: &mut gpui::Window,
    cx: &mut gpui::App,
) -> ValueChange {
    let now = cx.background_executor().now();
    let state = window.use_keyed_state(id, cx, |_, _| TrackedValue {
        from: value,
        to: value,
        generation: 0,
        started: now,
        duration,
    });
    state.update(cx, |s, _| {
        if s.to != value {
            let t = if s.duration.is_zero() {
                1.0
            } else {
                (now.saturating_duration_since(s.started).as_secs_f32() / s.duration.as_secs_f32())
                    .clamp(0.0, 1.0)
            };
            s.from = s.from + (s.to - s.from) * ease_out_quint()(t);
            s.to = value;
            s.generation += 1;
            s.started = now;
            s.duration = duration;
        }
        ValueChange {
            from: s.from,
            to: s.to,
            generation: s.generation,
        }
    })
}
