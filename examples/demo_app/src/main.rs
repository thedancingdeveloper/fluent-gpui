use fluent_app::FluentApp;
use fluent_core::ThemeProvider as _;
use fluent_layout::{
    CommandPalette, ConfirmModal, ContextMenu, DockPanel, DockPosition, MenuBar, MenuItemDef,
    ModalHost, ModalStack, PaletteEntry, Pane, SettingsNav, SettingsNavItem, SettingsNavSection,
    TabItem, TabStrip, Workspace,
};
use fluent_ribbon::RibbonBar;
use gpui::{
    actions, div, prelude::*, px, App, Context, Entity, FontWeight, IntoElement, KeyBinding,
    MouseButton, MouseDownEvent, Render, SharedString, Window,
};

actions!(demo_app, [TogglePalette, ToggleLightTheme, ToggleDarkTheme]);

// ---------------------------------------------------------------------------
// Data model
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Priority {
    fn label(self) -> &'static str {
        match self {
            Priority::Low => "Low",
            Priority::Medium => "Medium",
            Priority::High => "High",
            Priority::Critical => "Critical",
        }
    }

    fn color_hex(self) -> u32 {
        match self {
            Priority::Low => 0x4CAF50,
            Priority::Medium => 0x2196F3,
            Priority::High => 0xFF9800,
            Priority::Critical => 0xF44336,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Status {
    Todo,
    InProgress,
    InReview,
    Done,
}

impl Status {
    fn label(self) -> &'static str {
        match self {
            Status::Todo => "To Do",
            Status::InProgress => "In Progress",
            Status::InReview => "In Review",
            Status::Done => "Done",
        }
    }

    fn color_hex(self) -> u32 {
        match self {
            Status::Todo => 0x9E9E9E,
            Status::InProgress => 0x2196F3,
            Status::InReview => 0xFF9800,
            Status::Done => 0x4CAF50,
        }
    }
}

#[derive(Clone, Debug)]
struct Project {
    id: SharedString,
    name: SharedString,
    owner: SharedString,
    priority: Priority,
    status: Status,
    starred: bool,
}

#[derive(Clone, Debug)]
struct Board {
    name: SharedString,
    expanded: bool,
    projects: Vec<Project>,
}

// ---------------------------------------------------------------------------
// Project detail view — shown in a tab when a project is opened
// ---------------------------------------------------------------------------

struct ProjectView {
    project: Project,
}

impl Render for ProjectView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;

        let priority_color = gpui::rgb(self.project.priority.color_hex());
        let status_color = gpui::rgb(self.project.status.color_hex());

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors.surface)
            .child(
                // Header bar
                div()
                    .px(px(spacing.xl))
                    .py(px(spacing.lg))
                    .border_b_1()
                    .border_color(colors.stroke_neutral_subtle)
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.md))
                    .child(
                        div()
                            .text_size(px(typography.title.size))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.on_neutral)
                            .child(self.project.name.clone()),
                    )
                    .child(
                        div()
                            .px(px(spacing.sm))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .bg(status_color)
                            .text_color(gpui::rgb(0xFFFFFF))
                            .text_size(px(typography.caption.size))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.project.status.label()),
                    )
                    .child(
                        div()
                            .px(px(spacing.sm))
                            .py(px(2.0))
                            .rounded(px(4.0))
                            .border_1()
                            .border_color(priority_color)
                            .text_color(priority_color)
                            .text_size(px(typography.caption.size))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.project.priority.label()),
                    ),
            )
            .child(
                // Meta row
                div()
                    .px(px(spacing.xl))
                    .py(px(spacing.md))
                    .flex()
                    .flex_row()
                    .gap(px(spacing.xl))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(spacing.xs))
                            .child(
                                div()
                                    .text_size(px(typography.caption.size))
                                    .text_color(colors.on_subtle)
                                    .child("Owner"),
                            )
                            .child(
                                div()
                                    .text_size(px(typography.body.size))
                                    .text_color(colors.on_neutral)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.project.owner.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(spacing.xs))
                            .child(
                                div()
                                    .text_size(px(typography.caption.size))
                                    .text_color(colors.on_subtle)
                                    .child("ID"),
                            )
                            .child(
                                div()
                                    .text_size(px(typography.body.size))
                                    .text_color(colors.on_neutral)
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.project.id.clone()),
                            ),
                    ),
            )
            .child(
                // Placeholder content area
                div().flex_1().flex().items_center().justify_center().child(
                    div()
                        .px(px(spacing.xl))
                        .py(px(spacing.lg))
                        .rounded(px(8.0))
                        .bg(colors.surface_blur_layer)
                        .border_1()
                        .border_color(colors.stroke_neutral_subtle)
                        .text_size(px(typography.body.size))
                        .text_color(colors.on_subtle)
                        .child("[ project tasks and activity would render here ]"),
                ),
            )
    }
}

