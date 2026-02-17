use std::{
    collections::HashMap,
    hash::{DefaultHasher, Hash, Hasher},
    sync::LazyLock,
};

use markdown::{ParseOptions, to_mdast};
use ratatui::{
    style::{Color, Modifier, Style},
    text::Span,
};
use syntect::{easy::HighlightLines, highlighting::ThemeSet, parsing::SyntaxSet};

/// Dim dark gray style, reused across many node renderers.
const DIM_DARK_GRAY: Style = Style::new().fg(Color::DarkGray).add_modifier(Modifier::DIM);

// ---------------------------------------------------------------------------
// Syntax highlighting statics
// ---------------------------------------------------------------------------

static SYNTAXES: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEMES: LazyLock<ThemeSet> = LazyLock::new(ThemeSet::load_defaults);

fn highlight_theme() -> &'static syntect::highlighting::Theme {
    THEMES
        .themes
        .get("base16-ocean.dark")
        .or_else(|| THEMES.themes.values().next())
        .expect("syntect default ThemeSet is non-empty")
}

// ---------------------------------------------------------------------------
// Public types
// ---------------------------------------------------------------------------

/// Opaque render cache for markdown-to-ratatui rendering of response text segments.
///
/// Keyed by `(message_index, segment_index)`. Each entry is invalidated when
/// either the text content hash or the available width changes. The caller must
/// call [`MdRenderCache::clear`] when message indices are reused (e.g. after
/// `App::reset_all()` clears the message list).
///
/// Cached spans contain only the content portion of each line (no sidebar/gutter
/// prefix), so sidebar style changes (e.g. status transitions) do not require
/// cache invalidation.
#[derive(Default)]
pub struct MdRenderCache {
    entries: HashMap<(usize, usize), CacheEntry>,
}

struct CacheEntry {
    text_hash: u64,
    width: usize,
    content_spans: Vec<Vec<Span<'static>>>,
}

impl MdRenderCache {
    pub fn clear(&mut self) {
        self.entries.clear();
    }

    /// Return cached content spans, or render and cache.
    pub fn get_or_render(
        &mut self,
        msg_idx: usize,
        seg_idx: usize,
        text: &str,
        content_width: usize,
    ) -> &[Vec<Span<'static>>] {
        let hash = {
            let mut h = DefaultHasher::new();
            text.hash(&mut h);
            h.finish()
        };

        let key = (msg_idx, seg_idx);

        // Check if we have a valid cache hit
        let needs_render = !matches!(
            self.entries.get(&key),
            Some(entry) if entry.text_hash == hash && entry.width == content_width
        );

        if needs_render {
            let spans = render_markdown(text, content_width);
            self.entries.insert(
                key,
                CacheEntry {
                    text_hash: hash,
                    width: content_width,
                    content_spans: spans,
                },
            );
        }

        &self.entries[&key].content_spans
    }
}

// ---------------------------------------------------------------------------
// Parse options
// ---------------------------------------------------------------------------

fn tui_parse_options() -> ParseOptions {
    let mut opts = ParseOptions::gfm();
    opts.constructs.math_text = true;
    opts.constructs.math_flow = true;
    opts
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse markdown and return content spans per line (WITHOUT gutter/sidebar prefix).
/// The caller is responsible for prepending the gutter/sidebar to each line.
/// Falls back to plain text if parsing fails.
pub(super) fn render_markdown(text: &str, content_width: usize) -> Vec<Vec<Span<'static>>> {
    let Ok(tree) = to_mdast(text, &tui_parse_options()) else {
        return plain_text_lines(text, content_width);
    };

    let ctx = RenderContext {
        content_width,
        list_depth: 0,
        blockquote_depth: 0,
    };

    render_node(&tree, &ctx)
}

// ---------------------------------------------------------------------------
// Render context
// ---------------------------------------------------------------------------

struct RenderContext {
    content_width: usize,
    list_depth: usize,
    blockquote_depth: usize,
}

impl RenderContext {
    fn effective_width(&self) -> usize {
        self.content_width
            .saturating_sub(self.blockquote_depth * 2)
            .saturating_sub(self.list_depth * 2)
            .max(1)
    }

