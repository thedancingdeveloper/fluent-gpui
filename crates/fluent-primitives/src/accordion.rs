use std::{cell::RefCell, rc::Rc};

use gpui::{div, prelude::*, App, ElementId, IntoElement, KeyDownEvent, RenderOnce, Window};

use crate::{roving, Expander};

/// A group of expanders with roving keyboard focus.
///
/// Accordion state remains caller-owned by each [`Expander`]. In single-open
/// mode, opening one item calls the other expanded items' callbacks with
/// `false`. Set `collapsible(false)` to prevent the open item from closing.
#[derive(IntoElement)]
pub struct Accordion {
    id: ElementId,
    items: Vec<Expander>,
    multiple: bool,
    collapsible: bool,
}

impl Accordion {
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            items: Vec::new(),
            multiple: false,
            collapsible: true,
        }
    }

    pub fn item(mut self, item: Expander) -> Self {
        self.items.push(item);
        self
    }

    pub fn items(mut self, items: impl IntoIterator<Item = Expander>) -> Self {
        self.items = items.into_iter().collect();
        self
    }

    pub fn multiple(mut self, multiple: bool) -> Self {
        self.multiple = multiple;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
}

impl RenderOnce for Accordion {
    fn render(mut self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let enabled: Vec<bool> = self.items.iter().map(|item| !item.is_disabled()).collect();
        let handles = roving::handles(window, cx, &self.id, self.items.len());
        let stop = roving::stop_index(window, &handles, &enabled, None);
        let callbacks = Rc::new(RefCell::new(
            self.items
                .iter_mut()
                .map(Expander::take_on_toggle)
                .collect::<Vec<_>>(),
        ));
        let multiple = self.multiple;
        let collapsible = self.collapsible;
        let mut root = div().id(self.id);

        for (index, mut item) in self.items.into_iter().enumerate() {
            let item_callback = callbacks.borrow_mut()[index].take();
            let other_callbacks = callbacks.clone();
            let enabled_for_navigation = enabled.clone();
            let navigation_handles = handles.clone();
            let navigation = move |event: &KeyDownEvent, window: &mut Window, cx: &mut App| {
                if let Some(next) =
                    roving::target(event.keystroke.key.as_str(), index, &enabled_for_navigation)
                {
                    navigation_handles[next].focus(window, cx);
                    cx.stop_propagation();
                }
            };
            let callback = move |expanded: bool,
                                 event: &gpui::ClickEvent,
                                 window: &mut Window,
                                 cx: &mut App| {
                if !expanded && !collapsible {
                    return;
                }
                if expanded && !multiple {
                    for (other_index, callback) in other_callbacks.borrow().iter().enumerate() {
                        if other_index != index {
                            if let Some(callback) = callback {
                                callback(false, event, window, cx);
                            }
                        }
                    }
                }
                if let Some(callback) = &item_callback {
                    callback(expanded, event, window, cx);
                }
            };
            item = item.on_toggle(callback).with_focus(
                handles[index].clone(),
                stop == Some(index),
                navigation,
            );
            root = root.child(item);
        }
        root
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_to_single_open_and_collapsible() {
        let accordion = Accordion::new("settings");
        assert!(!accordion.multiple);
        assert!(accordion.collapsible);
        assert!(accordion.items.is_empty());
    }

    #[test]
    fn navigation_skips_disabled_items() {
        assert_eq!(roving::target("down", 0, &[true, false, true]), Some(2));
        assert_eq!(roving::target("up", 2, &[true, false, true]), Some(0));
    }
}