// ---------------------------------------------------------------------------
// Dashboard
// ---------------------------------------------------------------------------

struct DashboardView {
    boards: Vec<Board>,
}

impl Render for DashboardView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;

        let all_projects: Vec<&Project> = self.boards.iter().flat_map(|b| &b.projects).collect();
        let total = all_projects.len();
        let in_progress = all_projects
            .iter()
            .filter(|p| p.status == Status::InProgress)
            .count();
        let in_review = all_projects
            .iter()
            .filter(|p| p.status == Status::InReview)
            .count();
        let done = all_projects
            .iter()
            .filter(|p| p.status == Status::Done)
            .count();
        let critical = all_projects
            .iter()
            .filter(|p| p.priority == Priority::Critical)
            .count();

        let mut cards = div()
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(px(spacing.lg))
            .p(px(spacing.xl));

        for (label, value, hex) in [
            ("Total Projects", total.to_string(), 0x0078D4u32),
            ("In Progress", in_progress.to_string(), 0x2196F3u32),
            ("In Review", in_review.to_string(), 0xFF9800u32),
            ("Done", done.to_string(), 0x4CAF50u32),
            ("Critical", critical.to_string(), 0xF44336u32),
        ] {
            cards = cards.child(
                div()
                    .w(px(150.0))
                    .h(px(90.0))
                    .rounded(px(8.0))
                    .bg(colors.neutral)
                    .border_1()
                    .border_color(colors.stroke_neutral_subtle)
                    .p(px(spacing.md))
                    .flex()
                    .flex_col()
                    .gap(px(spacing.xs))
                    .child(div().size(px(8.0)).rounded(px(99.0)).bg(gpui::rgb(hex)))
                    .child(
                        div()
                            .text_size(px(typography.display.size))
                            .font_weight(FontWeight::BOLD)
                            .text_color(colors.on_neutral)
                            .child(value),
                    )
                    .child(
                        div()
                            .text_size(px(typography.caption.size))
                            .text_color(colors.on_subtle)
                            .child(label),
                    ),
            );
        }

        let starred: Vec<&Project> = all_projects.iter().filter(|p| p.starred).copied().collect();

        div()
            .id("dashboard-scroll")
            .size_full()
            .overflow_y_scroll()
            .bg(colors.surface)
            .child(
                div()
                    .p(px(spacing.xl))
                    .text_size(px(typography.title.size))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.on_neutral)
                    .child("Dashboard"),
            )
            .child(cards)
            .child(
                div()
                    .px(px(spacing.xl))
                    .pb(px(spacing.sm))
                    .text_size(px(typography.subtitle.size))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(colors.on_neutral)
                    .child("Starred"),
            )
            .child(
                div()
                    .px(px(spacing.xl))
                    .py(px(spacing.md))
                    .flex()
                    .flex_col()
                    .gap(px(spacing.xs))
                    .children(starred.iter().map(|p| {
                        let status_color = gpui::rgb(p.status.color_hex());
                        let priority_color = gpui::rgb(p.priority.color_hex());
                        div()
                            .flex()
                            .flex_row()
                            .items_center()
                            .gap(px(spacing.md))
                            .px(px(spacing.md))
                            .py(px(spacing.sm))
                            .rounded(px(4.0))
                            .bg(colors.neutral)
                            .child(
                                div()
                                    .text_size(px(10.0))
                                    .text_color(gpui::rgb(0xFF9800))
                                    .child("★"),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(typography.body.size))
                                    .text_color(colors.on_neutral)
                                    .child(p.name.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(typography.caption.size))
                                    .text_color(colors.on_subtle)
                                    .child(p.owner.clone()),
                            )
                            .child(
                                div()
                                    .px(px(spacing.xs))
                                    .py(px(1.0))
                                    .rounded(px(3.0))
                                    .bg(status_color)
                                    .text_color(gpui::rgb(0xFFFFFF))
                                    .text_size(px(typography.caption.size))
                                    .child(p.status.label()),
                            )
                            .child(
                                div()
                                    .px(px(spacing.xs))
                                    .py(px(1.0))
                                    .rounded(px(3.0))
                                    .border_1()
                                    .border_color(priority_color)
                                    .text_color(priority_color)
                                    .text_size(px(typography.caption.size))
                                    .child(p.priority.label()),
                            )
                    })),
            )
    }
}

// ---------------------------------------------------------------------------
// Sidebar — board/project tree
// ---------------------------------------------------------------------------

