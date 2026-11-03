use std::path::{Path, PathBuf};

use fluent_core::ThemeProvider as _;
use fluent_primitives::Button;
use gpui::{
    div, prelude::*, px, App, AsyncApp, ClickEvent, ElementId, IntoElement, PathPromptOptions,
    RenderOnce, Result, SharedString, Window,
};

/// A named set of accepted file extensions.
///
/// Extensions may be supplied with or without a leading dot and are matched
/// case-insensitively. GPUI does not currently pass native filters to its
/// platform picker, so open selections are filtered after the platform returns.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ExtensionFilter {
    label: SharedString,
    extensions: Vec<String>,
}

impl ExtensionFilter {
    /// Public API item.
    pub fn new(
        label: impl Into<SharedString>,
        extensions: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            label: label.into(),
            extensions: extensions
                .into_iter()
                .map(Into::into)
                .map(|extension: String| extension.trim_start_matches('.').to_ascii_lowercase())
                .filter(|extension| !extension.is_empty())
                .collect(),
        }
    }

    /// Public API item.
    pub fn label(&self) -> &SharedString {
        &self.label
    }

    /// Public API item.
    pub fn extensions(&self) -> &[String] {
        &self.extensions
    }

    fn accepts(&self, path: &Path) -> bool {
        path.extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| {
                self.extensions
                    .iter()
                    .any(|candidate| candidate.eq_ignore_ascii_case(extension))
            })
    }
}

/// Opens the native picker for one file.
///
/// On GPUI's test platform, answer the pending prompt with
/// `TestAppContext::simulate_path_prompt_response`. Cancellation returns `None`.
pub async fn open_file(cx: &AsyncApp, filters: &[ExtensionFilter]) -> Result<Option<PathBuf>> {
    Ok(open_paths(cx, false, filters).await?.and_then(|mut paths| {
        if paths.is_empty() {
            None
        } else {
            Some(paths.remove(0))
        }
    }))
}

/// Opens the native picker for one or more files.
///
/// Files that do not match any supplied extension filter are omitted. An empty
/// filter list accepts every file. Cancellation returns `None`.
pub async fn open_files(
    cx: &AsyncApp,
    filters: &[ExtensionFilter],
) -> Result<Option<Vec<PathBuf>>> {
    open_paths(cx, true, filters).await
}

/// Opens the native picker for one folder. Cancellation returns `None`.
pub async fn pick_folder(cx: &AsyncApp) -> Result<Option<PathBuf>> {
    let receiver = cx.update(|cx| {
        cx.prompt_for_paths(PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Select folder".into()),
        })
    });
    let selection = receiver.await??;

    Ok(selection.and_then(|mut paths| paths.pop()))
}

/// Opens the native save picker and applies the preferred extension.
///
/// If the selected path has no accepted extension, the first extension from
/// the first non-empty filter is appended. On GPUI's test platform, answer the
/// prompt with `TestAppContext::simulate_new_path_selection`.
pub async fn save_file(
    cx: &AsyncApp,
    directory: &Path,
    suggested_name: Option<&str>,
    filters: &[ExtensionFilter],
) -> Result<Option<PathBuf>> {
    let receiver = cx.update(|cx| cx.prompt_for_new_path(directory, suggested_name));
    let selection = receiver.await??;
    Ok(selection.map(|path| apply_preferred_extension(path, filters)))
}

async fn open_paths(
    cx: &AsyncApp,
    multiple: bool,
    filters: &[ExtensionFilter],
) -> Result<Option<Vec<PathBuf>>> {
    let receiver = cx.update(|cx| {
        cx.prompt_for_paths(PathPromptOptions {
            files: true,
            directories: false,
            multiple,
            prompt: Some(if multiple {
                "Select files".into()
            } else {
                "Select file".into()
            }),
        })
    });
    let selection = receiver.await??;

    Ok(selection.map(|paths| {
        paths
            .into_iter()
            .filter(|path| filters.is_empty() || filters.iter().any(|filter| filter.accepts(path)))
            .collect()
    }))
}

fn apply_preferred_extension(path: PathBuf, filters: &[ExtensionFilter]) -> PathBuf {
    if filters.is_empty() || filters.iter().any(|filter| filter.accepts(&path)) {
        return path;
    }

    filters
        .iter()
        .flat_map(|filter| filter.extensions.iter())
        .next()
        .map_or(path.clone(), |extension| path.with_extension(extension))
}

/// A read-only path display paired with a Fluent `Browse` button.
#[allow(clippy::type_complexity)]
#[derive(IntoElement)]
pub struct FilePickerField {
    id: ElementId,
    path: Option<PathBuf>,
    placeholder: SharedString,
    browse_label: SharedString,
    disabled: bool,
    on_browse: Option<Box<dyn Fn(&ClickEvent, &mut Window, &mut App) + 'static>>,
}

