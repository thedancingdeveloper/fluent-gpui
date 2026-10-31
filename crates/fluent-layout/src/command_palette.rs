use std::{fmt, rc::Rc};

use fluent_core::{Theme, ThemeProvider as _};
use gpui::{
    div, prelude::*, px, App, Context, FocusHandle, FontWeight, IntoElement, KeyDownEvent, Render,
    SharedString, Window,
};

const EXACT_MATCH_SCORE: i32 = 100;
const PREFIX_MATCH_SCORE: i32 = 80;
const SUBSTRING_MATCH_SCORE: i32 = 60;
const SUBTITLE_MATCH_SCORE: i32 = 50;
const KEYWORD_MATCH_SCORE: i32 = 40;
const GROUP_MATCH_SCORE: i32 = 35;
const FUZZY_MATCH_SCORE: i32 = 20;

// ---------------------------------------------------------------------------
// Entry type
// ---------------------------------------------------------------------------

/// Action callback stored directly on a [`PaletteEntry`].
pub type PaletteEntryAction = Rc<dyn Fn(&PaletteEntry, &mut App) + 'static>;

/// A single searchable entry in the `CommandPalette`.
#[derive(Clone)]
pub struct PaletteEntry {
    pub id: SharedString,
    /// Primary display text — also used in filtering.
    pub label: SharedString,
    /// Secondary line (e.g. host/path/shortcut).
    pub subtitle: Option<SharedString>,
    /// Optional section label shown above grouped results.
    pub group: Option<SharedString>,
    /// Optional trailing shortcut/hint text.
    pub hint: Option<SharedString>,
    /// Optional icon SVG path.
    pub icon: Option<SharedString>,
    /// Extra keywords included in the fuzzy match but not displayed.
    pub keywords: Vec<SharedString>,
    /// Disabled entries are shown but cannot be selected.
    pub disabled: bool,
    /// Optional direct action. If present, it is preferred over the palette's
    /// fallback `on_select` callback.
    pub action: Option<PaletteEntryAction>,
}

impl fmt::Debug for PaletteEntry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PaletteEntry")
            .field("id", &self.id)
            .field("label", &self.label)
            .field("subtitle", &self.subtitle)
            .field("group", &self.group)
            .field("hint", &self.hint)
            .field("icon", &self.icon)
            .field("keywords", &self.keywords)
            .field("disabled", &self.disabled)
            .field("has_action", &self.action.is_some())
            .finish()
    }
}

