use fluent_core::ThemeProvider as _;
use gpui::{
    div, img, prelude::*, px, App, FontWeight, IntoElement, RenderOnce, SharedString, Window,
};

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RichDocument {
    pub blocks: Vec<RichDocumentBlock>,
}

impl RichDocument {
    pub fn new() -> Self {
        Self { blocks: Vec::new() }
    }

    pub fn block(mut self, block: RichDocumentBlock) -> Self {
        self.blocks.push(block);
        self
    }

    pub fn from_plain_text(text: impl AsRef<str>) -> Self {
        let blocks = text
            .as_ref()
            .split("\n\n")
            .filter_map(|paragraph| {
                let text = paragraph.trim();
                (!text.is_empty()).then(|| {
                    RichDocumentBlock::Paragraph(vec![RichTextRun::new(text.replace('\n', " "))])
                })
            })
            .collect();
        Self { blocks }
    }

    pub fn from_html_fragment_lossy(html: impl AsRef<str>) -> Self {
        HtmlDocumentBuilder::parse(html.as_ref())
    }

    pub fn to_plain_text(&self) -> String {
        let mut lines = Vec::new();
        for block in &self.blocks {
            match block {
                RichDocumentBlock::Heading { runs, .. }
                | RichDocumentBlock::Paragraph(runs)
                | RichDocumentBlock::Preformatted(runs) => lines.push(runs_text(runs)),
                RichDocumentBlock::Table { rows } => {
                    for row in rows {
                        lines.push(
                            row.iter()
                                .map(|cell| cell.as_ref())
                                .collect::<Vec<_>>()
                                .join(" | "),
                        );
                    }
                }
                RichDocumentBlock::Image { alt, src } => {
                    let label = if alt.is_empty() {
                        "Image"
                    } else {
                        alt.as_ref()
                    };
                    lines.push(format!("{label}: {src}"));
                }
                RichDocumentBlock::Divider => lines.push(String::new()),
            }
        }
        lines.join("\n")
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RichDocumentBlock {
    Heading {
        level: u8,
        runs: Vec<RichTextRun>,
    },
    Paragraph(Vec<RichTextRun>),
    Preformatted(Vec<RichTextRun>),
    Table {
        rows: Vec<Vec<SharedString>>,
    },
    Image {
        alt: SharedString,
        src: SharedString,
    },
    Divider,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RichTextRun {
    pub text: SharedString,
    pub strong: bool,
    pub emphasis: bool,
    pub code: bool,
    pub link: Option<SharedString>,
}

impl RichTextRun {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            strong: false,
            emphasis: false,
            code: false,
            link: None,
        }
    }

    pub fn strong(mut self, strong: bool) -> Self {
        self.strong = strong;
        self
    }

    pub fn emphasis(mut self, emphasis: bool) -> Self {
        self.emphasis = emphasis;
        self
    }

    pub fn code(mut self, code: bool) -> Self {
        self.code = code;
        self
    }

    pub fn link(mut self, href: impl Into<SharedString>) -> Self {
        self.link = Some(href.into());
        self
    }
}

/// A safe rich document renderer for sanitized app content, email previews,
/// release notes, and help text. Raw HTML is converted into a lossy internal
/// document model before rendering; script, style, and unknown tags are ignored.
#[derive(IntoElement)]
pub struct DocumentViewer {
    document: RichDocument,
    empty_message: SharedString,
    max_height: Option<f32>,
}

impl DocumentViewer {
    pub fn new(document: RichDocument) -> Self {
        Self {
            document,
            empty_message: "No content".into(),
            max_height: None,
        }
    }

    pub fn plain_text(text: impl AsRef<str>) -> Self {
        Self::new(RichDocument::from_plain_text(text))
    }

    pub fn html_fragment(html: impl AsRef<str>) -> Self {
        Self::new(RichDocument::from_html_fragment_lossy(html))
    }

    pub fn empty_message(mut self, message: impl Into<SharedString>) -> Self {
        self.empty_message = message.into();
        self
    }

