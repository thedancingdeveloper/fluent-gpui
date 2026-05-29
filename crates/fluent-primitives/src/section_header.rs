use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, AnyElement, App, FontWeight, IntoElement, RenderOnce, SharedString, Window,
};

/// A small section / card header — title on the left, optional action on the
/// right. Used inside cards (Overview tiles, Notifications header).
#[derive(IntoElement)]
pub struct SectionHeader {
    title: SharedString,
    action: Option<AnyElement>,
}

impl SectionHeader {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            action: None,
        }
    }

    /// Trailing action — typically a `Button::Subtle` "See all" / "Open".
    pub fn action(mut self, action: impl IntoElement) -> Self {
        self.action = Some(action.into_any_element());
        self
    }
}

impl RenderOnce for SectionHeader {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let typography = &cx.theme().typography;
        let colors = &cx.theme().colors;
        let spacing = &cx.theme().spacing;
        let mut row = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.md))
            .mb(px(spacing.md + spacing.xs))
            .child(
                div()
                    .text_color(colors.on_neutral)
                    .text_size(px(typography.body.size))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title),
            );
        row = row.child(div().flex_1());
        if let Some(action) = self.action {
            row = row.child(action);
        }
        row
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_sets_title_without_action() {
        let header = SectionHeader::new("Storage");

        assert_eq!(header.title.as_ref(), "Storage");
        assert!(header.action.is_none());
    }

    #[test]
    fn action_records_trailing_element() {
        let header = SectionHeader::new("Recent").action(div());

        assert_eq!(header.title.as_ref(), "Recent");
        assert!(header.action.is_some());
    }
}