impl FilePickerField {
    /// Public API item.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            path: None,
            placeholder: "No path selected".into(),
            browse_label: "Browse".into(),
            disabled: false,
            on_browse: None,
        }
    }

    /// Public API item.
    pub fn path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Public API item.
    pub fn placeholder(mut self, placeholder: impl Into<SharedString>) -> Self {
        self.placeholder = placeholder.into();
        self
    }

    /// Public API item.
    pub fn browse_label(mut self, label: impl Into<SharedString>) -> Self {
        self.browse_label = label.into();
        self
    }

    /// Public API item.
    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Public API item.
    pub fn on_browse(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_browse = Some(Box::new(handler));
        self
    }
}

impl RenderOnce for FilePickerField {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let value = self
            .path
            .as_deref()
            .map(Path::display)
            .map(|path| path.to_string())
            .unwrap_or_else(|| self.placeholder.to_string());
        let muted = self.path.is_none() || self.disabled;
        let text_color = if muted {
            theme.colors.neutral_foreground_3.default
        } else {
            theme.colors.neutral_foreground_1.default
        };
        let mut button = Button::new((self.id.clone(), "browse"))
            .label(self.browse_label)
            .disabled(self.disabled);
        if let Some(on_browse) = self.on_browse {
            button = button.on_click(on_browse);
        }

        div()
            .id(self.id)
            .flex()
            .items_center()
            .gap(px(8.0))
            .child(
                div()
                    .flex_1()
                    .h(px(32.0))
                    .px(px(10.0))
                    .flex()
                    .items_center()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .rounded(px(4.0))
                    .border_1()
                    .border_color(theme.colors.neutral_stroke_1.default)
                    .bg(theme.colors.neutral_background_1.default)
                    .text_color(text_color)
                    .child(value),
            )
            .child(button)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[test]
    fn extension_filter_normalizes_and_matches_case_insensitively() {
        let filter = ExtensionFilter::new("Torrents", [".torrent"]);

        assert_eq!(filter.extensions(), &["torrent"]);
        assert!(filter.accepts(Path::new("ubuntu.TORRENT")));
        assert!(!filter.accepts(Path::new("ubuntu.iso")));
    }

    #[test]
    fn save_extension_preserves_allowed_extension_or_adds_preferred_one() {
        let filters = [ExtensionFilter::new("Images", ["png", "jpg"])];

        assert_eq!(
            apply_preferred_extension(PathBuf::from("image.JPG"), &filters),
            PathBuf::from("image.JPG")
        );
        assert_eq!(
            apply_preferred_extension(PathBuf::from("image"), &filters),
            PathBuf::from("image.png")
        );
    }

    #[gpui::test]
    async fn open_files_uses_test_platform_and_filters_response(cx: &mut TestAppContext) {
        let task = cx.update(|cx| {
            cx.spawn(async move |cx| {
                open_files(cx, &[ExtensionFilter::new("Torrents", ["torrent"])]).await
            })
        });
        cx.run_until_parked();
        assert!(cx.did_prompt_for_paths());
        cx.simulate_path_prompt_response(|options| {
            assert!(options.files);
            assert!(!options.directories);
            assert!(options.multiple);
            Some(vec!["one.torrent".into(), "two.txt".into()])
        });

        assert_eq!(task.await.unwrap(), Some(vec!["one.torrent".into()]));
    }

    #[gpui::test]
    async fn cancelled_folder_picker_returns_none(cx: &mut TestAppContext) {
        let task = cx.update(|cx| cx.spawn(async move |cx| pick_folder(cx).await));
        cx.run_until_parked();
        cx.simulate_path_prompt_response(|options| {
            assert!(!options.files);
            assert!(options.directories);
            assert!(!options.multiple);
            None
        });

        assert_eq!(task.await.unwrap(), None);
    }

    #[gpui::test]
    async fn cancelled_file_picker_returns_none(cx: &mut TestAppContext) {
        let task = cx.update(|cx| cx.spawn(async move |cx| open_file(cx, &[]).await));
        cx.run_until_parked();
        cx.simulate_path_prompt_response(|options| {
            assert!(options.files);
            assert!(!options.directories);
            assert!(!options.multiple);
            None
        });

        assert_eq!(task.await.unwrap(), None);
    }

    #[gpui::test]
    async fn save_picker_preserves_cancellation_and_applies_preferred_extension(
        cx: &mut TestAppContext,
    ) {
        let cancelled = cx.update(|cx| {
            cx.spawn(async move |cx| {
                save_file(
                    cx,
                    Path::new("/tmp"),
                    Some("report"),
                    &[ExtensionFilter::new("Text", ["txt"])],
                )
                .await
            })
        });
        cx.run_until_parked();
        assert!(cx.did_prompt_for_new_path());
        cx.simulate_new_path_selection(|path| {
            assert_eq!(path, Path::new("/tmp"));
            None
        });
        assert_eq!(cancelled.await.unwrap(), None);

        let selected = cx.update(|cx| {
            cx.spawn(async move |cx| {
                save_file(
                    cx,
                    Path::new("/tmp"),
                    Some("report"),
                    &[ExtensionFilter::new("Text", ["txt"])],
                )
                .await
            })
        });
        cx.run_until_parked();
        cx.simulate_new_path_selection(|_| Some(PathBuf::from("report")));
        assert_eq!(selected.await.unwrap(), Some(PathBuf::from("report.txt")));
    }
}