    pub fn max_height(mut self, height: f32) -> Self {
        self.max_height = Some(height.max(64.0));
        self
    }
}

impl RenderOnce for DocumentViewer {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let colors = theme.colors.clone();
        let spacing = theme.spacing;
        let typography = theme.typography;

        let mut root = div()
            .id("document-viewer")
            .flex()
            .flex_col()
            .gap(px(spacing.md))
            .w_full()
            .min_h_0()
            .text_color(colors.on_neutral);

        if let Some(height) = self.max_height {
            root = root.max_h(px(height)).overflow_y_scroll();
        }

        if self.document.blocks.is_empty() {
            return root.child(
                div()
                    .min_h(px(96.0))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(typography.body.size))
                    .text_color(colors.on_subtle)
                    .child(self.empty_message),
            );
        }

        for (idx, block) in self.document.blocks.into_iter().enumerate() {
            root = root.child(render_block(idx, block, cx));
        }

        root
    }
}

fn render_block(idx: usize, block: RichDocumentBlock, cx: &mut App) -> gpui::AnyElement {
    let theme = cx.theme();
    let colors = theme.colors.clone();
    let spacing = theme.spacing;
    let typography = theme.typography;

    match block {
        RichDocumentBlock::Heading { level, runs } => {
            let (size, weight) = match level {
                1 => (typography.title.size, FontWeight::SEMIBOLD),
                2 | 3 => (typography.subtitle.size, FontWeight::SEMIBOLD),
                _ => (typography.body.size, FontWeight::SEMIBOLD),
            };
            div()
                .id(("document-heading", idx as u64))
                .text_size(px(size))
                .font_weight(weight)
                .text_color(colors.on_neutral)
                .child(runs_text(&runs))
                .into_any_element()
        }
        RichDocumentBlock::Paragraph(runs) => div()
            .id(("document-paragraph", idx as u64))
            .flex()
            .flex_row()
            .flex_wrap()
            .gap(px(3.0))
            .line_height(px(typography.body.line_height))
            .children(runs.into_iter().map(|run| render_run(run, cx)))
            .into_any_element(),
        RichDocumentBlock::Preformatted(runs) => div()
            .id(("document-pre", idx as u64))
            .p(px(spacing.md))
            .rounded(px(theme.radii.sm))
            .bg(colors.neutral)
            .text_size(px(typography.caption.size))
            .line_height(px(typography.body.line_height))
            .children(runs.into_iter().map(|run| render_run(run.code(true), cx)))
            .into_any_element(),
        RichDocumentBlock::Table { rows } => render_table(idx, rows, cx).into_any_element(),
        RichDocumentBlock::Image { alt, src } => {
            let src_str = src.as_ref();
            if src_str.starts_with("http://")
                || src_str.starts_with("https://")
                || src_str.starts_with("data:")
            {
                // Render the image using GPUI's built-in async image loader.
                // max_w_full keeps it from overflowing the column; max_h caps
                // very tall images so the reading pane stays manageable.
                div()
                    .id(("document-image", idx as u64))
                    .max_w_full()
                    .child(
                        img(src_str)
                            .max_w_full()
                            .max_h(px(480.0))
                            .object_fit(gpui::ObjectFit::ScaleDown),
                    )
                    .into_any_element()
            } else {
                // cid: references, relative paths, etc. — show an alt label.
                div()
                    .id(("document-image", idx as u64))
                    .p(px(spacing.md))
                    .rounded(px(theme.radii.sm))
                    .border_1()
                    .border_color(colors.stroke_neutral_subtle)
                    .text_size(px(typography.caption.size))
                    .text_color(colors.on_subtle)
                    .child(if alt.is_empty() {
                        "Image".to_string()
                    } else {
                        alt.to_string()
                    })
                    .into_any_element()
            }
        }
        RichDocumentBlock::Divider => div()
            .id(("document-divider", idx as u64))
            .h(px(1.0))
            .bg(colors.stroke_neutral_subtle)
            .into_any_element(),
    }
}

