use tl::{HTMLTag, Parser};

use stencila_codec::stencila_schema::{
    AppendixBreak, Block, Claim, ClaimType, CodeBlock, Figure, Heading, HorizontalAlignment,
    ImageObject, Inline, List, ListItem, ListOrder, MathBlock, RawBlock, Section, SectionType,
    Table, TableCell, TableCellType, TableRow, TableRowType,
    shortcuts::{p, t},
};

use crate::decode_html_inlines::decode_inline;

use super::decode_html::{
    ArxivDecodeContext, extract_latex_and_mathml, get_attr, get_class, get_text,
};
use super::decode_html_inlines::{decode_a, decode_img, decode_inlines, decode_span};

/// Decode block elements
pub fn decode_blocks(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> Vec<Block> {
    let mut blocks = Vec::new();
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();
            let tag_class = tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            // If bibliography is within an inner section (rather than a direct child
            // of the <article>) then ignore it because handled in `decode_article`
            if tag_class.contains("ltx_bibliography") {
                continue;
            }

            match tag_name.as_ref() {
                "section" => {
                    if tag_class.contains("ltx_appendix") {
                        if !context.appendix_started {
                            blocks.push(Block::AppendixBreak(AppendixBreak::default()));
                            context.appendix_started = true;
                        }
                        blocks.push(decode_section(
                            parser,
                            tag,
                            SectionType::SupplementaryMaterials,
                            context,
                        ));
                    } else {
                        blocks.push(decode_section(parser, tag, SectionType::Main, context));
                    }
                }
                "h1" | "h2" | "h3" | "h4" | "h5" | "h6" => {
                    blocks.push(decode_heading(parser, tag, &tag_name, &tag_class, context))
                }
                "figure" => {
                    if tag_class.contains("ltx_table") {
                        blocks.push(decode_figure_table(parser, tag, context));
                    } else {
                        blocks.push(decode_figure(parser, tag, context));
                    }
                }
                "table" => {
                    if tag_class.contains("ltx_equation") || tag_class.contains("ltx_eqn_table") {
                        blocks.push(decode_math_block(parser, tag, context));
                    } else {
                        // Handle as regular table
                        blocks.push(decode_table(parser, tag, context));
                    }
                }
                "ul" => {
                    if tag_class.contains("ltx_biblist") {
                        // Handle bibliography list - decode children as individual references
                        blocks.append(&mut decode_blocks(parser, tag, context));
                    } else {
                        blocks.push(decode_list(parser, tag, false, context));
                    }
                }
                "ol" => {
                    blocks.push(decode_list(parser, tag, true, context));
                }
                "li" => {
                    if tag_class.contains("ltx_bibitem") {
                        // Bibliography items are handled in decode_bibliography, not here
                        // Just process as regular content for now
                        blocks.push(p(decode_inlines(parser, tag, context)));
                    } else {
                        blocks.push(p(decode_inlines(parser, tag, context)));
                    }
                }
                "p" => blocks.push(decode_p(parser, tag, context)),
                "svg" => {
                    if tag_class.contains("ltx_picture") {
                        blocks.push(decode_svg(parser, tag, context));
                    } else {
                        // Handle other SVG elements as needed
                        blocks.append(&mut decode_blocks(parser, tag, context));
                        context.add_loss(tag);
                    }
                }
                "div" => {
                    // decode_div may return multiple blocks, so extend rather than push
                    blocks.extend(decode_div(parser, tag, context));
                }
                // Inlines where block is expected
                "a" => blocks.push(p([decode_a(parser, tag, context)])),
                "img" => blocks.push(p([decode_img(parser, tag, context)])),
                "span" => blocks.push(p(decode_span(parser, tag, context))),
                _ => {
                    // Unhandled tag: just decode children into blocks but record loss
                    blocks.append(&mut decode_blocks(parser, tag, context));
                    context.add_loss(tag);
                }
            }
        } else if let Some(text) = child.as_raw() {
            // At block level, ignore whitespace
            let text = text.try_as_utf8_str().unwrap_or_default().trim();
            if !text.is_empty() {
                blocks.push(p([t(text)]))
            }
        }
    }
    blocks
}

