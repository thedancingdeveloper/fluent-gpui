use gpui::{App, Global, SharedString};

/// Global window title/status state for FluentGUI applications.
///
/// `FluentApp` initializes this at startup. Views can update it later to keep
/// platform chrome, custom title bars, and app-owned status surfaces in sync.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct WindowTitle {
    title: SharedString,
    status: Option<SharedString>,
}

impl Global for WindowTitle {}

impl WindowTitle {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            status: None,
        }
    }

    pub fn with_status(mut self, status: impl Into<SharedString>) -> Self {
        self.status = Some(status.into());
        self
    }

    pub fn title(&self) -> &SharedString {
        &self.title
    }

    pub fn status(&self) -> Option<&SharedString> {
        self.status.as_ref()
    }

    pub fn init(cx: &mut App, title: impl Into<SharedString>) {
        cx.set_global(Self::new(title));
    }

    pub fn set_title(cx: &mut App, title: impl Into<SharedString>) {
        if cx.has_global::<Self>() {
            cx.global_mut::<Self>().title = title.into();
        } else {
            Self::init(cx, title);
        }
    }

    pub fn set_status(cx: &mut App, status: impl Into<SharedString>) {
        if cx.has_global::<Self>() {
            cx.global_mut::<Self>().status = Some(status.into());
        } else {
            cx.set_global(Self::new("").with_status(status));
        }
    }

    pub fn clear_status(cx: &mut App) {
        if cx.has_global::<Self>() {
            cx.global_mut::<Self>().status = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::WindowTitle;

    #[test]
    fn window_title_stores_title_and_status() {
        let title = WindowTitle::new("Inbox").with_status("3 unread");

        assert_eq!(title.title().as_ref(), "Inbox");
        assert_eq!(title.status().map(|s| s.as_ref()), Some("3 unread"));
    }

    #[test]
    fn window_title_status_is_optional() {
        let title = WindowTitle::new("Inbox");

        assert_eq!(title.title().as_ref(), "Inbox");
        assert_eq!(title.status(), None);
    }
}
