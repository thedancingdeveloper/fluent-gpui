use fluent_core::{
    ColorScheme, ComponentTokens, RadiiTokens, SpacingTokens, Theme, TypographyTokens,
};
use gpui::{div, prelude::*, px, svg, AnyElement, ElementId, SharedString};

use crate::context_menu::ContextMenuItem;

pub(crate) struct MenuRenderEnv {
    pub(crate) colors: ColorScheme,
    pub(crate) spacing: SpacingTokens,
    pub(crate) typography: TypographyTokens,
    pub(crate) radii: RadiiTokens,
    pub(crate) components: ComponentTokens,
}

impl MenuRenderEnv {
    pub(crate) fn from_theme(theme: &Theme) -> Self {
        Self {
            colors: theme.colors.clone(),
            spacing: theme.spacing,
            typography: theme.typography,
            radii: theme.radii,
            components: theme.components,
        }
    }
}

pub(crate) fn items_at_parent<'a>(
    items: &'a [ContextMenuItem],
    parent: &[usize],
) -> Option<&'a [ContextMenuItem]> {
    if parent.is_empty() {
        return Some(items);
    }

    let mut current = items;
    for &idx in parent {
        match current.get(idx) {
            Some(ContextMenuItem::Submenu {
                items,
                disabled: false,
                ..
            }) => current = items,
            _ => return None,
        }
    }
    Some(current)
}

pub(crate) fn item_at_path<'a>(
    items: &'a [ContextMenuItem],
    path: &[usize],
) -> Option<&'a ContextMenuItem> {
    let (&last, parent) = path.split_last()?;
    items_at_parent(items, parent)?.get(last)
}

pub(crate) fn enabled_item_at_path<'a>(
    items: &'a [ContextMenuItem],
    path: &[usize],
) -> Option<&'a ContextMenuItem> {
    let item = item_at_path(items, path)?;
    item.enabled().then_some(item)
}

pub(crate) fn parent_path(path: &[usize]) -> Vec<usize> {
    path.get(..path.len().saturating_sub(1))
        .unwrap_or_default()
        .to_vec()
}

pub(crate) fn path_starts_with(path: &[usize], prefix: &[usize]) -> bool {
    prefix.len() <= path.len() && path[..prefix.len()] == *prefix
}

pub(crate) fn first_enabled_path(
    items: &[ContextMenuItem],
    parent: &[usize],
) -> Option<Vec<usize>> {
    items.iter().position(ContextMenuItem::enabled).map(|idx| {
        let mut path = parent.to_vec();
        path.push(idx);
        path
    })
}

pub(crate) fn last_enabled_path(items: &[ContextMenuItem], parent: &[usize]) -> Option<Vec<usize>> {
    items.iter().rposition(ContextMenuItem::enabled).map(|idx| {
        let mut path = parent.to_vec();
        path.push(idx);
        path
    })
}

pub(crate) fn ensure_selection(items: &[ContextMenuItem], selected_path: &mut Vec<usize>) {
    let selected_enabled = item_at_path(items, selected_path)
        .map(ContextMenuItem::enabled)
        .unwrap_or(false);
    if !selected_enabled {
        *selected_path = first_enabled_path(items, &[]).unwrap_or_default();
    }
}

pub(crate) fn move_selection(
    items: &[ContextMenuItem],
    selected_path: &mut Vec<usize>,
    delta: isize,
) -> bool {
    ensure_selection(items, selected_path);

    let parent = parent_path(selected_path);
    let current = selected_path.last().copied().unwrap_or(0);
    let Some(siblings) = items_at_parent(items, &parent) else {
        return false;
    };

    let enabled: Vec<usize> = siblings
        .iter()
        .enumerate()
        .filter_map(|(idx, item)| item.enabled().then_some(idx))
        .collect();
    if enabled.is_empty() {
        return false;
    }

    let pos = enabled.iter().position(|idx| *idx == current).unwrap_or(0);
    let next = if delta < 0 {
        if pos == 0 {
            enabled.len() - 1
        } else {
            pos - 1
        }
    } else {
        (pos + 1) % enabled.len()
    };

    *selected_path = parent;
    selected_path.push(enabled[next]);
    true
}