    fn indent_prefix(&self) -> Vec<Span<'static>> {
        let mut spans = Vec::new();
        for _ in 0..self.blockquote_depth {
            spans.push(Span::styled("\u{2502} ", DIM_DARK_GRAY));
        }
        if self.list_depth > 0 {
            spans.push(Span::raw("  ".repeat(self.list_depth.saturating_sub(1))));
        }
        spans
    }

    fn with_blockquote(&self) -> Self {
        Self {
            content_width: self.content_width,
            list_depth: self.list_depth,
            blockquote_depth: self.blockquote_depth + 1,
        }
    }

    fn with_list(&self) -> Self {
        Self {
            content_width: self.content_width,
            list_depth: self.list_depth + 1,
            blockquote_depth: self.blockquote_depth,
        }
    }
}

// ---------------------------------------------------------------------------
// Styled fragment for inline collection and word wrapping
// ---------------------------------------------------------------------------

#[derive(Clone)]
struct StyledFragment {
    text: String,
    style: Style,
}

// ---------------------------------------------------------------------------
// Plain text fallback
// ---------------------------------------------------------------------------

fn plain_text_lines(text: &str, width: usize) -> Vec<Vec<Span<'static>>> {
    let mut result = Vec::new();
    for line in text.lines() {
        let fragments = vec![StyledFragment {
            text: line.to_string(),
            style: Style::default(),
        }];
        for wrapped in wrap_fragments(&fragments, width) {
            result.push(wrapped);
        }
    }
    if result.is_empty() {
        result.push(vec![Span::raw("")]);
    }
    result
}

// ---------------------------------------------------------------------------
// Block-level rendering
// ---------------------------------------------------------------------------

/// Wrap styled fragments to the context's effective width and prepend the
/// context's indent prefix to each resulting line.
fn wrap_and_indent(fragments: &[StyledFragment], ctx: &RenderContext) -> Vec<Vec<Span<'static>>> {
    let indent = ctx.indent_prefix();
    let width = ctx.effective_width();
    let wrapped = wrap_fragments(fragments, width);

    wrapped
        .into_iter()
        .map(|mut spans| {
            let mut line = indent.clone();
            line.append(&mut spans);
            line
        })
        .collect()
}

fn render_node(node: &markdown::mdast::Node, ctx: &RenderContext) -> Vec<Vec<Span<'static>>> {
    use markdown::mdast::Node;

    match node {
        Node::Root(root) => render_children_with_spacing(&root.children, ctx),
        Node::Heading(heading) => render_heading(heading, ctx),
        Node::Paragraph(para) => render_paragraph(&para.children, ctx),
        Node::Code(code) => render_code_block(code, ctx),
        Node::Blockquote(bq) => render_blockquote(bq, ctx),
        Node::List(list) => render_list(list, ctx),
        Node::Table(table) => render_table(table, ctx),
        Node::ThematicBreak(_) => render_thematic_break(ctx),
        Node::Math(math) => render_math_block(math, ctx),
        // Inline nodes that can appear at block level (e.g. loose text)
        _ => {
            let fragments = collect_inlines(node);
            wrap_and_indent(&fragments, ctx)
        }
    }
}

fn render_children_with_spacing(
    children: &[markdown::mdast::Node],
    ctx: &RenderContext,
) -> Vec<Vec<Span<'static>>> {
    let mut result = Vec::new();
    for (i, child) in children.iter().enumerate() {
        if i > 0 && !result.is_empty() {
            // Blank line between blocks
            let mut blank = ctx.indent_prefix();
            if blank.is_empty() {
                blank.push(Span::raw(""));
            }
            result.push(blank);
        }
        result.extend(render_node(child, ctx));
    }
    if result.is_empty() {
        result.push(vec![Span::raw("")]);
    }
    result
}

fn render_heading(
    heading: &markdown::mdast::Heading,
    ctx: &RenderContext,
) -> Vec<Vec<Span<'static>>> {
    let depth = heading.depth as usize;
    let orange = Color::Rgb(255, 165, 0);

    let style = match depth {
        1 => Style::new()
            .fg(orange)
            .add_modifier(Modifier::BOLD | Modifier::UNDERLINED),
        2 => Style::new().fg(orange).add_modifier(Modifier::BOLD),
        3 => Style::new().fg(orange),
        _ => Style::new().fg(orange).add_modifier(Modifier::DIM),
    };

    let mut fragments = Vec::new();

    for child in &heading.children {
        let mut inline_frags = collect_inlines(child);
        for frag in &mut inline_frags {
            frag.style = frag.style.patch(style);
        }
        fragments.extend(inline_frags);
    }

    wrap_and_indent(&fragments, ctx)
}

