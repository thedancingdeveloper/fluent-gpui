use gpui::Styled;

/// Single-line truncation with a trailing ellipsis.
pub trait TruncateExt: Styled + Sized {
    /// Keeps the element on one line, clips overflow and ends it with an
    /// ellipsis. Also sets `min-width: 0` so a flex item can shrink below its
    /// content width.
    fn truncate_label(self) -> Self {
        self.min_w_0()
            .overflow_hidden()
            .whitespace_nowrap()
            .text_ellipsis()
    }
}

impl<T: Styled + Sized> TruncateExt for T {}
