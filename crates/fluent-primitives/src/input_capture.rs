//! [`InputCapture`]: a focusable surface that captures raw keyboard and pointer input, for
//! remote-desktop and emulator style views.
//!
//! # What it delivers
//!
//! While **grabbed**, the element emits an [`InputCaptureEvent`] for:
//!
//! - every key press and release, including keys the application would normally handle itself
//!   (Tab, Alt, Enter, F10, ...): app key bindings, Tab focus navigation and text input do not
//!   see them. A press repeated before its release is reported with `is_held: true`.
//! - lone modifier presses and releases (Control, Alt, Shift, Super/Platform, Function),
//!   synthesised by diffing the window's modifier state, because the Windows and Linux
//!   (Wayland, X11) platform layers report a lone modifier only as a modifiers change.
//! - pointer moves, button presses and releases, and wheel steps, with positions in
//!   element-local coordinates (origin at the element's top-left corner).
//!
//! While not grabbed it is an ordinary focusable control: keys pass through and Tab moves focus
//! on as usual. Pointer input over the element is still reported (hover tracking), but keys are
//! not.
//!
//! # Grab and release
//!
//! - **Grab:** a primary click on the element focuses and grabs it (that click is consumed, not
//!   reported). From the keyboard, focus it with Tab and press Enter or Space. Or call
//!   [`InputCapture::grab`].
//! - **Release:** Escape (unless [`InputCapture::release_on_escape`] turned it off), pressing
//!   Control and Alt together, focus leaving the element, the window deactivating, or
//!   [`InputCapture::release`]. On release the element reports a release for every key and
//!   modifier it reported as pressed, so the receiver never sees a stuck key, then emits
//!   [`InputCaptureEvent::GrabChanged`]`(false)`. The keystroke that released the grab is
//!   consumed, not forwarded.
//!
//! Every release path except the pointer one works from the keyboard, and blur and window
//! deactivation work with neither, so a user can always get out.
//!
//! # Key codes and the scancode limitation
//!
//! gpui's `Keystroke` carries the logical key and the produced text, not the physical key. The
//! [`InputCaptureEvent::Key`] `code` field is therefore a **best-effort** code in the W3C
//! `KeyboardEvent.code` vocabulary (`"KeyA"`, `"Digit1"`, `"Tab"`, `"AltLeft"`, ...), derived
//! from the logical key as if the layout were US QWERTY. It is right for US layouts and for
//! layout-independent keys (function, navigation, editing keys), and wrong for letters and
//! punctuation on other layouts. Left and right modifiers cannot be told apart; the left code
//! is reported.
//!
//! `scan_code` is reserved for the platform scancode (Windows `WM_KEYDOWN` scan code, Linux
//! evdev code). It is always `None` today: the vendored gpui platform layers read the scancode
//! and discard it, and carrying it through needs a vendored-gpui patch (WI-2804).
//!
//! Windows and Linux only.

use std::collections::HashSet;

use fluent_core::ThemeProvider as _;
use gpui::{
    canvas, div, prelude::*, px, AnyView, App, Bounds, Context, DispatchPhase, EventEmitter,
    FocusHandle, Focusable, Hitbox, HitboxBehavior, IntoElement, KeyUpEvent, KeystrokeEvent,
    Modifiers, ModifiersChangedEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    Pixels, Point, Render, ScrollDelta, ScrollEvent, Subscription, WeakEntity, Window,
};

