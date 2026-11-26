use families::{gallery_nav, DemoViews, Family, FamilyHost};
use fluent_app::{FluentApp, TitleBar};
use fluent_core::gpui::{div, prelude::*, App, ClickEvent, Context, IntoElement, Render, Window};
use fluent_layout::{
    DockPanel, DockPosition, MenuBar, MenuItemDef, ModalStack, Pane, TabItem, TabStrip, Workspace,
};
use fluent_ribbon::RibbonBar;
use std::path::PathBuf;
mod families;
mod pages;
mod pages_more;

// ---------------------------------------------------------------------------
// Root gallery view — wires the full shell together
// ---------------------------------------------------------------------------

struct GalleryRoot {
    workspace: fluent_core::gpui::Entity<Workspace>,
    /// Written once after the first frame, for the screenshot script.
    ready_file: Option<PathBuf>,
    #[cfg_attr(not(test), allow(dead_code))]
    host: fluent_core::gpui::Entity<FamilyHost>,
}

impl Render for GalleryRoot {
    fn render(&mut self, window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        if let Some(path) = self.ready_file.take() {
            window.on_next_frame(move |_, _| {
                let _ = std::fs::write(path, "1");
            });
        }
        div().size_full().child(self.workspace.clone())
    }
}

// ---------------------------------------------------------------------------
// Entry point
// ---------------------------------------------------------------------------

fn noop(_: &ClickEvent, _: &mut Window, _: &mut App) {}

/// How the gallery starts. The defaults (first page, dark theme, default
/// window size, no ready file) are what a plain launch gets; the screenshot
/// script sets the rest (WI-4838).
#[derive(Debug, Default, PartialEq)]
struct StartOptions {
    page: Option<Family>,
    light: bool,
    size: Option<(f32, f32)>,
    ready_file: Option<PathBuf>,
}

/// `ButtonsAndLinks` -> `buttons-and-links`: the page id used by `--page` and
/// in the screenshot file names.
fn family_slug(family: Family) -> String {
    let mut slug = String::new();
    for (i, c) in family.id().chars().enumerate() {
        if c.is_uppercase() && i > 0 {
            slug.push('-');
        }
        slug.extend(c.to_lowercase());
    }
    slug
}

fn family_from_slug(slug: &str) -> Option<Family> {
    let want = slug.to_ascii_lowercase().replace('_', "-");
    Family::ALL.into_iter().find(|f| family_slug(*f) == want)
}

impl StartOptions {
    /// Parses `--page <id>`, `--theme dark|light`, `--size WxH` and
    /// `--ready-file <path>` (each also as `--flag=value`). `env_page` is the
    /// `GALLERY_PAGE` fallback; `--page` wins over it.
    fn parse(args: &[String], env_page: Option<&str>) -> Result<Self, String> {
        let mut opts = StartOptions::default();
        let mut page = env_page.filter(|p| !p.is_empty()).map(str::to_owned);
        let mut it = args.iter();
        while let Some(arg) = it.next() {
            let (flag, inline) = match arg.split_once('=') {
                Some((f, v)) => (f, Some(v.to_owned())),
                None => (arg.as_str(), None),
            };
            let mut value = || {
                inline
                    .clone()
                    .or_else(|| it.next().cloned())
                    .ok_or_else(|| format!("{flag} needs a value"))
            };
            match flag {
                "--page" => page = Some(value()?),
                "--theme" => match value()?.as_str() {
                    "dark" => opts.light = false,
                    "light" => opts.light = true,
                    other => return Err(format!("--theme must be dark or light, not {other:?}")),
                },
                "--size" => {
                    let v = value()?;
                    let parsed = v
                        .split_once('x')
                        .and_then(|(w, h)| Some((w.parse::<f32>().ok()?, h.parse::<f32>().ok()?)));
                    match parsed {
                        Some((w, h)) if w > 0.0 && h > 0.0 => opts.size = Some((w, h)),
                        _ => return Err(format!("--size must be WIDTHxHEIGHT, not {v:?}")),
                    }
                }
                "--ready-file" => opts.ready_file = Some(PathBuf::from(value()?)),
                other => return Err(format!("unknown argument {other:?}")),
            }
        }
        if let Some(p) = page {
            opts.page = Some(family_from_slug(&p).ok_or_else(|| {
                let known: Vec<String> = Family::ALL.into_iter().map(family_slug).collect();
                format!("unknown page {p:?}; pages: {}", known.join(", "))
            })?);
        }
        Ok(opts)
    }
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--list-pages") {
        for family in Family::ALL {
            println!("{}", family_slug(family));
        }
        return;
    }
    let env_page = std::env::var("GALLERY_PAGE").ok();
    let opts = match StartOptions::parse(&args, env_page.as_deref()) {
        Ok(opts) => opts,
        Err(message) => {
            eprintln!("gallery: {message}");
            std::process::exit(2);
        }
    };
    let mut app = FluentApp::new("FluentGUI Gallery");
    app = if opts.light {
        app.light_theme()
    } else {
        app.dark_theme()
    };
    if let Some((w, h)) = opts.size {
        app = app.window_size(w, h);
    }
    app.run(move |cx| cx.new(|cx| build_gallery_with(cx, opts)));
}