#[allow(clippy::type_complexity)]
struct ProjectTree {
    boards: Vec<Board>,
    selected: Option<(usize, usize)>,
    on_open: Option<Box<dyn Fn(Project, &mut App) + 'static>>,
    context_menu: Option<Entity<ContextMenu>>,
}

impl ProjectTree {
    fn new(boards: Vec<Board>) -> Self {
        Self {
            boards,
            selected: None,
            on_open: None,
            context_menu: None,
        }
    }
}

impl Render for ProjectTree {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;

        let selected = self.selected;

        let mut tree = div()
            .id("project-tree-scroll")
            .flex()
            .flex_col()
            .size_full()
            .overflow_y_scroll()
            .bg(colors.panel_bg)
            .p(px(spacing.xs));

        tree = tree.child(
            div()
                .flex()
                .flex_row()
                .items_center()
                .mx(px(spacing.xs))
                .my(px(spacing.sm))
                .px(px(spacing.sm))
                .h(px(26.0))
                .rounded(px(4.0))
                .bg(colors.surface)
                .border_1()
                .border_color(colors.stroke_neutral)
                .text_size(px(typography.caption.size))
                .text_color(colors.on_subtle_disabled)
                .child("Search projects…"),
        );

        for (bi, board) in self.boards.iter().enumerate() {
            let expanded = board.expanded;
            let board_name = board.name.clone();

            let expand_handler = cx.listener(move |tree: &mut ProjectTree, _, _, cx| {
                tree.boards[bi].expanded = !tree.boards[bi].expanded;
                cx.notify();
            });

            tree = tree.child(
                div()
                    .id(("board", bi as u64))
                    .flex()
                    .flex_row()
                    .items_center()
                    .gap(px(spacing.xs))
                    .px(px(spacing.sm))
                    .h(px(24.0))
                    .rounded(px(4.0))
                    .cursor_pointer()
                    .hover(move |s| s.bg(colors.subtle_hover))
                    .on_click(expand_handler)
                    .child(
                        div()
                            .text_size(px(10.0))
                            .text_color(colors.on_subtle)
                            .child(if expanded { "▼" } else { "▶" }),
                    )
                    .child(div().text_size(px(14.0)).child("📋"))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(typography.body.size))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.on_neutral)
                            .child(board_name),
                    ),
            );

            if !expanded {
                continue;
            }

            for (pi, project) in board.projects.iter().enumerate() {
                let is_selected = selected == Some((bi, pi));
                let project_clone = project.clone();
                let status_color = gpui::rgb(project.status.color_hex());
                let project_name = project.name.clone();
                let starred = project.starred;
                let row_bg = if is_selected {
                    colors.neutral_selected
                } else {
                    colors.panel_bg
                };

                let click_handler = cx.listener(move |tree: &mut ProjectTree, _, _, cx| {
                    tree.selected = Some((bi, pi));
                    if let Some(f) = &tree.on_open {
                        f(project_clone.clone(), cx);
                    }
                    cx.notify();
                });

                let rclick_name = project.name.clone();
                let right_click_handler =
                    cx.listener(move |tree: &mut ProjectTree, ev: &MouseDownEvent, _, cx| {
                        let pos = ev.position;
                        let name = rclick_name.clone();
                        let menu = cx.new(|_| {
                            ContextMenu::build(pos)
                                .action("Open", move |_, _, _| {})
                                .separator()
                                .action("Edit…", move |_, _, _| {})
                                .action("Duplicate", |_, _, _| {})
                                .action("Move to Board", |_, _, _| {})
                                .separator()
                                .action("Archive", |_, _, _| {})
                                .action("Delete", |_, _, _| {})
                        });
                        tree.context_menu = Some(menu);
                        let _ = name;
                        cx.notify();
                    });

                tree = tree.child(
                    div()
                        .id(("project", (bi * 100 + pi) as u64))
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(spacing.xs))
                        .pl(px(spacing.xl))
                        .pr(px(spacing.sm))
                        .h(px(24.0))
                        .rounded(px(4.0))
                        .bg(row_bg)
                        .cursor_pointer()
                        .hover(move |s| s.bg(colors.subtle_hover))
                        .on_click(click_handler)
                        .on_mouse_down(MouseButton::Right, right_click_handler)
                        .child(div().size(px(7.0)).rounded(px(99.0)).bg(status_color))
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(typography.body.size))
                                .text_color(colors.on_neutral)
                                .truncate()
                                .child(project_name),
                        )
                        .child(if starred {
                            div()
                                .text_size(px(10.0))
                                .text_color(gpui::rgb(0xFF9800))
                                .child("★")
                        } else {
                            div()
                        }),
                );
            }
        }

        if let Some(menu_entity) = &self.context_menu {
            if menu_entity.read(cx).open {
                tree = tree.child(menu_entity.clone());
            } else {
                let _ = cx.listener(|tree: &mut ProjectTree, _: &gpui::ClickEvent, _, cx| {
                    tree.context_menu = None;
                    cx.notify();
                });
            }
        }

        tree
    }
}