/// One captured input event. See the [module docs](self).
#[derive(Clone, Debug)]
pub enum InputCaptureEvent {
    /// A key press (`pressed: true`) or release.
    Key {
        /// gpui's logical key name (`"a"`, `"tab"`, `"alt"`, `"f5"`, ...).
        key: String,
        /// Best-effort W3C `KeyboardEvent.code` (`"KeyA"`, `"Tab"`, `"AltLeft"`); `None` for a
        /// key with no mapping. Derived from the logical key, see the module docs.
        code: Option<String>,
        /// Physical scancode. Always `None` until the vendored gpui platform layers carry it.
        scan_code: Option<u32>,
        /// `true` for a press, `false` for a release.
        pressed: bool,
        /// A press reported again before its release (auto-repeat).
        is_held: bool,
        /// Modifiers held when the event happened.
        modifiers: Modifiers,
    },
    /// The pointer moved over the element (or anywhere, during a pointer capture).
    PointerMove {
        /// Element-local position.
        position: Point<Pixels>,
        /// The button held during the move, if any.
        pressed_button: Option<MouseButton>,
        /// Modifiers held.
        modifiers: Modifiers,
    },
    /// A pointer button was pressed or released.
    PointerButton {
        /// Which button.
        button: MouseButton,
        /// `true` for press, `false` for release.
        pressed: bool,
        /// Element-local position.
        position: Point<Pixels>,
        /// Modifiers held.
        modifiers: Modifiers,
    },
    /// One wheel or trackpad scroll step.
    Wheel {
        /// Element-local position.
        position: Point<Pixels>,
        /// The scroll amount, in lines or pixels as the platform reported it.
        delta: ScrollDelta,
        /// Modifiers held.
        modifiers: Modifiers,
    },
    /// The grab started (`true`) or ended (`false`).
    GrabChanged(bool),
}

/// The logical names gpui gives lone modifier keys, with the code reported for each.
const MODIFIER_KEYS: [(&str, &str); 5] = [
    ("control", "ControlLeft"),
    ("alt", "AltLeft"),
    ("shift", "ShiftLeft"),
    ("platform", "MetaLeft"),
    ("function", "Fn"),
];

fn modifier_held(modifiers: &Modifiers, key: &str) -> bool {
    match key {
        "control" => modifiers.control,
        "alt" => modifiers.alt,
        "shift" => modifiers.shift,
        "platform" => modifiers.platform,
        "function" => modifiers.function,
        _ => false,
    }
}

/// Best-effort W3C `KeyboardEvent.code` for a gpui logical key, assuming a US layout.
pub fn key_code_for(key: &str) -> Option<String> {
    let mut chars = key.chars();
    if let (Some(c), None) = (chars.next(), chars.next()) {
        if c.is_ascii_alphabetic() {
            return Some(format!("Key{}", c.to_ascii_uppercase()));
        }
        if c.is_ascii_digit() {
            return Some(format!("Digit{c}"));
        }
    }
    if let Some(n) = key.strip_prefix('f') {
        if let Ok(n) = n.parse::<u8>() {
            if (1..=24).contains(&n) {
                return Some(format!("F{n}"));
            }
        }
    }
    if let Some((_, code)) = MODIFIER_KEYS.iter().find(|(name, _)| *name == key) {
        return Some((*code).to_string());
    }
    let code = match key {
        "enter" => "Enter",
        "escape" => "Escape",
        "tab" => "Tab",
        "space" | " " => "Space",
        "backspace" => "Backspace",
        "delete" => "Delete",
        "insert" => "Insert",
        "home" => "Home",
        "end" => "End",
        "pageup" => "PageUp",
        "pagedown" => "PageDown",
        "left" => "ArrowLeft",
        "right" => "ArrowRight",
        "up" => "ArrowUp",
        "down" => "ArrowDown",
        "capslock" => "CapsLock",
        "menu" => "ContextMenu",
        "printscreen" => "PrintScreen",
        "-" => "Minus",
        "=" => "Equal",
        "[" => "BracketLeft",
        "]" => "BracketRight",
        "\\" => "Backslash",
        ";" => "Semicolon",
        "'" => "Quote",
        "," => "Comma",
        "." => "Period",
        "/" => "Slash",
        "`" => "Backquote",
        _ => return None,
    };
    Some(code.to_string())
}

