//! Roving-focus helpers: a group is one tab stop and arrows move within it.

use gpui::{App, ElementId, FocusHandle, Window};

/// Focus handles for a group of `n` items, kept across frames under `id`.
pub(crate) fn handles(
    window: &mut Window,
    cx: &mut App,
    id: &ElementId,
    n: usize,
) -> Vec<FocusHandle> {
    let state =
        window.use_keyed_state((id.clone(), "roving"), cx, |_, _| Vec::<FocusHandle>::new());
    state.update(cx, |v, cx| {
        while v.len() < n {
            v.push(cx.focus_handle());
        }
        v.truncate(n);
        v.clone()
    })
}

/// The `FocusHandle` of a single-stop composite (list, tree) rendered by a
/// stateless `RenderOnce` element, kept across frames under `id` and set up as
/// a Tab stop at `tab_index` 0. The whole composite is one stop; its active
/// item is tracked by the caller's selection, not by per-item focus.
pub(crate) fn container_handle(window: &mut Window, cx: &mut App, id: &ElementId) -> FocusHandle {
    let state = window.use_keyed_state((id.clone(), "fluent-focus"), cx, |_, cx| cx.focus_handle());
    let handle = state.read(cx).clone();
    handle.tab_index(0).tab_stop(true)
}

/// The item that is the group's tab stop: the focused item, else `default`
/// (when enabled), else the first enabled item.
pub(crate) fn stop_index(
    window: &Window,
    handles: &[FocusHandle],
    enabled: &[bool],
    default: Option<usize>,
) -> Option<usize> {
    handles
        .iter()
        .position(|h| h.is_focused(window))
        .or_else(|| default.filter(|&i| enabled.get(i).copied().unwrap_or(false)))
        .or_else(|| enabled.iter().position(|&e| e))
}

/// The item a navigation key moves to from `current`, skipping disabled items.
/// Arrows wrap; Home/End jump to the first/last enabled item.
pub(crate) fn target(key: &str, current: usize, enabled: &[bool]) -> Option<usize> {
    let n = enabled.len();
    if n == 0 {
        return None;
    }
    let step = |forward: bool| {
        (1..n)
            .map(|d| {
                if forward {
                    (current + d) % n
                } else {
                    (current + n - d) % n
                }
            })
            .find(|&i| enabled[i])
    };
    match key {
        "right" | "down" => step(true),
        "left" | "up" => step(false),
        "home" => enabled.iter().position(|&e| e),
        "end" => enabled.iter().rposition(|&e| e),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::target;

    #[test]
    fn arrows_wrap_and_skip_disabled() {
        let e = [true, false, true];
        assert_eq!(target("right", 0, &e), Some(2));
        assert_eq!(target("right", 2, &e), Some(0));
        assert_eq!(target("left", 0, &e), Some(2));
        assert_eq!(target("home", 2, &e), Some(0));
        assert_eq!(target("end", 0, &e), Some(2));
        assert_eq!(target("a", 0, &e), None);
    }
}

/// Shared helpers for the keyboard-focus tests of the composite controls.
#[cfg(test)]
pub(crate) mod test_support {
    use fluent_core::ThemeProvider as _;
    use fluent_test_utils::FluentInteractionExt as _;
    use gpui::VisualTestContext;

    /// Presses Tab `stops` times from a fresh window, ending in keyboard modality.
    pub(crate) fn tab(cx: &mut VisualTestContext, stops: usize) {
        cx.update(|_, app| fluent_primitives::install_focus_keybindings(app));
        cx.update(|window, app| window.draw(app).clear(app));
        // Nothing is focused yet, so the first stop comes from the window; the
        // rest are real Tab presses.
        cx.update(|window, app| window.focus_next(app));
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
        for _ in 1..stops {
            cx.type_keys("tab");
            cx.update(|window, app| window.draw(app).clear(app));
        }
    }

    /// Presses `key` and releases it, as a button needs the key-up to activate.
    pub(crate) fn press(cx: &mut VisualTestContext, key: &'static str) {
        use gpui::InputEvent as _;
        cx.type_keys(key);
        let keystroke = gpui::Keystroke::parse(key).expect("valid test keystroke");
        cx.update(|window, app| {
            window.dispatch_event(
                gpui::KeyUpEvent {
                    keystroke,
                    scan_code: None,
                }
                .to_platform_input(),
                app,
            );
        });
        cx.run_until_parked();
        cx.update(|window, app| window.draw(app).clear(app));
    }

    /// Outer focus-ring quads (as `"bounds vs clip"` text) that an
    /// `overflow_hidden` ancestor cuts off. gpui splits a hollow bordered quad
    /// into edge strips, each with its own content mask, so a ring's clip is the
    /// bounding box of the masks of the quads that share its bounds; the ring is
    /// whole when that box covers its bounds. Empty when every ring is whole.
    pub(crate) fn clipped_ring_quads(cx: &mut VisualTestContext) -> Vec<String> {
        cx.update(|window, app| window.draw(app).clear(app));
        cx.update(|window, app| {
            let outer = app.theme().colors.stroke_focus_outer;
            let quads = window.painted_quads();
            let rings: Vec<_> = quads.iter().filter(|q| q.border_color == outer).collect();
            let mut clipped = Vec::new();
            for ring in &rings {
                let (mut l, mut t, mut r, mut b) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
                for q in rings.iter().filter(|q| q.bounds == ring.bounds) {
                    let m = q.content_mask.bounds;
                    l = l.min(m.origin.x.0);
                    t = t.min(m.origin.y.0);
                    r = r.max(m.right().0);
                    b = b.max(m.bottom().0);
                }
                let bb = ring.bounds;
                if bb.origin.x.0 < l || bb.origin.y.0 < t || bb.right().0 > r || bb.bottom().0 > b {
                    clipped.push(format!("{:?} vs clip ({l},{t})-({r},{b})", bb));
                }
            }
            clipped.sort();
            clipped.dedup();
            clipped
        })
    }

    /// Number of distinct painted rings in the Fluent 2 outer focus-ring colour.
    pub(crate) fn outer_ring_quads(cx: &mut VisualTestContext) -> usize {
        cx.update(|window, app| window.draw(app).clear(app));
        cx.update(|window, app| {
            let outer = app.theme().colors.stroke_focus_outer;
            let mut rings: Vec<String> = window
                .painted_quads()
                .iter()
                .filter(|q| q.border_color == outer)
                .map(|q| format!("{:?}", q.bounds))
                .collect();
            rings.sort();
            rings.dedup();
            rings.len()
        })
    }
}
