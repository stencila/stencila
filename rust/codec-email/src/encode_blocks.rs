use stencila_codec::{
    Losses,
    stencila_format::Format,
    stencila_schema::{
        Block, CodeBlock, CodeChunk, Datatable, Heading, ImageObject, LabelType, List, ListItem,
        ListOrder, MathBlock, Node, Primitive, QuoteBlock, Table, TableCell, TableRow,
    },
};
use stencila_codec_dom_trait::to_dom;
use stencila_codec_text_trait::TextCodec;
use stencila_convert::html_to_png_data_uri;

use crate::{encode_inlines, html_escape};

/// Encode a list of blocks to MJML
pub(super) fn encode_blocks(blocks: &[Block], mjml: &mut String, losses: &mut Losses) {
    for block in blocks {
        encode_block(block, mjml, losses);
    }
}

/// Encode a single block to MJML
fn encode_block(block: &Block, mjml: &mut String, losses: &mut Losses) {
    match block {
        Block::Paragraph(para) => {
            mjml.push_str("        <mj-text>\n");
            mjml.push_str("          <p>");
            encode_inlines(&para.content, mjml, losses);
            mjml.push_str("</p>\n");
            mjml.push_str("        </mj-text>\n");
        }
        Block::Heading(heading) => {
            encode_heading(heading, mjml, losses);
        }
        Block::List(list) => {
            encode_list(list, mjml, losses);
        }
        Block::Table(table) => {
            encode_table(table, mjml, losses);
        }
        Block::CodeBlock(code_block) => {
            encode_code_block(code_block, mjml);
        }
        Block::CodeChunk(code_chunk) => {
            encode_code_chunk(code_chunk, mjml, losses);
        }
        Block::QuoteBlock(quote) => {
            encode_quote_block(quote, mjml, losses);
        }
        Block::ThematicBreak(_) => {
            mjml.push_str("        <mj-divider/>\n");
        }
        Block::MathBlock(math) => {
            encode_math_block(math, mjml, losses);
        }
        Block::Section(section) => {
            // Encode section content directly
            encode_blocks(&section.content, mjml, losses);
        }
        _ => {
            // Track unsupported block types
            losses.add(format!("Block::{}", block.node_type()));
        }
    }
}

/// Encode a heading
fn encode_heading(heading: &Heading, mjml: &mut String, losses: &mut Losses) {
    let level = heading.level.clamp(1, 6);
    mjml.push_str("        <mj-text>\n");
    mjml.push_str(&format!("          <h{level}>"));
    encode_inlines(&heading.content, mjml, losses);
    mjml.push_str(&format!("</h{level}>\n"));
    mjml.push_str("        </mj-text>\n");
}

/// Encode a list
fn encode_list(list: &List, mjml: &mut String, losses: &mut Losses) {
    let tag = if matches!(list.order, ListOrder::Ascending) {
        "ol"
    } else {
        "ul"
    };
    mjml.push_str("        <mj-text>\n");
    mjml.push_str(&format!("          <{tag}>\n"));
    for item in &list.items {
        encode_list_item(item, mjml, losses);
    }
    mjml.push_str(&format!("          </{tag}>\n"));
    mjml.push_str("        </mj-text>\n");
}

/// Encode a list item
fn encode_list_item(item: &ListItem, mjml: &mut String, losses: &mut Losses) {
    mjml.push_str("            <li>");
    for block in &item.content {
        match block {
            Block::Paragraph(para) => {
                encode_inlines(&para.content, mjml, losses);
            }
            _ => {
                // Nested blocks in list items are simplified
                encode_block(block, mjml, losses);
            }
        }
    }
    mjml.push_str("</li>\n");
}

/// Encode a table
fn encode_table(table: &Table, mjml: &mut String, losses: &mut Losses) {
    mjml.push_str("        <mj-table css-class=\"content-table\">\n");
    for row in &table.rows {
        encode_table_row(row, mjml, losses);
    }
    mjml.push_str("        </mj-table>\n");
}

/// Encode a table row
fn encode_table_row(row: &TableRow, mjml: &mut String, losses: &mut Losses) {
    mjml.push_str("          <tr>\n");
    for cell in &row.cells {
        encode_table_cell(cell, mjml, losses);
    }
    mjml.push_str("          </tr>\n");
}

/// Encode a table cell
fn encode_table_cell(cell: &TableCell, mjml: &mut String, losses: &mut Losses) {
    let tag = match cell.cell_type {
        Some(stencila_codec::stencila_schema::TableCellType::HeaderCell) => "th",
        _ => "td",
    };
    mjml.push_str(&format!("            <{tag}>"));
    for block in &cell.content {
        match block {
            Block::Paragraph(para) => {
                encode_inlines(&para.content, mjml, losses);
            }
            _ => {
                losses.add(format!("TableCell content: {}", block.node_type()));
            }
        }
    }
    mjml.push_str(&format!("</{tag}>\n"));
}

