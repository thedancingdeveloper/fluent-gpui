use std::sync::Arc;

use gpui::{FontFeatures, Styled};

/// Extension trait that adds typography helpers on top of GPUI's `Styled`.
///
/// Implemented blanket-style for every `Styled` element, so any `div()` /
/// `Label` / `Button` chain can call `.tabular_nums()` etc. without an
/// explicit `use`.
pub trait FluentTextExt: Styled + Sized {
    /// Enable the `tnum` OpenType feature on this element and its children —
    /// digits render at uniform widths so things like "78%" / "12:43" don't
    /// jiggle horizontally as the values change. Idempotent: calling twice
    /// only sets the feature once.
    fn tabular_nums(mut self) -> Self {
        let style = self.text_style().get_or_insert_with(Default::default);
        let mut features: Vec<(String, u32)> = style
            .font_features
            .as_ref()
            .map(|f| f.tag_value_list().to_vec())
            .unwrap_or_default();
        if !features.iter().any(|(tag, _)| tag == "tnum") {
            features.push(("tnum".to_owned(), 1));
        }
        style.font_features = Some(FontFeatures(Arc::new(features)));
        self
    }
}

impl<T: Styled> FluentTextExt for T {}