/// A focusable surface that captures raw key, pointer and wheel input while grabbed.
///
/// Create it with `cx.new(InputCapture::new)` and subscribe to its
/// [`InputCaptureEvent`]s. See the [module docs](self) for grab, release and key-code details.
pub struct InputCapture {
    focus_handle: FocusHandle,
    grabbed: bool,
    release_on_escape: bool,
    /// Modifier state last reported, for synthesising lone modifier presses and releases.
    modifiers: Modifiers,
    /// Non-modifier keys reported as pressed and not yet released.
    pressed_keys: HashSet<String>,
    /// Swallow the mouse-up of the click that started the grab.
    swallow_mouse_up: bool,
    content: Option<AnyView>,
    /// Whether the blur and window-activation watches are set up (done on first render).
    window_watched: bool,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<InputCaptureEvent> for InputCapture {}

impl Focusable for InputCapture {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl InputCapture {
    /// A new, ungrabbed capture surface.
    ///
    /// Needs no window: the blur and window-activation watches that release the grab are set up
    /// on first render, before the surface can be focused or grabbed.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle().tab_stop(true);
        let weak = cx.weak_entity();
        let interceptor = cx.intercept_keystrokes(move |event, window, cx| {
            Self::intercept(&weak, event, window, cx);
        });
        Self {
            focus_handle,
            grabbed: false,
            release_on_escape: true,
            modifiers: Modifiers::default(),
            pressed_keys: HashSet::new(),
            swallow_mouse_up: false,
            content: None,
            window_watched: false,
            _subscriptions: vec![interceptor],
        }
    }

    /// Release on blur and on window deactivation; done once, on first render.
    fn watch_window(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.window_watched {
            return;
        }
        self.window_watched = true;
        let blur = cx.on_blur(&self.focus_handle, window, |this, window, cx| {
            this.release(window, cx);
        });
        let activation = cx.observe_window_activation(window, |this, window, cx| {
            if !window.is_window_active() {
                this.release(window, cx);
            }
        });
        self._subscriptions.push(blur);
        self._subscriptions.push(activation);
    }

    /// Whether Escape releases the grab (default `true`). Turn it off when the remote side needs
    /// Escape; Control+Alt, blur and window deactivation still release.
    pub fn release_on_escape(mut self, release: bool) -> Self {
        self.release_on_escape = release;
        self
    }

    /// A view rendered inside the capture surface (for example the remote frame).
    pub fn content(mut self, content: AnyView) -> Self {
        self.content = Some(content);
        self
    }

    /// Whether input is currently grabbed.
    pub fn is_grabbed(&self) -> bool {
        self.grabbed
    }

