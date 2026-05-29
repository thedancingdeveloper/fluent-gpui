use gpui::{App, ClickEvent, SharedString, Window};

use crate::{ContextMenuItem, PaletteEntry, ToolbarItem};

/// Shared command metadata for menus, toolbars, ribbons, and palettes.
#[derive(Clone, Debug)]
pub struct CommandDef {
    pub id: SharedString,
    pub label: SharedString,
    pub subtitle: Option<SharedString>,
    pub group: Option<SharedString>,
    pub icon: Option<SharedString>,
    pub shortcut: Option<SharedString>,
    pub keywords: Vec<SharedString>,
    pub disabled: bool,
    pub checked: Option<bool>,
}

impl CommandDef {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            subtitle: None,
            group: None,
            icon: None,
            shortcut: None,
            keywords: Vec::new(),
            disabled: false,
            checked: None,
        }
    }

    pub fn subtitle(mut self, subtitle: impl Into<SharedString>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn icon(mut self, icon: impl Into<SharedString>) -> Self {
        self.icon = Some(icon.into());
        self
    }

    pub fn shortcut(mut self, shortcut: impl Into<SharedString>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    pub fn keyword(mut self, keyword: impl Into<SharedString>) -> Self {
        self.keywords.push(keyword.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn checked(mut self, checked: bool) -> Self {
        self.checked = Some(checked);
        self
    }

    pub fn palette_entry(&self) -> PaletteEntry {
        let mut entry = PaletteEntry::new(self.id.clone(), self.label.clone());
        if let Some(subtitle) = &self.subtitle {
            entry = entry.subtitle(subtitle.clone());
        }
        if let Some(group) = &self.group {
            entry = entry.group(group.clone());
        }
        if let Some(shortcut) = &self.shortcut {
            entry = entry.shortcut(shortcut.clone());
        }
        if let Some(icon) = &self.icon {
            entry = entry.icon(icon.clone());
        }
        for keyword in &self.keywords {
            entry = entry.keyword(keyword.clone());
        }
        entry.disabled(self.disabled)
    }

    pub fn menu_item(
        &self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> ContextMenuItem {
        let mut item = if let Some(icon) = &self.icon {
            ContextMenuItem::action_with_icon(self.label.clone(), icon.clone(), on_click)
        } else {
            ContextMenuItem::action(self.label.clone(), on_click)
        };
        if let Some(shortcut) = &self.shortcut {
            item = item.shortcut(shortcut.clone());
        }
        item.disabled(self.disabled)
    }

    pub fn toolbar_item(
        &self,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> ToolbarItem {
        let item = if let Some(icon) = &self.icon {
            ToolbarItem::action_with_icon(
                self.id.clone(),
                self.label.clone(),
                icon.clone(),
                on_click,
            )
        } else {
            ToolbarItem::action(self.id.clone(), self.label.clone(), on_click)
        };
        item.disabled(self.disabled)
    }
}
