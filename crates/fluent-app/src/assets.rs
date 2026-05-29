use std::{borrow::Cow, sync::Arc};

use gpui::{AssetSource, Result, SharedString};

const ADD: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M7 2h2v5h5v2H9v5H7V9H2V7h5z"/></svg>"#;
const ARROW_FORWARD: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M9.3 3.3 14 8l-4.7 4.7-1.4-1.4L10.2 9H2V7h8.2L7.9 4.7z"/></svg>"#;
const CHECKMARK: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="m6.2 11.4-3.3-3.3 1.4-1.4 1.9 1.9 5.5-5.5 1.4 1.4z"/></svg>"#;
const CHEVRON_DOWN: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M4.2 6 8 9.8 11.8 6l1.2 1.2-5 5-5-5z"/></svg>"#;
const CHEVRON_RIGHT: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="m6 3 5 5-5 5-1.2-1.2L8.6 8 4.8 4.2z"/></svg>"#;
const CHEVRON_UP: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M4.2 10 8 6.2l3.8 3.8 1.2-1.2-5-5-5 5z"/></svg>"#;
const CLEAR: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M4.3 3 8 6.7 11.7 3 13 4.3 9.3 8l3.7 3.7-1.3 1.3L8 9.3 4.3 13 3 11.7 6.7 8 3 4.3z"/></svg>"#;
const COPY: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M5 2h7v9H5zm-2 3h1v7h6v1H3z"/></svg>"#;
const CUT: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M4 5a2 2 0 1 1 1.4-.6L8 7l2.6-2.6A2 2 0 1 1 12 5a2 2 0 0 1-.7-.1L9.4 8l1.9 3.1A2 2 0 1 1 10 12l-2-2-2 2a2 2 0 1 1-1.3-.9L6.6 8 4.7 4.9A2 2 0 0 1 4 5z"/></svg>"#;
const DARK: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M9.6 1.2a6.8 6.8 0 1 0 5.2 5.2A5.2 5.2 0 1 1 9.6 1.2z"/></svg>"#;
const DISMISS: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="m4.3 3 3.7 3.7L11.7 3 13 4.3 9.3 8l3.7 3.7-1.3 1.3L8 9.3 4.3 13 3 11.7 6.7 8 3 4.3z"/></svg>"#;
const FOLDER: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M2 4h5l1 1h6v7.5A1.5 1.5 0 0 1 12.5 14h-9A1.5 1.5 0 0 1 2 12.5z"/></svg>"#;
const GRID: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M2 2h5v5H2zm7 0h5v5H9zM2 9h5v5H2zm7 0h5v5H9z"/></svg>"#;
const LIGHT: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 4a4 4 0 1 1 0 8A4 4 0 0 1 8 4zm0-3h1v2H7V1zm0 12h1v2H7v-2zM1 7h2v2H1zm12 0h2v2h-2zM3 2.3 4.4 3.7 3.7 4.4 2.3 3zm9.3 9.3 1.4 1.4-.7.7-1.4-1.4zM13.7 3l-1.4 1.4-.7-.7L13 2.3zM4.4 12.3 3 13.7l-.7-.7 1.4-1.4z"/></svg>"#;
const LIST_TREE: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 2h2v2H3zm4 .5h7v1H7zM3 7h2v2H3zm4 .5h7v1H7zM3 12h2v2H3zm4 .5h7v1H7z"/></svg>"#;
const MINIMIZE: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 8h10v1.5H3z"/></svg>"#;
const MAXIMIZE: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 3h10v10H3zm1.5 1.5v7h7v-7z"/></svg>"#;
const PANEL_OPEN: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M2 3h12v10H2zm2 1.5v7h2.5v-7zm4 0v7h4.5v-7z"/></svg>"#;
const PASTE: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M6 1h4l.5 1H13v12H3V2h2.5zm0 2v1h4V3zM5 6v6h6V6z"/></svg>"#;
const PLUG: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M6 1h1v4h2V1h1v4h1v3a4 4 0 0 1-3 3.9V15H7v-3.1A4 4 0 0 1 4 8V5h2z"/></svg>"#;
const SEARCH: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M7 2a5 5 0 0 1 3.9 8.1l3 3-1.4 1.4-3-3A5 5 0 1 1 7 2zm0 2a3 3 0 1 0 0 6 3 3 0 0 0 0-6z"/></svg>"#;
const SETTINGS: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M7 1h2l.4 2 1.7.7 1.7-1.1 1.4 1.4-1.1 1.7.7 1.7 2 .4v2l-2 .4-.7 1.7 1.1 1.7-1.4 1.4-1.7-1.1-1.7.7-.4 2H7l-.4-2-1.7-.7-1.7 1.1-1.4-1.4 1.1-1.7-.7-1.7-2-.4V8l2-.4.7-1.7-1.1-1.7 1.4-1.4 1.7 1.1 1.7-.7zM8 6a2 2 0 1 0 0 4 2 2 0 0 0 0-4z"/></svg>"#;
const SPINNER: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 1a7 7 0 0 1 7 7h-2a5 5 0 0 0-5-5z"/></svg>"#;
const DOCUMENT: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 2h7l3 3v9H3V2zm7 0v3h3"/></svg>"#;
const DOWNLOAD: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 11 4 7h2.5V2h3v5H12zm-6 2h12v1.5H2z"/></svg>"#;
const UPLOAD: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 2l4 4H9.5v5h-3V6H4zm-6 11h12v1.5H2z"/></svg>"#;
const GENERIC: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 3h10v10H3z"/></svg>"#;