/// Decode a div.ltx_listing into a CodeBlock
fn decode_code_block(parser: &Parser, tag: &HTMLTag, _context: &mut ArxivDecodeContext) -> Block {
    let code = get_text(parser, tag);

    Block::CodeBlock(CodeBlock {
        code: code.into(),
        ..Default::default()
    })
}

/// Decode a <div> element into appropriate Block type(s) based on class
pub fn decode_div(parser: &Parser, tag: &HTMLTag, context: &mut ArxivDecodeContext) -> Vec<Block> {
    let class = get_class(tag);

    if class.contains("ltx_para") {
        // ltx_para is just a paragraph wrapper, decode children directly
        decode_blocks(parser, tag, context)
    } else if class.contains("ltx_theorem") || class.contains("ltx_proof") {
        vec![decode_theorem_or_proof(parser, tag, context)]
    } else if class.contains("ltx_listing") {
        vec![decode_code_block(parser, tag, context)]
    } else {
        // Unhandled div: just decode children into blocks and record loss
        context.add_loss(tag);
        decode_blocks(parser, tag, context)
    }
}

/// Extract label and caption content from a figure caption, handling ltx_tag_figure spans
fn extract_figure_label_and_caption(
    parser: &Parser,
    figcaption: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> (Option<String>, Vec<Block>) {
    let mut label = None;
    let mut caption_inlines = Vec::new();

    for child in figcaption
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            if child_class.contains("ltx_tag_figure") {
                // Extract label from figure tag like "Figure 2.1: " -> "2.1"
                let text = get_text(parser, child_tag);
                label = extract_number_from_figure_title(&text);
                // Don't include this in caption content
            } else {
                // Include all other content in caption
                caption_inlines.extend(decode_inline(parser, child_tag, context));
            }
        } else if let Some(text) = child.as_raw() {
            let text_content = text.try_as_utf8_str().unwrap_or_default();
            // Only include non-whitespace text in caption
            if !text_content.trim().is_empty() {
                caption_inlines.push(t(text_content));
            }
        }
    }

    // Wrap caption inlines in a paragraph
    let caption_blocks = if caption_inlines.is_empty() {
        Vec::new()
    } else {
        vec![p(caption_inlines)]
    };

    (label, caption_blocks)
}

/// Extract number from figure title like "Figure 2.1: " -> "2.1"
fn extract_number_from_figure_title(text: &str) -> Option<String> {
    let trimmed = text.trim().trim_end_matches(':').trim();
    let words: Vec<&str> = trimmed.split_whitespace().collect();

    // Look for the last word that contains numbers and periods
    for word in words.iter().rev() {
        if word.chars().any(|c| c.is_ascii_digit()) && word.contains('.') {
            return Some(word.to_string());
        }
    }

    // Fallback: look for any word with numbers
    for word in words.iter().rev() {
        if word.chars().any(|c| c.is_ascii_digit()) {
            return Some(word.to_string());
        }
    }

    None
}

/// Decode a figure element
pub fn decode_figure(parser: &Parser, tag: &HTMLTag, context: &mut ArxivDecodeContext) -> Block {
    let mut caption = None;
    let mut label = None;
    let mut content = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_name = child_tag.name().as_utf8_str();
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            if child_name == "figcaption" && child_class.contains("ltx_caption") {
                let (extracted_label, caption_content) =
                    extract_figure_label_and_caption(parser, child_tag, context);
                label = extracted_label;
                caption = Some(caption_content);
            } else if child_name == "img" {
                // Handle img elements as ImageObjects in the figure content
                let raw_url = get_attr(child_tag, "src").unwrap_or_default();
                let content_url = context.resolve_url(&raw_url);
                let image_caption = get_attr(child_tag, "alt").map(|alt| vec![t(alt)]);
                let image_title =
                    get_attr(child_tag, "title").map(|title_text| vec![t(title_text)]);

                let image_object = ImageObject {
                    content_url,
                    caption: image_caption,
                    title: image_title,
                    ..Default::default()
                };

                content.push(Block::ImageObject(image_object));
            } else {
                content.append(&mut decode_blocks(parser, child_tag, context));
            }
        }
    }

    Block::Figure(Figure {
        content,
        caption,
        label,
        ..Default::default()
    })
}

