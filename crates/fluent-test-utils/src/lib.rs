use fluent_core::Theme;

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

#[cfg(test)]
mod tests {
    use super::*;
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
                .on_key_down(cx.listener(|this, e: &gpui::KeyDownEvent, _, _| {
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
            a: cx.focus_handle(),
            b: cx.focus_handle(),
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
