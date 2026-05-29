use std::sync::Arc;

use gpui::{App, ClickEvent, SharedString, Window};

/// A click callback stored in ribbon item definitions.
pub type RibbonCallback = Arc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>;

/// A click callback for toggle buttons — first arg is the *new* desired state.
pub type RibbonToggleCallback = Arc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>;

// ---------------------------------------------------------------------------
// Item definitions
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
#[derive(Clone)]
pub enum RibbonItemDef {
    /// Tall button: 32px icon above a text label. Fills the full content-row height.
    LargeButton {
        label: SharedString,
        icon: SharedString,
        on_click: RibbonCallback,
    },
    /// Compact button: 16px icon + label side-by-side. Stacks 2–3 per column.
    Button {
        label: SharedString,
        icon: SharedString,
        on_click: RibbonCallback,
    },
    /// Compact icon-only button.
    IconButton {
        icon: SharedString,
        tooltip: SharedString,
        on_click: RibbonCallback,
    },
    /// Compact toggle button with external `selected` state.
    ToggleButton {
        label: SharedString,
        icon: SharedString,
        selected: bool,
        on_click: RibbonToggleCallback,
    },
    /// 2–3 compact items stacked in a single column.
    Stack(Vec<RibbonItemDef>),
    /// Thin vertical separator within a group.
    Separator,
}

// ---------------------------------------------------------------------------
// Group definition + builder
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RibbonGroupDef {
    pub name: SharedString,
    pub items: Vec<RibbonItemDef>,
}

/// Builder for a single ribbon group.
pub struct RibbonGroupBuilder(RibbonGroupDef);

impl RibbonGroupBuilder {
    pub(crate) fn new(name: impl Into<SharedString>) -> Self {
        Self(RibbonGroupDef {
            name: name.into(),
            items: vec![],
        })
    }

    pub fn large_button(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.items.push(RibbonItemDef::LargeButton {
            label: label.into(),
            icon: icon.into(),
            on_click: Arc::new(on_click),
        });
        self
    }

    pub fn button(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.items.push(RibbonItemDef::Button {
            label: label.into(),
            icon: icon.into(),
            on_click: Arc::new(on_click),
        });
        self
    }

    pub fn icon_button(
        mut self,
        icon: impl Into<SharedString>,
        tooltip: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.items.push(RibbonItemDef::IconButton {
            icon: icon.into(),
            tooltip: tooltip.into(),
            on_click: Arc::new(on_click),
        });
        self
    }

    pub fn toggle_button(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.items.push(RibbonItemDef::ToggleButton {
            label: label.into(),
            icon: icon.into(),
            selected,
            on_click: Arc::new(on_click),
        });
        self
    }

    /// Stack 2–3 compact items in a single column.
    pub fn stack(mut self, build: impl FnOnce(RibbonStackBuilder) -> RibbonStackBuilder) -> Self {
        let items = build(RibbonStackBuilder::new()).build();
        self.0.items.push(RibbonItemDef::Stack(items));
        self
    }

    pub fn separator(mut self) -> Self {
        self.0.items.push(RibbonItemDef::Separator);
        self
    }

    pub(crate) fn build(self) -> RibbonGroupDef {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Stack builder (compact items within a column)
// ---------------------------------------------------------------------------

pub struct RibbonStackBuilder(Vec<RibbonItemDef>);

impl RibbonStackBuilder {
    pub(crate) fn new() -> Self {
        Self(vec![])
    }

    pub fn button(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.push(RibbonItemDef::Button {
            label: label.into(),
            icon: icon.into(),
            on_click: Arc::new(on_click),
        });
        self
    }

    pub fn icon_button(
        mut self,
        icon: impl Into<SharedString>,
        tooltip: impl Into<SharedString>,
        on_click: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.push(RibbonItemDef::IconButton {
            icon: icon.into(),
            tooltip: tooltip.into(),
            on_click: Arc::new(on_click),
        });
        self
    }

    pub fn toggle_button(
        mut self,
        label: impl Into<SharedString>,
        icon: impl Into<SharedString>,
        selected: bool,
        on_click: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.0.push(RibbonItemDef::ToggleButton {
            label: label.into(),
            icon: icon.into(),
            selected,
            on_click: Arc::new(on_click),
        });
        self
    }

    pub(crate) fn build(self) -> Vec<RibbonItemDef> {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Tab definition + builder
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RibbonTabDef {
    pub name: SharedString,
    pub groups: Vec<RibbonGroupDef>,
}

pub struct RibbonTabBuilder(RibbonTabDef);

impl RibbonTabBuilder {
    pub(crate) fn new(name: impl Into<SharedString>) -> Self {
        Self(RibbonTabDef {
            name: name.into(),
            groups: vec![],
        })
    }

    pub fn group(
        mut self,
        name: impl Into<SharedString>,
        build: impl FnOnce(RibbonGroupBuilder) -> RibbonGroupBuilder,
    ) -> Self {
        let group = build(RibbonGroupBuilder::new(name)).build();
        self.0.groups.push(group);
        self
    }

    pub(crate) fn build(self) -> RibbonTabDef {
        self.0
    }
}

// ---------------------------------------------------------------------------
// Contextual tab definition
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct ContextualTabDef {
    pub name: SharedString,
    /// Whether this tab is currently visible in the tab strip.
    pub visible: bool,
    pub tab: RibbonTabDef,
}