/// Extract label and caption content from a table caption, handling ltx_tag_table spans
fn extract_table_label_and_caption(
    parser: &Parser,
    figcaption_tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> (Option<String>, Vec<Block>) {
    let mut label = None;
    let mut caption_inlines = Vec::new();

    for child in figcaption_tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            if child_class.contains("ltx_tag_table") {
                // Extract label from table tag like "Table 3.1: " -> "3.1"
                let text = get_text(parser, child_tag);
                label = extract_number_from_table_title(&text);
                // Don't include this in caption content
            } else {
                // Include all other content in caption
                caption_inlines.extend(decode_inline(parser, child_tag, context));
            }
        } else if let Some(text) = child.as_raw() {
            let text_content = text.try_as_utf8_str().unwrap_or_default();
            // Only include non-whitespace text in caption
            if !text_content.trim().is_empty() {
                caption_inlines.push(t(text_content));
            }
        }
    }

    // Wrap caption inlines in a paragraph
    let caption_blocks = if caption_inlines.is_empty() {
        Vec::new()
    } else {
        vec![p(caption_inlines)]
    };

    (label, caption_blocks)
}

/// Extract number from table title like "Table 3.1: " -> "3.1"
fn extract_number_from_table_title(text: &str) -> Option<String> {
    let trimmed = text.trim().trim_end_matches(':').trim();
    let words: Vec<&str> = trimmed.split_whitespace().collect();

    // Look for the last word that contains numbers and periods
    for word in words.iter().rev() {
        if word.chars().any(|c| c.is_ascii_digit()) && word.contains('.') {
            return Some(word.to_string());
        }
    }

    // Fallback: look for any word with numbers
    for word in words.iter().rev() {
        if word.chars().any(|c| c.is_ascii_digit()) {
            return Some(word.to_string());
        }
    }

    None
}

/// Decode a figure with class ltx_table as a [`Table`]
pub fn decode_figure_table(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> Block {
    // Extract label and caption properly, handling ltx_tag_table spans
    let mut label = None;
    let mut caption = None;

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            if child_class.contains("ltx_caption") {
                let (extracted_label, caption_content) =
                    extract_table_label_and_caption(parser, child_tag, context);
                label = extracted_label;
                caption = Some(caption_content);
                break; // Stop after finding the caption
            }
        }
    }

    // Find the actual table element within the figure
    let mut rows = Vec::new();
    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag()
            && child_tag.name().as_utf8_str() == "table"
        {
            rows = extract_table_rows(parser, child_tag, context);
            break;
        }
    }

    Block::Table(Table {
        rows,
        label,
        caption,
        ..Default::default()
    })
}

/// Extract table rows from a table element
pub fn extract_table_rows(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> Vec<TableRow> {
    let mut rows = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_name = child_tag.name().as_utf8_str();
            match child_name.as_ref() {
                "thead" => {
                    // Process rows within thead
                    for row_handle in child_tag
                        .children()
                        .top()
                        .iter()
                        .flat_map(|h| h.get(parser))
                    {
                        if let Some(row_tag) = row_handle.as_tag()
                            && row_tag.name().as_utf8_str() == "tr"
                        {
                            rows.push(decode_table_row(
                                parser,
                                row_tag,
                                context,
                                Some(TableRowType::HeaderRow),
                            ));
                        }
                    }
                }
                "tbody" => {
                    // Process rows within tbody
                    for row_handle in child_tag
                        .children()
                        .top()
                        .iter()
                        .flat_map(|h| h.get(parser))
                    {
                        if let Some(row_tag) = row_handle.as_tag()
                            && row_tag.name().as_utf8_str() == "tr"
                        {
                            rows.push(decode_table_row(
                                parser,
                                row_tag,
                                context,
                                Some(TableRowType::BodyRow),
                            ));
                        }
                    }
                }
                "tr" => {
                    // Default to body row for direct tr elements
                    rows.push(decode_table_row(
                        parser,
                        child_tag,
                        context,
                        Some(TableRowType::BodyRow),
                    ));
                }
                _ => {
                    // Ignore other elements like caption, etc.
                }
            }
        }
    }

    rows
}

