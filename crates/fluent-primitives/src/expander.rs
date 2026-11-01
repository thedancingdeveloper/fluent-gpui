use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, svg, AnyElement, App, ClickEvent, Div, ElementId, FontWeight, IntoElement,
    RenderOnce, Role, SharedString, Stateful, Window,
};

/// A Fluent 2-styled expander (collapsible section): a header row with a
/// chevron and a title that toggles a content area.
///
/// The caller owns the `expanded` state and passes it in on each render; the
/// `on_toggle` callback receives the *new* desired state. The header is a
/// keyboard-focusable button: Enter and Space toggle it, and it reports its
/// expanded state to assistive technology. The content is only built into the
/// element tree while expanded. The toggle is instant (no animation).
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Expander {
    id: ElementId,
    title: SharedString,
    expanded: bool,
    disabled: bool,
    content: Option<AnyElement>,
    on_toggle: Option<Box<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Expander {
    pub fn new(id: impl Into<ElementId>, title: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            expanded: false,
            disabled: false,
            content: None,
            on_toggle: None,
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// The collapsible body, shown while expanded.
    pub fn content(mut self, content: impl IntoElement) -> Self {
        self.content = Some(content.into_any_element());
        self
    }

    /// Callback receives the *new* desired expanded state.
    pub fn on_toggle(
        mut self,
        handler: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Box::new(handler));
        self
    }
}

/// The header element with its accessibility semantics: a button named by the
/// title that reports its expanded state.
fn header_base(id: ElementId, title: SharedString, expanded: bool) -> Stateful<Div> {
    div()
        .id(id)
        .role(Role::Button)
        .aria_label(title)
        .aria_expanded(expanded)
}

impl RenderOnce for Expander {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;
        let spacing = &theme.spacing;
        let typography = &theme.typography;
        let components = &theme.components;

        let disabled = self.disabled;
        let expanded = self.expanded;
        let next_state = !expanded;
        let on_toggle = self.on_toggle;

        let fg = if disabled {
            colors.on_neutral_disabled
        } else {
            colors.on_neutral
        };
        let chevron = if expanded {
            "icons/chevron_down.svg"
        } else {
            "icons/chevron_right.svg"
        };
        let hover_bg = colors.subtle_hover;
        let focus_border = colors.stroke_accent;

        let mut header = header_base(self.id, self.title.clone(), expanded)
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.md))
            .px(px(spacing.md))
            .py(px(spacing.sm))
            .rounded(px(radii.md))
            .border_1()
            .border_color(gpui::transparent_black())
            .text_color(fg)
            .child(
                svg()
                    .path(chevron)
                    .size(px(components.popup_icon_slot))
                    .flex_none()
                    .text_color(fg),
            )
            .child(
                div()
                    .text_size(px(typography.body.size))
                    .line_height(px(typography.body.line_height))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title),
            );

        if !disabled {
            header = header
                .focusable()
                .tab_stop(true)
                .cursor_pointer()
                .hover(move |s| s.bg(hover_bg))
                .focus_visible(move |s| s.border_color(focus_border));
            if let Some(handler) = on_toggle {
                header = header.on_click(move |ev, win, app| handler(next_state, ev, win, app));
            }
        }

        let mut root = div().flex().flex_col().child(header);
        if expanded {
            if let Some(content) = self.content {
                root = root.child(
                    div()
                        .pl(px(spacing.xxl + spacing.md))
                        .pr(px(spacing.md))
                        .py(px(spacing.md))
                        .child(content),
                );
            }
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use std::{cell::RefCell, rc::Rc};

    use fluent_test_utils::open_fluent_test_window;
    use gpui::{
        Context, InputEvent as _, KeyDownEvent, KeyUpEvent, Keystroke, Render, TestAppContext,
        VisualTestContext,
    };

    use super::*;

    /// Test view owning the expanded state, as an app would.
    struct Host {
        expanded: bool,
        disabled: bool,
        toggles: Rc<RefCell<Vec<bool>>>,
    }

    impl Render for Host {
        fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            let entity = cx.entity();
            let toggles = self.toggles.clone();
            div().size_full().child(
                Expander::new("exp", "Workspace limits")
                    .expanded(self.expanded)
                    .disabled(self.disabled)
                    .content(div().debug_selector(|| "exp-content".into()).child("Body"))
                    .on_toggle(move |new, _, _, cx| {
                        toggles.borrow_mut().push(new);
                        entity.update(cx, |host, cx| {
                            host.expanded = new;
                            cx.notify();
                        });
                    }),
            )
        }
    }

    fn open(
        cx: &mut TestAppContext,
        expanded: bool,
        disabled: bool,
    ) -> (
        gpui::Entity<Host>,
        &mut VisualTestContext,
        Rc<RefCell<Vec<bool>>>,
    ) {
        let toggles = Rc::new(RefCell::new(Vec::new()));
        let t = toggles.clone();
        let (host, vcx) = open_fluent_test_window(cx, move |_, _| Host {
            expanded,
            disabled,
            toggles: t,
        });
        vcx.executor().allow_parking();
        (host, vcx, toggles)
    }

    fn press(vcx: &mut VisualTestContext, key: &'static str) {
        let keystroke = Keystroke::parse(key).unwrap();
        vcx.update(|window, cx| {
            window.dispatch_event(
                KeyDownEvent {
                    keystroke: keystroke.clone(),
                    is_held: false,
                    prefer_character_input: false,
                }
                .to_platform_input(),
                cx,
            );
            window.dispatch_event(KeyUpEvent { keystroke }.to_platform_input(), cx);
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| window.draw(cx).clear(cx));
    }

    fn focus_header(vcx: &mut VisualTestContext) {
        vcx.update(|window, cx| window.focus_next(cx));
        vcx.run_until_parked();
        vcx.update(|window, cx| window.draw(cx).clear(cx));
    }

    #[gpui::test]
    fn content_is_built_only_while_expanded(cx: &mut TestAppContext) {
        let (host, vcx, _) = open(cx, false, false);
        vcx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(vcx.debug_bounds("exp-content").is_none());

        host.update(vcx, |host, cx| {
            host.expanded = true;
            cx.notify();
        });
        vcx.update(|window, cx| window.draw(cx).clear(cx));
        assert!(vcx.debug_bounds("exp-content").is_some());
    }

    #[gpui::test]
    fn enter_and_space_toggle_the_focused_header(cx: &mut TestAppContext) {
        let (host, vcx, toggles) = open(cx, false, false);
        vcx.update(|window, cx| window.draw(cx).clear(cx));
        focus_header(vcx);

        press(vcx, "enter");
        assert_eq!(*toggles.borrow(), vec![true]);
        assert!(host.read_with(vcx, |h, _| h.expanded));

        vcx.update(|window, cx| window.draw(cx).clear(cx));
        press(vcx, "space");
        assert_eq!(*toggles.borrow(), vec![true, false]);
        assert!(!host.read_with(vcx, |h, _| h.expanded));
    }

    #[gpui::test]
    fn disabled_header_is_not_focusable_and_ignores_keys(cx: &mut TestAppContext) {
        let (host, vcx, toggles) = open(cx, false, true);
        vcx.update(|window, cx| window.draw(cx).clear(cx));
        focus_header(vcx);

        press(vcx, "enter");
        press(vcx, "space");
        assert!(toggles.borrow().is_empty());
        assert!(!host.read_with(vcx, |h, _| h.expanded));
    }

    #[test]
    fn header_exposes_button_role_label_and_expanded_state() {
        for expanded in [false, true] {
            let header = header_base("h".into(), "Workspace limits".into(), expanded);
            assert_eq!(header.a11y_role(), Some(Role::Button));
            let mut node = gpui::accesskit::Node::new(Role::Button);
            header.write_a11y_info(&mut node);
            assert_eq!(node.label(), Some("Workspace limits"));
            assert_eq!(node.is_expanded(), Some(expanded));
        }
    }
}
