use fluent_core::{LayoutDirection, Theme};

pub use gpui::{Context, Entity, FocusHandle, TestAppContext, VisualTestContext, Window};

use gpui::{Bounds, Modifiers, MouseButton, Pixels, Point};

pub fn init_fluent_test(cx: &mut TestAppContext) {
    cx.update(Theme::init);
}

pub fn open_fluent_test_window<V: gpui::Render>(
    cx: &mut TestAppContext,
    build: impl FnOnce(&mut Window, &mut Context<V>) -> V,
) -> (Entity<V>, &mut VisualTestContext) {
    init_fluent_test(cx);
    cx.add_window_view(build)
}

/// Interaction helpers for a [`VisualTestContext`]: type keys, click an
/// element by debug selector and assert which element holds focus.
///
/// Elements are addressed by the name given to
/// `InteractiveElement::debug_selector`, which is only recorded in builds with
/// `debug_assertions` (the default for tests).
pub trait FluentInteractionExt {
    /// Renders the window with accessibility active and returns a stable,
    /// compact list of each AccessKit node's role, accessible name and state.
    /// Bounds, node ids and debug provenance are deliberately omitted.
    fn accessibility_snapshot(&mut self) -> Vec<String>;

    /// Types a space-separated list of keystrokes (`"tab"`, `"shift-tab"`,
    /// `"enter"`, `"a"`) and runs the executor until parked.
    ///
    /// ```
    /// use fluent_test_utils::*;
    /// struct Empty;
    /// impl gpui::Render for Empty {
    ///     fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
    ///         gpui::div()
    ///     }
    /// }
    /// let mut cx = TestAppContext::single();
    /// let (_view, cx) = open_fluent_test_window(&mut cx, |_, _| Empty);
    /// cx.type_keys("tab shift-tab");
    /// ```
    fn type_keys(&mut self, keystrokes: &str);

    /// Types literal text, one character per keystroke.
    fn type_text(&mut self, text: &str);