/// Decode a table row
fn decode_table_row(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
    row_type: Option<TableRowType>,
) -> TableRow {
    let mut cells = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_name = child_tag.name().as_utf8_str();
            if child_name == "td" || child_name == "th" {
                cells.push(decode_table_cell(parser, child_tag, context));
            }
        }
    }

    TableRow {
        cells,
        row_type,
        ..Default::default()
    }
}

/// Decode a table cell with proper cell type and alignment
fn decode_table_cell(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> TableCell {
    // Determine cell type based on tag name
    let cell_type = match tag.name().as_utf8_str().as_ref() {
        "th" => Some(TableCellType::HeaderCell),
        "td" => Some(TableCellType::DataCell),
        _ => None,
    };

    // Extract horizontal alignment from class attributes
    let class = get_class(tag);
    let horizontal_alignment = if class.contains("ltx_align_center") {
        Some(HorizontalAlignment::AlignCenter)
    } else if class.contains("ltx_align_left") {
        Some(HorizontalAlignment::AlignLeft)
    } else if class.contains("ltx_align_right") {
        Some(HorizontalAlignment::AlignRight)
    } else {
        None
    };

    let content = decode_inlines(parser, tag, context);
    let content = if content.is_empty() {
        Vec::new()
    } else {
        vec![p(content)]
    };

    TableCell {
        cell_type,
        horizontal_alignment,
        content,
        ..Default::default()
    }
}

/// Decode a heading element with class-aware level mapping
pub fn decode_heading(
    parser: &Parser,
    tag: &HTMLTag,
    tag_name: &str,
    tag_class: &str,
    context: &mut ArxivDecodeContext,
) -> Block {
    use crate::decode_html::extract_label_and_inlines;

    // Extract label and content from heading
    let (label, content) = extract_label_and_inlines(parser, tag, context);

    // Create heading with label if found
    let level = if tag_class.contains("ltx_title_abstract") {
        // Abstract title should always be h1 as it's a top-level section
        1
    } else if tag_class.contains("ltx_title_section") {
        1
    } else if tag_class.contains("ltx_title_subsection") {
        2
    } else if tag_class.contains("ltx_title_subsubsection") {
        3
    } else {
        match tag_name {
            "h1" => 1,
            "h2" => 2,
            "h3" => 3,
            "h4" => 4,
            "h5" => 5,
            _ => 6,
        }
    };

    Block::Heading(Heading {
        level,
        label,
        content,
        ..Default::default()
    })
}

/// Decode blocks within a list item, stripping leading numbers for ordered lists
fn decode_list_item_blocks(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
    ordered: bool,
) -> Vec<Block> {
    let mut blocks = decode_blocks(parser, tag, context);

    // For ordered lists, try to remove leading list numbers
    if ordered && !blocks.is_empty() {
        // Check if the first block is a paragraph containing only a list number
        let should_remove_first_block = if let Block::Paragraph(paragraph) = &blocks[0] {
            if paragraph.content.len() == 1 {
                if let Inline::Text(text_node) = &paragraph.content[0] {
                    is_list_number_only(&text_node.value)
                } else {
                    false
                }
            } else {
                false
            }
        } else {
            false
        };

        if should_remove_first_block {
            // Remove the entire first paragraph that contains only the list number
            blocks.remove(0);
        } else {
            // Try to strip list number from the beginning of the first paragraph
            if let Block::Paragraph(paragraph) = &mut blocks[0]
                && let Some(Inline::Text(text_node)) = paragraph.content.first_mut()
            {
                let text = &text_node.value;
                if let Some(content_start) = find_content_after_list_number(text) {
                    if content_start < text.len() {
                        // Update the text content, removing the list number
                        text_node.value = text[content_start..].to_string().into();
                    } else {
                        // The text was only a number, remove this text node entirely
                        paragraph.content.remove(0);
                        // If paragraph is now empty, remove it too
                        if paragraph.content.is_empty() {
                            blocks.remove(0);
                        }
                    }
                }
            }
        }
    }

    blocks
}

/// Check if text contains only a list number (e.g., "1." or "2. ")
fn is_list_number_only(text: &str) -> bool {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return false;
    }

    // Check if it's just digits followed by a period
    let mut chars = trimmed.chars();
    let mut found_digits = false;

    // Look for digits at the start
    while let Some(ch) = chars.next() {
        if ch.is_ascii_digit() {
            found_digits = true;
        } else if ch == '.' && found_digits {
            // Check that there's nothing meaningful after the period (only whitespace)
            return chars.all(|c| c.is_whitespace());
        } else {
            return false;
        }
    }

    false
}