// ---------------------------------------------------------------------------
// New Project modal
// ---------------------------------------------------------------------------

struct NewProjectModal {
    nav: Entity<SettingsNav>,
    active_section: SharedString,
}

impl NewProjectModal {
    fn new(cx: &mut Context<Self>) -> Self {
        cx.observe_global::<fluent_core::Theme>(|_, cx| cx.notify())
            .detach();
        let active: SharedString = "details".into();
        let active_clone = active.clone();
        let nav = cx.new(move |cx| {
            SettingsNav::new(cx)
                .active(active_clone.clone())
                .section(
                    SettingsNavSection::new("Project")
                        .item(SettingsNavItem::new("details", "Details"))
                        .item(SettingsNavItem::new("team", "Team"))
                        .item(SettingsNavItem::new("schedule", "Schedule")),
                )
                .section(
                    SettingsNavSection::new("Settings")
                        .item(SettingsNavItem::new("workflow", "Workflow"))
                        .item(SettingsNavItem::new("notifications", "Notifications"))
                        .item(SettingsNavItem::new("permissions", "Permissions")),
                )
        });
        Self {
            nav,
            active_section: active,
        }
    }
}

impl Render for NewProjectModal {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;
        let radii = theme.radii;

        let active = self.active_section.clone();

        let self_weak = cx.entity().downgrade();
        self.nav.update(cx, |nav, _| {
            let sw = self_weak.clone();
            nav.on_select = Some(std::sync::Arc::new(
                move |id: SharedString, cx: &mut App| {
                    sw.update(cx, |modal: &mut NewProjectModal, cx| {
                        modal.active_section = id;
                        cx.notify();
                    })
                    .ok();
                },
            ));
        });

        let section_title: &str = match active.as_ref() {
            "details" => "Project Details",
            "team" => "Team Members",
            "schedule" => "Schedule",
            "workflow" => "Workflow",
            "notifications" => "Notifications",
            "permissions" => "Permissions",
            _ => "Settings",
        };

        let field = |label: &str,
                     placeholder: &str,
                     colors: &fluent_core::ColorScheme,
                     spacing: fluent_core::SpacingTokens,
                     typography: fluent_core::TypographyTokens,
                     _radii: fluent_core::RadiiTokens| {
            div()
                .flex()
                .flex_row()
                .items_center()
                .gap(px(spacing.md))
                .py(px(spacing.sm))
                .child(
                    div()
                        .w(px(130.0))
                        .text_size(px(typography.body.size))
                        .text_color(colors.on_neutral)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(label.to_string()),
                )
                .child(
                    div()
                        .flex_1()
                        .h(px(28.0))
                        .px(px(spacing.sm))
                        .border_b_1()
                        .border_color(colors.accent)
                        .text_size(px(typography.body.size))
                        .text_color(colors.on_subtle_disabled)
                        .flex()
                        .items_center()
                        .child(placeholder.to_string()),
                )
        };

        let colors2 = colors.clone();
        let form_body = match active.as_ref() {
            "details" => div()
                .flex()
                .flex_col()
                .gap(px(spacing.xs))
                .child(field(
                    "Project Name:",
                    "My New Project",
                    &colors,
                    spacing,
                    typography,
                    radii,
                ))
                .child(field(
                    "Description:",
                    "What is this project about?",
                    &colors2,
                    spacing,
                    typography,
                    radii,
                ))
                .child(field(
                    "Owner:",
                    "Assign an owner…",
                    &colors2,
                    spacing,
                    typography,
                    radii,
                ))
                .child(
                    div()
                        .flex()
                        .flex_row()
                        .items_center()
                        .gap(px(spacing.md))
                        .py(px(spacing.sm))
                        .child(
                            div()
                                .w(px(130.0))
                                .text_size(px(typography.body.size))
                                .text_color(colors2.on_neutral)
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Priority:"),
                        )
                        .child(
                            div()
                                .w(px(120.0))
                                .h(px(28.0))
                                .px(px(spacing.sm))
                                .border_1()
                                .border_color(colors2.stroke_neutral)
                                .rounded(px(radii.sm))
                                .text_size(px(typography.body.size))
                                .text_color(colors2.on_neutral)
                                .flex()
                                .items_center()
                                .child("Medium ▾"),
                        )
                        .child(
                            div()
                                .w(px(100.0))
                                .text_size(px(typography.body.size))
                                .text_color(colors2.on_neutral)
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Board:"),
                        )
                        .child(
                            div()
                                .w(px(120.0))
                                .h(px(28.0))
                                .px(px(spacing.sm))
                                .border_1()
                                .border_color(colors2.stroke_neutral)
                                .rounded(px(radii.sm))
                                .text_size(px(typography.body.size))
                                .text_color(colors2.on_neutral)
                                .flex()
                                .items_center()
                                .child("Active ▾"),
                        ),
                ),
            "team" => div()
                .flex()
                .flex_col()
                .gap(px(spacing.xs))
                .child(field(
                    "Add Member:",
                    "Search by name or email…",
                    &colors,
                    spacing,
                    typography,
                    radii,
                ))
                .child(field(
                    "Role:",
                    "Contributor ▾",
                    &colors2,
                    spacing,
                    typography,
                    radii,
                )),
            "schedule" => div()
                .flex()
                .flex_col()
                .gap(px(spacing.xs))
                .child(field(
                    "Start Date:",
                    "YYYY-MM-DD",
                    &colors,
                    spacing,
                    typography,
                    radii,
                ))
                .child(field(
                    "Due Date:",
                    "YYYY-MM-DD",
                    &colors2,
                    spacing,
                    typography,
                    radii,
                ))
                .child(field(
                    "Milestone:",
                    "Optional milestone name…",
                    &colors2,
                    spacing,
                    typography,
                    radii,
                )),
            _ => div()
                .flex()
                .items_center()
                .justify_center()
                .text_size(px(typography.body.size))
                .text_color(colors2.on_subtle_disabled)
                .child(format!("{} settings coming soon.", active.as_ref())),
        };