    /// Focus the element and grab input. No-op if already grabbed.
    pub fn grab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle, cx);
        }
        if self.grabbed {
            return;
        }
        self.grabbed = true;
        // Modifiers already held when the grab starts belong to the gesture that started it.
        self.modifiers = window.modifiers();
        self.pressed_keys.clear();
        cx.emit(InputCaptureEvent::GrabChanged(true));
        cx.notify();
    }

    /// Release the grab, reporting a release for every key and modifier still down. No-op if
    /// not grabbed. Focus stays where it is.
    pub fn release(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.grabbed {
            return;
        }
        let mut keys: Vec<String> = self.pressed_keys.drain().collect();
        keys.sort();
        for key in keys {
            self.emit_key(key, false, false, self.modifiers, cx);
        }
        let held = self.modifiers;
        for (name, _) in MODIFIER_KEYS {
            if modifier_held(&held, name) {
                self.emit_key(name.to_string(), false, false, Modifiers::default(), cx);
            }
        }
        self.modifiers = Modifiers::default();
        self.grabbed = false;
        self.swallow_mouse_up = false;
        window.release_pointer();
        cx.emit(InputCaptureEvent::GrabChanged(false));
        cx.notify();
    }

    fn emit_key(
        &self,
        key: String,
        pressed: bool,
        is_held: bool,
        modifiers: Modifiers,
        cx: &mut Context<Self>,
    ) {
        cx.emit(InputCaptureEvent::Key {
            code: key_code_for(&key),
            key,
            scan_code: None,
            pressed,
            is_held,
            modifiers,
        });
    }

    /// Report lone modifier presses and releases for the difference between the last reported
    /// modifier state and `now`. Returns `true` when Control and Alt are both down (release).
    fn sync_modifiers(&mut self, now: Modifiers, cx: &mut Context<Self>) -> bool {
        let before = self.modifiers;
        if before == now {
            return false;
        }
        if now.control && now.alt {
            // Leave `self.modifiers` as last reported, so the release that follows reports
            // only what the receiver saw pressed.
            return true;
        }
        self.modifiers = now;
        for (name, _) in MODIFIER_KEYS {
            let was = modifier_held(&before, name);
            let is = modifier_held(&now, name);
            if was != is {
                self.emit_key(name.to_string(), is, false, now, cx);
            }
        }
        false
    }

    /// Runs before key bindings for every keystroke in every window.
    fn intercept(
        weak: &WeakEntity<Self>,
        event: &KeystrokeEvent,
        window: &mut Window,
        cx: &mut App,
    ) {
        let Ok((focused, grabbed)) = weak.read_with(cx, |this, _| {
            (this.focus_handle.is_focused(window), this.grabbed)
        }) else {
            return;
        };
        if !focused {
            return;
        }
        let keystroke = &event.keystroke;
        let no_mods = !keystroke.modifiers.modified();
        if !grabbed {
            if no_mods && matches!(keystroke.key.as_str(), "enter" | "space") {
                weak.update(cx, |this, cx| this.grab(window, cx)).ok();
                cx.stop_propagation();
            }
            return;
        }
        cx.stop_propagation();
        let window_modifiers = window.modifiers();
        weak.update(cx, |this, cx| {
            if keystroke.is_modifier_key() {
                // A lone modifier tap; the modifier diff carries it.
                if this.sync_modifiers(window_modifiers, cx) {
                    this.release(window, cx);
                }
                return;
            }
            if this.release_on_escape && no_mods && keystroke.key == "escape" {
                this.release(window, cx);
                return;
            }
            // Simulated or synthetic input can carry modifiers no modifiers-changed event
            // announced; report them before the key.
            if this.sync_modifiers(keystroke.modifiers, cx) {
                this.release(window, cx);
                return;
            }
            let is_held = !this.pressed_keys.insert(keystroke.key.clone());
            this.emit_key(
                keystroke.key.clone(),
                true,
                is_held,
                keystroke.modifiers,
                cx,
            );
        })
        .ok();
    }

    fn on_key_up(&mut self, event: &KeyUpEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.grabbed || !self.focus_handle.is_focused(window) {
            return;
        }
        cx.stop_propagation();
        if self.pressed_keys.remove(&event.keystroke.key) {
            self.emit_key(
                event.keystroke.key.clone(),
                false,
                false,
                event.keystroke.modifiers,
                cx,
            );
        }
    }

    fn on_modifiers_changed(
        &mut self,
        event: &ModifiersChangedEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.grabbed || !self.focus_handle.is_focused(window) {
            return;
        }
        cx.stop_propagation();
        if self.sync_modifiers(event.modifiers, cx) {
            self.release(window, cx);
        }
    }

    fn on_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        hitbox: &Hitbox,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.stop_propagation();
        window.capture_pointer(hitbox.id);
        if !self.grabbed {
            if event.button == MouseButton::Left {
                self.grab(window, cx);
                self.swallow_mouse_up = true;
            } else {
                window.focus(&self.focus_handle, cx);
            }
            return;
        }
        cx.emit(InputCaptureEvent::PointerButton {
            button: event.button,
            pressed: true,
            position: event.position - hitbox.bounds.origin,
            modifiers: event.modifiers,
        });
    }

    fn on_mouse_up(&mut self, event: &MouseUpEvent, origin: Point<Pixels>, cx: &mut Context<Self>) {
        cx.stop_propagation();
        if std::mem::take(&mut self.swallow_mouse_up) || !self.grabbed {
            return;
        }
        cx.emit(InputCaptureEvent::PointerButton {
            button: event.button,
            pressed: false,
            position: event.position - origin,
            modifiers: event.modifiers,
        });
    }

    fn paint_listeners(weak: WeakEntity<Self>, hitbox: Hitbox, window: &mut Window) {
        {
            let weak = weak.clone();
            let hitbox = hitbox.clone();
            window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !hitbox.is_hovered(window) {
                    return;
                }
                let position = event.position - hitbox.bounds.origin;
                weak.update(cx, |_, cx| {
                    cx.emit(InputCaptureEvent::PointerMove {
                        position,
                        pressed_button: event.pressed_button,
                        modifiers: event.modifiers,
                    });
                })
                .ok();
            });
        }
        {
            let weak = weak.clone();
            let hitbox = hitbox.clone();
            window.on_mouse_event(move |event: &MouseDownEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !hitbox.is_hovered(window) {
                    return;
                }
                weak.update(cx, |this, cx| {
                    this.on_mouse_down(event, &hitbox, window, cx)
                })
                .ok();
            });
        }
        {
            let weak = weak.clone();
            let hitbox = hitbox.clone();
            window.on_mouse_event(move |event: &MouseUpEvent, phase, window, cx| {
                if phase != DispatchPhase::Bubble || !hitbox.is_hovered(window) {
                    return;
                }
                let origin = hitbox.bounds.origin;
                weak.update(cx, |this, cx| this.on_mouse_up(event, origin, cx))
                    .ok();
            });
        }
        window.on_mouse_event(move |event: &ScrollEvent, phase, window, cx| {
            if phase != DispatchPhase::Bubble || !hitbox.should_handle_scroll(window) {
                return;
            }
            let position = event.position - hitbox.bounds.origin;
            weak.update(cx, |this, cx| {
                if !this.grabbed {
                    return;
                }
                cx.stop_propagation();
                cx.emit(InputCaptureEvent::Wheel {
                    position,
                    delta: event.delta,
                    modifiers: event.modifiers,
                });
            })
            .ok();
        });
    }
}