    /// Bounds of the element with the given debug selector, or `None` if no
    /// such element was painted in the last frame.
    fn selector_bounds(&mut self, selector: &'static str) -> Option<Bounds<Pixels>>;

    /// Left-clicks the centre of the element with the given debug selector.
    ///
    /// # Panics
    /// Panics if no element with that selector was painted.
    ///
    /// ```
    /// use fluent_test_utils::*;
    /// use gpui::{InteractiveElement, ParentElement, Styled, px};
    /// struct Button;
    /// impl gpui::Render for Button {
    ///     fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
    ///         gpui::div().size(px(40.)).debug_selector(|| "button".into())
    ///     }
    /// }
    /// let mut cx = TestAppContext::single();
    /// let (_view, cx) = open_fluent_test_window(&mut cx, |_, _| Button);
    /// cx.click_selector("button");
    /// ```
    fn click_selector(&mut self, selector: &'static str);

    /// Clicks at a window position with the given button and modifiers.
    fn click_at(&mut self, position: Point<Pixels>, button: MouseButton, modifiers: Modifiers);

    /// Whether `handle` is the focused element of the window.
    fn is_focused(&mut self, handle: &FocusHandle) -> bool;

    /// Asserts that `handle` holds focus.
    ///
    /// ```
    /// use fluent_test_utils::*;
    /// use gpui::{InteractiveElement, ParentElement};
    /// struct Field(FocusHandle);
    /// impl gpui::Render for Field {
    ///     fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl gpui::IntoElement {
    ///         gpui::div().track_focus(&self.0)
    ///     }
    /// }
    /// let mut cx = TestAppContext::single();
    /// let (view, cx) = open_fluent_test_window(&mut cx, |_, cx| Field(cx.focus_handle()));
    /// let handle = view.read_with(cx, |v, _| v.0.clone());
    /// cx.update(|window, cx| window.focus(&handle, cx));
    /// cx.assert_focused(&handle);
    /// ```
    fn assert_focused(&mut self, handle: &FocusHandle);

    /// Asserts that `handle` does not hold focus.
    fn assert_not_focused(&mut self, handle: &FocusHandle);

    /// Asserts that nothing in the window holds focus.
    fn assert_nothing_focused(&mut self);
}

impl FluentInteractionExt for VisualTestContext {
    fn accessibility_snapshot(&mut self) -> Vec<String> {
        self.activate_accessibility();
        self.run_until_parked();
        let json = self.update(|window, cx| {
            window.draw(cx).clear(cx);
            window.debug_a11y_tree_json()
        });
        let tree: serde_json::Value = serde_json::from_str(
            json.as_deref()
                .expect("accessibility tree was not captured after rendering"),
        )
        .expect("GPUI accessibility debug output was not valid JSON");

        let nodes = tree["nodes"]
            .as_object()
            .expect("accessibility debug output did not contain nodes");
        let root = tree["root"]
            .as_str()
            .expect("accessibility debug output did not identify its root");
        let mut snapshot = Vec::with_capacity(nodes.len());
        append_accessibility_node(root, nodes, &mut snapshot);
        snapshot
    }

    fn type_keys(&mut self, keystrokes: &str) {
        self.simulate_keystrokes(keystrokes);
    }

    fn type_text(&mut self, text: &str) {
        self.simulate_input(text);
    }

    fn selector_bounds(&mut self, selector: &'static str) -> Option<Bounds<Pixels>> {
        self.debug_bounds(selector)
    }

    fn click_selector(&mut self, selector: &'static str) {
        let bounds = self
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("no element with debug selector {selector:?} was painted"));
        self.click_at(bounds.center(), MouseButton::Left, Modifiers::none());
    }

    fn click_at(&mut self, position: Point<Pixels>, button: MouseButton, modifiers: Modifiers) {
        self.simulate_mouse_down(position, button, modifiers);
        self.simulate_mouse_up(position, button, modifiers);
        self.run_until_parked();
    }

    fn is_focused(&mut self, handle: &FocusHandle) -> bool {
        self.update(|window, _| handle.is_focused(window))
    }

    fn assert_focused(&mut self, handle: &FocusHandle) {
        assert!(
            self.is_focused(handle),
            "expected the given focus handle to hold focus"
        );
    }

    fn assert_not_focused(&mut self, handle: &FocusHandle) {
        assert!(
            !self.is_focused(handle),
            "expected the given focus handle not to hold focus"
        );
    }

    fn assert_nothing_focused(&mut self) {
        let focused = self.update(|window, cx| window.focused(cx).is_some());
        assert!(!focused, "expected no element to hold focus");
    }
}

fn append_accessibility_node(
    id: &str,
    nodes: &serde_json::Map<String, serde_json::Value>,
    snapshot: &mut Vec<String>,
) {
    let Some(node) = nodes.get(id).and_then(serde_json::Value::as_object) else {
        return;
    };
    let Some(aria) = node.get("aria").and_then(serde_json::Value::as_object) else {
        return;
    };

    let role = aria
        .get("role")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Unknown");
    let mut line = role.to_string();
    if let Some(name) = aria.get("label").and_then(serde_json::Value::as_str) {
        line.push_str(" \"");
        line.push_str(name);
        line.push('"');
    }

    let states = aria
        .iter()
        .filter(|(key, _)| {
            !matches!(
                key.as_str(),
                "role" | "label" | "description" | "value" | "on_action"
            )
        })
        .map(|(key, value)| format!("{key}={}", compact_json(value)))
        .collect::<Vec<_>>();
    if !states.is_empty() {
        line.push_str(" [");
        line.push_str(&states.join(", "));
        line.push(']');
    }
    snapshot.push(line);

    if let Some(children) = node.get("children").and_then(serde_json::Value::as_array) {
        for child in children.iter().filter_map(serde_json::Value::as_str) {
            append_accessibility_node(child, nodes, snapshot);
        }
    }
}

fn compact_json(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

/// Asserts that an [`FluentInteractionExt::accessibility_snapshot`] contains a
/// node with the given role, accessible name and states.
///
/// `name` of `None` matches any name (including none). Every entry of
/// `states` (for example `"toggled=True"`) must appear in the node's state
/// list; extra states are allowed, so the helper never needs updating when a
/// component gains an unrelated state.
///
/// ```
/// use fluent_test_utils::assert_a11y_node;
/// let snap = vec!["Window".to_string(), "Switch \"Wi-Fi\" [toggled=True]".to_string()];
/// assert_a11y_node(&snap, "Switch", Some("Wi-Fi"), &["toggled=True"]);
/// ```
///
/// # Panics
/// Panics, printing the whole snapshot, if no node matches.
#[track_caller]
pub fn assert_a11y_node(snapshot: &[String], role: &str, name: Option<&str>, states: &[&str]) {
    let found = snapshot.iter().any(|line| {
        let (head, state_list) = match line.split_once(" [") {
            Some((head, rest)) => (head, rest.trim_end_matches(']')),
            None => (line.as_str(), ""),
        };
        let (line_role, line_name) = match head.split_once(" \"") {
            Some((role, rest)) => (role, Some(rest.trim_end_matches('"'))),
            None => (head, None),
        };
        line_role == role
            && name.is_none_or(|n| line_name == Some(n))
            && states
                .iter()
                .all(|wanted| state_list.split(", ").any(|have| have == *wanted))
    });
    assert!(
        found,
        "no accessibility node with role {role:?}, name {name:?}, states {states:?} in {snapshot:#?}"
    );
}

/// Asserts that no node with the given role and name is exposed.
#[track_caller]
pub fn assert_no_a11y_node(snapshot: &[String], role: &str, name: Option<&str>) {
    let prefix = match name {
        Some(n) => format!("{role} \"{n}\""),
        None => role.to_string(),
    };
    assert!(
        !snapshot
            .iter()
            .any(|line| line == &prefix || line.starts_with(&format!("{prefix} ["))),
        "unexpected accessibility node {prefix:?} in {snapshot:#?}"
    );
}

/// WCAG relative luminance of a colour composited over `backdrop`.
fn luminance_over(color: gpui::Hsla, backdrop: gpui::Hsla) -> f32 {
    let (c, b) = (color.to_rgb(), backdrop.to_rgb());
    let mix = |fg: f32, bg: f32| fg * c.a + bg * (1.0 - c.a);
    let lin = |v: f32| {
        if v <= 0.03928 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(mix(c.r, b.r)) + 0.7152 * lin(mix(c.g, b.g)) + 0.0722 * lin(mix(c.b, b.b))
}

/// WCAG 2.x contrast ratio (1.0 to 21.0) of `foreground` on an opaque
/// `background`; a translucent foreground is composited over it first.
pub fn contrast_ratio(foreground: gpui::Hsla, background: gpui::Hsla) -> f32 {
    let fg = luminance_over(foreground, background);
    let bg = luminance_over(background, gpui::black());
    let (hi, lo) = if fg > bg { (fg, bg) } else { (bg, fg) };
    (hi + 0.05) / (lo + 0.05)
}

/// Asserts `foreground` on `background` reaches `minimum` (4.5 for body text,
/// 3.0 for large text and UI components under WCAG AA).
///
/// ```
/// use fluent_test_utils::assert_contrast;
/// assert_contrast("text", gpui::black(), gpui::white(), 4.5);
/// ```
#[track_caller]
pub fn assert_contrast(label: &str, foreground: gpui::Hsla, background: gpui::Hsla, minimum: f32) {
    let ratio = contrast_ratio(foreground, background);
    assert!(
        ratio >= minimum,
        "{label}: contrast {ratio:.2}:1 is below the required {minimum}:1"
    );
}

/// Semantic, focus, keyboard and directionality assertions for a
/// [`VisualTestContext`].
pub trait FluentAssertExt {
    /// Asserts the rendered accessibility tree has a node with this role,
    /// name and states (see [`assert_a11y_node`]).
    fn assert_a11y(&mut self, role: &str, name: Option<&str>, states: &[&str]);

    /// Asserts that pressing `tab` visits exactly `order` (starting from the
    /// first handle, which is focused first) and that `shift-tab` walks it back.
    fn assert_tab_order(&mut self, order: &[&FocusHandle]);

    /// Types `keys`, then asserts `observe` returns `expected`. Use it to pin
    /// a keyboard outcome (state change, value, selection) in one line.
    fn assert_keys_yield<T: PartialEq + std::fmt::Debug>(
        &mut self,
        keys: &str,
        observe: impl FnOnce(&mut Self) -> T,
        expected: T,
    );

    /// Sets the application layout direction and lets the window re-render.
    fn set_layout_direction(&mut self, direction: LayoutDirection);

    /// Asserts the element `leading` sits on the start side of `trailing`:
    /// left of it in LTR, right of it in RTL. Both are debug selectors.
    fn assert_inline_order(&mut self, leading: &'static str, trailing: &'static str);
}

impl FluentAssertExt for VisualTestContext {
    #[track_caller]
    fn assert_a11y(&mut self, role: &str, name: Option<&str>, states: &[&str]) {
        let snapshot = self.accessibility_snapshot();
        assert_a11y_node(&snapshot, role, name, states);
    }

    #[track_caller]
    fn assert_tab_order(&mut self, order: &[&FocusHandle]) {
        let Some(first) = order.first() else { return };
        self.update(|window, cx| window.focus(first, cx));
        self.run_until_parked();
        for (i, handle) in order.iter().enumerate() {
            assert!(
                self.is_focused(handle),
                "tab order: expected stop {i} of {} to hold focus",
                order.len()
            );
            if i + 1 < order.len() {
                self.type_keys("tab");
            }
        }
        for (i, handle) in order.iter().enumerate().rev().skip(1) {
            self.type_keys("shift-tab");
            assert!(
                self.is_focused(handle),
                "reverse tab order: expected stop {i} of {} to hold focus",
                order.len()
            );
        }
    }

    #[track_caller]
    fn assert_keys_yield<T: PartialEq + std::fmt::Debug>(
        &mut self,
        keys: &str,
        observe: impl FnOnce(&mut Self) -> T,
        expected: T,
    ) {
        self.type_keys(keys);
        assert_eq!(observe(self), expected, "after typing {keys:?}");
    }

    fn set_layout_direction(&mut self, direction: LayoutDirection) {
        self.update(|window, cx| {
            LayoutDirection::set(cx, direction);
            window.refresh();
        });
        self.run_until_parked();
    }

    #[track_caller]
    fn assert_inline_order(&mut self, leading: &'static str, trailing: &'static str) {
        let rtl = self.update(|_, cx| LayoutDirection::current(cx).is_rtl());
        let a = self
            .selector_bounds(leading)
            .unwrap_or_else(|| panic!("no element with debug selector {leading:?}"));
        let b = self
            .selector_bounds(trailing)
            .unwrap_or_else(|| panic!("no element with debug selector {trailing:?}"));
        let ok = if rtl {
            a.left() >= b.right()
        } else {
            a.right() <= b.left()
        };
        assert!(
            ok,
            "{leading:?} should start-side of {trailing:?} in {} but is at {a:?} vs {b:?}",
            if rtl { "RTL" } else { "LTR" }
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::prelude::FluentBuilder as _;
    use gpui::{
        actions, div, px, InteractiveElement, IntoElement, KeyBinding, ParentElement, Render,
        StatefulInteractiveElement, Styled,
    };

    actions!(test_utils, [Bump]);

    struct Probe {
        a: FocusHandle,
        b: FocusHandle,
        clicks: usize,
        bumps: usize,
        text: String,
    }

    impl Render for Probe {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .on_action(cx.listener(|this, _: &Bump, _, _| this.bumps += 1))
                .on_key_down(cx.listener(|this, e: &gpui::KeyDownEvent, window, cx| {
                    if e.keystroke.key == "tab" {
                        if e.keystroke.modifiers.shift {
                            window.focus_prev(cx);
                        } else {
                            window.focus_next(cx);
                        }
                        return;
                    }
                    if let Some(c) = &e.keystroke.key_char {
                        this.text.push_str(c);
                    }
                }))
                .child(
                    div()
                        .id("a")
                        .size(px(50.))
                        .track_focus(&self.a)
                        .debug_selector(|| "a".into())
                        .on_click(cx.listener(|this, _, _, _| this.clicks += 1)),
                )
                .child(
                    div()
                        .id("b")
                        .size(px(50.))
                        .track_focus(&self.b)
                        .debug_selector(|| "b".into()),
                )
        }
    }

    fn open(cx: &mut TestAppContext) -> (Entity<Probe>, &mut VisualTestContext) {
        open_fluent_test_window(cx, |_, cx| Probe {
            a: cx.focus_handle().tab_stop(true),
            b: cx.focus_handle().tab_stop(true),
            clicks: 0,
            bumps: 0,
            text: String::new(),
        })
    }

    #[gpui::test]
    fn click_selector_clicks_and_focuses(cx: &mut TestAppContext) {
        let (view, cx) = open(cx);
        let (a, b) = view.read_with(cx, |v, _| (v.a.clone(), v.b.clone()));
        cx.assert_nothing_focused();
        cx.click_selector("a");
        cx.assert_focused(&a);
        cx.assert_not_focused(&b);
        assert_eq!(view.read_with(cx, |v, _| v.clicks), 1);
        cx.click_selector("b");
        cx.assert_focused(&b);
        cx.assert_not_focused(&a);
    }

    #[gpui::test]
    fn type_keys_and_text_reach_focused_element(cx: &mut TestAppContext) {
        let (view, cx) = open(cx);
        cx.update(|_, cx| cx.bind_keys([KeyBinding::new("ctrl-b", Bump, None)]));
        cx.click_selector("a");
        cx.type_text("hi");
        cx.type_keys("ctrl-b ctrl-b");
        view.read_with(cx, |v, _| {
            assert_eq!(v.text, "hi");
            assert_eq!(v.bumps, 2);
        });
    }

    #[test]
    fn a11y_node_matching() {
        let snap = vec![
            "Window".to_string(),
            "Switch \"Wi-Fi\" [toggled=True, disabled=true]".to_string(),
        ];
        assert_a11y_node(&snap, "Switch", Some("Wi-Fi"), &["toggled=True"]);
        assert_a11y_node(&snap, "Switch", None, &[]);
        assert_a11y_node(&snap, "Window", None, &[]);
        assert_no_a11y_node(&snap, "Button", None);
    }

    #[test]
    #[should_panic(expected = "no accessibility node")]
    fn a11y_node_wrong_state_panics() {
        let snap = vec!["Switch \"Wi-Fi\" [toggled=False]".to_string()];
        assert_a11y_node(&snap, "Switch", Some("Wi-Fi"), &["toggled=True"]);
    }

    #[test]
    fn contrast_known_values() {
        let max = contrast_ratio(gpui::black(), gpui::white());
        assert!((max - 21.0).abs() < 0.01, "{max}");
        assert!((contrast_ratio(gpui::white(), gpui::white()) - 1.0).abs() < 1e-4);
    }

    #[test]
    fn theme_text_meets_aa() {
        for theme in [Theme::dark(), Theme::light(), Theme::high_contrast()] {
            let c = &theme.colors;
            assert_contrast("on_neutral", c.on_neutral, c.surface, 4.5);
        }
    }

    #[gpui::test]
    fn tab_order_and_inline_order(cx: &mut TestAppContext) {
        let (view, cx) = open(cx);
        let (a, b) = view.read_with(cx, |v, _| (v.a.clone(), v.b.clone()));
        cx.assert_tab_order(&[&a, &b]);
        cx.assert_keys_yield("tab", |cx| cx.is_focused(&a), false);
    }

    #[gpui::test]
    fn layout_direction_flips_inline_order(cx: &mut TestAppContext) {
        struct Row;
        impl Render for Row {
            fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let rtl = LayoutDirection::current(cx).is_rtl();
                div()
                    .flex()
                    .when(rtl, |d| d.flex_row_reverse())
                    .child(
                        div()
                            .w(px(20.))
                            .h(px(20.))
                            .debug_selector(|| "first".into()),
                    )
                    .child(
                        div()
                            .w(px(20.))
                            .h(px(20.))
                            .debug_selector(|| "second".into()),
                    )
            }
        }
        let (_view, cx) = open_fluent_test_window(cx, |_, _| Row);
        cx.assert_inline_order("first", "second");
        cx.set_layout_direction(LayoutDirection::Rtl);
        cx.assert_inline_order("first", "second");
    }

    #[gpui::test]
    fn selector_bounds_missing_is_none(cx: &mut TestAppContext) {
        let (_view, cx) = open(cx);
        assert!(cx.selector_bounds("a").is_some());
        assert!(cx.selector_bounds("nope").is_none());
    }

    #[gpui::test]
    #[should_panic(expected = "no element with debug selector")]
    fn click_unknown_selector_panics(cx: &mut TestAppContext) {
        let (_view, cx) = open(cx);
        cx.click_selector("nope");
    }

    #[gpui::test]
    #[should_panic(expected = "to hold focus")]
    fn assert_focused_fails_when_unfocused(cx: &mut TestAppContext) {
        let (view, cx) = open(cx);
        let a = view.read_with(cx, |v, _| v.a.clone());
        cx.assert_focused(&a);
    }
}