impl PaletteEntry {
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            subtitle: None,
            group: None,
            hint: None,
            icon: None,
            keywords: vec![],
            disabled: false,
            action: None,
        }
    }

    pub fn subtitle(mut self, s: impl Into<SharedString>) -> Self {
        self.subtitle = Some(s.into());
        self
    }

    pub fn group(mut self, group: impl Into<SharedString>) -> Self {
        self.group = Some(group.into());
        self
    }

    pub fn hint(mut self, hint: impl Into<SharedString>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    pub fn shortcut(self, shortcut: impl Into<SharedString>) -> Self {
        self.hint(shortcut)
    }

    pub fn icon(mut self, path: impl Into<SharedString>) -> Self {
        self.icon = Some(path.into());
        self
    }

    pub fn keyword(mut self, kw: impl Into<SharedString>) -> Self {
        self.keywords.push(kw.into());
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    pub fn action(mut self, f: impl Fn(&PaletteEntry, &mut App) + 'static) -> Self {
        self.action = Some(Rc::new(f));
        self
    }
}

// ---------------------------------------------------------------------------
// CommandPalette entity
// ---------------------------------------------------------------------------

/// A Ctrl+K style floating search / action palette.
///
/// Include one instance as a child of your root view. Call `.show()` (or set
/// `open = true` + `cx.notify()`) to reveal it. It self-focuses on the next
/// frame and handles all keyboard input internally.
///
/// Results filter in real time as the user types (case-insensitive substring
/// and fuzzy match against `label`, `subtitle`, `group`, and `keywords`).
/// Entries can own direct action callbacks, or the palette can use `on_select`
/// as a fallback for older call sites.
#[allow(clippy::type_complexity)]
pub struct CommandPalette {
    pub open: bool,
    focus_handle: FocusHandle,
    query: String,
    entries: Vec<PaletteEntry>,
    filtered: Vec<usize>,
    selected: usize,
    max_results: Option<usize>,
    pub on_select: Option<Box<dyn Fn(&PaletteEntry, &mut App) + 'static>>,
}

impl CommandPalette {
    pub fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<Theme>(|_, cx| cx.notify()).detach();
        Self {
            open: false,
            focus_handle: cx.focus_handle(),
            query: String::new(),
            entries: vec![],
            filtered: vec![],
            selected: 0,
            max_results: None,
            on_select: None,
        }
    }

    // ---- Public API ----

    pub fn entries(mut self, entries: Vec<PaletteEntry>) -> Self {
        self.entries = entries;
        self.refilter();
        self
    }

    pub fn max_results(mut self, max_results: usize) -> Self {
        self.max_results = Some(max_results.max(1));
        self.refilter();
        self
    }

    pub fn set_entries(&mut self, entries: Vec<PaletteEntry>, cx: &mut Context<Self>) {
        self.entries = entries;
        self.refilter();
        cx.notify();
    }

    pub fn set_max_results(&mut self, max_results: Option<usize>, cx: &mut Context<Self>) {
        self.max_results = max_results.map(|max_results| max_results.max(1));
        self.refilter();
        cx.notify();
    }

    pub fn set_query(&mut self, query: impl Into<String>, cx: &mut Context<Self>) {
        self.query = query.into();
        self.refilter();
        cx.notify();
    }

    pub fn on_select(mut self, f: impl Fn(&PaletteEntry, &mut App) + 'static) -> Self {
        self.on_select = Some(Box::new(f));
        self
    }

    pub fn show(&mut self, cx: &mut Context<Self>) {
        self.open = true;
        self.query.clear();
        self.selected = 0;
        self.refilter();
        cx.notify();
    }

    pub fn hide(&mut self, cx: &mut Context<Self>) {
        self.open = false;
        cx.notify();
    }

    // ---- Internal ----

    fn refilter(&mut self) {
        self.filtered = filter_entries(&self.entries, &self.query, self.max_results);
        self.selected = first_enabled_row(&self.filtered, &self.entries)
            .unwrap_or_else(|| self.selected.min(self.filtered.len().saturating_sub(1)));
    }

    fn move_up(&mut self, cx: &mut Context<Self>) {
        if self.selected > 0 {
            for row in (0..self.selected).rev() {
                if self.is_row_enabled(row) {
                    self.selected = row;
                    break;
                }
            }
        }
        cx.notify();
    }

    fn move_down(&mut self, cx: &mut Context<Self>) {
        for row in self.selected.saturating_add(1)..self.filtered.len() {
            if self.is_row_enabled(row) {
                self.selected = row;
                break;
            }
        }
        cx.notify();
    }

    fn is_row_enabled(&self, row: usize) -> bool {
        self.filtered
            .get(row)
            .and_then(|&entry_idx| self.entries.get(entry_idx))
            .map(|entry| !entry.disabled)
            .unwrap_or(false)
    }

    fn confirm(&mut self, cx: &mut Context<Self>) {
        if let Some(&entry_idx) = self.filtered.get(self.selected) {
            if let Some(entry) = self.entries.get(entry_idx) {
                if entry.disabled {
                    return;
                }
                let entry = entry.clone();
                if let Some(action) = entry.action.clone() {
                    action(&entry, cx);
                } else if let Some(f) = &self.on_select {
                    f(&entry, cx);
                }
            }
        }
        self.hide(cx);
    }

    fn handle_key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        match ev.keystroke.key.as_str() {
            "escape" => self.hide(cx),
            "return" => self.confirm(cx),
            "up" => self.move_up(cx),
            "down" => self.move_down(cx),
            "backspace" => {
                self.query.pop();
                self.refilter();
                cx.notify();
            }
            _ => {
                // Printable character: use key_char (includes shifted characters)
                if let Some(ch) = &ev.keystroke.key_char {
                    if !ev.keystroke.modifiers.control && !ev.keystroke.modifiers.platform {
                        self.query.push_str(ch);
                        self.refilter();
                        cx.notify();
                    }
                }
            }
        }
    }
}

