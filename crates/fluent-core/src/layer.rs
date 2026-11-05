//! Paint order for deferred popups.
//!
//! `deferred(..).with_priority(n)` paints higher priorities on top. Every popup
//! picks its priority from this table so independent popups stack the same way
//! wherever they are opened: tooltip > context menu > submenu > menu >
//! dropdown / popover. A popup nested in another uses the parent's priority
//! plus its depth (`SUBMENU + depth`, `CONTEXT_MENU + depth`), which stays
//! inside its tier because the tiers are spaced well apart.

/// Popovers, suggestion lists and anything else with no stronger claim.
pub const POPOVER: usize = 0;
/// Dropdown, combobox and date/time picker lists.
pub const DROPDOWN: usize = 0;
/// Menu-button and menu-bar menus, and `Flyout` by default.
pub const MENU: usize = 10;
/// Submenus of menu-button and menu-bar menus; add the nesting depth.
pub const SUBMENU: usize = 20;
/// Pointer-positioned context menus and their submenus; add the nesting depth.
pub const CONTEXT_MENU: usize = 40;
/// Tooltips paint over every other popup.
pub const TOOLTIP: usize = 80;