pub(crate) fn enter_submenu(
    items: &[ContextMenuItem],
    selected_path: &mut Vec<usize>,
    open_path: &mut Vec<usize>,
) -> bool {
    let Some(ContextMenuItem::Submenu {
        items: submenu,
        disabled,
        ..
    }) = item_at_path(items, selected_path)
    else {
        return false;
    };
    if *disabled {
        return false;
    }

    if let Some(first) = first_enabled_path(submenu, selected_path) {
        *open_path = selected_path.clone();
        *selected_path = first;
        true
    } else {
        false
    }
}

pub(crate) fn leave_submenu(selected_path: &mut Vec<usize>, open_path: &mut Vec<usize>) {
    if selected_path.len() <= 1 {
        open_path.clear();
    } else {
        let parent = parent_path(selected_path);
        *selected_path = parent.clone();
        *open_path = parent_path(&parent);
    }
}

fn row_id(id_prefix: &'static str, path: &[usize]) -> ElementId {
    let mut id = ElementId::Name(id_prefix.into());
    for idx in path {
        id = (id, idx.to_string()).into();
    }
    id
}

pub(crate) fn row_base(
    id_prefix: &'static str,
    path: &[usize],
    selected: bool,
    fg: gpui::Hsla,
    env: &MenuRenderEnv,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(row_id(id_prefix, path))
        .flex()
        .flex_row()
        .items_center()
        .gap(px(env.spacing.sm))
        .px(px(env.spacing.md))
        .h(px(env.components.menu_item_height))
        .bg(if selected {
            env.colors.neutral_selected
        } else {
            env.colors.surface
        })
        .text_size(px(env.typography.body.size))
        .text_color(fg)
}

pub(crate) fn icon_slot(
    icon: Option<SharedString>,
    fg: gpui::Hsla,
    env: &MenuRenderEnv,
) -> AnyElement {
    match icon {
        Some(icon) => svg()
            .path(icon)
            .size(px(env.components.popup_icon_slot))
            .text_color(fg)
            .into_any_element(),
        None => div()
            .size(px(env.components.popup_icon_slot))
            .into_any_element(),
    }
}

#[cfg(test)]
mod tests {
    use gpui::ClickEvent;

    use super::*;

    fn item(label: &'static str) -> ContextMenuItem {
        ContextMenuItem::action(label, |_: &ClickEvent, _, _| {})
    }

    #[test]
    fn moves_selection_across_enabled_siblings() {
        let items = vec![
            item("One"),
            ContextMenuItem::separator(),
            item("Two"),
            item("Three").disabled(true),
        ];
        let mut selected = vec![0];

        assert!(move_selection(&items, &mut selected, 1));
        assert_eq!(selected, vec![2]);

        assert!(move_selection(&items, &mut selected, 1));
        assert_eq!(selected, vec![0]);

        assert!(move_selection(&items, &mut selected, -1));
        assert_eq!(selected, vec![2]);
    }

    #[test]
    fn navigates_submenus_by_path() {
        let items = vec![ContextMenuItem::submenu(
            "File",
            vec![item("New"), item("Open")],
        )];
        let mut selected = vec![0];
        let mut open = Vec::new();

        assert!(enter_submenu(&items, &mut selected, &mut open));
        assert_eq!(open, vec![0]);
        assert_eq!(selected, vec![0, 0]);

        leave_submenu(&mut selected, &mut open);
        assert_eq!(selected, vec![0]);
        assert!(open.is_empty());
    }

    #[test]
    fn disabled_items_and_disabled_submenu_children_are_not_actionable() {
        let items = vec![
            item("Disabled").disabled(true),
            ContextMenuItem::submenu("Disabled submenu", vec![item("Child")]).disabled(true),
        ];

        assert!(enabled_item_at_path(&items, &[0]).is_none());
        assert!(enabled_item_at_path(&items, &[1, 0]).is_none());
        assert!(items_at_parent(&items, &[1]).is_none());
    }

    #[test]
    fn row_ids_include_the_full_menu_path() {
        assert_eq!(row_id("menu-item", &[1]).to_string(), "menu-item-1");
        assert_eq!(row_id("menu-item", &[1, 2]).to_string(), "menu-item-1-2");
    }
}