fn render_paragraph(
    children: &[markdown::mdast::Node],
    ctx: &RenderContext,
) -> Vec<Vec<Span<'static>>> {
    let mut fragments = Vec::new();
    for child in children {
        fragments.extend(collect_inlines(child));
    }

    wrap_and_indent(&fragments, ctx)
}

fn render_code_block(code: &markdown::mdast::Code, ctx: &RenderContext) -> Vec<Vec<Span<'static>>> {
    let indent = ctx.indent_prefix();
    let bar = Span::styled("\u{2502} ", DIM_DARK_GRAY);

    let mut result = Vec::new();

    // Language label
    if let Some(lang) = &code.lang {
        let mut line = indent.clone();
        line.push(bar.clone());
        line.push(Span::styled(lang.clone(), DIM_DARK_GRAY));
        result.push(line);
    }

    // Syntax-highlighted code lines
    let highlighted = highlight_code(code.lang.as_deref(), &code.value);

    for spans in highlighted {
        let mut line = indent.clone();
        line.push(bar.clone());
        line.extend(spans);
        result.push(line);
    }

    result
}

fn render_blockquote(
    bq: &markdown::mdast::Blockquote,
    ctx: &RenderContext,
) -> Vec<Vec<Span<'static>>> {
    let inner_ctx = ctx.with_blockquote();
    let mut lines = render_children_with_spacing(&bq.children, &inner_ctx);
    for line in &mut lines {
        for span in line.iter_mut() {
            span.style = span.style.add_modifier(Modifier::ITALIC);
        }
    }
    lines
}

fn render_list(list: &markdown::mdast::List, ctx: &RenderContext) -> Vec<Vec<Span<'static>>> {
    let mut result = Vec::new();
    let start_num = list.start.unwrap_or(1);

    for (i, child) in list.children.iter().enumerate() {
        if let markdown::mdast::Node::ListItem(item) = child {
            let bullet = if list.ordered {
                format!("{}. ", start_num as usize + i)
            } else {
                "\u{2022} ".to_string()
            };

            // Check for task list item
            let bullet = if let Some(checked) = item.checked {
                if checked {
                    "[x] ".to_string()
                } else {
                    "[ ] ".to_string()
                }
            } else {
                bullet
            };

            let inner_ctx = ctx.with_list();
            let indent = ctx.indent_prefix();

            // Render item children
            let mut first_block = true;
            for block_child in &item.children {
                let child_lines = render_node(block_child, &inner_ctx);
                for (j, mut spans) in child_lines.into_iter().enumerate() {
                    let mut line = indent.clone();
                    if first_block && j == 0 {
                        line.push(Span::styled(bullet.clone(), Style::new().fg(Color::Blue)));
                        first_block = false;
                    } else {
                        line.push(Span::raw(" ".repeat(bullet.len())));
                    }
                    // Remove the inner_ctx indent prefix from the child spans
                    // since we handle indentation via the bullet prefix
                    let inner_prefix_len = inner_ctx.indent_prefix().len();
                    if spans.len() >= inner_prefix_len {
                        spans = spans[inner_prefix_len..].to_vec();
                    }
                    line.extend(spans);
                    result.push(line);
                }
            }
        }
    }

    result
}