fn render_run(run: RichTextRun, cx: &mut App) -> gpui::AnyElement {
    let theme = cx.theme();
    let colors = theme.colors.clone();
    let typography = theme.typography;

    let mut el = div()
        .text_size(px(if run.code {
            typography.caption.size
        } else {
            typography.body.size
        }))
        .text_color(if run.link.is_some() {
            colors.on_neutral_accent
        } else {
            colors.on_neutral
        })
        .font_weight(if run.strong {
            FontWeight::SEMIBOLD
        } else {
            FontWeight::NORMAL
        });

    if run.emphasis {
        el = el.italic();
    }
    if run.link.is_some() {
        el = el.underline();
    }
    if run.code {
        el = el
            .px(px(3.0))
            .rounded(px(theme.radii.sm))
            .bg(colors.neutral);
    }

    el.child(run.text).into_any_element()
}

fn render_table(idx: usize, rows: Vec<Vec<SharedString>>, cx: &mut App) -> impl IntoElement {
    let theme = cx.theme();
    let colors = theme.colors.clone();
    let spacing = theme.spacing;
    let typography = theme.typography;
    let mut table = div()
        .id(("document-table", idx as u64))
        .flex()
        .flex_col()
        .border_1()
        .border_color(colors.stroke_neutral_subtle)
        .rounded(px(theme.radii.sm))
        .overflow_hidden();

    for (row_idx, row) in rows.into_iter().enumerate() {
        let mut row_el = div()
            .id(("document-table-row", idx as u64 * 1000 + row_idx as u64))
            .flex()
            .flex_row()
            .bg(if row_idx == 0 {
                colors.surface_dim
            } else {
                colors.surface
            });
        for (cell_idx, cell) in row.into_iter().enumerate() {
            row_el = row_el.child(
                div()
                    .id((
                        "document-table-cell",
                        idx as u64 * 1_000_000 + row_idx as u64 * 1000 + cell_idx as u64,
                    ))
                    .flex_1()
                    .p(px(spacing.sm))
                    .border_r_1()
                    .border_color(colors.stroke_neutral_subtle)
                    .text_size(px(typography.caption.size))
                    .font_weight(if row_idx == 0 {
                        FontWeight::SEMIBOLD
                    } else {
                        FontWeight::NORMAL
                    })
                    .child(cell),
            );
        }
        table = table.child(row_el);
    }

    table
}

fn runs_text(runs: &[RichTextRun]) -> String {
    runs.iter()
        .map(|run| run.text.as_ref())
        .collect::<Vec<_>>()
        .join("")
        .trim()
        .to_string()
}

#[derive(Default)]
struct HtmlDocumentBuilder {
    document: RichDocument,
    runs: Vec<RichTextRun>,
    heading: Option<u8>,
    strong_depth: usize,
    emphasis_depth: usize,
    code_depth: usize,
    link_stack: Vec<SharedString>,
    skip_stack: Vec<String>,
    table: Option<Vec<Vec<SharedString>>>,
    row: Option<Vec<SharedString>>,
    cell: Option<String>,
}

impl HtmlDocumentBuilder {
    fn parse(html: &str) -> RichDocument {
        let mut builder = Self::default();
        let mut pos = 0;
        while let Some(start) = html[pos..].find('<') {
            let start = pos + start;
            builder.push_text(&html[pos..start]);
            let Some(end) = html[start..].find('>') else {
                builder.push_text(&html[start..]);
                return builder.finish();
            };
            let tag = &html[start + 1..start + end];
            builder.handle_tag(tag);
            pos = start + end + 1;
        }
        builder.push_text(&html[pos..]);
        builder.finish()
    }

    fn finish(mut self) -> RichDocument {
        self.flush_runs();
        self.finish_table();
        self.document
    }

