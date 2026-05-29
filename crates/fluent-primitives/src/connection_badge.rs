use std::time::Duration;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, AnimationExt, App, ElementId, Hsla, IntoElement, RenderOnce, SharedString,
    Window,
};
use gpui::{pulsating_between, Animation};

/// State of a [`ConnectionBadge`] — drives label, colour pair and whether the
/// status dot pulses.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConnectionBadgeState {
    Disconnected,
    Pairing,
    Paired,
    Error,
}

/// A small status pill (`[● label]`) for the top-bar / status indicator. The
/// dot pulses on `Pairing` / `Paired`, sits static on `Disconnected` / `Error`.
#[derive(IntoElement)]
pub struct ConnectionBadge {
    id: ElementId,
    state: ConnectionBadgeState,
    label: Option<SharedString>,
}

impl ConnectionBadge {
    pub fn new(id: impl Into<ElementId>, state: ConnectionBadgeState) -> Self {
        Self {
            id: id.into(),
            state,
            label: None,
        }
    }

    /// Override the default text — useful for surfacing the peer name or
    /// "Pairing with Pixel 8…".
    pub fn label(mut self, label: impl Into<SharedString>) -> Self {
        self.label = Some(label.into());
        self
    }
}

pub(crate) fn tokens(
    state: ConnectionBadgeState,
    colors: &fluent_core::ColorScheme,
) -> (Hsla, Hsla, &'static str) {
    match state {
        ConnectionBadgeState::Disconnected => (colors.neutral, colors.on_subtle, "Disconnected"),
        ConnectionBadgeState::Pairing => {
            (colors.status_warning_bg, colors.status_warning, "Pairing…")
        }
        ConnectionBadgeState::Paired => (
            colors.status_success_bg,
            colors.status_success,
            "Paired · Encrypted",
        ),
        ConnectionBadgeState::Error => (colors.status_error_bg, colors.status_error, "Error"),
    }
}

impl RenderOnce for ConnectionBadge {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;
        let typography = &theme.typography;

        let (bg, fg, default_label) = tokens(self.state, colors);
        let label = self.label.unwrap_or_else(|| default_label.into());

        let pulses = matches!(
            self.state,
            ConnectionBadgeState::Pairing | ConnectionBadgeState::Paired
        );

        let dot_color = fg;
        let dot = div().size(px(8.0)).rounded(px(radii.pill)).bg(dot_color);

        let dot_el: gpui::AnyElement = if pulses {
            dot.with_animation(
                self.id.clone(),
                Animation::new(Duration::from_millis(1600))
                    .repeat()
                    .with_easing(pulsating_between(0.35, 1.0)),
                move |this, delta| {
                    let mut c = dot_color;
                    c.a = delta;
                    this.bg(c)
                },
            )
            .into_any_element()
        } else {
            dot.into_any_element()
        };

        div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(6.0))
            .px(px(10.0))
            .py(px(3.0))
            .rounded(px(radii.pill))
            .bg(bg)
            .text_color(fg)
            .text_size(px(typography.caption.size))
            .child(dot_el)
            .child(div().font_weight(gpui::FontWeight::SEMIBOLD).child(label))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fluent_core::ColorScheme;

    #[test]
    fn token_map_uses_distinct_palettes_per_state() {
        let scheme = ColorScheme::dark();
        let disc = tokens(ConnectionBadgeState::Disconnected, &scheme);
        let pair = tokens(ConnectionBadgeState::Pairing, &scheme);
        let paired = tokens(ConnectionBadgeState::Paired, &scheme);
        let err = tokens(ConnectionBadgeState::Error, &scheme);
        // Backgrounds differ across all four states.
        assert_ne!(disc.0, pair.0);
        assert_ne!(pair.0, paired.0);
        assert_ne!(paired.0, err.0);
        assert_ne!(disc.0, err.0);
    }

    #[test]
    fn default_labels_match_states() {
        let scheme = ColorScheme::dark();
        assert_eq!(
            tokens(ConnectionBadgeState::Paired, &scheme).2,
            "Paired · Encrypted"
        );
        assert_eq!(tokens(ConnectionBadgeState::Pairing, &scheme).2, "Pairing…");
        assert_eq!(tokens(ConnectionBadgeState::Error, &scheme).2, "Error");
    }
}
