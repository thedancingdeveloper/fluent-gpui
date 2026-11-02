use std::rc::Rc;

use fluent_core::ThemeProvider as _;
use gpui::{
    div, prelude::*, px, App, ClickEvent, ElementId, IntoElement, KeyDownEvent, RenderOnce,
    SharedString, Window,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Available tag appearance options for configuring the control.
pub enum TagAppearance {
    #[default]
    /// The neutral option.
    Neutral,
    /// The accent option.
    Accent,
    /// The informative option.
    Informative,
    /// The success option.
    Success,
    /// The warning option.
    Warning,
    /// The danger option.
    Danger,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Available tag shape options for configuring the control.
pub enum TagShape {
    #[default]
    /// The rounded option.
    Rounded,
    /// The circular option.
    Circular,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
/// Available tag size options for configuring the control.
pub enum TagSize {
    /// The small option.
    Small,
    #[default]
    /// The medium option.
    Medium,
    /// The large option.
    Large,
}

#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
/// A Fluent primitive for representing tag in an interface.
pub struct Tag {
    id: ElementId,
    label: SharedString,
    size: TagSize,
    shape: TagShape,
    appearance: TagAppearance,
    selected: bool,
    disabled: bool,
    dismissible: bool,
    on_dismiss: Option<Rc<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
    on_select: Option<Rc<dyn Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl Tag {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            size: TagSize::Medium,
            shape: TagShape::Rounded,
            appearance: TagAppearance::Neutral,
            selected: false,
            disabled: false,
            dismissible: false,
            on_dismiss: None,
            on_select: None,
        }
    }
    /// Sets the size configuration for this control.
    pub fn size(mut self, v: TagSize) -> Self {
        self.size = v;
        self
    }
    /// Sets the shape configuration for this control.
    pub fn shape(mut self, v: TagShape) -> Self {
        self.shape = v;
        self
    }
    /// Sets the appearance configuration for this control.
    pub fn appearance(mut self, v: TagAppearance) -> Self {
        self.appearance = v;
        self
    }
    /// Returns the current selected.
    pub fn selected(mut self, v: bool) -> Self {
        self.selected = v;
        self
    }
    /// Sets the disabled configuration for this control.
    pub fn disabled(mut self, v: bool) -> Self {
        self.disabled = v;
        self
    }
    /// Sets the dismissible configuration for this control.
    pub fn dismissible(mut self, v: bool) -> Self {
        self.dismissible = v;
        self
    }
    /// Registers a handler for dismiss events.
    pub fn on_dismiss(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_dismiss = Some(Rc::new(f));
        self
    }
    /// Registers a handler for select events.
    pub fn on_select(
        mut self,
        f: impl Fn(bool, &ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_select = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for Tag {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let t = cx.theme();
        let c = &t.colors;
        let (bg, fg) = match self.appearance {
            TagAppearance::Accent => (c.accent, c.on_accent),
            TagAppearance::Informative => (c.status_info_bg, c.status_info),
            TagAppearance::Success => (c.status_success_bg, c.status_success),
            TagAppearance::Warning => (c.status_warning_bg, c.status_warning),
            TagAppearance::Danger => (c.status_error_bg, c.status_error),
            TagAppearance::Neutral => (c.neutral, c.on_neutral),
        };
        let (pxv, pyv, fs) = match self.size {
            TagSize::Small => (6., 2., t.typography.caption.size),
            TagSize::Medium => (8., 4., t.typography.body.size),
            TagSize::Large => (10., 6., t.typography.body.size + 1.),
        };
        let radius = match self.shape {
            TagShape::Circular => t.radii.pill,
            TagShape::Rounded => t.radii.md,
        };
        let disabled = self.disabled;
        let selected = self.selected;
        let select = self.on_select.clone();
        let dismiss = self.on_dismiss.clone();
        let mut el = div()
            .id(self.id)
            .flex()
            .items_center()
            .gap(px(4.))
            .px(px(pxv))
            .py(px(pyv))
            .rounded(px(radius))
            .bg(if selected { c.neutral_selected } else { bg })
            .text_size(px(fs))
            .text_color(if disabled { c.on_neutral_disabled } else { fg })
            .child(self.label);
        if !disabled && (select.is_some() || self.dismissible) {
            let key_select = select.clone();
            let key_dismiss = dismiss.clone();
            el = el
                .focusable()
                .tab_stop(true)
                .cursor_pointer()
                .on_key_down(move |ev: &KeyDownEvent, w, a| {
                    if matches!(ev.keystroke.key.as_str(), "enter" | "space") {
                        if let Some(f) = &key_select {
                            f(!selected, &ClickEvent::default(), w, a);
                        }
                    }
                    if matches!(ev.keystroke.key.as_str(), "delete" | "backspace") {
                        if let Some(f) = &key_dismiss {
                            f(&ClickEvent::default(), w, a);
                        }
                    }
                })
                .when_some(select.clone(), |e, f| {
                    e.on_click(move |ev, w, a| f(!selected, ev, w, a))
                });
        }
        if !disabled && self.dismissible {
            el = el.child(div().child("×"));
        }
        el
    }
}

#[derive(IntoElement)]
/// A Fluent primitive for representing tag group in an interface.
pub struct TagGroup {
    id: ElementId,
    children: Vec<Tag>,
    size: TagSize,
}
impl TagGroup {
    /// Creates a new instance for use in a Fluent interface.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            children: Vec::new(),
            size: TagSize::Medium,
        }
    }
    /// Sets the tags configuration for this control.
    pub fn tags(mut self, tags: impl IntoIterator<Item = Tag>) -> Self {
        self.children = tags.into_iter().collect();
        self
    }
    /// Sets the tag configuration for this control.
    pub fn tag(mut self, tag: Tag) -> Self {
        self.children.push(tag);
        self
    }
    /// Sets the size configuration for this control.
    pub fn size(mut self, size: TagSize) -> Self {
        self.size = size;
        self
    }
}
impl RenderOnce for TagGroup {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        div()
            .id(self.id)
            .flex()
            .flex_wrap()
            .gap(px(8.))
            .children(self.children)
    }
}