        let header = div()
            .flex()
            .flex_row()
            .items_center()
            .gap(px(spacing.lg))
            .px(px(spacing.xl))
            .py(px(spacing.lg))
            .border_b_1()
            .border_color(colors.stroke_neutral_subtle)
            .child(
                div()
                    .size(px(48.0))
                    .rounded(px(radii.lg))
                    .bg(colors.accent)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(24.0))
                    .text_color(colors.on_accent)
                    .child("📋"),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(spacing.xs))
                    .child(
                        div()
                            .text_size(px(typography.subtitle.size))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(colors.on_neutral)
                            .child(section_title),
                    )
                    .child(
                        div()
                            .text_size(px(typography.caption.size))
                            .text_color(colors.on_subtle)
                            .child("Create a new project"),
                    ),
            );

        let footer = div()
            .flex()
            .flex_row()
            .items_center()
            .justify_end()
            .gap(px(spacing.sm))
            .px(px(spacing.xl))
            .py(px(spacing.md))
            .border_t_1()
            .border_color(colors.stroke_neutral_subtle)
            .child(
                div()
                    .id("modal-cancel-btn")
                    .px(px(spacing.md))
                    .py(px(spacing.sm))
                    .rounded(px(radii.md))
                    .bg(colors.neutral)
                    .border_1()
                    .border_color(colors.stroke_neutral)
                    .text_size(px(typography.body.size))
                    .text_color(colors.on_neutral)
                    .cursor_pointer()
                    .hover(move |s| s.bg(colors.neutral_hover))
                    .on_click(move |_, _, cx| ModalStack::pop(cx))
                    .child("Cancel"),
            )
            .child(
                div()
                    .id("modal-create-btn")
                    .px(px(spacing.md))
                    .py(px(spacing.sm))
                    .rounded(px(radii.md))
                    .bg(colors.accent)
                    .text_size(px(typography.body.size))
                    .text_color(colors.on_accent)
                    .cursor_pointer()
                    .hover(move |s| s.bg(colors.accent_hover))
                    .on_click(move |_, _, cx| ModalStack::pop(cx))
                    .child("Create Project"),
            );

        div()
            .size_full()
            .flex()
            .flex_col()
            .bg(colors.surface)
            .child(header)
            .child(
                div()
                    .flex()
                    .flex_row()
                    .flex_1()
                    .min_h_0()
                    .child(self.nav.clone())
                    .child(
                        div()
                            .flex_1()
                            .p(px(spacing.xl))
                            .overflow_hidden()
                            .child(form_body),
                    ),
            )
            .child(footer)
    }
}

// ---------------------------------------------------------------------------
// Root app state
// ---------------------------------------------------------------------------

#[allow(dead_code)]
struct DemoApp {
    ribbon: Entity<RibbonBar>,
    tab_strip: Entity<TabStrip>,
    pane: Entity<Pane>,
    project_tree: Entity<ProjectTree>,
    sidebar: Entity<DockPanel>,
    workspace: Entity<Workspace>,
    modal_host: Entity<ModalHost>,
    palette: Entity<CommandPalette>,
}

