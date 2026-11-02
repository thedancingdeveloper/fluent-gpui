//! Shared loading, empty, no-results and error states for data-heavy views.

use fluent_core::ThemeProvider as _;
use gpui::{div, prelude::*, px, App, Role, SharedString};

/// The four non-ready states of a data view.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataStatusKind {
    /// Data is on its way.
    Loading,
    /// There is no data at all.
    Empty,
    /// There is data, but the filter or search hides all of it.
    NoResults,
    /// Loading failed.
    Error,
}

impl DataStatusKind {
    /// Accessibility role: errors are alerts, everything else a polite status.
    pub fn role(self) -> Role {
        match self {
            Self::Error => Role::Alert,
            _ => Role::Status,
        }
    }

    /// Spoken label: the message, prefixed with the kind so it is never colour-only.
    pub fn announcement(self, message: &str) -> String {
        let prefix = match self {
            Self::Loading => "Loading",
            Self::Empty => "Empty",
            Self::NoResults => "No results",
            Self::Error => "Error",
        };
        format!("{prefix}: {message}")
    }
}

/// A fixed-height status block, so a view does not jump when its state changes.
pub(crate) fn status_block(
    id: &'static str,
    kind: DataStatusKind,
    message: SharedString,
    cx: &App,
) -> gpui::Stateful<gpui::Div> {
    let theme = cx.theme();
    let colors = &theme.colors;
    let color = match kind {
        DataStatusKind::Error => colors.status_error,
        DataStatusKind::Loading => colors.on_subtle,
        DataStatusKind::Empty | DataStatusKind::NoResults => colors.on_subtle_disabled,
    };
    div()
        .id(id)
        .role(kind.role())
        .aria_label(SharedString::from(kind.announcement(&message)))
        .min_h(px(theme.components.menu_item_height * 3.0))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(theme.typography.body.size))
        .text_color(color)
        .child(message)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_is_an_alert_others_are_status() {
        assert_eq!(DataStatusKind::Error.role(), Role::Alert);
        assert_eq!(DataStatusKind::Loading.role(), Role::Status);
        assert_eq!(DataStatusKind::NoResults.role(), Role::Status);
    }

    #[test]
    fn announcement_names_the_kind() {
        assert_eq!(
            DataStatusKind::NoResults.announcement("Nothing matches"),
            "No results: Nothing matches"
        );
    }
}