    fn handle_tag(&mut self, raw: &str) {
        let raw = raw.trim();
        if raw.is_empty() || raw.starts_with('!') || raw.starts_with('?') {
            return;
        }

        let closing = raw.starts_with('/');
        let tag = if closing { raw[1..].trim() } else { raw };
        let self_closing = tag.ends_with('/');
        let name = tag
            .split_whitespace()
            .next()
            .unwrap_or_default()
            .trim_end_matches('/')
            .to_ascii_lowercase();

        if !self.skip_stack.is_empty() {
            if closing && self.skip_stack.last().is_some_and(|skip| skip == &name) {
                self.skip_stack.pop();
            } else if !closing && is_skipped_tag(&name) && !self_closing {
                self.skip_stack.push(name);
            }
            return;
        }

        if closing {
            self.end_tag(&name);
        } else {
            self.start_tag(&name, tag);
            if self_closing {
                self.end_tag(&name);
            }
        }
    }

    fn start_tag(&mut self, name: &str, raw: &str) {
        match name {
            "script" | "style" | "head" | "title" => self.skip_stack.push(name.to_string()),
            "p" | "div" | "section" | "article" | "body" | "html" => self.flush_runs(),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                self.flush_runs();
                self.heading = name[1..].parse().ok();
            }
            "br" => self.push_run("\n".into()),
            "hr" => {
                self.flush_runs();
                self.document.blocks.push(RichDocumentBlock::Divider);
            }
            "strong" | "b" => self.strong_depth += 1,
            "em" | "i" => self.emphasis_depth += 1,
            "code" | "pre" => {
                self.flush_runs();
                self.code_depth += 1;
            }
            "a" => {
                if let Some(href) = attr_value(raw, "href") {
                    self.link_stack.push(href.into());
                }
            }
            "ul" | "ol" => self.flush_runs(),
            "li" => {
                self.flush_runs();
                self.push_run("- ".to_string());
            }
            "table" => {
                self.flush_runs();
                self.table = Some(Vec::new());
            }
            "tr" => self.row = Some(Vec::new()),
            "td" | "th" => self.cell = Some(String::new()),
            "img" => {
                self.flush_runs();
                let src = attr_value(raw, "src").unwrap_or_default();
                if !src.is_empty() {
                    self.document.blocks.push(RichDocumentBlock::Image {
                        alt: attr_value(raw, "alt").unwrap_or_default().into(),
                        src: src.into(),
                    });
                }
            }
            _ => {}
        }
    }

    fn end_tag(&mut self, name: &str) {
        match name {
            "p" | "div" | "section" | "article" | "body" | "html" | "li" => self.flush_runs(),
            "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => self.flush_runs(),
            "strong" | "b" => self.strong_depth = self.strong_depth.saturating_sub(1),
            "em" | "i" => self.emphasis_depth = self.emphasis_depth.saturating_sub(1),
            "code" | "pre" => {
                self.flush_runs();
                self.code_depth = self.code_depth.saturating_sub(1);
            }
            "a" => {
                self.link_stack.pop();
            }
            "td" | "th" => {
                if let (Some(row), Some(cell)) = (self.row.as_mut(), self.cell.take()) {
                    row.push(cell.trim().to_string().into());
                }
            }
            "tr" => {
                if let (Some(table), Some(row)) = (self.table.as_mut(), self.row.take()) {
                    if !row.is_empty() {
                        table.push(row);
                    }
                }
            }
            "table" => self.finish_table(),
            _ => {}
        }
    }

    fn push_text(&mut self, raw: &str) {
        if !self.skip_stack.is_empty() || raw.is_empty() {
            return;
        }
        let text = collapse_whitespace(&decode_entities(raw));
        if text.trim().is_empty() {
            return;
        }

        if let Some(cell) = self.cell.as_mut() {
            if !cell.is_empty() {
                cell.push(' ');
            }
            cell.push_str(text.trim());
            return;
        }

        self.push_run(text);
    }

    fn push_run(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        let mut run = RichTextRun::new(text)
            .strong(self.strong_depth > 0)
            .emphasis(self.emphasis_depth > 0)
            .code(self.code_depth > 0);
        if let Some(link) = self.link_stack.last() {
            run = run.link(link.clone());
        }
        self.runs.push(run);
    }

    fn flush_runs(&mut self) {
        trim_runs(&mut self.runs);
        if self.runs.is_empty() {
            self.heading = None;
            return;
        }

        let runs = std::mem::take(&mut self.runs);
        if let Some(level) = self.heading.take() {
            self.document
                .blocks
                .push(RichDocumentBlock::Heading { level, runs });
        } else if self.code_depth > 0 {
            self.document
                .blocks
                .push(RichDocumentBlock::Preformatted(runs));
        } else {
            self.document
                .blocks
                .push(RichDocumentBlock::Paragraph(runs));
        }
    }

    fn finish_table(&mut self) {
        if let Some(rows) = self.table.take() {
            if !rows.is_empty() {
                self.document.blocks.push(RichDocumentBlock::Table { rows });
            }
        }
    }
}