// --- AndroidConnect design-system icons (Fluent System Icons-style, 16×16) ---
const HOME: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 1.5 14.5 7H13v6.5a.5.5 0 0 1-.5.5H10V10H6v4H3.5a.5.5 0 0 1-.5-.5V7H1.5z"/></svg>"#;
const MIRROR: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M4 2h8a1.5 1.5 0 0 1 1.5 1.5v9A1.5 1.5 0 0 1 12 14H4a1.5 1.5 0 0 1-1.5-1.5v-9A1.5 1.5 0 0 1 4 2zm0 1.5v9h8v-9zM6.5 12h3v.5h-3z"/></svg>"#;
const BELL: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 1.5a.75.75 0 0 1 .75.75v.55A4.5 4.5 0 0 1 12.5 7v2.5l1 1.5v.5h-11v-.5l1-1.5V7a4.5 4.5 0 0 1 3.75-4.2v-.55A.75.75 0 0 1 8 1.5zM6.5 13h3a1.5 1.5 0 0 1-3 0z"/></svg>"#;
const BELL_OFF: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="m2 2 12 12-1 1L1.5 3zm6 0a.75.75 0 0 1 .75.75v.55A4.5 4.5 0 0 1 12.5 7v2.5l.62.93L4.4 1.81A4.5 4.5 0 0 1 7.25 3.3v-.55A.75.75 0 0 1 8 2zM3.5 7c0-.4.05-.78.14-1.14l9 9H2.5v-.5l1-1.5V7zm3 6h3a1.5 1.5 0 0 1-3 0z"/></svg>"#;
const CHAT: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 2h10a1.5 1.5 0 0 1 1.5 1.5v7A1.5 1.5 0 0 1 13 12H7l-3.5 2.5V12H3A1.5 1.5 0 0 1 1.5 10.5v-7A1.5 1.5 0 0 1 3 2z"/></svg>"#;
const PHONE: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M5 1.5h6A1.5 1.5 0 0 1 12.5 3v10a1.5 1.5 0 0 1-1.5 1.5H5A1.5 1.5 0 0 1 3.5 13V3A1.5 1.5 0 0 1 5 1.5zm0 1.5v8h6V3zm2.25 9.25a.75.75 0 1 1 1.5 0 .75.75 0 0 1-1.5 0z"/></svg>"#;
const QR: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M2 2h5v5H2zm1.5 1.5v2h2v-2zM9 2h5v5H9zm1.5 1.5v2h2v-2zM2 9h5v5H2zm1.5 1.5v2h2v-2zM9 9h2v2H9zm3 0h2v2h-2zm-3 3h2v2H9zm3 0h2v2h-2z"/></svg>"#;
const BATTERY: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M2 5h10a1 1 0 0 1 1 1v4a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V6a1 1 0 0 1 1-1zm0 1v4h10V6zm12 1h1v2h-1zM3 7h6v2H3z"/></svg>"#;
const WIFI: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 12.5a1 1 0 1 1 0 2 1 1 0 0 1 0-2zM3.5 9.5a6 6 0 0 1 9 0l-1 1a4.5 4.5 0 0 0-7 0zM1.5 7a9 9 0 0 1 13 0l-1 1a7.5 7.5 0 0 0-11 0z"/></svg>"#;
const SIGNAL: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M2 12h2v2H2zm3-2h2v4H5zm3-3h2v7H8zm3-4h2v11h-2z"/></svg>"#;
const BLUETOOTH: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M7 1.5 11 5.5 8 8l3 2.5-4 4V9.5L4 12 3 11l4-3-4-3 1-1 3 2.5V1.5zm1.5 2.4v3.2L10 5.5zm0 5.7v3.2l1.5-1.6z"/></svg>"#;
const SUN: &[u8] = LIGHT;
const MOON: &[u8] = DARK;
const VOL: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 2.5v11l-3.5-3H2v-5h2.5zm2.8 1.6c1.3 1 2.2 2.5 2.2 3.9s-.9 2.9-2.2 3.9l-.8-1.2c1-.7 1.5-1.7 1.5-2.7s-.5-2-1.5-2.7zm-1.4 2c.7.5 1.1 1.2 1.1 1.9 0 .7-.4 1.4-1.1 1.9l-.7-1.1c.4-.3.6-.6.6-.8 0-.2-.2-.5-.6-.8z"/></svg>"#;
const VOL_MUTE: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 2.5v11l-3.5-3H2v-5h2.5zm3 3 1.5 1.5L14 5.5 15 6.5 13.5 8 15 9.5 14 10.5l-1.5-1.5L11 10.5 10 9.5 11.5 8 10 6.5z"/></svg>"#;
const PLAY: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M4 2.5v11L13 8z"/></svg>"#;
const PAUSE: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M4 3h3v10H4zm5 0h3v10H9z"/></svg>"#;
const PREV: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 3h2v10H3zm10 0v10L6 8z"/></svg>"#;
const NEXT: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M11 3h2v10h-2zM3 3l7 5-7 5z"/></svg>"#;
const SEND: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M2 2 14 8 2 14l2-6zm1.7 1.4L4.8 7H8l-3.5.8z"/></svg>"#;
const BACK: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M10 3 5 8l5 5 1.2-1.2L7.4 8l3.8-3.8z"/></svg>"#;
const MORE: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 8a1.25 1.25 0 1 1 2.5 0A1.25 1.25 0 0 1 3 8zm3.75 0a1.25 1.25 0 1 1 2.5 0 1.25 1.25 0 0 1-2.5 0zm3.75 0a1.25 1.25 0 1 1 2.5 0 1.25 1.25 0 0 1-2.5 0z"/></svg>"#;
const REFRESH: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M8 2a6 6 0 0 1 5.5 3.6L15 3.5V8h-4.5l1.6-1.5A4.5 4.5 0 0 0 3.5 8H2A6 6 0 0 1 8 2zm0 12a6 6 0 0 1-5.5-3.6L1 12.5V8h4.5L3.9 9.5A4.5 4.5 0 0 0 12.5 8H14A6 6 0 0 1 8 14z"/></svg>"#;
const IMAGE: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 2.5h10A1.5 1.5 0 0 1 14.5 4v8a1.5 1.5 0 0 1-1.5 1.5H3A1.5 1.5 0 0 1 1.5 12V4A1.5 1.5 0 0 1 3 2.5zm0 1.5v6.5l3-3 2 2 3-3 2 2V4zM5 6a1 1 0 1 1 0 2 1 1 0 0 1 0-2z"/></svg>"#;
const VIDEO: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M2 4h8a1 1 0 0 1 1 1v6a1 1 0 0 1-1 1H2a1 1 0 0 1-1-1V5a1 1 0 0 1 1-1zm10 1.5 3-1.5v8l-3-1.5z"/></svg>"#;
const ZIP: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M3 1.5h6L13 5v9.5H3zm6 0V5h4M6 3h1v1H6zm1 1h1v1H7zm-1 1h1v1H6zm1 1h1v1H7zm-1 1h1v1H6zm0 2h2v3H6z"/></svg>"#;
const PIN: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M10.5 1.5 14.5 5.5 13 7l-1-1-3.5 3.5 1 4-1 1-3-3-2.5 2.5L2 13.5l2.5-2.5-3-3 1-1 4 1L10 4.5l-1-1z"/></svg>"#;
const LOGO: &[u8] = br#"<svg viewBox="0 0 16 16" xmlns="http://www.w3.org/2000/svg"><path fill="currentColor" d="M5 1h6a1 1 0 0 1 1 1v12a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1zm1 2v9h4V3zM8 13a.75.75 0 1 1 0 1.5A.75.75 0 0 1 8 13z"/></svg>"#;

