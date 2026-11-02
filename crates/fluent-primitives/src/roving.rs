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