impl Render for DemoApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("app-root")
            .size_full()
            .on_action(cx.listener(|demo: &mut DemoApp, _: &TogglePalette, _, cx| {
                demo.palette.update(cx, |p, cx| p.show(cx));
            }))
            .on_action(cx.listener(|_: &mut DemoApp, _: &ToggleLightTheme, _, cx| {
                fluent_core::Theme::apply_light(cx);
            }))
            .on_action(cx.listener(|_: &mut DemoApp, _: &ToggleDarkTheme, _, cx| {
                fluent_core::Theme::apply_dark(cx);
            }))
            .child(self.workspace.clone())
            .child(self.palette.clone())
    }
}

// ---------------------------------------------------------------------------
// Wiring helpers
// ---------------------------------------------------------------------------

fn make_ribbon(project_open: bool, cx: &mut App) -> Entity<RibbonBar> {
    cx.new(|cx| {
        let bar = RibbonBar::new(cx)
            .tab("Home", |t| {
                t.group("Projects", |g| {
                    g.large_button("New Project", "icons/add.svg", |_, _, cx| {
                        let modal = cx.new(NewProjectModal::new);
                        ModalStack::push(
                            modal.into(),
                            fluent_layout::ModalSize::Fixed(760.0, 540.0),
                            cx,
                        );
                    })
                    .large_button(
                        "Archive",
                        "icons/folder_open.svg",
                        |_, _, _| {},
                    )
                })
                .group("Edit", |g| {
                    g.stack(|s| {
                        s.button("Rename", "icons/settings.svg", |_, _, _| {})
                            .button("Duplicate", "icons/copy.svg", |_, _, _| {})
                            .button("Delete", "icons/delete.svg", |_, _, _| {})
                    })
                })
                .group("Assign", |g| {
                    g.large_button("Assign Owner", "icons/plug.svg", |_, _, _| {})
                        .large_button("Add to Board", "icons/folder_add.svg", |_, _, _| {})
                })
            })
            .tab("View", |t| {
                t.group("Layout", |g| {
                    g.toggle_button("List", "icons/list_tree.svg", true, |_, _, _, _| {})
                        .toggle_button("Board", "icons/grid.svg", false, |_, _, _, _| {})
                })
                .group("Theme", |g| {
                    g.toggle_button("Dark", "icons/dark_theme.svg", true, |_, _, _, cx| {
                        fluent_core::Theme::apply_dark(cx);
                    })
                    .toggle_button(
                        "Light",
                        "icons/light_theme.svg",
                        false,
                        |_, _, _, cx| {
                            fluent_core::Theme::apply_light(cx);
                        },
                    )
                })
            });

        bar.contextual_tab("Project", project_open, |t| {
            t.group("Status", |g| {
                g.large_button("Mark Done", "icons/plug.svg", |_, _, _| {})
                    .stack(|s| {
                        s.button("To Do", "icons/copy.svg", |_, _, _| {})
                            .button("In Progress", "icons/clipboard_paste.svg", |_, _, _| {})
                            .button("In Review", "icons/search.svg", |_, _, _| {})
                    })
            })
            .group("Actions", |g| {
                g.large_button("Add Task", "icons/add.svg", |_, _, _| {})
                    .button("Attach File", "icons/folder_sync.svg", |_, _, _| {})
                    .button("Comment", "icons/copy.svg", |_, _, _| {})
            })
        })
    })
}

fn make_menu_bar(cx: &mut App) -> Entity<MenuBar> {
    cx.new(|cx| {
        MenuBar::new(cx)
            .menu(
                "File",
                vec![
                    MenuItemDef::action("New Project", |_, _, cx| {
                        let modal = cx.new(NewProjectModal::new);
                        ModalStack::push(
                            modal.into(),
                            fluent_layout::ModalSize::Fixed(760.0, 540.0),
                            cx,
                        );
                    }),
                    MenuItemDef::action_with_shortcut("Quick Open…", "Ctrl+Q", |_, _, _| {}),
                    MenuItemDef::separator(),
                    MenuItemDef::action("Import Projects…", |_, _, _| {}),
                    MenuItemDef::action("Export Projects…", |_, _, _| {}),
                    MenuItemDef::separator(),
                    MenuItemDef::action_with_shortcut("Quit", "Ctrl+W", |_, _, cx| cx.quit()),
                ],
            )
            .menu(
                "View",
                vec![
                    MenuItemDef::action_with_shortcut("Toggle Sidebar", "Ctrl+B", |_, _, _| {}),
                    MenuItemDef::separator(),
                    MenuItemDef::action("Board View", |_, _, _| {}),
                    MenuItemDef::action("List View", |_, _, _| {}),
                    MenuItemDef::separator(),
                    MenuItemDef::action_with_shortcut("Dark Theme", "Ctrl+Shift+D", |_, _, cx| {
                        fluent_core::Theme::apply_dark(cx);
                    }),
                    MenuItemDef::action_with_shortcut("Light Theme", "Ctrl+Shift+L", |_, _, cx| {
                        fluent_core::Theme::apply_light(cx);
                    }),
                ],
            )
            .menu(
                "Tools",
                vec![
                    MenuItemDef::action("Team Members", |_, _, _| {}),
                    MenuItemDef::action("Labels & Tags", |_, _, _| {}),
                    MenuItemDef::separator(),
                    MenuItemDef::action("Activity Log", |_, _, _| {}),
                    MenuItemDef::action_with_shortcut("Preferences", "Ctrl+,", |_, _, cx| {
                        let modal = cx.new(NewProjectModal::new);
                        ModalStack::push(
                            modal.into(),
                            fluent_layout::ModalSize::Fixed(760.0, 540.0),
                            cx,
                        );
                    }),
                ],
            )
            .menu(
                "Help",
                vec![
                    MenuItemDef::action("Documentation", |_, _, _| {}),
                    MenuItemDef::action("Check for Updates…", |_, _, _| {}),
                    MenuItemDef::separator(),
                    MenuItemDef::action("About FluentGUI Demo", |_, _, _| {}),
                ],
            )
    })
}