fn trim_runs(runs: &mut Vec<RichTextRun>) {
    while runs
        .first()
        .is_some_and(|run| run.text.as_ref().trim().is_empty())
    {
        runs.remove(0);
    }
    while runs
        .last()
        .is_some_and(|run| run.text.as_ref().trim().is_empty())
    {
        runs.pop();
    }
    if let Some(first) = runs.first_mut() {
        first.text = first.text.as_ref().trim_start().to_string().into();
    }
    if let Some(last) = runs.last_mut() {
        last.text = last.text.as_ref().trim_end().to_string().into();
    }
}

fn is_skipped_tag(name: &str) -> bool {
    matches!(name, "script" | "style" | "head" | "title")
}

fn collapse_whitespace(value: &str) -> String {
    let mut out = String::new();
    let mut previous_space = false;
    for ch in value.chars() {
        if ch.is_whitespace() {
            if !previous_space {
                out.push(' ');
            }
            previous_space = true;
        } else {
            out.push(ch);
            previous_space = false;
        }
    }
    out
}

fn decode_entities(value: &str) -> String {
    value
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
}

fn attr_value(raw: &str, name: &str) -> Option<String> {
    let mut rest = raw;
    loop {
        let idx = rest.to_ascii_lowercase().find(name)?;
        rest = &rest[idx + name.len()..];
        let trimmed = rest.trim_start();
        if !trimmed.starts_with('=') {
            continue;
        }
        let value = trimmed[1..].trim_start();
        if let Some(stripped) = value.strip_prefix('"') {
            let end = stripped.find('"')?;
            return Some(decode_entities(&stripped[..end]));
        }
        if let Some(stripped) = value.strip_prefix('\'') {
            let end = stripped.find('\'')?;
            return Some(decode_entities(&stripped[..end]));
        }
        let end = value.find(char::is_whitespace).unwrap_or(value.len());
        return Some(decode_entities(value[..end].trim_end_matches('/')));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_splits_paragraphs() {
        let doc = RichDocument::from_plain_text("One\nline\n\nTwo");

        assert_eq!(doc.blocks.len(), 2);
        assert_eq!(doc.to_plain_text(), "One line\nTwo");
    }

    #[test]
    fn html_fragment_extracts_safe_blocks() {
        let doc = RichDocument::from_html_fragment_lossy(
            r#"<h1>Title</h1><p>Hello <strong>world</strong> &amp; <a href="https://example.test">link</a>.</p><table><tr><th>A</th><th>B</th></tr><tr><td>1</td><td>2</td></tr></table>"#,
        );

        assert!(matches!(
            doc.blocks.first(),
            Some(RichDocumentBlock::Heading { level: 1, .. })
        ));
        assert!(doc.to_plain_text().contains("Hello world & link."));
        assert!(doc.to_plain_text().contains("A | B"));
    }

    #[test]
    fn html_fragment_ignores_script_and_style() {
        let doc = RichDocument::from_html_fragment_lossy(
            "<style>.x{}</style><script>alert(1)</script><p>Visible</p>",
        );

        assert_eq!(doc.to_plain_text(), "Visible");
    }

    #[test]
    fn attr_value_reads_quoted_and_unquoted_values() {
        assert_eq!(
            attr_value(r#"a href="https://example.test?a=1&amp;b=2""#, "href").as_deref(),
            Some("https://example.test?a=1&b=2")
        );
        assert_eq!(
            attr_value("img src=https://example.test/x.png/", "src").as_deref(),
            Some("https://example.test/x.png")
        );
    }
}
