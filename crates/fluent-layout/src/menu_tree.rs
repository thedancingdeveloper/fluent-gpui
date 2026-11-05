use fluent_core::TextRampExt;
use fluent_core::{
    ColorScheme, ComponentTokens, RadiiTokens, SpacingTokens, Theme, TypographyTokens,
};
use std::time::Duration;

use gpui::{
    deferred, div, point, prelude::*, px, svg, AnyElement, App, Bounds, Display, Edges, Element,
    ElementId, GlobalElementId, InspectorElementId, Keystroke, LayoutId, Pixels, Position,
    SharedString, Style, Window,
};

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

/// Splits a menu label into its display text and access key.
///
/// `&` marks the access key (`"&File"` shows "File" with F underlined); `&&`
/// is a literal ampersand. Returns the display text and the access key's
/// `(byte offset, char)` within it.
pub(crate) fn parse_label(label: &str) -> (String, Option<(usize, char)>) {
    let mut display = String::with_capacity(label.len());
    let mut key = None;
    let mut chars = label.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '&' {
            match chars.peek().copied() {
                Some('&') => {
                    chars.next();
                    display.push('&');
                }
                Some(next) if key.is_none() && !next.is_whitespace() => {
                    key = Some((display.len(), next));
                }
                _ => {}
            }
        } else {
            display.push(c);
        }
    }
    (display, key)
}

pub(crate) fn access_key(label: &str) -> Option<char> {
    parse_label(label)
        .1
        .map(|(_, c)| c.to_lowercase().next().unwrap_or(c))
}

/// Label text with the access key underlined when `show_underline` is set.
pub(crate) fn label_element(label: &str, show_underline: bool) -> AnyElement {
    let (display, key) = parse_label(label);
    match key {
        Some((at, c)) if show_underline => {
            let end = at + c.len_utf8();
            div()
                .flex()
                .flex_row()
                .child(display[..at].to_string())
                .child(div().underline().child(display[at..end].to_string()))
                .child(display[end..].to_string())
                .into_any_element()
        }
        _ => div().child(display).into_any_element(),
    }
}

fn item_label(item: &ContextMenuItem) -> Option<&SharedString> {
    match item {
        ContextMenuItem::Action { label, .. }
        | ContextMenuItem::Checkbox { label, .. }
        | ContextMenuItem::Radio { label, .. }
        | ContextMenuItem::Submenu { label, .. } => Some(label),
        ContextMenuItem::Separator => None,
    }
}

/// Finds the enabled sibling under `parent` whose access key is `ch`,
/// cycling from just after `current` so repeated presses visit each match.
pub(crate) fn find_access_key(
    items: &[ContextMenuItem],
    parent: &[usize],
    ch: char,
    current: Option<usize>,
) -> Option<Vec<usize>> {
    let ch = ch.to_lowercase().next().unwrap_or(ch);
    let siblings = items_at_parent(items, parent)?;
    let start = current.map_or(0, |c| c + 1);
    (0..siblings.len())
        .map(|offset| (start + offset) % siblings.len())
        .find(|&idx| {
            let item = &siblings[idx];
            item.enabled() && item_label(item).and_then(|l| access_key(l)) == Some(ch)
        })
        .map(|idx| {
            let mut path = parent.to_vec();
            path.push(idx);
            path
        })
}

/// The access character of an unmodified or Alt-only keystroke.
pub(crate) fn typed_access_char(keystroke: &Keystroke) -> Option<char> {
    let m = &keystroke.modifiers;
    if m.control || m.platform || m.function {
        return None;
    }
    let mut chars = keystroke.key.chars();
    let c = chars.next()?;
    (chars.next().is_none() && c.is_alphanumeric()).then_some(c)
}

