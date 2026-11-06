//! Public API for this crate.
#![deny(missing_docs)]

mod app_builder;
mod assets;
mod chrome;
mod pickers;
mod title_bar;
mod window_title;

pub use app_builder::{title_bar, window_bounds, window_title_bar, FluentApp};
pub use assets::FluentAssets;
pub use chrome::{wrap_with_chrome, WindowChrome};
pub use pickers::{
    open_file, open_files, pick_folder, save_file, ExtensionFilter, FilePickerField,
};
pub use title_bar::TitleBar;
pub use window_title::WindowTitle;