/// Find the position where actual content starts after a list number (e.g., "1. " -> 3)
fn find_content_after_list_number(text: &str) -> Option<usize> {
    let trimmed = text.trim_start();
    let mut chars = trimmed.char_indices();

    // Look for digits at the start
    let mut last_digit_end = 0;
    let mut found_digits = false;

    for (i, ch) in chars.by_ref() {
        if ch.is_ascii_digit() {
            found_digits = true;
            last_digit_end = i + ch.len_utf8();
        } else {
            break;
        }
    }

    if !found_digits {
        return None;
    }

    // Check for period after digits
    let remaining = &trimmed[last_digit_end..];
    if remaining.starts_with('.') {
        let after_period = last_digit_end + 1;
        // Skip any whitespace after the period
        let final_start = trimmed[after_period..]
            .chars()
            .take_while(|c| c.is_whitespace())
            .map(|c| c.len_utf8())
            .sum::<usize>()
            + after_period;

        // Account for leading whitespace that was trimmed
        let leading_whitespace = text.len() - trimmed.len();
        Some(leading_whitespace + final_start)
    } else {
        None
    }
}

/// Decode a list element (ul/ol)
pub fn decode_list(
    parser: &Parser,
    tag: &HTMLTag,
    ordered: bool,
    context: &mut ArxivDecodeContext,
) -> Block {
    let mut items = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag()
            && child_tag.name().as_utf8_str() == "li"
        {
            let item_content = decode_list_item_blocks(parser, child_tag, context, ordered);
            items.push(ListItem {
                content: item_content,
                ..Default::default()
            });
        }
    }

    Block::List(List {
        items,
        order: if ordered {
            ListOrder::Ascending
        } else {
            ListOrder::Unordered
        },
        ..Default::default()
    })
}

/// Decode a mathematical equation block (table.ltx_equation) to MathBlock
pub fn decode_math_block(
    parser: &Parser,
    tag: &HTMLTag,
    _context: &mut ArxivDecodeContext,
) -> Block {
    let (latex_code, mathml_content) = extract_latex_and_mathml(parser, tag);

    let mut math_block = MathBlock {
        code: latex_code.into(),
        math_language: Some("latex".to_string()),
        ..Default::default()
    };

    // Set MathML in options if available
    if !mathml_content.is_empty() {
        math_block.options.mathml = Some(mathml_content);
    }

    Block::MathBlock(math_block)
}

/// Decode a <p> element into a [`Paragraph`]
pub fn decode_p(parser: &Parser, tag: &HTMLTag, context: &mut ArxivDecodeContext) -> Block {
    p(decode_inlines(parser, tag, context))
}

/// Decode a <section> element into a [`Section`] node
pub fn decode_section(
    parser: &Parser,
    tag: &HTMLTag,
    section_type: SectionType,
    context: &mut ArxivDecodeContext,
) -> Block {
    let content = decode_blocks(parser, tag, context);
    Block::Section(Section {
        section_type: Some(section_type),
        content,
        ..Default::default()
    })
}

