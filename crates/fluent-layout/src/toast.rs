use std::time::Duration;

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    deferred, div, prelude::*, px, Context, Global, IntoElement, Render, SharedString, Window,
};

use crate::{MessageBar, MessageIntent};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ToastPlacement {
    TopStart,
    TopCenter,
    #[default]
    TopEnd,
    BottomStart,
    BottomCenter,
    BottomEnd,
}

#[derive(Clone, Debug)]
pub struct Toast {
    pub id: SharedString,
    pub intent: MessageIntent,
    pub title: Option<SharedString>,
    pub message: SharedString,
    pub timeout: Option<Duration>,
}

impl Toast {
    pub fn new(id: impl Into<SharedString>, message: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            intent: MessageIntent::Info,
            title: None,
            message: message.into(),
            timeout: Some(Duration::from_secs(5)),
        }
    }

    pub fn intent(mut self, intent: MessageIntent) -> Self {
        self.intent = intent;
        self
    }

    pub fn title(mut self, title: impl Into<SharedString>) -> Self {
        self.title = Some(title.into());
        self
    }

    pub fn timeout(mut self, timeout: impl Into<Option<Duration>>) -> Self {
        self.timeout = timeout.into();
        self
    }

    pub fn persistent(mut self) -> Self {
        self.timeout = None;
        self
    }
}

pub struct ToastStack {
    entries: Vec<Toast>,
    max_entries: usize,
}

impl Global for ToastStack {}

impl ToastStack {
    pub fn init(cx: &mut gpui::App) {
        cx.set_global(ToastStack {
            entries: Vec::new(),
            max_entries: 5,
        });
    }

    pub fn push(toast: Toast, cx: &mut gpui::App) {
        let id = toast.id.clone();
        let timeout = toast.timeout;
        {
            let stack = cx.global_mut::<ToastStack>();
            stack.entries.retain(|entry| entry.id != id);
            stack.entries.push(toast);
            let overflow = stack.entries.len().saturating_sub(stack.max_entries);
            if overflow > 0 {
                stack.entries.drain(0..overflow);
            }
        }

        if let Some(timeout) = timeout {
            cx.spawn(async move |cx| {
                cx.background_executor().timer(timeout).await;
                cx.update(|cx| ToastStack::dismiss(id.as_ref(), cx)).ok();
            })
            .detach();
        }
    }

    pub fn set_max_entries(max_entries: usize, cx: &mut gpui::App) {
        let stack = cx.global_mut::<ToastStack>();
        stack.max_entries = max_entries.max(1);
        let overflow = stack.entries.len().saturating_sub(stack.max_entries);
        if overflow > 0 {
            stack.entries.drain(0..overflow);
        }
    }

    pub fn max_entries(cx: &gpui::App) -> usize {
        cx.global::<ToastStack>().max_entries
    }

    pub fn dismiss(id: &str, cx: &mut gpui::App) {
        cx.global_mut::<ToastStack>()
            .entries
            .retain(|toast| toast.id.as_ref() != id);
    }

    pub fn clear(cx: &mut gpui::App) {
        cx.global_mut::<ToastStack>().entries.clear();
    }
}

#[derive(Default)]
pub struct ToastHost {
    observing: bool,
    placement: ToastPlacement,
    width: f32,
}

impl ToastHost {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        cx.observe_global::<ToastStack>(|_, cx| cx.notify())
            .detach();
        Self {
            observing: true,
            placement: ToastPlacement::TopEnd,
            width: 360.0,
        }
    }

    pub fn placement(mut self, placement: ToastPlacement) -> Self {
        self.placement = placement;
        self
    }

    pub fn width(mut self, width: f32) -> Self {
        self.width = width;
        self
    }
}

impl Render for ToastHost {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.observing {
            cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
            cx.observe_global::<ToastStack>(|_, cx| cx.notify())
                .detach();
            self.observing = true;
        }

        let theme = cx.theme();
        let spacing = theme.spacing;
        let toasts = cx
            .try_global::<ToastStack>()
            .map(|stack| stack.entries.clone())
            .unwrap_or_default();

        if toasts.is_empty() {
            return div();
        }

        let mut stack = div()
            .absolute()
            .w(px(self.width))
            .flex()
            .flex_col()
            .gap(px(spacing.sm));

        stack = match self.placement {
            ToastPlacement::TopStart => stack.top(px(spacing.xl)).left(px(spacing.xl)),
            ToastPlacement::TopCenter => stack
                .top(px(spacing.xl))
                .left(px(0.0))
                .right(px(0.0))
                .mx_auto(),
            ToastPlacement::TopEnd => stack.top(px(spacing.xl)).right(px(spacing.xl)),
            ToastPlacement::BottomStart => stack.bottom(px(spacing.xl)).left(px(spacing.xl)),
            ToastPlacement::BottomCenter => stack
                .bottom(px(spacing.xl))
                .left(px(0.0))
                .right(px(0.0))
                .mx_auto(),
            ToastPlacement::BottomEnd => stack.bottom(px(spacing.xl)).right(px(spacing.xl)),
        };

        for toast in toasts {
            let id = toast.id.clone();
            let mut bar = MessageBar::new(toast.message).intent(toast.intent);
            if let Some(title) = toast.title {
                bar = bar.title(title);
            }
            stack = stack.child(bar.on_dismiss(move |_, _, cx| {
                ToastStack::dismiss(id.as_ref(), cx);
            }));
        }

        div().absolute().inset_0().child(deferred(stack))
    }
}