fn first_enabled_row(filtered: &[usize], entries: &[PaletteEntry]) -> Option<usize> {
    filtered
        .iter()
        .position(|&entry_idx| entries.get(entry_idx).is_some_and(|entry| !entry.disabled))
}

fn filter_entries(entries: &[PaletteEntry], query: &str, max_results: Option<usize>) -> Vec<usize> {
    let query = normalize_query(query);
    let limit = max_results.unwrap_or(usize::MAX);
    if query.is_empty() {
        return entries
            .iter()
            .enumerate()
            .map(|(index, _)| index)
            .take(limit)
            .collect();
    }

    let mut matches = entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| match_score(entry, &query).map(|score| (index, score)))
        .collect::<Vec<_>>();

    matches.sort_by(|(left_index, left_score), (right_index, right_score)| {
        right_score
            .cmp(left_score)
            .then_with(|| left_index.cmp(right_index))
    });

    matches
        .into_iter()
        .map(|(index, _)| index)
        .take(limit)
        .collect()
}

fn match_score(entry: &PaletteEntry, query: &str) -> Option<i32> {
    let label = entry.label.to_lowercase();
    let mut best = weighted_text_score(
        &label,
        query,
        EXACT_MATCH_SCORE,
        PREFIX_MATCH_SCORE,
        SUBSTRING_MATCH_SCORE,
        FUZZY_MATCH_SCORE,
    );

    if let Some(subtitle) = &entry.subtitle {
        best = best.max(weighted_text_score(
            &subtitle.to_lowercase(),
            query,
            SUBTITLE_MATCH_SCORE + 20,
            SUBTITLE_MATCH_SCORE + 10,
            SUBTITLE_MATCH_SCORE,
            0,
        ));
    }

    if let Some(group) = &entry.group {
        best = best.max(weighted_text_score(
            &group.to_lowercase(),
            query,
            GROUP_MATCH_SCORE + 20,
            GROUP_MATCH_SCORE + 10,
            GROUP_MATCH_SCORE,
            0,
        ));
    }

    for keyword in &entry.keywords {
        best = best.max(weighted_text_score(
            &keyword.to_lowercase(),
            query,
            KEYWORD_MATCH_SCORE + 20,
            KEYWORD_MATCH_SCORE + 10,
            KEYWORD_MATCH_SCORE,
            0,
        ));
    }

    (best > 0).then_some(best)
}

fn weighted_text_score(
    text: &str,
    query: &str,
    exact: i32,
    prefix: i32,
    substring: i32,
    fuzzy: i32,
) -> i32 {
    if text == query {
        exact
    } else if text.starts_with(query) {
        prefix
    } else if text.contains(query) {
        substring
    } else if fuzzy > 0 && fuzzy_matches(text, query) {
        fuzzy
    } else {
        0
    }
}

fn fuzzy_matches(text: &str, query: &str) -> bool {
    let mut text_chars = text.chars();
    for query_char in query.chars().filter(|ch| !ch.is_whitespace()) {
        if !text_chars.any(|text_char| text_char == query_char) {
            return false;
        }
    }
    true
}