impl Render for InputCapture {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.watch_window(window, cx);
        let colors = &cx.theme().colors;
        let border = if self.grabbed {
            colors.stroke_accent
        } else if self.focus_handle.is_focused(window) {
            colors.stroke_neutral
        } else {
            colors.stroke_neutral_subtle
        };
        let weak = cx.weak_entity();
        let surface = canvas(
            |bounds: Bounds<Pixels>, window, _| {
                window.insert_hitbox(bounds, HitboxBehavior::Normal)
            },
            move |_, hitbox, window, _| Self::paint_listeners(weak, hitbox, window),
        )
        .absolute()
        .size_full();

        div()
            .id("fluent-input-capture")
            .track_focus(&self.focus_handle)
            .relative()
            .size_full()
            .overflow_hidden()
            .rounded(px(4.0))
            .capture_key_up(cx.listener(Self::on_key_up))
            .on_modifiers_changed(cx.listener(Self::on_modifiers_changed))
            .children(self.content.clone())
            .child(surface)
            // The focus/grab ring is an overlay so it does not shift the capture surface's
            // origin away from the element's top-left corner.
            .child(
                div()
                    .absolute()
                    .size_full()
                    .border_2()
                    .border_color(border)
                    .rounded(px(4.0)),
            )
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use fluent_test_utils::{open_fluent_test_window, Entity, VisualTestContext};
    use gpui::{
        actions, point, Capslock, KeyBinding, Keystroke, ModifiersChangedEvent, TestAppContext,
        TouchPhase,
    };

