use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, AnyElement, App, ClickEvent, ElementId, IntoElement, RenderOnce, Window,
};

/// Canonical card recipe: `neutral` fill, 1px subtle border, `radii.md`. All
/// dashboard tiles, notification rows, and panel groups should start from
/// here so spacing/borders stay consistent.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct Card {
    id: Option<ElementId>,
    padding: f32,
    gap: f32,
    row: bool,
    flex_1: bool,
    items_center: bool,
    hoverable: bool,
    on_click: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    children: Vec<AnyElement>,
}

impl Card {
    pub fn new() -> Self {
        Self {
            id: None,
            padding: 12.0,
            gap: 0.0,
            row: false,
            flex_1: false,
            items_center: false,
            hoverable: false,
            on_click: None,
            children: Vec::new(),
        }
    }

    pub fn id(mut self, id: impl Into<ElementId>) -> Self {
        self.id = Some(id.into());
        self
    }

    /// Override default 12px padding. Common alternates: 16, 20.
    pub fn padding(mut self, p: f32) -> Self {
        self.padding = p;
        self
    }

    /// Gap between children when stacked vertically.
    pub fn gap(mut self, g: f32) -> Self {
        self.gap = g;
        self
    }

    pub fn row(mut self) -> Self {
        self.row = true;
        self
    }

    pub fn flex_1(mut self) -> Self {
        self.flex_1 = true;
        self
    }

    pub fn items_center(mut self) -> Self {
        self.items_center = true;
        self
    }

    pub fn hoverable(mut self) -> Self {
        self.hoverable = true;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Box::new(handler));
        self
    }

    pub fn child(mut self, child: impl IntoElement) -> Self {
        self.children.push(child.into_any_element());
        self
    }

    pub fn children(mut self, children: impl IntoIterator<Item = impl IntoElement>) -> Self {
        for c in children {
            self.children.push(c.into_any_element());
        }
        self
    }
}

impl Default for Card {
    fn default() -> Self {
        Self::new()
    }
}

impl RenderOnce for Card {
    fn render(self, _: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = &theme.colors;
        let radii = &theme.radii;
        let hover_bg = colors.neutral_hover;
        let hover_border = colors.stroke_neutral;
        let id = self.id;
        let row = self.row;
        let flex_1 = self.flex_1;
        let items_center = self.items_center;
        let hoverable = self.hoverable;
        let on_click = self.on_click;
        let card = div()
            .p(px(self.padding))
            .rounded(px(radii.md))
            .bg(colors.neutral)
            .border_1()
            .border_color(colors.stroke_neutral_subtle)
            .flex()
            .gap(px(self.gap));

        if id.is_some() || on_click.is_some() {
            let mut card = card.id(id.unwrap_or_else(|| "card-interactive".into()));
            card = if row {
                card.flex_row()
            } else {
                card.flex_col()
            };
            if flex_1 {
                card = card.flex_1();
            }
            if items_center {
                card = card.items_center();
            }
            if hoverable {
                card = card
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg).border_color(hover_border));
            }
            if let Some(handler) = on_click {
                card = card.on_click(move |ev, win, app| handler(ev, win, app));
            }
            for c in self.children {
                card = card.child(c);
            }
            card.into_any_element()
        } else {
            let mut card = if row {
                card.flex_row()
            } else {
                card.flex_col()
            };
            if flex_1 {
                card = card.flex_1();
            }
            if items_center {
                card = card.items_center();
            }
            if hoverable {
                card = card
                    .cursor_pointer()
                    .hover(move |s| s.bg(hover_bg).border_color(hover_border));
            }
            for c in self.children {
                card = card.child(c);
            }
            card.into_any_element()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_card_recipe() {
        let card = Card::new();

        assert!(card.id.is_none());
        assert_eq!(card.padding, 12.0);
        assert_eq!(card.gap, 0.0);
        assert!(!card.row);
        assert!(!card.flex_1);
        assert!(!card.items_center);
        assert!(!card.hoverable);
        assert!(card.on_click.is_none());
        assert_eq!(card.children.len(), 0);
    }

    #[test]
    fn builder_methods_record_configuration() {
        let card = Card::new()
            .id("quick-action")
            .padding(14.0)
            .gap(12.0)
            .row()
            .flex_1()
            .items_center()
            .hoverable()
            .on_click(|_, _, _| {})
            .child(div());

        assert!(card.id.is_some());
        assert_eq!(card.padding, 14.0);
        assert_eq!(card.gap, 12.0);
        assert!(card.row);
        assert!(card.flex_1);
        assert!(card.items_center);
        assert!(card.hoverable);
        assert!(card.on_click.is_some());
        assert_eq!(card.children.len(), 1);
    }
}