/// Built-in FluentGUI assets plus an optional application asset fallback.
#[derive(Default)]
pub struct FluentAssets {
    fallback: Option<Arc<dyn AssetSource>>,
}

impl FluentAssets {
    pub fn new(fallback: Option<Arc<dyn AssetSource>>) -> Self {
        Self { fallback }
    }

    fn builtin(path: &str) -> Option<&'static [u8]> {
        Some(match path {
            "icons/add.svg" | "icons/folder_add.svg" => ADD,
            "icons/arrow_forward.svg" => ARROW_FORWARD,
            "icons/checkmark.svg" | "icons/check.svg" => CHECKMARK,
            "icons/chevron_down.svg" => CHEVRON_DOWN,
            "icons/chevron_right.svg" | "icons/chevron.svg" => CHEVRON_RIGHT,
            "icons/chevron_up.svg" => CHEVRON_UP,
            "icons/clear.svg" | "icons/eraser.svg" => CLEAR,
            "icons/copy.svg" => COPY,
            "icons/cut.svg" => CUT,
            "icons/dismiss.svg"
            | "icons/delete.svg"
            | "icons/plug_disconnected.svg"
            | "icons/close.svg" => DISMISS,
            "icons/document.svg" | "icons/file.svg" | "icons/doc.svg" => DOCUMENT,
            "icons/download.svg" | "icons/download_arrow.svg" => DOWNLOAD,
            "icons/upload.svg" | "icons/upload_arrow.svg" => UPLOAD,
            "icons/folder.svg" | "icons/folder_open.svg" | "icons/folder_sync.svg" => FOLDER,
            "icons/grid.svg" => GRID,
            "icons/light_theme.svg" | "icons/sun.svg" => SUN,
            "icons/dark_theme.svg" | "icons/moon.svg" => MOON,
            "icons/list_tree.svg" => LIST_TREE,
            "icons/minimize.svg" => MINIMIZE,
            "icons/maximize.svg" | "icons/full_screen.svg" | "icons/fit_width.svg" => MAXIMIZE,
            "icons/panel_open.svg" => PANEL_OPEN,
            "icons/paste.svg" | "icons/clipboard_paste.svg" | "icons/clipboard_sync.svg" => PASTE,
            "icons/plug.svg" | "icons/connect.svg" | "icons/network.svg" => PLUG,
            "icons/search.svg" | "icons/find_replace.svg" => SEARCH,
            "icons/settings.svg" => SETTINGS,
            "icons/spinner.svg" => SPINNER,
            // AndroidConnect design-system additions
            "icons/home.svg" => HOME,
            "icons/mirror.svg" => MIRROR,
            "icons/bell.svg" => BELL,
            "icons/bell_off.svg" => BELL_OFF,
            "icons/chat.svg" => CHAT,
            "icons/phone.svg" => PHONE,
            "icons/qr.svg" => QR,
            "icons/battery.svg" => BATTERY,
            "icons/wifi.svg" => WIFI,
            "icons/signal.svg" => SIGNAL,
            "icons/bluetooth.svg" => BLUETOOTH,
            "icons/vol.svg" | "icons/volume.svg" | "icons/speaker.svg" => VOL,
            "icons/vol_mute.svg" | "icons/volume_mute.svg" | "icons/speaker_mute.svg" => VOL_MUTE,
            "icons/play.svg" => PLAY,
            "icons/pause.svg" => PAUSE,
            "icons/prev.svg" | "icons/previous.svg" => PREV,
            "icons/next.svg" => NEXT,
            "icons/send.svg" => SEND,
            "icons/back.svg" | "icons/chevron_left.svg" => BACK,
            "icons/more.svg" => MORE,
            "icons/refresh.svg" => REFRESH,
            "icons/image.svg" => IMAGE,
            "icons/video.svg" => VIDEO,
            "icons/zip.svg" => ZIP,
            "icons/pin.svg" => PIN,
            "icons/logo.svg" => LOGO,
            _ => return None,
        })
    }

    fn generic_icon(path: &str) -> Option<&'static [u8]> {
        (path.starts_with("icons/") && path.ends_with(".svg")).then_some(GENERIC)
    }
}