    use super::*;

    actions!(input_capture_test, [TestTab]);

    struct Host {
        capture: Entity<InputCapture>,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .pt(px(50.0))
                .pl(px(20.0))
                .child(div().w(px(200.0)).h(px(100.0)).child(self.capture.clone()))
        }
    }

    type Log = Rc<RefCell<Vec<InputCaptureEvent>>>;

    /// `ScrollDelta` has no `PartialEq`, so events are compared through `Debug`.
    fn dbg(events: &[InputCaptureEvent]) -> String {
        format!("{events:?}")
    }

    fn logged(log: &Log) -> String {
        dbg(&log.borrow())
    }

    fn last(log: &Log) -> String {
        format!("{:?}", log.borrow().last())
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<InputCapture>, Log, &mut VisualTestContext) {
        cx.executor().allow_parking();
        let log: Log = Rc::default();
        let sink = log.clone();
        let (host, vcx) = open_fluent_test_window(cx, move |_window, cx| {
            let capture = cx.new(InputCapture::new);
            cx.subscribe(&capture, move |_, _, event: &InputCaptureEvent, _| {
                sink.borrow_mut().push(event.clone());
            })
            .detach();
            Host { capture }
        });
        let capture = host.read_with(vcx, |host, _| host.capture.clone());
        vcx.run_until_parked();
        (capture, log, vcx)
    }

    /// Window coordinates of element-local (x, y): the capture sits at (20, 50).
    fn at(x: f32, y: f32) -> Point<Pixels> {
        point(px(20.0 + x), px(50.0 + y))
    }

    fn grabbed(capture: &Entity<InputCapture>, vcx: &mut VisualTestContext) -> bool {
        capture.read_with(vcx, |c, _| c.is_grabbed())
    }

    fn key(log: &Log) -> Vec<(String, Option<String>, bool, bool)> {
        log.borrow()
            .iter()
            .filter_map(|e| match e {
                InputCaptureEvent::Key {
                    key,
                    code,
                    pressed,
                    is_held,
                    ..
                } => Some((key.clone(), code.clone(), *pressed, *is_held)),
                _ => None,
            })
            .collect()
    }

    fn k(key: &str, code: &str, pressed: bool) -> (String, Option<String>, bool, bool) {
        (key.into(), Some(code.into()), pressed, false)
    }

    fn key_up(vcx: &mut VisualTestContext, source: &str) {
        let keystroke = Keystroke {
            modifiers: Modifiers::default(),
            key: source.into(),
            key_char: None,
        };
        vcx.simulate_event(KeyUpEvent { keystroke });
    }

    #[gpui::test]
    fn click_grabs_and_is_consumed(cx: &mut TestAppContext) {
        let (capture, log, vcx) = setup(cx);
        assert!(!grabbed(&capture, vcx));
        vcx.simulate_click(at(10.0, 10.0), Modifiers::default());
        assert!(grabbed(&capture, vcx));
        assert_eq!(logged(&log), dbg(&[InputCaptureEvent::GrabChanged(true)]));
    }

    #[gpui::test]
    fn keys_are_delivered_with_press_release_and_tab_is_captured(cx: &mut TestAppContext) {
        let (capture, log, vcx) = setup(cx);
        let tab_fired = Rc::new(RefCell::new(0));
        let counter = tab_fired.clone();
        vcx.update(|_, cx| {
            cx.bind_keys([KeyBinding::new("tab", TestTab, None)]);
            cx.on_action(move |_: &TestTab, _| *counter.borrow_mut() += 1);
        });
        vcx.simulate_click(at(10.0, 10.0), Modifiers::default());
        log.borrow_mut().clear();

        vcx.simulate_keystrokes("a");
        key_up(vcx, "a");
        vcx.simulate_keystrokes("tab");
        vcx.simulate_keystrokes("tab");
        key_up(vcx, "tab");

        assert_eq!(
            key(&log),
            vec![
                k("a", "KeyA", true),
                k("a", "KeyA", false),
                k("tab", "Tab", true),
                ("tab".into(), Some("Tab".into()), true, true),
                k("tab", "Tab", false),
            ]
        );
        assert_eq!(
            *tab_fired.borrow(),
            0,
            "Tab binding must not fire while grabbed"
        );
        assert!(grabbed(&capture, vcx));
    }

    #[gpui::test]
    fn not_grabbed_lets_keys_through(cx: &mut TestAppContext) {
        let (capture, log, vcx) = setup(cx);
        let tab_fired = Rc::new(RefCell::new(0));
        let counter = tab_fired.clone();
        vcx.update(|window, cx| {
            cx.bind_keys([KeyBinding::new("tab", TestTab, None)]);
            cx.on_action(move |_: &TestTab, _| *counter.borrow_mut() += 1);
            let handle = capture.read(cx).focus_handle.clone();
            window.focus(&handle, cx);
        });
        vcx.simulate_keystrokes("a tab");
        assert!(key(&log).is_empty());
        assert_eq!(*tab_fired.borrow(), 1);
        assert!(!grabbed(&capture, vcx));
    }

    #[gpui::test]
    fn enter_grabs_from_keyboard_and_escape_releases(cx: &mut TestAppContext) {
        let (capture, log, vcx) = setup(cx);
        vcx.update(|window, cx| {
            let handle = capture.read(cx).focus_handle.clone();
            window.focus(&handle, cx);
        });
        vcx.simulate_keystrokes("enter");
        key_up(vcx, "enter");
        assert!(grabbed(&capture, vcx));
        // The grabbing Enter is consumed: neither its press nor its release is reported.
        assert_eq!(logged(&log), dbg(&[InputCaptureEvent::GrabChanged(true)]));

        vcx.simulate_keystrokes("x");
        vcx.simulate_keystrokes("escape");
        assert!(!grabbed(&capture, vcx));
        assert_eq!(
            key(&log),
            vec![k("x", "KeyX", true), k("x", "KeyX", false)],
            "release reports a release for the still-held key, not Escape"
        );
        assert_eq!(last(&log), "Some(GrabChanged(false))");
    }

    #[gpui::test]
    fn escape_is_forwarded_when_release_on_escape_is_off(cx: &mut TestAppContext) {
        let (capture, log, vcx) = setup(cx);
        capture.update(vcx, |c, _| c.release_on_escape = false);
        vcx.simulate_click(at(1.0, 1.0), Modifiers::default());
        vcx.simulate_keystrokes("escape");
        assert!(grabbed(&capture, vcx));
        assert_eq!(key(&log), vec![k("escape", "Escape", true)]);
    }

    #[gpui::test]
    fn modifier_diff_reports_lone_modifiers_and_alt_is_captured(cx: &mut TestAppContext) {
        let (capture, log, vcx) = setup(cx);
        vcx.simulate_click(at(1.0, 1.0), Modifiers::default());
        log.borrow_mut().clear();

        vcx.simulate_modifiers_change(Modifiers::alt());
        vcx.simulate_modifiers_change(Modifiers::default());
        vcx.simulate_modifiers_change(Modifiers::shift());
        vcx.simulate_keystrokes("shift-b");
        vcx.simulate_modifiers_change(Modifiers::default());

        assert_eq!(
            key(&log),
            vec![
                k("alt", "AltLeft", true),
                k("alt", "AltLeft", false),
                k("shift", "ShiftLeft", true),
                k("b", "KeyB", true),
                k("shift", "ShiftLeft", false),
            ]
        );
        assert!(grabbed(&capture, vcx));
    }

    #[gpui::test]
    fn control_alt_releases(cx: &mut TestAppContext) {
        let (capture, log, vcx) = setup(cx);
        vcx.simulate_click(at(1.0, 1.0), Modifiers::default());
        vcx.simulate_modifiers_change(Modifiers::control());
        vcx.simulate_event(ModifiersChangedEvent {
            modifiers: Modifiers {
                control: true,
                alt: true,
                ..Modifiers::default()
            },
            capslock: Capslock { on: false },
        });
        assert!(!grabbed(&capture, vcx));
        // Control was reported pressed, so it is reported released; Alt never reached the
        // receiver.
        assert_eq!(
            key(&log),
            vec![
                k("control", "ControlLeft", true),
                k("control", "ControlLeft", false)
            ]
        );
        assert_eq!(last(&log), "Some(GrabChanged(false))");
    }

    #[gpui::test]
    fn blur_releases(cx: &mut TestAppContext) {
        let (capture, log, vcx) = setup(cx);
        // Focus events only fire for an active window.
        vcx.update(|window, _| window.activate_window());
        vcx.run_until_parked();
        vcx.simulate_click(at(1.0, 1.0), Modifiers::default());
        assert!(grabbed(&capture, vcx));
        vcx.update(|window, cx| {
            // Draw once so the rendered frame's focus path contains the capture.
            window.draw(cx).clear(cx);
            window.blur(cx);
            // Focus listeners run when the next frame is drawn.
            window.draw(cx).clear(cx);
        });
        vcx.run_until_parked();
        assert!(!grabbed(&capture, vcx));
        assert_eq!(last(&log), "Some(GrabChanged(false))");
    }

    #[gpui::test]
    fn pointer_and_wheel_positions_are_element_local(cx: &mut TestAppContext) {
        let (_capture, log, vcx) = setup(cx);
        vcx.simulate_click(at(1.0, 1.0), Modifiers::default());
        log.borrow_mut().clear();

        vcx.simulate_mouse_move(at(30.0, 40.0), None, Modifiers::default());
        vcx.simulate_mouse_down(at(30.0, 40.0), MouseButton::Right, Modifiers::default());
        vcx.simulate_mouse_up(at(31.0, 41.0), MouseButton::Right, Modifiers::default());
        vcx.simulate_event(ScrollEvent {
            position: at(5.0, 6.0),
            delta: ScrollDelta::Lines(point(0.0, -1.0)),
            modifiers: Modifiers::default(),
            touch_phase: TouchPhase::Moved,
            ..Default::default()
        });

        let none = Modifiers::default();
        assert_eq!(
            logged(&log),
            dbg(&[
                InputCaptureEvent::PointerMove {
                    position: point(px(30.0), px(40.0)),
                    pressed_button: None,
                    modifiers: none,
                },
                InputCaptureEvent::PointerButton {
                    button: MouseButton::Right,
                    pressed: true,
                    position: point(px(30.0), px(40.0)),
                    modifiers: none,
                },
                InputCaptureEvent::PointerButton {
                    button: MouseButton::Right,
                    pressed: false,
                    position: point(px(31.0), px(41.0)),
                    modifiers: none,
                },
                InputCaptureEvent::Wheel {
                    position: point(px(5.0), px(6.0)),
                    delta: ScrollDelta::Lines(point(0.0, -1.0)),
                    modifiers: none,
                },
            ])
        );
    }

    #[test]
    fn key_codes_follow_w3c_names() {
        assert_eq!(key_code_for("q").as_deref(), Some("KeyQ"));
        assert_eq!(key_code_for("7").as_deref(), Some("Digit7"));
        assert_eq!(key_code_for("f12").as_deref(), Some("F12"));
        assert_eq!(key_code_for("alt").as_deref(), Some("AltLeft"));
        assert_eq!(key_code_for("pageup").as_deref(), Some("PageUp"));
        assert_eq!(key_code_for("/").as_deref(), Some("Slash"));
        assert_eq!(key_code_for("f99"), None);
        assert_eq!(key_code_for("unknownkey"), None);
    }
}