/// Shift+F10 or the dedicated Menu key.
pub fn is_context_menu_key(keystroke: &Keystroke) -> bool {
    let m = &keystroke.modifiers;
    match keystroke.key.as_str() {
        "menu" => !m.control && !m.alt && !m.platform,
        "f10" => m.shift && !m.control && !m.alt && !m.platform,
        _ => false,
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

/// How long the pointer may rest on a sibling row before the open submenu
/// closes in its favour. A diagonal move from a submenu-trigger row into its
/// submenu crosses sibling rows on the way; without the delay each crossing
/// closes the submenu it is heading for.
pub(crate) const SUBMENU_CLOSE_DELAY: Duration = Duration::from_millis(200);

/// A hover that would close the open submenu, waiting out
/// [`SUBMENU_CLOSE_DELAY`].
#[derive(Default)]
pub(crate) struct HoverIntent {
    pending: Option<PendingHover>,
    epoch: u64,
}

struct PendingHover {
    select: Vec<usize>,
    open: Vec<usize>,
    /// The selection when the hover was deferred; if it has changed (keyboard
    /// navigation) the deferred hover is stale and is dropped.
    from_selected: Vec<usize>,
}

/// A menu whose rows react to pointer hover.
pub(crate) trait HoverMenu: Sized + 'static {
    fn is_closing(&self) -> bool;
    fn hover_intent(&mut self) -> &mut HoverIntent;
    fn paths_mut(&mut self) -> (&mut Vec<usize>, &mut Vec<usize>);
}

/// The pointer moved onto the row at `select`, whose own menu level is `open`.
///
/// Moving onto a row that would close an open submenu waits
/// [`SUBMENU_CLOSE_DELAY`] first and is dropped if the pointer reaches any
/// other row meanwhile, such as one inside the submenu. Everything else
/// applies at once.
pub(crate) fn hover_row<T: HoverMenu>(
    menu: &mut T,
    select: Vec<usize>,
    open: Vec<usize>,
    cx: &mut gpui::Context<T>,
) {
    if menu.is_closing() {
        return;
    }
    let (selected_path, open_path) = menu.paths_mut();
    let closes_submenu = open_path.len() > open.len() && path_starts_with(open_path, &open);
    if !closes_submenu {
        *selected_path = select;
        *open_path = open;
        let intent = menu.hover_intent();
        intent.pending = None;
        intent.epoch = intent.epoch.wrapping_add(1);
        cx.notify();
        return;
    }

    let from_selected = selected_path.clone();
    let intent = menu.hover_intent();
    if intent.pending.as_ref().is_some_and(|p| p.select == select) {
        return;
    }
    intent.epoch = intent.epoch.wrapping_add(1);
    let epoch = intent.epoch;
    intent.pending = Some(PendingHover {
        select,
        open,
        from_selected,
    });

    cx.spawn(move |this: gpui::WeakEntity<T>, cx: &mut gpui::AsyncApp| {
        let mut cx = cx.clone();
        async move {
            cx.background_executor().timer(SUBMENU_CLOSE_DELAY).await;
            this.update(&mut cx, move |menu, cx| {
                if menu.is_closing() || menu.hover_intent().epoch != epoch {
                    return;
                }
                let Some(pending) = menu.hover_intent().pending.take() else {
                    return;
                };
                let (selected_path, open_path) = menu.paths_mut();
                if *selected_path == pending.from_selected {
                    *selected_path = pending.select;
                    *open_path = pending.open;
                    cx.notify();
                }
            })
            .ok();
        }
    })
    .detach();
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
    role: gpui::Role,
    label: &SharedString,
) -> gpui::Stateful<gpui::Div> {
    div()
        .id(row_id(id_prefix, path))
        .debug_selector(|| format!("{id_prefix}-row-{path:?}"))
        .role(role)
        .aria_label(label.clone())
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
        .text_ramp(&env.typography.body)
        .text_color(fg)
}

/// Gap kept between a flyout submenu and the window edge.
pub(crate) const SUBMENU_WINDOW_MARGIN: f32 = 8.0;

/// Lays a submenu beside the submenu-trigger row it is a child of.
///
/// The element fills the row (`absolute`, all insets zero), so its prepaint
/// bounds are the row's bounds and no row offset is tracked by hand. The
/// submenu opens to the right of the row with its first item level with the
/// row, overlapping the parent's border by one pixel as Zed's submenus do. When
/// it would run past the right window edge and fits on the left, it opens to the
/// left instead. It is then snapped inside the window with
/// [`SUBMENU_WINDOW_MARGIN`], like `anchored().snap_to_window_with_margin`.
struct SubmenuPlacement {
    child: Option<AnyElement>,
    /// How far above the row the submenu's top edge sits (the parent's top
    /// border and padding), so the first items align.
    top_inset: Pixels,
}

impl Element for SubmenuPlacement {
    type RequestLayoutState = LayoutId;
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let child_id = self
            .child
            .as_mut()
            .map(|child| child.request_layout(window, cx))
            .expect("submenu placement child");
        let zero = gpui::Length::Definite(px(0.).into());
        let style = Style {
            position: Position::Absolute,
            display: Display::Flex,
            inset: Edges {
                top: zero,
                right: zero,
                bottom: zero,
                left: zero,
            },
            ..Style::default()
        };
        (window.request_layout(style, [child_id], cx), child_id)
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        row: Bounds<Pixels>,
        child_id: &mut LayoutId,
        window: &mut Window,
        cx: &mut App,
    ) {
        let child_bounds = window.layout_bounds(*child_id);
        let size = child_bounds.size;
        let viewport = window.viewport_size();
        let inset = window.client_inset().unwrap_or(px(0.));
        let margin = px(SUBMENU_WINDOW_MARGIN) + inset;
        let overlap = px(1.);

        let mut x = row.right() - overlap;
        if x + size.width > viewport.width - margin && row.left() + overlap - size.width >= margin {
            x = row.left() + overlap - size.width;
        }
        let mut y = row.top() - self.top_inset;

        if x + size.width > viewport.width - margin {
            x = viewport.width - margin - size.width;
        }
        if x < margin {
            x = margin;
        }
        if y + size.height > viewport.height - margin {
            y = viewport.height - margin - size.height;
        }
        if y < margin {
            y = margin;
        }

        let offset = point(x, y) - child_bounds.origin;
        let offset = point(offset.x.round(), offset.y.round());
        if let Some(child) = self.child.as_mut() {
            window.with_element_offset(offset, |window| child.prepaint(window, cx));
        }
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _child_id: &mut LayoutId,
        _prepaint: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) {
        if let Some(child) = self.child.as_mut() {
            child.paint(window, cx);
        }
    }
}