impl AssetSource for FluentAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if let Some(bytes) = Self::builtin(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        if let Some(fallback) = &self.fallback {
            if let Some(bytes) = fallback.load(path)? {
                return Ok(Some(bytes));
            }
        }
        if let Some(bytes) = Self::generic_icon(path) {
            return Ok(Some(Cow::Borrowed(bytes)));
        }
        Ok(None)
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        if let Some(fallback) = &self.fallback {
            return fallback.list(path);
        }
        let names = if path == "icons" {
            vec![
                "add.svg",
                "arrow_forward.svg",
                "checkmark.svg",
                "chevron_down.svg",
                "chevron_right.svg",
                "chevron_up.svg",
                "clear.svg",
                "copy.svg",
                "cut.svg",
                "dark_theme.svg",
                "dismiss.svg",
                "folder_open.svg",
                "grid.svg",
                "light_theme.svg",
                "list_tree.svg",
                "maximize.svg",
                "minimize.svg",
                "panel_open.svg",
                "paste.svg",
                "plug.svg",
                "search.svg",
                "settings.svg",
                "spinner.svg",
            ]
        } else {
            vec![]
        };
        Ok(names.into_iter().map(SharedString::from).collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CUSTOM_ICON: &[u8] = br#"<svg viewBox="0 0 1 1"><path d="M0 0h1v1H0z"/></svg>"#;

    struct CustomAssets;

    impl AssetSource for CustomAssets {
        fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
            Ok((path == "icons/custom.svg").then_some(Cow::Borrowed(CUSTOM_ICON)))
        }

        fn list(&self, _path: &str) -> Result<Vec<SharedString>> {
            Ok(Vec::new())
        }
    }

    #[test]
    fn app_icon_assets_are_loaded_before_generic_icon_fallback() {
        let assets = FluentAssets::new(Some(Arc::new(CustomAssets)));
        let loaded = assets.load("icons/custom.svg").unwrap().unwrap();

        assert_eq!(&*loaded, CUSTOM_ICON);
    }

    #[test]
    fn unknown_icons_still_use_generic_fallback() {
        let assets = FluentAssets::new(None);
        let loaded = assets.load("icons/missing.svg").unwrap().unwrap();

        assert_eq!(&*loaded, GENERIC);
    }

    #[test]
    fn chevron_right_has_a_specific_builtin_icon() {
        let assets = FluentAssets::new(None);
        let loaded = assets.load("icons/chevron_right.svg").unwrap().unwrap();

        assert_eq!(&*loaded, CHEVRON_RIGHT);
        assert_ne!(&*loaded, GENERIC);
    }
}