/// Encode a code block
fn encode_code_block(code_block: &CodeBlock, mjml: &mut String) {
    mjml.push_str("        <mj-text>\n");
    mjml.push_str("          <pre><code>");
    mjml.push_str(&html_escape(&code_block.code));
    mjml.push_str("</code></pre>\n");
    mjml.push_str("        </mj-text>\n");
}

/// Encode a code chunk with its outputs, label, and caption
fn encode_code_chunk(code_chunk: &CodeChunk, mjml: &mut String, losses: &mut Losses) {
    // Determine the label prefix based on label_type
    let label_prefix = match &code_chunk.label_type {
        Some(LabelType::TableLabel) => "Table",
        Some(LabelType::FigureLabel) => "Figure",
        Some(LabelType::AppendixLabel) => "Appendix",
        Some(LabelType::SupplementLabel) => "Supplement",
        None => "Figure", // Default to figure if not specified
    };

    // Encode the label if present
    if let Some(label) = &code_chunk.label {
        mjml.push_str("        <mj-text>\n");
        mjml.push_str(&format!(
            "          <p class=\"figure-label\"><strong>{} {}.</strong></p>\n",
            label_prefix,
            html_escape(label)
        ));
        mjml.push_str("        </mj-text>\n");
    }

    // Encode outputs if present
    if let Some(outputs) = &code_chunk.outputs {
        for output in outputs {
            encode_code_chunk_output(output, mjml, losses);
        }
    }

    // Encode caption if present
    if let Some(caption) = &code_chunk.caption {
        mjml.push_str("        <mj-text>\n");
        mjml.push_str("          <div class=\"figure-caption\">\n");
        for block in caption {
            match block {
                Block::Paragraph(para) => {
                    mjml.push_str("            <p>");
                    encode_inlines(&para.content, mjml, losses);
                    mjml.push_str("</p>\n");
                }
                _ => {
                    losses.add(format!("CodeChunk caption block: {}", block.node_type()));
                }
            }
        }
        mjml.push_str("          </div>\n");
        mjml.push_str("        </mj-text>\n");
    }
}

/// Encode a code chunk output
fn encode_code_chunk_output(output: &Node, mjml: &mut String, losses: &mut Losses) {
    match output {
        Node::Datatable(datatable) => {
            encode_datatable(datatable, mjml, losses);
        }
        Node::ImageObject(image) => {
            encode_output_image(image, mjml, losses);
        }
        Node::Paragraph(para) => {
            mjml.push_str("        <mj-text>\n");
            mjml.push_str("          <p>");
            encode_inlines(&para.content, mjml, losses);
            mjml.push_str("</p>\n");
            mjml.push_str("        </mj-text>\n");
        }
        Node::String(s) => {
            // Render string output as preformatted text
            mjml.push_str("        <mj-text>\n");
            mjml.push_str("          <pre>");
            mjml.push_str(&html_escape(s));
            mjml.push_str("</pre>\n");
            mjml.push_str("        </mj-text>\n");
        }
        _ => {
            // For other output types, try to render as text
            let text = output.to_text();
            if !text.is_empty() {
                mjml.push_str("        <mj-text>\n");
                mjml.push_str("          <pre>");
                mjml.push_str(&html_escape(&text));
                mjml.push_str("</pre>\n");
                mjml.push_str("        </mj-text>\n");
            } else {
                losses.add(format!("CodeChunk output: {}", output.node_type()));
            }
        }
    }
}

/// Encode a datatable as an MJML table
fn encode_datatable(datatable: &Datatable, mjml: &mut String, _losses: &mut Losses) {
    mjml.push_str("        <mj-table css-class=\"content-table\">\n");

    // Encode header row with column names
    mjml.push_str("          <tr>\n");
    for column in &datatable.columns {
        mjml.push_str(&format!(
            "            <th>{}</th>\n",
            html_escape(&column.name)
        ));
    }
    mjml.push_str("          </tr>\n");

    // Encode data rows
    let num_rows = datatable.rows();
    for row_index in 0..num_rows {
        mjml.push_str("          <tr>\n");
        for column in &datatable.columns {
            if let Some(value) = column.values.get(row_index) {
                let text = match value {
                    Primitive::Null(_) => String::new(),
                    Primitive::Boolean(b) => b.to_string(),
                    Primitive::Integer(i) => i.to_string(),
                    Primitive::UnsignedInteger(u) => u.to_string(),
                    Primitive::Number(n) => n.to_string(),
                    Primitive::String(s) => s.clone(),
                    _ => serde_json::to_string(value).unwrap_or_default(),
                };
                mjml.push_str(&format!("            <td>{}</td>\n", html_escape(&text)));
            } else {
                mjml.push_str("            <td></td>\n");
            }
        }
        mjml.push_str("          </tr>\n");
    }

    mjml.push_str("        </mj-table>\n");
}