#[allow(
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss
)]
fn render_table(table: &markdown::mdast::Table, ctx: &RenderContext) -> Vec<Vec<Span<'static>>> {
    let indent = ctx.indent_prefix();
    let width = ctx.effective_width();
    let border_style = DIM_DARK_GRAY;

    // Collect all rows and their cell contents as plain text
    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut num_cols = 0;

    for child in &table.children {
        if let markdown::mdast::Node::TableRow(row) = child {
            let mut cells = Vec::new();
            for cell_node in &row.children {
                if let markdown::mdast::Node::TableCell(cell) = cell_node {
                    let frags = collect_inlines_from_children(&cell.children);
                    let text: String = frags.iter().map(|f| f.text.as_str()).collect();
                    cells.push(text);
                }
            }
            num_cols = num_cols.max(cells.len());
            rows.push(cells);
        }
    }

    if rows.is_empty() || num_cols == 0 {
        return Vec::new();
    }

    // Pad rows to same number of columns
    for row in &mut rows {
        while row.len() < num_cols {
            row.push(String::new());
        }
    }

    // Check if we have enough width for table borders
    let min_table_width = num_cols * 3 + num_cols + 1;
    if min_table_width > width {
        // Narrow-width fallback: record layout
        return render_table_narrow(&rows, ctx);
    }

    // Compute column widths based on content (char count, not bytes)
    let mut col_widths: Vec<usize> = vec![0; num_cols];
    for row in &rows {
        for (c, cell) in row.iter().enumerate() {
            col_widths[c] = col_widths[c].max(cell.chars().count()).max(1);
        }
    }

    // Shrink proportionally if total exceeds available width
    let border_overhead = num_cols + 1; // one │ per column + 1
    let padding_overhead = num_cols * 2; // 1 space each side per column
    let avail_for_content = width.saturating_sub(border_overhead + padding_overhead);
    let total_content: usize = col_widths.iter().sum();

    if total_content > avail_for_content && avail_for_content > 0 {
        let ratio = avail_for_content as f64 / total_content as f64;
        for w in &mut col_widths {
            *w = ((*w as f64 * ratio).floor() as usize).max(1);
        }
    }

    // Get alignments
    let aligns = &table.align;

    let mut result = Vec::new();

    // Top border: ┌─────┬─────┐
    let top = build_border_line(&col_widths, "\u{250c}", "\u{2500}", "\u{252c}", "\u{2510}");
    let mut line = indent.clone();
    line.push(Span::styled(top, border_style));
    result.push(line);

    for (r, row) in rows.iter().enumerate() {
        // Content row: │ val │ val │
        let mut line = indent.clone();
        let is_header = r == 0;
        let cell_style = if is_header {
            Style::new().add_modifier(Modifier::BOLD)
        } else {
            Style::default()
        };

        for (c, cell) in row.iter().enumerate() {
            line.push(Span::styled("\u{2502} ", border_style));
            let w = col_widths[c];
            let cell_chars = cell.chars().count();
            let padded = if cell_chars > w {
                truncate_chars(cell, w)
            } else {
                let align = aligns
                    .get(c)
                    .copied()
                    .unwrap_or(markdown::mdast::AlignKind::Left);
                match align {
                    markdown::mdast::AlignKind::Right => {
                        format!("{cell:>w$}")
                    }
                    markdown::mdast::AlignKind::Center => {
                        let pad = w.saturating_sub(cell_chars);
                        let left_pad = pad / 2;
                        let right_pad = pad - left_pad;
                        format!("{}{}{}", " ".repeat(left_pad), cell, " ".repeat(right_pad))
                    }
                    _ => format!("{cell:<w$}"),
                }
            };
            line.push(Span::styled(padded, cell_style));
            line.push(Span::raw(" "));
        }
        line.push(Span::styled("\u{2502}", border_style));
        result.push(line);

        // Header separator
        if r == 0 && rows.len() > 1 {
            let sep =
                build_border_line(&col_widths, "\u{251c}", "\u{2500}", "\u{253c}", "\u{2524}");
            let mut line = indent.clone();
            line.push(Span::styled(sep, border_style));
            result.push(line);
        }
    }

    // Bottom border: └─────┴─────┘
    let bottom = build_border_line(&col_widths, "\u{2514}", "\u{2500}", "\u{2534}", "\u{2518}");
    let mut line = indent.clone();
    line.push(Span::styled(bottom, border_style));
    result.push(line);

    result
}

fn build_border_line(
    col_widths: &[usize],
    left: &str,
    fill: &str,
    mid: &str,
    right: &str,
) -> String {
    let mut s = left.to_string();
    for (i, &w) in col_widths.iter().enumerate() {
        // +2 for padding on each side
        for _ in 0..(w + 2) {
            s.push_str(fill);
        }
        if i < col_widths.len() - 1 {
            s.push_str(mid);
        }
    }
    s.push_str(right);
    s
}

fn render_table_narrow(rows: &[Vec<String>], ctx: &RenderContext) -> Vec<Vec<Span<'static>>> {
    let indent = ctx.indent_prefix();
    let bold = Style::new().add_modifier(Modifier::BOLD);

    let mut result = Vec::new();

    // First row is header
    let headers: Vec<&str> = rows
        .first()
        .map(|r| r.iter().map(String::as_str).collect())
        .unwrap_or_default();

    if rows.len() <= 1 {
        // Header-only table: render each header cell as a bold line
        for header in &headers {
            let mut line = indent.clone();
            line.push(Span::styled((*header).to_string(), bold));
            result.push(line);
        }
        return result;
    }

    for (r, row) in rows.iter().enumerate().skip(1) {
        if r > 1 {
            let mut line = indent.clone();
            line.push(Span::styled("\u{2500}\u{2500}\u{2500}", DIM_DARK_GRAY));
            result.push(line);
        }
        for (c, cell) in row.iter().enumerate() {
            let header = headers.get(c).copied().unwrap_or("?");
            let mut line = indent.clone();
            line.push(Span::styled(format!("{header}: "), bold));
            line.push(Span::raw(cell.clone()));
            result.push(line);
        }
    }

    result
}