/// Decode an SVG as RawBlock
pub fn decode_svg(parser: &Parser, tag: &HTMLTag, _context: &mut ArxivDecodeContext) -> Block {
    let svg = tag.outer_html(parser);

    Block::RawBlock(RawBlock {
        content: svg.into(),
        format: "svg".into(),
        ..Default::default()
    })
}

/// Decode a table element
pub fn decode_table(parser: &Parser, tag: &HTMLTag, context: &mut ArxivDecodeContext) -> Block {
    let rows = extract_table_rows(parser, tag, context);

    Block::Table(Table {
        rows,
        ..Default::default()
    })
}

/// Extract label and content from a theorem, handling the special heading structure
fn extract_theorem_label_and_content(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> (Option<String>, Vec<Block>) {
    let mut label = None;
    let mut content = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_name = child_tag.name().as_utf8_str();
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            // Check if this is the theorem title heading
            if (child_name == "h6"
                || child_name == "h5"
                || child_name == "h4"
                || child_name == "h3")
                && child_class.contains("ltx_title_theorem")
            {
                // Extract label from ltx_tag_theorem spans within the heading
                label = extract_theorem_label(parser, child_tag);
                // Don't include the heading in content
            } else {
                // Include all other content (paragraphs, etc.)
                content.append(&mut decode_blocks(parser, child_tag, context));
            }
        } else if let Some(text) = child.as_raw() {
            let text_content = text.try_as_utf8_str().unwrap_or_default().trim();
            if !text_content.is_empty() {
                content.push(p([t(text_content)]));
            }
        }
    }

    (label, content)
}

/// Extract the label (like "2.1") from a theorem heading
fn extract_theorem_label(parser: &Parser, heading_tag: &HTMLTag) -> Option<String> {
    // Look for ltx_tag_theorem spans within the heading
    for child in heading_tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let child_class = child_tag
                .attributes()
                .class()
                .map(|cls| cls.as_utf8_str())
                .unwrap_or_default();

            if child_class.contains("ltx_tag_theorem") {
                // Get the text content and extract just the number part
                let text = get_text(parser, child_tag);
                // Extract number from text like "Proposition 2.1" -> "2.1"
                if let Some(number_part) = extract_number_from_theorem_title(&text) {
                    return Some(number_part);
                }
            }
        }
    }
    None
}

/// Extract number from theorem title like "Proposition 2.1" -> "2.1"
fn extract_number_from_theorem_title(text: &str) -> Option<String> {
    let words: Vec<&str> = text.split_whitespace().collect();
    // Look for the last word that contains numbers and periods
    for word in words.iter().rev() {
        if word.chars().any(|c| c.is_ascii_digit()) && word.contains('.') {
            return Some(word.to_string());
        }
    }

    // Fallback: look for any word with numbers
    for word in words.iter().rev() {
        if word.chars().any(|c| c.is_ascii_digit()) {
            return Some(word.to_string());
        }
    }

    None
}

/// Decode a div.ltx_theorem or div.ltx_proof into a Stencila [`Claim`]
pub fn decode_theorem_or_proof(
    parser: &Parser,
    tag: &HTMLTag,
    context: &mut ArxivDecodeContext,
) -> Block {
    let class = get_class(tag);

    // Determine claim type from class
    let claim_type = if class.contains("proposition") {
        ClaimType::Proposition
    } else if class.contains("lemma") {
        ClaimType::Lemma
    } else if class.contains("ltx_theorem_cor") || class.contains("corollary") {
        ClaimType::Corollary
    } else if class.contains("proof") {
        ClaimType::Proof
    } else if class.contains("hypothesis") {
        ClaimType::Hypothesis
    } else if class.contains("postulate") {
        ClaimType::Postulate
    } else {
        // Default to theorem for ltx_theorem class
        ClaimType::Theorem
    };

    // Extract label and content specially for theorems
    let (label, content) = extract_theorem_label_and_content(parser, tag, context);

    Block::Claim(Claim {
        claim_type,
        label,
        content,
        ..Default::default()
    })
}