impl IntoElement for SubmenuPlacement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

/// The deferred flyout for an open submenu. Add it as a child of the
/// submenu-trigger row. `priority` must exceed the parent menu's so a nested
/// flyout paints over it.
pub(crate) fn submenu_flyout(
    submenu: AnyElement,
    env: &MenuRenderEnv,
    priority: usize,
) -> AnyElement {
    deferred(SubmenuPlacement {
        child: Some(div().absolute().child(submenu).into_any_element()),
        // Parent border (1) plus its vertical padding.
        top_inset: px(env.spacing.xs + 1.0),
    })
    .with_priority(priority)
    .into_any_element()
}

/// A window-sized layer that closes its menu on any click outside the menu.
/// Place it inside a `deferred` overlay, before the menu so the menu paints
/// above it.
pub(crate) fn window_backdrop(
    id: &'static str,
    window: &Window,
    on_close: impl Fn(&mut Window, &mut App) + 'static,
) -> AnyElement {
    let viewport = window.viewport_size();
    gpui::anchored()
        .position(point(px(0.), px(0.)))
        .child(
            div()
                .id(id)
                .w(viewport.width)
                .h(viewport.height)
                .on_mouse_down(gpui::MouseButton::Left, move |_, window, cx| {
                    cx.stop_propagation();
                    on_close(window, cx);
                }),
        )
        .into_any_element()
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
    fn parses_access_keys() {
        assert_eq!(parse_label("&File"), ("File".into(), Some((0, 'F'))));
        assert_eq!(parse_label("Save &As"), ("Save As".into(), Some((5, 'A'))));
        assert_eq!(parse_label("Tom && Jerry"), ("Tom & Jerry".into(), None));
        assert_eq!(parse_label("Plain"), ("Plain".into(), None));
        assert_eq!(access_key("E&xit"), Some('x'));
    }

    #[test]
    fn access_keys_cycle_among_enabled_siblings() {
        let items = vec![
            item("&Save"),
            item("S&ave as").disabled(true),
            item("Se&nd"),
            item("&Sync"),
        ];
        assert_eq!(find_access_key(&items, &[], 's', None), Some(vec![0]));
        assert_eq!(find_access_key(&items, &[], 'S', Some(0)), Some(vec![3]));
        assert_eq!(find_access_key(&items, &[], 's', Some(3)), Some(vec![0]));
        assert_eq!(find_access_key(&items, &[], 'a', None), None);
        assert_eq!(find_access_key(&items, &[], 'n', None), Some(vec![2]));
    }

    #[test]
    fn recognises_context_menu_keys() {
        assert!(is_context_menu_key(&Keystroke::parse("shift-f10").unwrap()));
        assert!(is_context_menu_key(&Keystroke::parse("menu").unwrap()));
        assert!(!is_context_menu_key(&Keystroke::parse("f10").unwrap()));
        assert!(!is_context_menu_key(
            &Keystroke::parse("ctrl-menu").unwrap()
        ));
    }

    #[test]
    fn row_ids_include_the_full_menu_path() {
        assert_eq!(row_id("menu-item", &[1]).to_string(), "menu-item-1");
        assert_eq!(row_id("menu-item", &[1, 2]).to_string(), "menu-item-1-2");
    }
}