fn render_thematic_break(ctx: &RenderContext) -> Vec<Vec<Span<'static>>> {
    let indent = ctx.indent_prefix();
    let width = ctx.effective_width();
    let mut line = indent;
    line.push(Span::styled("\u{2500}".repeat(width), DIM_DARK_GRAY));
    vec![line]
}

fn render_math_block(math: &markdown::mdast::Math, ctx: &RenderContext) -> Vec<Vec<Span<'static>>> {
    let indent = ctx.indent_prefix();
    let bar = Span::styled("\u{2502} ", DIM_DARK_GRAY);

    let mut result = Vec::new();

    // Label
    let mut label_line = indent.clone();
    label_line.push(bar.clone());
    label_line.push(Span::styled("$$", DIM_DARK_GRAY));
    result.push(label_line);

    for text_line in math.value.lines() {
        let mut line = indent.clone();
        line.push(bar.clone());
        line.push(Span::styled(
            text_line.to_string(),
            Style::new().fg(Color::Cyan),
        ));
        result.push(line);
    }

    result
}

// ---------------------------------------------------------------------------
// Inline-level collection
// ---------------------------------------------------------------------------

fn collect_inlines(node: &markdown::mdast::Node) -> Vec<StyledFragment> {
    use markdown::mdast::Node;

    match node {
        Node::Text(t) => vec![StyledFragment {
            text: t.value.clone(),
            style: Style::default(),
        }],
        Node::Strong(s) => collect_with_modifier(&s.children, Modifier::BOLD),
        Node::Emphasis(e) => collect_with_modifier(&e.children, Modifier::ITALIC),
        Node::InlineCode(c) => vec![StyledFragment {
            text: c.value.clone(),
            style: Style::new().fg(Color::Rgb(200, 160, 255)),
        }],
        Node::Link(link) => {
            let mut frags = collect_inlines_from_children(&link.children);
            let link_style = Style::new()
                .fg(Color::Blue)
                .add_modifier(Modifier::UNDERLINED);
            for f in &mut frags {
                f.style = f.style.patch(link_style);
            }
            // Append URL in dim if reasonably short
            if link.url.len() <= 50 {
                frags.push(StyledFragment {
                    text: format!(" ({})", link.url),
                    style: DIM_DARK_GRAY,
                });
            }
            frags
        }
        Node::Delete(d) => collect_with_modifier(&d.children, Modifier::CROSSED_OUT),
        Node::Image(img) => {
            let alt = if img.alt.is_empty() {
                "image".to_string()
            } else {
                img.alt.clone()
            };
            vec![StyledFragment {
                text: format!("[image: {alt}]"),
                style: Style::new()
                    .fg(Color::DarkGray)
                    .add_modifier(Modifier::DIM | Modifier::ITALIC),
            }]
        }
        Node::InlineMath(m) => vec![StyledFragment {
            text: m.value.clone(),
            style: Style::new().fg(Color::Cyan),
        }],
        Node::Break(_) => vec![StyledFragment {
            text: "\n".to_string(),
            style: Style::default(),
        }],
        // For block nodes encountered inline, recursively collect text
        Node::Paragraph(p) => collect_inlines_from_children(&p.children),
        _ => {
            // Fallback: try to get children
            if let Some(children) = node_children(node) {
                collect_inlines_from_children(children)
            } else {
                Vec::new()
            }
        }
    }
}

fn collect_inlines_from_children(children: &[markdown::mdast::Node]) -> Vec<StyledFragment> {
    children.iter().flat_map(collect_inlines).collect()
}

/// Collect inline fragments from children and apply a modifier to all of them.
fn collect_with_modifier(
    children: &[markdown::mdast::Node],
    modifier: Modifier,
) -> Vec<StyledFragment> {
    let mut frags = collect_inlines_from_children(children);
    for f in &mut frags {
        f.style = f.style.add_modifier(modifier);
    }
    frags
}