fn open_project(
    project: Project,
    pane: &Entity<Pane>,
    tabs: &Entity<TabStrip>,
    ribbon: &Entity<RibbonBar>,
    cx: &mut App,
) {
    let tab_id: SharedString = project.id.clone();
    let project_name = project.name.clone();

    tabs.update(cx, |ts, cx| {
        let already_open = ts.tabs.iter().any(|t| t.id == tab_id);
        if !already_open {
            ts.add_tab(
                TabItem::new(tab_id.clone())
                    .label(project_name.clone())
                    .closable(true),
            );
            ts.active = ts.tabs.len() - 1;
        }
        cx.notify();
    });

    let view = cx.new(|cx| {
        cx.observe_global::<fluent_core::Theme>(|_, cx| cx.notify())
            .detach();
        ProjectView { project }
    });
    pane.update(cx, |p, cx| {
        p.set_content(view.into(), cx);
    });

    ribbon.update(cx, |bar, cx| {
        bar.set_contextual_visible("Project", true);
        cx.notify();
    });

    ModalStack::push(
        cx.new(|_| {
            ConfirmModal::new(
                format!("Opened {project_name}"),
                "Project is now active in the editor.".to_string(),
            )
        })
        .into(),
        fluent_layout::ModalSize::Fixed(360.0, 160.0),
        cx,
    );
}