fn normalize_query(query: &str) -> String {
    query
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if !self.open {
            return div();
        }

        // Self-focus on the first frame after opening
        if !self.focus_handle.is_focused(window) {
            window.focus(&self.focus_handle, cx);
        }

        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;
        let components = theme.components;

        let query_display = if self.query.is_empty() {
            "Type to search…".to_string()
        } else {
            format!("{}_", self.query) // underscore acts as cursor indicator
        };

        let input_text_color = if self.query.is_empty() {
            colors.on_subtle_disabled
        } else {
            colors.on_neutral
        };

        let selected = self.selected;
        let filtered = self.filtered.clone();
        let entries = &self.entries;

        // Build result rows
        let mut results = div().flex().flex_col().max_h(px(360.0)).overflow_hidden();

        let mut previous_group: Option<SharedString> = None;
        for (row_idx, &entry_idx) in filtered.iter().enumerate() {
            let Some(entry) = entries.get(entry_idx) else {
                continue;
            };
            let is_selected = row_idx == selected && !entry.disabled;
            let label = entry.label.clone();
            let subtitle = entry.subtitle.clone();
            let group = entry.group.clone();
            let hint = entry.hint.clone();
            let icon = entry.icon.clone();
            let disabled = entry.disabled;

            if group.is_some() && group != previous_group {
                if let Some(group_label) = group.clone() {
                    results = results.child(
                        div()
                            .px(px(spacing.md))
                            .pt(px(spacing.sm))
                            .pb(px(4.0))
                            .text_size(px(typography.caption.size))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.on_subtle)
                            .child(group_label),
                    );
                }
            }
            previous_group = group;

            let row_bg = if is_selected {
                colors.neutral_selected
            } else {
                colors.surface
            };
            let row_hover = colors.neutral_hover;
            let label_fg = if disabled {
                colors.on_subtle_disabled
            } else {
                colors.on_neutral
            };
            let sub_fg = if disabled {
                colors.on_subtle_disabled
            } else {
                colors.on_subtle
            };

            let row = cx.listener(move |p: &mut CommandPalette, _, _, cx| {
                p.selected = row_idx;
                if disabled {
                    cx.notify();
                } else {
                    p.confirm(cx);
                }
            });

            let mut row_el = div()
                .id(("palette-row", row_idx as u64))
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing.sm))
                .px(px(spacing.md))
                .py(px(spacing.sm))
                .bg(row_bg)
                .when(disabled, |d| d.opacity(0.72))
                .when(!disabled, |d| {
                    d.cursor_pointer()
                        .hover(move |s| s.bg(row_hover))
                        .on_click(row)
                });

            if let Some(icon_path) = icon {
                row_el = row_el.child(
                    gpui::svg()
                        .path(icon_path)
                        .size(px(components.command_palette_icon_slot))
                        .text_color(colors.on_neutral),
                );
            } else {
                row_el = row_el.child(div().size(px(components.command_palette_icon_slot)));
            }

            let mut text_col = div().flex().flex_col().flex_1().child(
                div()
                    .text_size(px(typography.body.size))
                    .text_color(label_fg)
                    .when(is_selected, |d| d.font_weight(FontWeight::SEMIBOLD))
                    .child(label),
            );

            if let Some(sub) = subtitle {
                text_col = text_col.child(
                    div()
                        .text_size(px(typography.caption.size))
                        .text_color(sub_fg)
                        .child(sub),
                );
            }

            row_el = row_el.child(text_col);
            if let Some(hint) = hint {
                row_el = row_el.child(
                    div()
                        .px(px(6.0))
                        .py(px(2.0))
                        .rounded(px(radii.sm))
                        .border_1()
                        .border_color(colors.stroke_neutral_subtle)
                        .text_size(px(typography.caption.size))
                        .text_color(sub_fg)
                        .child(hint),
                );
            }
            results = results.child(row_el);
        }

        // Empty state
        if filtered.is_empty() {
            results = results.child(
                div()
                    .px(px(spacing.md))
                    .py(px(spacing.lg))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(typography.body.size))
                    .text_color(colors.on_subtle_disabled)
                    .child("No results"),
            );
        }

        // Main palette panel
        let panel = div()
            .w(px(560.0))
            .bg(colors.surface)
            .border_1()
            .border_color(colors.stroke_neutral)
            .rounded(px(radii.lg))
            .shadow_lg()
            .overflow_hidden()
            // Search input row
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.sm))
                    .px(px(spacing.md))
                    .h(px(components.command_palette_search_height))
                    .border_b_1()
                    .border_color(colors.stroke_neutral_subtle)
                    .child(
                        gpui::svg()
                            .path("icons/search.svg")
                            .size(px(components.command_palette_icon_slot))
                            .text_color(colors.on_subtle),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(typography.body.size))
                            .text_color(input_text_color)
                            .child(query_display),
                    ),
            )
            .child(results)
            // Footer hint
            .child(
                div()
                    .flex()
                    .flex_row()
                    .items_center()
                    .justify_center()
                    .gap(px(spacing.md))
                    .px(px(spacing.md))
                    .h(px(components.command_palette_footer_height))
                    .border_t_1()
                    .border_color(colors.stroke_neutral_subtle)
                    .child(
                        div()
                            .text_size(px(typography.caption.size))
                            .text_color(colors.on_subtle_disabled)
                            .child("↑↓ navigate · ↵ select · Esc close"),
                    ),
            );

        // Backdrop + centered panel. No deferred() — like ModalHost, we rely on being the last
        // child of a `.relative()` parent (FluentRdpApp root) so absolute().inset_0() fills the
        // window correctly without nested-deferred issues.
        div()
            .absolute()
            .inset_0()
            .flex()
            .items_center()
            .justify_center()
            .pt(px(80.0)) // Appear in the upper-centre (like VS Code)
            .items_start()
            // Backdrop closes on click
            .child(
                div()
                    .id("palette-backdrop")
                    .absolute()
                    .inset_0()
                    .on_mouse_down(
                        gpui::MouseButton::Left,
                        cx.listener(|p: &mut CommandPalette, _, _, cx| {
                            cx.stop_propagation();
                            p.hide(cx);
                        }),
                    ),
            )
            .child(
                // Panel container: relative so it's above the backdrop
                div()
                    .id("palette-panel")
                    .relative()
                    .track_focus(&self.focus_handle)
                    .on_key_down(
                        cx.listener(|p: &mut CommandPalette, ev: &KeyDownEvent, _, cx| {
                            p.handle_key(ev, cx);
                        }),
                    )
                    .child(panel),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::{filter_entries, first_enabled_row, match_score, PaletteEntry};

    #[test]
    fn palette_entry_records_group_hint_disabled_and_action() {
        let entry = PaletteEntry::new("settings", "Open Settings")
            .group("Actions")
            .hint("Ctrl+,")
            .disabled(true)
            .action(|_, _| {});

        assert_eq!(
            entry.group.as_ref().map(ToString::to_string).as_deref(),
            Some("Actions")
        );
        assert_eq!(
            entry.hint.as_ref().map(ToString::to_string).as_deref(),
            Some("Ctrl+,")
        );
        assert!(entry.disabled);
        assert!(entry.action.is_some());
    }

    #[test]
    fn empty_query_preserves_order_and_optional_limit() {
        let entries = vec![
            PaletteEntry::new("a", "Archive"),
            PaletteEntry::new("b", "Delete"),
            PaletteEntry::new("c", "Reply"),
        ];

        assert_eq!(filter_entries(&entries, "", None), vec![0, 1, 2]);
        assert_eq!(filter_entries(&entries, "", Some(2)), vec![0, 1]);
    }

    #[test]
    fn weighted_filter_prefers_exact_prefix_and_substring_over_fuzzy() {
        let entries = vec![
            PaletteEntry::new("fuzzy", "Message Archive List"),
            PaletteEntry::new("substring", "Monthly Mail"),
            PaletteEntry::new("prefix", "Mail Settings"),
            PaletteEntry::new("exact", "Mail"),
        ];

        assert_eq!(filter_entries(&entries, "mail", None), vec![3, 2, 1, 0]);
    }

    #[test]
    fn filter_matches_subtitle_group_and_keywords() {
        let entries = vec![
            PaletteEntry::new("calendar", "Calendar").group("Navigate"),
            PaletteEntry::new("invoice", "Invoice").subtitle("Finance folder"),
            PaletteEntry::new("compose", "New Message").keyword("write"),
        ];

        assert_eq!(filter_entries(&entries, "navigate", None), vec![0]);
        assert_eq!(filter_entries(&entries, "finance", None), vec![1]);
        assert_eq!(filter_entries(&entries, "write", None), vec![2]);
    }

    #[test]
    fn first_enabled_row_skips_disabled_entries() {
        let entries = vec![
            PaletteEntry::new("compose", "New Message").disabled(true),
            PaletteEntry::new("settings", "Open Settings"),
        ];

        assert_eq!(first_enabled_row(&[0, 1], &entries), Some(1));
    }

    #[test]
    fn match_score_normalizes_whitespace_and_case() {
        let entry = PaletteEntry::new("advanced-search", "Advanced Search");
        assert!(match_score(&entry, "advanced search").is_some());
        assert!(filter_entries(&[entry], "  ADVANCED   SEARCH  ", None).len() == 1);
    }
}