fn node_children(node: &markdown::mdast::Node) -> Option<&Vec<markdown::mdast::Node>> {
    use markdown::mdast::Node;
    match node {
        Node::Root(n) => Some(&n.children),
        Node::Paragraph(n) => Some(&n.children),
        Node::Heading(n) => Some(&n.children),
        Node::Blockquote(n) => Some(&n.children),
        Node::List(n) => Some(&n.children),
        Node::ListItem(n) => Some(&n.children),
        Node::Emphasis(n) => Some(&n.children),
        Node::Strong(n) => Some(&n.children),
        Node::Link(n) => Some(&n.children),
        Node::Delete(n) => Some(&n.children),
        Node::Table(n) => Some(&n.children),
        Node::TableRow(n) => Some(&n.children),
        Node::TableCell(n) => Some(&n.children),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Word-aware wrapping
// ---------------------------------------------------------------------------

fn wrap_fragments(fragments: &[StyledFragment], width: usize) -> Vec<Vec<Span<'static>>> {
    if width == 0 {
        return vec![
            fragments
                .iter()
                .map(|f| Span::styled(f.text.clone(), f.style))
                .collect(),
        ];
    }

    let mut lines: Vec<Vec<Span<'static>>> = Vec::new();
    let mut current_line: Vec<Span<'static>> = Vec::new();
    let mut col = 0usize;

    for frag in fragments {
        // Handle explicit line breaks
        for (sub_idx, sub_text) in frag.text.split('\n').enumerate() {
            if sub_idx > 0 {
                // Explicit newline
                lines.push(std::mem::take(&mut current_line));
                col = 0;
            }

            if sub_text.is_empty() {
                continue;
            }

            // Split into words (preserving leading/trailing whitespace)
            let words = split_words(sub_text);

            for word in &words {
                let word_len = word.chars().count();

                if word_len == 0 {
                    continue;
                }

                // If adding this word would exceed width
                if col > 0 && col + word_len > width {
                    // Wrap to next line (only if the word isn't just whitespace)
                    if word.trim().is_empty() {
                        // Skip trailing whitespace at line break
                        continue;
                    }
                    lines.push(std::mem::take(&mut current_line));
                    col = 0;
                }

                // If a single word exceeds width, break it character-by-character
                if word_len > width && col == 0 {
                    let chars: Vec<char> = word.chars().collect();
                    let mut pos = 0;
                    while pos < chars.len() {
                        let chunk_end = (pos + width).min(chars.len());
                        let chunk: String = chars[pos..chunk_end].iter().collect();
                        let chunk_len = chunk.chars().count();
                        if !current_line.is_empty() && col > 0 {
                            lines.push(std::mem::take(&mut current_line));
                            col = 0;
                        }
                        current_line.push(Span::styled(chunk, frag.style));
                        col += chunk_len;
                        pos = chunk_end;
                        if pos < chars.len() {
                            lines.push(std::mem::take(&mut current_line));
                            col = 0;
                        }
                    }
                } else {
                    current_line.push(Span::styled(word.clone(), frag.style));
                    col += word_len;
                }
            }
        }
    }

    if !current_line.is_empty() {
        lines.push(current_line);
    }

    if lines.is_empty() {
        lines.push(vec![Span::raw("")]);
    }

    lines
}

/// Split text into words, keeping whitespace attached to preceding word.
fn split_words(text: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut in_whitespace = false;

    for ch in text.chars() {
        if ch.is_whitespace() {
            if !in_whitespace && !current.is_empty() {
                current.push(ch);
                in_whitespace = true;
            } else {
                current.push(ch);
            }
        } else {
            if in_whitespace {
                words.push(std::mem::take(&mut current));
                in_whitespace = false;
            }
            current.push(ch);
        }
    }

    if !current.is_empty() {
        words.push(current);
    }

    words
}

/// Truncate a string to at most `max_chars` characters (safe for multibyte).
fn truncate_chars(s: &str, max_chars: usize) -> String {
    s.chars().take(max_chars).collect()
}

// ---------------------------------------------------------------------------
// Syntax highlighting
// ---------------------------------------------------------------------------

fn highlight_code(lang: Option<&str>, code: &str) -> Vec<Vec<Span<'static>>> {
    let fallback_style = DIM_DARK_GRAY;

    let syntax = lang.and_then(|l| {
        // Normalize: extract first token (e.g., "rust,ignore" -> "rust")
        let normalized = l.split([' ', ',', '\t']).next().unwrap_or(l);
        SYNTAXES
            .find_syntax_by_extension(normalized)
            .or_else(|| SYNTAXES.find_syntax_by_name(normalized))
            // Case-insensitive name lookup (e.g., "rust" → "Rust", "python" → "Python")
            .or_else(|| {
                SYNTAXES
                    .syntaxes()
                    .iter()
                    .find(|s| s.name.eq_ignore_ascii_case(normalized))
            })
    });

    let Some(syntax) = syntax else {
        // No syntax found: render with uniform dim style
        return code
            .lines()
            .map(|line| vec![Span::styled(line.to_string(), fallback_style)])
            .collect();
    };

    let theme = highlight_theme();
    let mut highlighter = HighlightLines::new(syntax, theme);
    let mut result = Vec::new();

    for line in code.lines() {
        // Skip highlighting for very long lines (perf guard)
        if line.len() > 500 {
            result.push(vec![Span::styled(line.to_string(), fallback_style)]);
            continue;
        }

        // Append newline for syntect (load_defaults_newlines expects it)
        let line_nl = format!("{line}\n");
        match highlighter.highlight_line(&line_nl, &SYNTAXES) {
            Ok(ranges) => {
                let spans: Vec<Span<'static>> = ranges
                    .into_iter()
                    .map(|(style, text)| {
                        let fg =
                            Color::Rgb(style.foreground.r, style.foreground.g, style.foreground.b);
                        // Trim trailing newline from output
                        let trimmed = text.trim_end_matches('\n');
                        Span::styled(trimmed.to_string(), Style::new().fg(fg))
                    })
                    .filter(|s| !s.content.is_empty())
                    .collect();
                result.push(spans);
            }
            Err(_) => {
                result.push(vec![Span::styled(line.to_string(), fallback_style)]);
            }
        }
    }

    if result.is_empty() {
        result.push(vec![Span::raw("")]);
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn spans_to_text(spans: &[Span]) -> String {
        spans.iter().map(|s| s.content.as_ref()).collect()
    }

    fn lines_to_text(lines: &[Vec<Span>]) -> Vec<String> {
        lines.iter().map(|l| spans_to_text(l)).collect()
    }

    #[test]
    fn test_plain_text_fallback() {
        let result = render_markdown("Hello, world!", 80);
        assert!(!result.is_empty());
        let text = spans_to_text(&result[0]);
        assert!(text.contains("Hello, world!"));
    }

    #[test]
    fn test_heading() {
        let result = render_markdown("# Title", 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("Title")));
        // No hash prefix
        assert!(!text.iter().any(|l| l.contains('#')));
    }

    #[test]
    fn test_bold_and_italic() {
        let result = render_markdown("**bold** and *italic*", 80);
        assert!(!result.is_empty());
        let text = lines_to_text(&result);
        assert!(
            text.iter()
                .any(|l| l.contains("bold") && l.contains("italic"))
        );
    }

    #[test]
    fn test_inline_code() {
        let result = render_markdown("Use `foo()` here", 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("foo()")));
    }

    #[test]
    fn test_code_block() {
        let md = "```rust\nfn main() {}\n```";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("fn main()")));
    }

    #[test]
    fn test_code_block_no_lang() {
        let md = "```\nsome code\n```";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("some code")));
    }

    #[test]
    fn test_unordered_list() {
        let md = "- item one\n- item two";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(
            text.iter()
                .any(|l| l.contains("\u{2022}") && l.contains("item one"))
        );
        assert!(
            text.iter()
                .any(|l| l.contains("\u{2022}") && l.contains("item two"))
        );
    }

    #[test]
    fn test_ordered_list() {
        let md = "1. first\n2. second";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("1.") && l.contains("first")));
        assert!(
            text.iter()
                .any(|l| l.contains("2.") && l.contains("second"))
        );
    }

    #[test]
    fn test_blockquote() {
        let md = "> quoted text";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(
            text.iter()
                .any(|l| l.contains("\u{2502}") && l.contains("quoted text"))
        );
    }

    #[test]
    fn test_thematic_break() {
        let md = "above\n\n---\n\nbelow";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("\u{2500}")));
    }

    #[test]
    fn test_link() {
        let md = "[click](https://example.com)";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("click")));
    }

    #[test]
    fn test_table() {
        let md = "| A | B |\n| - | - |\n| 1 | 2 |";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        // Should have box-drawing characters
        assert!(text.iter().any(|l| l.contains("\u{250c}")));
        assert!(text.iter().any(|l| l.contains("\u{2514}")));
    }

    #[test]
    fn test_table_multibyte() {
        // Ensure non-ASCII content doesn't panic (CJK, emoji, accented chars)
        let md = "| 名前 | 値 |\n| - | - |\n| こんにちは | 🎉🎊 |";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("こんにちは")));
        assert!(text.iter().any(|l| l.contains("🎉🎊")));
    }

    #[test]
    fn test_table_multibyte_narrow() {
        // Force truncation on multibyte content -- must not panic
        let md = "| 名前 | 値 |\n| - | - |\n| あいうえおかきくけこ | さしすせそ |";
        let result = render_markdown(md, 30);
        assert!(!result.is_empty());
    }

    #[test]
    fn test_table_narrow() {
        let md = "| A | B | C | D | E |\n| - | - | - | - | - |\n| 1 | 2 | 3 | 4 | 5 |";
        // Very narrow width forces record layout
        let result = render_markdown(md, 10);
        let text = lines_to_text(&result);
        // Should fall back to header: value format
        assert!(text.iter().any(|l| l.contains("A: ")));
    }

    #[test]
    fn test_table_header_only_narrow() {
        // Header-only table at narrow width should still render header cells
        let md = "| Name | Age | City |\n| - | - | - |";
        let result = render_markdown(md, 10);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("Name")));
        assert!(text.iter().any(|l| l.contains("Age")));
        assert!(text.iter().any(|l| l.contains("City")));
    }

    #[test]
    fn test_word_wrap() {
        let frags = vec![StyledFragment {
            text: "hello world foo bar".to_string(),
            style: Style::default(),
        }];
        let wrapped = wrap_fragments(&frags, 10);
        assert!(wrapped.len() >= 2);
    }

    #[test]
    fn test_word_wrap_long_word() {
        let frags = vec![StyledFragment {
            text: "abcdefghijklmnop".to_string(),
            style: Style::default(),
        }];
        let wrapped = wrap_fragments(&frags, 5);
        // Should break the long word across multiple lines
        assert!(wrapped.len() >= 3);
    }

    #[test]
    fn test_cache_hit() {
        let mut cache = MdRenderCache::default();
        let text = "# Hello\n\nWorld";
        let width = 80;

        let first = cache.get_or_render(0, 0, text, width);
        let first_len = first.len();
        assert!(first_len > 0);

        let second = cache.get_or_render(0, 0, text, width);
        assert_eq!(second.len(), first_len);

        // Only one entry in the cache
        assert_eq!(cache.entries.len(), 1);
    }

    #[test]
    fn test_cache_miss_on_content_change() {
        let mut cache = MdRenderCache::default();

        let _ = cache.get_or_render(0, 0, "Hello", 80);
        let _ = cache.get_or_render(0, 0, "Hello World", 80);

        // Same key but hash changed, should re-render
        assert_eq!(cache.entries.len(), 1);
        let entry = &cache.entries[&(0, 0)];
        // Should reflect the new text
        let text: String = entry
            .content_spans
            .iter()
            .flat_map(|l| l.iter().map(|s| s.content.to_string()))
            .collect();
        assert!(text.contains("World"));
    }

    #[test]
    fn test_cache_miss_on_resize() {
        let mut cache = MdRenderCache::default();

        let _ = cache.get_or_render(0, 0, "Hello World", 80);
        let _ = cache.get_or_render(0, 0, "Hello World", 40);

        assert_eq!(cache.entries.len(), 1);
        let entry = &cache.entries[&(0, 0)];
        assert_eq!(entry.width, 40);
    }

    #[test]
    fn test_cache_clear_on_reset() {
        let mut cache = MdRenderCache::default();
        let _ = cache.get_or_render(0, 0, "test", 80);
        assert_eq!(cache.entries.len(), 1);
        cache.clear();
        assert_eq!(cache.entries.len(), 0);
    }

    #[test]
    fn test_strikethrough() {
        let md = "~~deleted~~";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("deleted")));
    }

    #[test]
    fn test_image() {
        let md = "![alt text](image.png)";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("[image: alt text]")));
    }

    #[test]
    fn test_empty_text() {
        let result = render_markdown("", 80);
        // Should produce at least one empty line
        assert!(!result.is_empty());
    }

    #[test]
    fn test_streaming_partial_code_fence() {
        // Simulate streaming: unclosed code fence
        let md = "```python\nprint('hello')";
        let result = render_markdown(md, 80);
        let text = lines_to_text(&result);
        assert!(text.iter().any(|l| l.contains("print")));
    }
}