fn demo_boards() -> Vec<Board> {
    vec![
        Board {
            name: "Active".into(),
            expanded: true,
            projects: vec![
                Project {
                    id: "PRJ-001".into(),
                    name: "Onboarding Redesign".into(),
                    owner: "Alice Chen".into(),
                    priority: Priority::High,
                    status: Status::InProgress,
                    starred: true,
                },
                Project {
                    id: "PRJ-002".into(),
                    name: "API Rate Limiting".into(),
                    owner: "Bob Okafor".into(),
                    priority: Priority::Critical,
                    status: Status::InReview,
                    starred: false,
                },
                Project {
                    id: "PRJ-003".into(),
                    name: "Design System Audit".into(),
                    owner: "Clara Novak".into(),
                    priority: Priority::Medium,
                    status: Status::Todo,
                    starred: true,
                },
                Project {
                    id: "PRJ-004".into(),
                    name: "Mobile Push Notifications".into(),
                    owner: "Alice Chen".into(),
                    priority: Priority::High,
                    status: Status::InProgress,
                    starred: false,
                },
            ],
        },
        Board {
            name: "Backlog".into(),
            expanded: true,
            projects: vec![
                Project {
                    id: "PRJ-005".into(),
                    name: "Dark Mode Support".into(),
                    owner: "David Kim".into(),
                    priority: Priority::Low,
                    status: Status::Todo,
                    starred: false,
                },
                Project {
                    id: "PRJ-006".into(),
                    name: "CSV Export".into(),
                    owner: "Clara Novak".into(),
                    priority: Priority::Medium,
                    status: Status::Todo,
                    starred: false,
                },
                Project {
                    id: "PRJ-007".into(),
                    name: "Keyboard Shortcuts".into(),
                    owner: "Bob Okafor".into(),
                    priority: Priority::Low,
                    status: Status::Todo,
                    starred: true,
                },
            ],
        },
        Board {
            name: "Completed".into(),
            expanded: false,
            projects: vec![
                Project {
                    id: "PRJ-008".into(),
                    name: "User Profile Page".into(),
                    owner: "Alice Chen".into(),
                    priority: Priority::Medium,
                    status: Status::Done,
                    starred: false,
                },
                Project {
                    id: "PRJ-009".into(),
                    name: "Email Notifications".into(),
                    owner: "David Kim".into(),
                    priority: Priority::High,
                    status: Status::Done,
                    starred: false,
                },
            ],
        },
    ]
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn main() {
    let boards = demo_boards();

    FluentApp::new("FluentGUI Demo — Project Tracker")
        .window_size(1280.0, 800.0)
        .dark_theme()
        .run(move |cx| {
            cx.bind_keys([
                KeyBinding::new("ctrl-k", TogglePalette, None),
                KeyBinding::new("ctrl-shift-l", ToggleLightTheme, None),
                KeyBinding::new("ctrl-shift-d", ToggleDarkTheme, None),
            ]);
            ModalStack::init(cx);
            let modal_host = cx.new(ModalHost::new);

            let palette_entries: Vec<PaletteEntry> = boards
                .iter()
                .flat_map(|b| {
                    b.projects.iter().map(|p| {
                        PaletteEntry::new(p.id.clone(), p.name.clone())
                            .group("Projects")
                            .subtitle(format!(
                                "{} — {} — {}",
                                p.owner,
                                p.status.label(),
                                p.priority.label()
                            ))
                            .hint(p.priority.label())
                            .keyword(p.owner.clone())
                            .keyword(p.status.label())
                    })
                })
                .collect();

            let boards_clone = boards.clone();
            let boards_for_tree = boards.clone();

            let dashboard = cx.new(|cx| {
                cx.observe_global::<fluent_core::Theme>(|_, cx| cx.notify())
                    .detach();
                DashboardView {
                    boards: boards_clone,
                }
            });

            let tab_strip = cx.new(|cx: &mut gpui::Context<TabStrip>| {
                let mut ts = TabStrip::new(cx);
                ts.add_tab(TabItem::new("dashboard").label("Dashboard"));
                ts
            });

            let pane = cx.new(|cx| Pane::new(cx).with_tab_strip(tab_strip.clone()));
            pane.update(cx, |p, cx| {
                p.set_content(dashboard.into(), cx);
            });

            let ribbon = make_ribbon(false, cx);

            let project_tree_entity: Entity<ProjectTree> = {
                let pane_handle = pane.clone();
                let tab_strip_handle = tab_strip.clone();
                let ribbon_handle = ribbon.clone();

                cx.new(move |cx| {
                    cx.observe_global::<fluent_core::Theme>(|_, cx| cx.notify())
                        .detach();
                    let mut tree = ProjectTree::new(boards_for_tree);

                    let pane_h = pane_handle.clone();
                    let tabs_h = tab_strip_handle.clone();
                    let ribbon_h = ribbon_handle.clone();
                    tree.on_open = Some(Box::new(move |project: Project, cx: &mut App| {
                        open_project(project, &pane_h, &tabs_h, &ribbon_h, cx);
                    }));

                    tree
                })
            };

            let sidebar = cx.new(|cx| {
                DockPanel::new(cx, DockPosition::Left, "Boards")
                    .size(240.0)
                    .content(project_tree_entity.clone().into())
            });

            let boards_for_palette = boards.clone();
            let pane_for_palette = pane.clone();
            let tabs_for_palette = tab_strip.clone();
            let ribbon_for_palette = ribbon.clone();
            let palette = cx.new(move |cx| {
                let mut p = CommandPalette::new(cx);
                p.set_entries(palette_entries, cx);
                let pane_handle = pane_for_palette.clone();
                let tab_strip_handle = tabs_for_palette.clone();
                let ribbon_handle = ribbon_for_palette.clone();
                p.on_select = Some(Box::new(move |entry: &PaletteEntry, cx: &mut App| {
                    let id = entry.id.clone();
                    for board in &boards_for_palette {
                        for project in &board.projects {
                            if project.id == id {
                                open_project(
                                    project.clone(),
                                    &pane_handle,
                                    &tab_strip_handle,
                                    &ribbon_handle,
                                    cx,
                                );
                                return;
                            }
                        }
                    }
                }));
                p
            });

            let menu_bar = make_menu_bar(cx);
            let workspace = cx.new(|cx| {
                Workspace::new(cx)
                    .menu_bar(menu_bar)
                    .ribbon(ribbon.clone().into())
                    .left_dock(sidebar.clone())
                    .content(pane.clone().into())
                    .modal_host(modal_host.clone())
            });

            cx.new(|_cx| DemoApp {
                ribbon,
                tab_strip,
                pane,
                project_tree: project_tree_entity,
                sidebar,
                workspace,
                modal_host,
                palette,
            })
        });
}