/// Builds the full gallery shell: title bar, menu bar, ribbon, settings-nav
/// sidebar, tabbed content pane and modal host.
#[cfg(test)]
fn build_gallery(cx: &mut App) -> GalleryRoot {
    build_gallery_with(cx, StartOptions::default())
}

fn build_gallery_with(cx: &mut App, opts: StartOptions) -> GalleryRoot {
    let family_host = cx.new(|cx| FamilyHost::new(DemoViews::build(cx)));
    // Title bar
    let title_bar = cx.new(|cx| TitleBar::new(cx, "FluentGUI Gallery"));

    // Menu bar
    let menu_bar = cx.new(|cx| {
        MenuBar::new(cx)
            .menu(
                "&File",
                vec![
                    MenuItemDef::action_with_shortcut("New", "Ctrl+N", noop),
                    MenuItemDef::action_with_shortcut("Open...", "Ctrl+O", noop),
                    MenuItemDef::submenu(
                        "Open Recent",
                        vec![
                            MenuItemDef::action("Project One", noop),
                            MenuItemDef::submenu(
                                "Older",
                                vec![
                                    MenuItemDef::action("Project Two", noop),
                                    MenuItemDef::submenu(
                                        "Archive",
                                        vec![MenuItemDef::action("Project Three", noop)],
                                    ),
                                ],
                            ),
                        ],
                    ),
                    MenuItemDef::separator(),
                    MenuItemDef::action_with_shortcut("Exit", "Alt+F4", noop),
                ],
            )
            .menu(
                "&Edit",
                vec![
                    MenuItemDef::action_with_shortcut("Cut", "Ctrl+X", noop),
                    MenuItemDef::action_with_shortcut("Copy", "Ctrl+C", noop),
                    MenuItemDef::action_with_shortcut("Paste", "Ctrl+V", noop),
                ],
            )
            .menu(
                "&View",
                vec![
                    MenuItemDef::submenu(
                        "Zoom",
                        vec![
                            MenuItemDef::action("Zoom In", noop),
                            MenuItemDef::action("Zoom Out", noop),
                        ],
                    ),
                    MenuItemDef::separator(),
                    MenuItemDef::checkbox("Status Bar", true, |_, _, _, _| {}),
                    MenuItemDef::checkbox("Line Numbers", false, |_, _, _, _| {}),
                    MenuItemDef::separator(),
                    MenuItemDef::radio("Compact", false, noop),
                    MenuItemDef::radio("Comfortable", true, noop),
                    MenuItemDef::separator(),
                    MenuItemDef::action_with_shortcut("Full Screen", "F11", noop),
                ],
            )
            .menu(
                "&Help",
                vec![
                    MenuItemDef::submenu("Learn", vec![MenuItemDef::action("Documentation", noop)]),
                    MenuItemDef::action("About", noop),
                ],
            )
    });

    // Ribbon
    let ribbon = cx.new(|cx| {
        RibbonBar::new(cx)
            .tab("Home", |t| {
                t.group("Actions", |g| {
                    g.large_button("New", "icons/add.svg", |_, _, _| {})
                        .large_button("Open", "icons/folder_open.svg", |_, _, _| {})
                        .separator()
                        .stack(|s| {
                            s.button("Cut", "icons/cut.svg", |_, _, _| {})
                                .button("Copy", "icons/copy.svg", |_, _, _| {})
                                .button("Paste", "icons/paste.svg", |_, _, _| {})
                        })
                })
                .group("View", |g| {
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
            })
            .tab("Edit", |t| {
                t.group("Text", |g| {
                    g.button("Find", "icons/search.svg", |_, _, _| {}).button(
                        "Replace",
                        "icons/find_replace.svg",
                        |_, _, _| {},
                    )
                })
            })
            .contextual_tab("Session", true, |t| {
                t.group("Terminal", |g| {
                    g.large_button("Clear", "icons/clear.svg", |_, _, _| {})
                        .icon_button("icons/copy.svg", "Copy", |_, _, _| {})
                })
            })
    });

    // Left sidebar
    let nav = cx.new(|cx| gallery_nav(family_host.clone(), cx));
    if let Some(family) = opts.page {
        family_host.update(cx, |h, cx| h.set_active(family, cx));
        nav.update(cx, |n, cx| n.set_active(family.id().into(), cx));
    }
    let sidebar = cx.new(|cx| {
        DockPanel::new(cx, DockPosition::Left, "Navigation")
            .size(240.0)
            .content(nav.into())
    });

    // Main pane with a tab strip
    let tab_strip = cx.new(|cx: &mut fluent_core::gpui::Context<TabStrip>| {
        let mut ts = TabStrip::new(cx);
        ts.add_tab(TabItem::new("dashboard").label("Dashboard").closable(true));
        ts.add_tab(TabItem::new("settings").label("Settings").closable(true));
        ts
    });
    let main_pane = cx.new(|cx| Pane::new(cx).with_tab_strip(tab_strip));
    // set content separately since we need the entity
    main_pane.update(cx, |p, cx| {
        p.set_content(family_host.clone().into(), cx);
    });

    // Modal host
    ModalStack::init(cx);
    let modal_host = cx.new(fluent_layout::ModalHost::new);

    // Workspace
    let workspace = cx.new(|cx| {
        Workspace::new(cx)
            .title_bar(title_bar.into())
            .menu_bar(menu_bar)
            .with_file_menu("FluentGUI Gallery")
            .ribbon(ribbon.into())
            .left_dock(sidebar)
            .content(main_pane.into())
            .modal_host(modal_host)
    });
    GalleryRoot {
        workspace,
        ready_file: opts.ready_file,
        host: family_host,
    }
}

#[cfg(test)]
mod tests {
    //! In-process smoke tests: no window system and no GPU, so they check that
    //! each page renders without panic and that the theme controls act, not
    //! that anything looks right.

    use super::families::{DemoViews, Family, FamilyHost};
    use fluent_core::gpui::TestAppContext;
    use fluent_core::Theme;
    use fluent_layout::{MenuBar, MenuItemDef};
    use fluent_test_utils::{open_fluent_test_window, FluentInteractionExt as _};

    fn is_dark(cx: &mut fluent_test_utils::VisualTestContext) -> bool {
        cx.update(|_, cx| cx.global::<Theme>().is_dark())
    }

    #[test]
    fn every_family_page_renders() {
        let mut app = TestAppContext::single();
        let (host, cx) =
            open_fluent_test_window(&mut app, |_, cx| FamilyHost::new(DemoViews::build(cx)));
        for family in Family::ALL {
            host.update(cx, |h, cx| h.set_active(family, cx));
            let snapshot = cx.accessibility_snapshot();
            assert!(!snapshot.is_empty(), "{family:?} rendered no nodes");
        }
    }

    use super::{
        build_gallery, build_gallery_with, family_from_slug, family_slug, GalleryRoot, StartOptions,
    };
    use crate::pages_more::{MenusPage, OverlaysPage};
    use std::path::PathBuf;

    fn open_shell(
        app: &mut TestAppContext,
    ) -> (
        fluent_core::gpui::Entity<GalleryRoot>,
        &mut fluent_test_utils::VisualTestContext,
    ) {
        open_fluent_test_window(app, |_, cx| build_gallery(cx))
    }

    fn host_of(
        root: &fluent_core::gpui::Entity<GalleryRoot>,
        cx: &mut fluent_test_utils::VisualTestContext,
    ) -> fluent_core::gpui::Entity<FamilyHost> {
        root.read_with(cx, |r, _| r.host.clone())
    }

    #[test]
    fn menu_bar_shows_one_trigger_per_defined_menu() {
        // WI-4835: `with_file_menu` used to prepend a second "File" trigger
        // because the gallery's own menu is labelled "&File".
        use fluent_test_utils::FluentInteractionExt as _;
        let mut app = TestAppContext::single();
        let (_root, cx) = open_shell(&mut app);
        let triggers: Vec<String> = cx
            .accessibility_snapshot()
            .into_iter()
            .filter(|n| n.starts_with("MenuItem \"") && n.contains("[expanded="))
            .collect();
        assert_eq!(triggers.len(), 4, "menu bar triggers: {triggers:?}");
    }

    #[test]
    fn every_family_page_renders_inside_the_full_shell() {
        // The shell adds the title bar, menu bar, ribbon, tab strip and the
        // settings nav around each page; a page that repeats one of their
        // element ids would panic on a duplicate a11y node.
        let mut app = TestAppContext::single();
        let (root, cx) = open_shell(&mut app);
        let host = host_of(&root, cx);
        for family in Family::ALL {
            host.update(cx, |h, cx| h.set_active(family, cx));
            assert!(
                !cx.accessibility_snapshot().is_empty(),
                "{family:?} rendered no nodes in the shell"
            );
        }
    }

    #[test]
    fn settings_nav_selection_switches_the_page() {
        let mut app = TestAppContext::single();
        let (root, cx) = open_shell(&mut app);
        let host = host_of(&root, cx);
        for family in Family::ALL.into_iter().rev() {
            let (si, ii) = family.nav_position();
            let selector: &'static str = Box::leak(format!("nav-item-{si}-{ii}").into_boxed_str());
            cx.click_selector(selector);
            assert_eq!(
                host.read_with(cx, |h, _| h.active()),
                family,
                "clicking {selector} did not show {family:?}"
            );
            assert!(!cx.accessibility_snapshot().is_empty());
        }
    }

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| (*s).to_owned()).collect()
    }

    #[test]
    fn start_options_default_to_the_plain_launch() {
        let opts = StartOptions::parse(&[], None).unwrap();
        assert_eq!(opts, StartOptions::default());
        assert!(!opts.light && opts.page.is_none() && opts.size.is_none());
    }

    #[test]
    fn start_options_parse_page_theme_size_and_ready_file() {
        let opts = StartOptions::parse(
            &args(&[
                "--page",
                "selection",
                "--theme=light",
                "--size",
                "1280x800",
                "--ready-file",
                "/x/ready",
            ]),
            None,
        )
        .unwrap();
        assert_eq!(opts.page, Some(Family::Selection));
        assert!(opts.light);
        assert_eq!(opts.size, Some((1280.0, 800.0)));
        assert_eq!(opts.ready_file, Some(PathBuf::from("/x/ready")));
    }

    #[test]
    fn start_page_comes_from_the_env_and_the_flag_wins() {
        let env = StartOptions::parse(&[], Some("menus")).unwrap();
        assert_eq!(env.page, Some(Family::Menus));
        let flag = StartOptions::parse(&args(&["--page", "layout"]), Some("menus")).unwrap();
        assert_eq!(flag.page, Some(Family::Layout));
    }

    #[test]
    fn start_options_reject_bad_input() {
        for bad in [
            &["--page", "nope"][..],
            &["--theme", "blue"],
            &["--size", "wide"],
            &["--size", "0x800"],
            &["--page"],
            &["--bogus"],
        ] {
            assert!(StartOptions::parse(&args(bad), None).is_err(), "{bad:?}");
        }
    }

    #[test]
    fn every_family_has_a_unique_slug_that_round_trips() {
        let mut seen = std::collections::HashSet::new();
        for family in Family::ALL {
            let slug = family_slug(family);
            assert!(seen.insert(slug.clone()), "duplicate slug {slug}");
            assert_eq!(family_from_slug(&slug), Some(family));
        }
        assert_eq!(family_slug(Family::ButtonsAndLinks), "buttons-and-links");
    }

    #[test]
    fn starting_on_a_page_selects_it_in_host_and_nav() {
        let mut app = TestAppContext::single();
        let (root, cx) = open_fluent_test_window(&mut app, |_, cx| {
            build_gallery_with(
                cx,
                StartOptions {
                    page: Some(Family::Menus),
                    ..Default::default()
                },
            )
        });
        assert_eq!(
            host_of(&root, cx).read_with(cx, |h, _| h.active()),
            Family::Menus
        );
        let mut app2 = TestAppContext::single();
        let (plain, cx2) = open_shell(&mut app2);
        assert_eq!(
            host_of(&plain, cx2).read_with(cx2, |h, _| h.active()),
            Family::ButtonsAndLinks
        );
    }

    #[test]
    fn every_nav_item_is_a_family_and_back() {
        for family in Family::ALL {
            assert_eq!(Family::from_id(&family.id()), Some(family));
        }
        assert_eq!(Family::from_id("nope"), None);
    }

    /// Names listed in `ALL_COMPONENTS` of the screenshot script.
    fn script_components() -> Vec<&'static str> {
        let script = include_str!("../../../scripts/screenshots.sh");
        let start = script.find("ALL_COMPONENTS=(").expect("array") + "ALL_COMPONENTS=(".len();
        let end = start + script[start..].find(')').expect("closed");
        script[start..end].split_whitespace().collect()
    }

    #[test]
    fn gallery_pages_and_screenshot_list_stay_in_sync() {
        let script = script_components();
        let binary = include_str!("../../screenshots/src/main.rs");
        let mut claimed = Vec::new();
        for family in Family::ALL {
            for slug in family.screenshots() {
                assert!(
                    script.contains(slug),
                    "{family:?}: {slug} is missing from ALL_COMPONENTS in scripts/screenshots.sh"
                );
                assert!(
                    binary.contains(&format!("\"{slug}\"")),
                    "{family:?}: {slug} has no arm in examples/screenshots"
                );
                assert!(!claimed.contains(slug), "{slug} claimed by two families");
                claimed.push(*slug);
            }
        }
        for slug in script {
            assert!(
                claimed.contains(&slug),
                "{slug} is in the screenshot script but on no gallery page"
            );
        }
    }

    #[test]
    fn overlay_and_menu_buttons_open_their_popups() {
        let mut app = TestAppContext::single();
        let (root, cx) = open_shell(&mut app);
        let host = host_of(&root, cx);

        host.update(cx, |h, cx| h.set_active(Family::Overlays, cx));
        let overlays = host
            .read_with(cx, |h, _| h.page_view(Family::Overlays))
            .and_then(|v| v.downcast::<OverlaysPage>().ok())
            .expect("overlays page");
        assert!(!overlays.read_with(cx, |p, _| p.flyout_open()));
        cx.click_selector("gal-open-flyout");
        assert!(overlays.read_with(cx, |p, _| p.flyout_open()));
        // Pushing toasts replaces the open one; stacking would collide on ids.
        cx.click_selector("gal-push-gal-t-info");
        cx.click_selector("gal-push-gal-t-err");
        assert!(!cx.accessibility_snapshot().is_empty());

        host.update(cx, |h, cx| h.set_active(Family::Menus, cx));
        let menus = host
            .read_with(cx, |h, _| h.page_view(Family::Menus))
            .and_then(|v| v.downcast::<MenusPage>().ok())
            .expect("menus page");
        assert!(!menus.read_with(cx, |p, cx| p.palette_open(cx)));
        cx.click_selector("gal-open-palette");
        assert!(menus.read_with(cx, |p, cx| p.palette_open(cx)));
        assert!(!cx.accessibility_snapshot().is_empty());
    }

    #[test]
    fn variant_buttons_switch_the_theme() {
        let mut app = TestAppContext::single();
        let (_host, cx) =
            open_fluent_test_window(&mut app, |_, cx| FamilyHost::new(DemoViews::build(cx)));
        cx.click_selector("v-light");
        assert!(!is_dark(cx), "Light button left the dark theme active");
        cx.click_selector("v-dark");
        assert!(is_dark(cx), "Dark button left the light theme active");
    }

    #[test]
    fn menu_bar_with_checkbox_radio_and_shortcut_items_renders() {
        let mut app = TestAppContext::single();
        let (_bar, cx) = open_fluent_test_window(&mut app, |_, cx| {
            MenuBar::new(cx).menu(
                "&View",
                vec![
                    MenuItemDef::checkbox("Status Bar", true, |_, _, _, _| {}),
                    MenuItemDef::radio("Compact", true, |_, _, _| {}),
                    MenuItemDef::action_with_shortcut("Full Screen", "F11", |_, _, _| {}),
                ],
            )
        });
        assert!(!cx.accessibility_snapshot().is_empty());
    }
}
