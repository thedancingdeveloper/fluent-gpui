use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, RenderOnce, SharedString, Window,
};

/// A dismissable tag / token chip.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Chip {
    id: ElementId,
    label: SharedString,
    on_dismiss: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Chip {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            on_dismiss: None,
        }
    }

    pub fn on_dismiss(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_dismiss = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for Chip {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
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
            .py(px(spacing.xs))
            .rounded(px(radii.pill))
            .bg(colors.neutral)
            .border_1()
            .border_color(colors.stroke_neutral)
            .text_size(px(typography.caption.size))
            .text_color(colors.on_neutral)
            .child(self.label);

        if has_dismiss {
            let dismiss_id = ElementId::NamedChild(Box::new(self.id), "dismiss".into());
            el = el.child(
                div()
                    .id(dismiss_id)
                    .size(px(14.0))
                    .rounded(px(9999.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .cursor_pointer()
                    .hover(move |s| s.bg(colors.neutral_hover))
                    .on_click(move |ev, win, app| {
                        if let Some(handler) = &on_dismiss {
                            handler(ev, win, app);
                        }
                    })
                    .child(
                        gpui::svg()
                            .path("icons/dismiss.svg")
                            .size(px(10.0))
                            .text_color(colors.on_neutral),
                    ),
            );
        }

        el
    }
}
