use crate::focus::{control_focus_handle, focus_ring, is_keyboard_focused, FluentFocus as _};
use fluent_core::TextRampExt;
use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window,
};

/// Fluent tag height (medium).
pub(crate) const CHIP_HEIGHT: f32 = 32.0;
/// Hit box of the dismiss button inside a chip.
pub(crate) const CHIP_DISMISS_BOX: f32 = 20.0;
/// Glyph size of the dismiss icon.
pub(crate) const CHIP_DISMISS_GLYPH: f32 = 12.0;

/// A dismissable tag / token chip.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    label: SharedString,
    on_dismiss: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Chip {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            on_dismiss: None,
        }
    }

    /// Registers a handler for dismiss events.
    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let dismiss_id = ElementId::NamedChild(std::sync::Arc::new(self.id), "dismiss".into());
        let focus = self
            .on_dismiss
            .is_some()
            .then(|| control_focus_handle(&dismiss_id, None, window, cx));
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;
        let spacing = &theme.spacing;
        let typography = &theme.typography;

        let on_dismiss = self.on_dismiss;
        let has_dismiss = on_dismiss.is_some();

        let mut el = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.xs))
            .px(px(spacing.sm))
            .h(px(CHIP_HEIGHT))
            .rounded(px(radii.pill))
            .bg(colors.neutral)
            .border_1()
            .border_color(colors.stroke_neutral)
            .text_ramp(&typography.caption)
            .text_color(colors.on_neutral)
            .child(self.label);

        if has_dismiss {
            let focus = focus.expect("dismissable chip has a focus handle");
            let ring = is_keyboard_focused(&focus, window);
            let dismiss = div()
                .id(dismiss_id)
                .track_focus(&focus)
                .key_context("Chip")
                .fluent_tab_stop(0)
                .size(px(CHIP_DISMISS_BOX))
                .rounded(px(radii.pill))
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .hover(move |s| s.bg(colors.neutral_hover))
                .active(move |s| s.bg(colors.neutral_selected))
                .on_click(move |ev, win, app| {
                    if let Some(handler) = &on_dismiss {
                        handler(ev, win, app);
                    }
                })
                .child(
                    gpui::svg()
                        .path("icons/dismiss.svg")
                        .size(px(CHIP_DISMISS_GLYPH))
                        .text_color(colors.on_neutral),
                );
            el = el.child(focus_ring(dismiss, ring, colors, px(radii.pill)));
        }

        el
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chip_is_fluent_medium_height() {
        assert_eq!(CHIP_HEIGHT, 32.0);
    }

    #[test]
    fn dismiss_box_fits_inside_chip_without_growing_it() {
        // 1px border top and bottom plus the dismiss box must fit the fixed height.
        const { assert!(CHIP_DISMISS_BOX + 2.0 <= CHIP_HEIGHT) };
        assert_eq!(CHIP_DISMISS_BOX, 20.0);
        assert_eq!(CHIP_DISMISS_GLYPH, 12.0);
    }
}