/// Encode an image output from a code chunk
///
/// For visualizations (Plotly, Vega, Mermaid, etc.), this function renders the
/// visualization to a PNG using headless Chrome and embeds it as a data URI.
fn encode_output_image(image: &ImageObject, mjml: &mut String, losses: &mut Losses) {
    // Check if this is a visualization that needs to be rendered to PNG
    let needs_screenshot = image
        .media_type
        .as_ref()
        .and_then(|mt| Format::from_media_type(mt).ok())
        .is_some_and(|format| format.is_viz());

    if needs_screenshot {
        // Render the visualization to HTML and then to PNG
        let dom_html = to_dom(&Node::ImageObject(image.clone()));
        match html_to_png_data_uri(&dom_html) {
            Ok(data_uri) => {
                mjml.push_str(&format!(
                    "        <mj-image src=\"{}\" alt=\"{}\"/>\n",
                    data_uri,
                    image
                        .caption
                        .as_ref()
                        .map(|c| html_escape(&c.to_text()))
                        .unwrap_or_default()
                ));
            }
            Err(e) => {
                tracing::warn!("Failed to render visualization to PNG: {}", e);
                losses.add(format!("ImageObject visualization render error: {}", e));
                // Fall back to showing a placeholder or the original URL if available
                if !image.content_url.is_empty()
                    && (image.content_url.starts_with("http")
                        || image.content_url.starts_with("data:"))
                {
                    mjml.push_str(&format!(
                        "        <mj-image src=\"{}\" alt=\"{}\"/>\n",
                        html_escape(&image.content_url),
                        image
                            .caption
                            .as_ref()
                            .map(|c| html_escape(&c.to_text()))
                            .unwrap_or_default()
                    ));
                }
            }
        }
    } else {
        // Regular image - just use the URL directly
        mjml.push_str(&format!(
            "        <mj-image src=\"{}\" alt=\"{}\"/>\n",
            html_escape(&image.content_url),
            image
                .caption
                .as_ref()
                .map(|c| html_escape(&c.to_text()))
                .unwrap_or_default()
        ));
    }
}

/// Encode a quote block
fn encode_quote_block(quote: &QuoteBlock, mjml: &mut String, losses: &mut Losses) {
    mjml.push_str("        <mj-text>\n");
    mjml.push_str("          <blockquote>\n");
    for block in &quote.content {
        match block {
            Block::Paragraph(para) => {
                mjml.push_str("            <p>");
                encode_inlines(&para.content, mjml, losses);
                mjml.push_str("</p>\n");
            }
            _ => {
                losses.add(format!("QuoteBlock content: {}", block.node_type()));
            }
        }
    }
    mjml.push_str("          </blockquote>\n");
    mjml.push_str("        </mj-text>\n");
}

/// Encode a math block
///
/// Renders the math to a PNG image since email clients don't support MathML.
fn encode_math_block(math: &MathBlock, mjml: &mut String, losses: &mut Losses) {
    // Render math to DOM HTML, then to PNG data URI
    let dom_html = to_dom(&Node::MathBlock(math.clone()));
    match html_to_png_data_uri(&dom_html) {
        Ok(data_uri) => {
            mjml.push_str("        <mj-text>\n");

            // Add label if present
            if let Some(label) = &math.label {
                mjml.push_str(&format!(
                    "          <p class=\"equation-label\"><strong>Equation {}.</strong></p>\n",
                    html_escape(label)
                ));
            }

            // Use img tag centered, not mj-image which stretches to full width
            mjml.push_str(&format!(
                "          <p style=\"text-align: center;\"><img src=\"{}\" alt=\"{}\" style=\"max-width: 100%; height: auto;\"/></p>\n",
                data_uri,
                html_escape(&math.code)
            ));

            mjml.push_str("        </mj-text>\n");
        }
        Err(e) => {
            // Fallback to showing the math code in a code block
            tracing::warn!("Failed to render MathBlock to PNG: {}", e);
            mjml.push_str("        <mj-text>\n");
            mjml.push_str("          <pre><code>");
            mjml.push_str(&html_escape(&math.code));
            mjml.push_str("</code></pre>\n");
            mjml.push_str("        </mj-text>\n");
            losses.add(format!("MathBlock render error: {e}"));
        }
    }
}
