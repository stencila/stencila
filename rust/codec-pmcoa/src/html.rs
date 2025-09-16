//! HTML decoder for PMC article pages.
//!
//! This module handles the conversion of HTML from PubMed Central article pages
//! into Stencila document structures. PMC HTML pages contain semantic markup
//! that can be parsed to extract article metadata and content.

use std::{fs::read_to_string, path::Path};

use tl::{HTMLTag, Parser, ParserOptions, parse};

use stencila_codec::{
    DecodeInfo, DecodeOptions,
    eyre::{Result, bail},
    stencila_schema::{
        Article, Author, Block, Citation, CreativeWorkVariant, Date, Figure, Heading, ImageObject,
        Inline, IntegerOrString, Node, Organization, Paragraph, Periodical, Person, Primitive,
        PropertyValue, PropertyValueOrString, PublicationIssue, PublicationVolume, Reference,
        Section, SectionType, Supplement, Table, TableCell, TableCellOptions, TableRow,
        TableRowType,
        shortcuts::{em, h1, h2, lnk, p, stg, sub, sup, t, u},
    },
};
use stencila_codec_biblio::decode::{text_to_author, text_to_reference};

/// Decode a PMC HTML file to a Stencila [`Node`]
#[tracing::instrument]
pub(super) fn decode_html_path(
    path: &Path,
    options: Option<DecodeOptions>,
) -> Result<(Node, Option<Node>, DecodeInfo)> {
    // Read the HTML content
    let html = read_to_string(path)?;
    if html.trim().is_empty() {
        bail!("HTML is empty");
    }

    // Parse the HTML
    let dom = parse(&html, ParserOptions::default())?;
    let parser = dom.parser();

    // Extract metadata from meta tags
    let mut article = extract_metadata(&dom)?;

    // Extract title from front-matter section
    if let Some(front_matter) = dom
        .query_selector("section.front-matter")
        .and_then(|mut nodes| nodes.next())
        .and_then(|node| node.get(parser))
        .and_then(|node| node.as_tag())
    {
        if let Some(h1_tag) = front_matter
            .query_selector(parser, "h1")
            .and_then(|mut nodes| nodes.next())
            .and_then(|node| node.get(parser))
            .and_then(|node| node.as_tag())
        {
            let title_inlines = decode_inlines(parser, h1_tag)?;
            if !title_inlines.is_empty() {
                article.title = Some(title_inlines);
            }
        }
    }

    // Extract abstract, content, and references
    if let Some(body_section) = dom
        .query_selector("section.body.main-article-body")
        .and_then(|mut nodes| nodes.next())
        .and_then(|node| node.get(parser))
        .and_then(|node| node.as_tag())
    {
        let rest = decode_article(parser, body_section)?;
        article.r#abstract = rest.r#abstract;
        article.content = rest.content;
        article.references = rest.references;
    }

    Ok((Node::Article(article), None, DecodeInfo::none()))
}

/// Extract metadata from HTML meta tags
fn extract_metadata(dom: &tl::VDom) -> Result<Article> {
    let mut article = Article::default();

    let mut identifiers = Vec::new();
    let mut journal_title = None;
    let mut volume_number = None;
    let mut issue_number = None;

    // Extract PMCID from canonical link tag
    if let Some(canonical_node) = dom
        .query_selector("link[rel='canonical']")
        .and_then(|mut nodes| nodes.next())
        .and_then(|node| node.get(dom.parser()))
        .and_then(|node| node.as_tag())
        && let Some(href) = get_attr(canonical_node, "href")
    {
        if let Some(pmc) = href.strip_prefix("https://pmc.ncbi.nlm.nih.gov/articles/PMC") {
            identifiers.push(PropertyValueOrString::PropertyValue(PropertyValue {
                property_id: Some("pmc".to_string()),
                value: Primitive::String(pmc.trim_end_matches("/").into()),
                ..Default::default()
            }));
        }
    }

    // Extract metadata from meta tags
    let meta_nodes: Vec<_> = dom
        .query_selector("meta")
        .map(|iter| iter.collect())
        .unwrap_or_default();

    let mut authors = Vec::new();
    for node_handle in meta_nodes {
        if let Some(node) = node_handle.get(dom.parser())
            && let Some(tag) = node.as_tag()
            && let Some(name) = get_attr(tag, "name")
            && let Some(content) = get_attr(tag, "content")
        {
            match name.as_str() {
                "citation_doi" => {
                    article.doi = Some(content);
                }
                "citation_publication_date" => {
                    article.date_published = Some(Date::new(content));
                }
                "citation_author" => {
                    let author = text_to_author(&content)
                        .unwrap_or_else(|| Author::Person(Person::from(content)));
                    authors.push(author)
                }
                "citation_author_institution" => {
                    let org = Organization {
                        name: Some(content),
                        ..Default::default()
                    };
                    if let Some(Author::Person(Person { affiliations, .. })) = authors.last_mut() {
                        affiliations.get_or_insert_default().push(org);
                    }
                }
                "citation_pmid" => {
                    identifiers.push(PropertyValueOrString::PropertyValue(PropertyValue {
                        property_id: Some("pmid".to_string()),
                        value: Primitive::String(content),
                        ..Default::default()
                    }));
                }
                "citation_journal_title" => {
                    journal_title = Some(content);
                }
                "citation_volume" => {
                    volume_number = Some(content);
                }
                "citation_issue" => {
                    issue_number = Some(content);
                }
                "citation_firstpage" => {
                    article.options.page_start = Some(IntegerOrString::from(content));
                }
                "citation_lastpage" => {
                    article.options.page_end = Some(IntegerOrString::from(content));
                }
                _ => {}
            }
        }
    }

    // Build isPartOf structure if we have the necessary data
    if let Some(journal) = journal_title {
        let periodical = Periodical {
            name: Some(journal),
            ..Default::default()
        };

        let work = if let Some(volume) = volume_number {
            let publication_volume = PublicationVolume {
                is_part_of: Some(Box::new(CreativeWorkVariant::Periodical(periodical))),
                volume_number: Some(IntegerOrString::from(&volume)),
                ..Default::default()
            };

            if let Some(issue) = issue_number {
                let publication_issue = PublicationIssue {
                    is_part_of: Some(Box::new(CreativeWorkVariant::PublicationVolume(
                        publication_volume,
                    ))),
                    issue_number: Some(IntegerOrString::from(&issue)),
                    ..Default::default()
                };
                CreativeWorkVariant::PublicationIssue(publication_issue)
            } else {
                CreativeWorkVariant::PublicationVolume(publication_volume)
            }
        } else {
            CreativeWorkVariant::Periodical(periodical)
        };

        article.options.is_part_of = Some(work);
    }

    if !authors.is_empty() {
        article.authors = Some(authors);
    }

    if !identifiers.is_empty() {
        article.options.identifiers = Some(identifiers);
    }

    Ok(article)
}

/// Decode the main article from the body section
fn decode_article(parser: &Parser, body_section: &HTMLTag) -> Result<Article> {
    let mut r#abstract = None;
    let mut content = Vec::new();
    let mut references = None;

    // Process all children of the body section
    for child in body_section
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();
            let class = get_class(tag);

            match tag_name.as_ref() {
                "section" if class.contains("front-matter") => {
                    // Front matter is outside main-article-body, skip if found here
                    // (we already extracted metadata from meta tags)
                }
                "section" if class.contains("abstract") => {
                    r#abstract = Some(decode_abstract(parser, tag)?);
                }
                "section" if class.contains("ref-list") => {
                    references = Some(decode_references(parser, tag)?);
                }
                "section" if class.contains("associated-data") => {
                    // Skip as repeats supplementary materials section
                }
                "section" => {
                    // Regular content section
                    let section = decode_section(parser, tag)?;
                    // Only add non-empty sections
                    if !section.content.is_empty() {
                        content.push(Block::Section(section));
                    }
                }
                _ => {}
            }
        }
    }

    Ok(Article {
        r#abstract,
        content,
        references,
        ..Default::default()
    })
}

/// Get an attribute value as Option<String>
fn get_attr(tag: &HTMLTag, name: &str) -> Option<String> {
    tag.attributes()
        .get(name)
        .flatten()
        .map(|bytes| bytes.as_utf8_str().to_string())
}

/// Get the class attribute as a string, or empty string if not present
fn get_class(tag: &HTMLTag) -> String {
    tag.attributes()
        .class()
        .map(|cls| cls.as_utf8_str().to_string())
        .unwrap_or_default()
}

/// Extract text content from an HTML element
fn get_text(parser: &Parser, tag: &HTMLTag) -> String {
    let mut text_parts = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            text_parts.push(get_text(parser, child_tag));
        } else if let Some(text) = child.as_raw()
            && let Some(text_str) = text.try_as_utf8_str()
        {
            text_parts.push(decode_html_entities(text_str));
        }
    }

    text_parts.join(" ").trim().to_string()
}

/// Decode common HTML entities
fn decode_html_entities(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#x27;", "'")
        .replace("&#39;", "'")
        .replace("&apos;", "'")
        .replace("&nbsp;", " ")
}

/// Decode abstract section
fn decode_abstract(parser: &Parser, abstract_section: &HTMLTag) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();

    for child in abstract_section
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();

            match tag_name.as_ref() {
                "section" => {
                    // Abstract subsection - convert to section without recursive headings
                    let mut section_type = None;
                    let mut content = Vec::new();

                    for section_child in tag
                        .children()
                        .top()
                        .iter()
                        .flat_map(|handle| handle.get(parser))
                    {
                        if let Some(section_tag) = section_child.as_tag() {
                            let child_tag_name = section_tag.name().as_utf8_str();

                            match child_tag_name.as_ref() {
                                "h3" => {
                                    let heading_text = get_text(parser, section_tag);
                                    section_type = SectionType::from_text(&heading_text).ok();

                                    content.push(h2([t(heading_text)]));
                                }
                                "p" => {
                                    let paragraph = decode_paragraph(parser, section_tag)?;
                                    content.push(paragraph);
                                }
                                _ => {}
                            }
                        }
                    }

                    if !content.is_empty() {
                        let section = Section {
                            section_type,
                            content,
                            ..Default::default()
                        };
                        blocks.push(Block::Section(section));
                    }
                }
                "p" => {
                    // Direct paragraph in abstract
                    let paragraph = decode_paragraph(parser, tag)?;
                    blocks.push(paragraph);
                }
                "h2" => {
                    // Skip abstract heading
                }
                _ => {}
            }
        }
    }

    Ok(blocks)
}

/// Decode a section
fn decode_section(parser: &Parser, section: &HTMLTag) -> Result<Section> {
    let mut section_type = None;
    let mut content = Vec::new();

    for child in section
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        let Some(tag) = child.as_tag() else {
            continue;
        };

        let tag_name = tag.name().as_utf8_str();
        match tag_name.as_ref() {
            "h2" | "h3" => {
                let heading_text = get_text(parser, tag);
                let level = if tag_name == "h2" { 1 } else { 2 };

                // Determine section type from heading text
                if section_type.is_none() {
                    section_type = SectionType::from_text(&heading_text).ok();
                }

                let heading = Heading {
                    level: level as i64,
                    content: vec![t(heading_text)],
                    ..Default::default()
                };
                content.push(Block::Heading(heading));
            }
            "p" => {
                let paragraph = decode_paragraph(parser, tag)?;
                content.push(paragraph);
            }
            "section" => {
                let class = get_class(tag);
                if class.contains("tw") {
                    if let Some(table) = decode_table(parser, tag)? {
                        content.push(Block::Table(table));
                    }
                } else if class.contains("sm") {
                    if let Some(supplement) = decode_supplement(parser, tag)? {
                        content.push(Block::Supplement(supplement));
                    }
                } else {
                    let subsection = decode_section(parser, tag)?;
                    if !subsection.content.is_empty() {
                        content.push(Block::Section(subsection));
                    }
                }
            }
            "figure" => {
                if let Some(figure) = decode_figure(parser, tag)? {
                    content.push(Block::Figure(figure));
                }
            }
            _ => {}
        }
    }

    Ok(Section {
        section_type,
        content,
        ..Default::default()
    })
}

/// Decode block elements
fn decode_blocks(parser: &Parser, tag: &HTMLTag) -> Result<Vec<Block>> {
    let mut blocks = Vec::new();

    for child in tag.children().all(parser) {
        let Some(child_tag) = child.as_tag() else {
            continue;
        };
        let tag_name = child_tag.name().as_utf8_str();

        match tag_name.as_ref() {
            "p" => {
                let paragraph = decode_paragraph(parser, child_tag)?;
                blocks.push(paragraph);
            }
            _ => {}
        }
    }

    Ok(blocks)
}

/// Decode a paragraph
fn decode_paragraph(parser: &Parser, paragraph: &HTMLTag) -> Result<Block> {
    let content = decode_inlines(parser, paragraph)?;
    Ok(p(content))
}

/// Decode inline elements
fn decode_inlines(parser: &Parser, tag: &HTMLTag) -> Result<Vec<Inline>> {
    let mut inlines = Vec::new();

    for child in tag
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(child_tag) = child.as_tag() {
            let tag_name = child_tag.name().as_utf8_str();
            let content = decode_inlines(parser, child_tag)?;

            let inline = match tag_name.as_ref() {
                "em" | "i" => em(content),
                "strong" | "b" => stg(content),
                "u" => u(content),
                "sub" => sub(content),
                "sup" => sup(content),
                "a" => {
                    let href = get_attr(child_tag, "href");
                    if let Some(href_val) = href {
                        if href_val.starts_with('#') {
                            let target = href_val.trim_start_matches('#');

                            // Check if this is a figure or table reference
                            let target_lower = target.to_lowercase();
                            if target_lower.starts_with("fig")
                                || target_lower.contains(".g")
                                || target_lower.starts_with("tab")
                                || target_lower.contains(".t")
                                || target_lower.starts_with("sec")
                                || target_lower.contains(".s")
                            {
                                // This is a figure, table, or supplement reference
                                lnk(content, ["#", target].concat())
                            } else {
                                // This is a citation reference (.ref001)
                                let mut citation = Citation::new(target.to_string());
                                citation.options.content = Some(content);
                                Inline::Citation(citation)
                            }
                        } else {
                            // This is an external link
                            lnk(content, href_val)
                        }
                    } else {
                        lnk(content, "#")
                    }
                }
                _ => {
                    // Recurse into other tags
                    inlines.extend(decode_inlines(parser, child_tag)?);
                    continue;
                }
            };
            inlines.push(inline);
        } else if let Some(text) = child.as_raw()
            && let Some(text_str) = text.try_as_utf8_str()
        {
            let decoded_text = decode_html_entities(text_str);
            if !decoded_text.trim().is_empty() {
                inlines.push(t(decoded_text));
            }
        }
    }

    // Concatenate adjacent text nodes
    Ok(concatenate_text_nodes(inlines))
}

/// Concatenate adjacent Text nodes in inline content
fn concatenate_text_nodes(inlines: Vec<Inline>) -> Vec<Inline> {
    let mut result = Vec::new();
    let mut current_text = String::new();

    for inline in inlines {
        match inline {
            Inline::Text(text) => {
                current_text.push_str(&text.value);
            }
            other => {
                // If we have accumulated text, add it to result
                if !current_text.is_empty() {
                    result.push(t(current_text.clone()));
                    current_text.clear();
                }
                // Add the non-text element
                result.push(other);
            }
        }
    }

    // Don't forget any remaining text
    if !current_text.is_empty() {
        result.push(t(current_text));
    }

    result
}

/// Decode a table section (section with class "tw xbox")
fn decode_table(parser: &Parser, section: &HTMLTag) -> Result<Option<Table>> {
    let id = get_attr(section, "id");
    let mut caption = Vec::new();
    let mut label = None;
    let mut rows = Vec::new();
    let mut notes = Vec::new();

    for child in section
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();
            let tag_class = get_class(tag);

            match tag_name.as_ref() {
                "h4" if tag_class.contains("obj_head") => {
                    let mut caption_inlines = decode_inlines(parser, tag)?;
                    label = extract_and_clean_table_label(&mut caption_inlines);
                    if !caption_inlines.is_empty() {
                        caption.push(p(caption_inlines));
                    }
                }
                "div" if tag_class.contains("caption") => {
                    let blocks = decode_blocks(parser, tag)?;
                    caption.extend(blocks);
                }
                "div" if tag_class.contains("tbl-box") => {
                    if let Some(table_tag) = tag
                        .query_selector(parser, "table")
                        .and_then(|mut nodes| nodes.next())
                        .and_then(|node| node.get(parser))
                        .and_then(|node| node.as_tag())
                    {
                        rows = decode_table_rows(parser, table_tag)?;
                    }
                }
                "div" if tag_class.contains("tw-foot") => {
                    if let Some(mut ps) = tag.query_selector(parser, "p") {
                        while let Some(p) = ps
                            .next()
                            .and_then(|node| node.get(parser))
                            .and_then(|node| node.as_tag())
                        {
                            if let Ok(paragraph) = decode_paragraph(parser, p) {
                                notes.push(paragraph);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if rows.is_empty() {
        return Ok(None);
    }

    let label_automatically = label.is_some().then_some(false);
    let caption = (!caption.is_empty()).then_some(caption);
    let notes = (!notes.is_empty()).then_some(notes);

    Ok(Some(Table {
        id,
        label,
        label_automatically,
        caption,
        rows,
        notes,
        ..Default::default()
    }))
}

/// Extract table label from inlines and clean the prefix from the first text element
///
/// This function looks for "Table X" at the beginning of the first text element,
/// extracts "X" as the label, and removes "Table X." from the text element.
fn extract_and_clean_table_label(inlines: &mut Vec<Inline>) -> Option<String> {
    const PREFIXES: &[&str] = &["Table", "table", "TABLE"];

    if let Some(Inline::Text(text)) = inlines.first_mut() {
        let original_value = text.value.clone();

        for prefix in PREFIXES {
            if let Some(stripped) = original_value.strip_prefix(prefix) {
                let trimmed = stripped.trim_start_matches(['.', ':', ' ']);

                // Extract the number part
                if let Some((number_part, rest)) = trimmed.split_once('.') {
                    let label = number_part.trim().to_string();

                    // Update the text element to remove the "Table X." prefix
                    let cleaned_text = rest.trim_start();
                    if cleaned_text.is_empty() {
                        // If nothing remains, remove this text element entirely
                        inlines.remove(0);
                    } else {
                        text.value = cleaned_text.into();
                    }

                    return Some(label);
                } else if let Some(first_word) = trimmed.split_whitespace().next() {
                    // Handle cases where there's no dot after the number
                    let label = first_word.to_string();

                    // Remove "Table X " from the beginning
                    let to_remove = format!("{} {} ", prefix, first_word);
                    if let Some(cleaned) = original_value.strip_prefix(&to_remove) {
                        text.value = cleaned.into();
                    }

                    return Some(label);
                }
            }
        }
    }

    None
}

/// Decode table rows
fn decode_table_rows(parser: &Parser, table: &HTMLTag) -> Result<Vec<TableRow>> {
    let mut rows = Vec::new();

    // Process thead and tbody
    for child in table
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();

            match tag_name.as_ref() {
                "thead" | "tbody" => {
                    // Determine row type based on section
                    let row_type = match tag_name.as_ref() {
                        "thead" => Some(TableRowType::HeaderRow),
                        _ => None,
                    };

                    // Process tr elements within thead/tbody
                    for tr_child in tag
                        .children()
                        .top()
                        .iter()
                        .flat_map(|handle| handle.get(parser))
                    {
                        if let Some(tr_tag) = tr_child.as_tag()
                            && tr_tag.name().as_utf8_str() == "tr"
                        {
                            let row = decode_table_row(parser, tr_tag, row_type)?;
                            if !row.cells.is_empty() {
                                rows.push(row);
                            }
                        }
                    }
                }
                "tr" => {
                    // Direct tr element
                    let row = decode_table_row(parser, tag, None)?;
                    if !row.cells.is_empty() {
                        rows.push(row);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(rows)
}

/// Decode a table row
fn decode_table_row(
    parser: &Parser,
    tr: &HTMLTag,
    row_type: Option<TableRowType>,
) -> Result<TableRow> {
    let mut cells = Vec::new();
    for child in tr
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();

            if tag_name == "td" || tag_name == "th" {
                let cell_blocks = decode_blocks(parser, tag)?;

                let content = if cell_blocks.is_empty() {
                    let cell_inlines = decode_inlines(parser, tag)?;
                    if cell_inlines.is_empty() {
                        Vec::new()
                    } else {
                        vec![Block::Paragraph(Paragraph {
                            content: cell_inlines,
                            ..Default::default()
                        })]
                    }
                } else {
                    cell_blocks
                };

                // Extract colspan and rowspan attributes
                let row_span = get_attr(tag, "rowspan")
                    .and_then(|attr| attr.parse().ok())
                    .and_then(|span: i64| (span != 1).then_some(span));

                let column_span = get_attr(tag, "colspan")
                    .and_then(|attr| attr.parse().ok())
                    .and_then(|span: i64| (span != 1).then_some(span));

                cells.push(TableCell {
                    content,
                    options: Box::new(TableCellOptions {
                        row_span,
                        column_span,
                        ..Default::default()
                    }),
                    ..Default::default()
                });
            }
        }
    }

    Ok(TableRow {
        cells,
        row_type,
        ..Default::default()
    })
}

/// Decode a figure element
fn decode_figure(parser: &Parser, figure: &HTMLTag) -> Result<Option<Figure>> {
    let id = get_attr(figure, "id");
    let mut caption = None;
    let mut content = Vec::new();

    for child in figure
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();

            match tag_name.as_ref() {
                "h4" if get_class(tag).contains("obj_head") => {
                    // Figure caption from heading
                    let caption_text = get_text(parser, tag);
                    let paragraph = Paragraph {
                        content: vec![t(caption_text)],
                        ..Default::default()
                    };
                    caption = Some(vec![Block::Paragraph(paragraph)]);
                }
                "figcaption" => {
                    // Figure caption
                    let caption_inlines = decode_inlines(parser, tag)?;
                    let paragraph = Paragraph {
                        content: caption_inlines,
                        ..Default::default()
                    };
                    caption = Some(vec![Block::Paragraph(paragraph)]);
                }
                "p" if get_class(tag).contains("img-box") => {
                    // Find image elements
                    if let Some(img_tag) = tag
                        .query_selector(parser, "img")
                        .and_then(|mut nodes| nodes.next())
                        .and_then(|node| node.get(parser))
                        .and_then(|node| node.as_tag())
                    {
                        let src = get_attr(img_tag, "src");

                        if let Some(content_url) = src {
                            let image = ImageObject {
                                content_url,
                                ..Default::default()
                            };
                            content.push(Block::ImageObject(image));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    if content.is_empty() && caption.is_none() {
        return Ok(None);
    }

    Ok(Some(Figure {
        id,
        caption,
        content,
        ..Default::default()
    }))
}

/// Decode a supplement material element (section with class "sm")
fn decode_supplement(parser: &Parser, supplement_section: &HTMLTag) -> Result<Option<Supplement>> {
    let id = get_attr(supplement_section, "id");
    let mut label = None;
    let mut caption = None;
    let mut target = None;

    for child in supplement_section
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();
            let tag_class = get_class(tag);

            match tag_name.as_ref() {
                "div" if tag_class.contains("caption p") => {
                    let mut caption_blocks = Vec::new();

                    // Extract label and caption content from the first span
                    if let Some(span_tag) = tag
                        .query_selector(parser, "span")
                        .and_then(|mut nodes| nodes.next())
                        .and_then(|node| node.get(parser))
                        .and_then(|node| node.as_tag())
                    {
                        let span_text = get_text(parser, span_tag);

                        // Extract label (e.g., "S1 Fig", "S1 Raw Images", "S2 Data")
                        let parts: Vec<&str> = span_text.splitn(2, '.').collect();
                        if parts.len() >= 2 {
                            label = Some(parts[0].trim().to_string());

                            // Create heading with the remaining text
                            let heading_text = parts[1].trim();
                            if !heading_text.is_empty() {
                                caption_blocks.push(h1([t(heading_text)]));
                            }
                        } else {
                            // If no period found, use the whole text as heading
                            if !span_text.trim().is_empty() {
                                caption_blocks.push(h1([t(span_text)]));
                            }
                        }
                    }

                    // Extract format information from any <p> tags
                    if let Some(mut paras) = tag.query_selector(parser, "p") {
                        while let Some(para) = paras
                            .next()
                            .and_then(|node| node.get(parser))
                            .and_then(|node| node.as_tag())
                        {
                            let para_text = get_text(parser, para);
                            if !para_text.trim().is_empty() {
                                caption_blocks.push(p([t(para_text)]));
                            }
                        }
                    }

                    if !caption_blocks.is_empty() {
                        caption = Some(caption_blocks);
                    }
                }
                "div" if tag_class.contains("media p") => {
                    // Extract the target URL from the link
                    if let Some(a_tag) = tag
                        .query_selector(parser, "a")
                        .and_then(|mut nodes| nodes.next())
                        .and_then(|node| node.get(parser))
                        .and_then(|node| node.as_tag())
                    {
                        if let Some(href) = get_attr(a_tag, "href") {
                            target = Some(["https://pmc.ncbi.nlm.nih.gov", &href].concat());
                        }
                    }
                }
                _ => {}
            }
        }
    }

    // Only create supplement if we have essential content
    if label.is_some() || caption.is_some() || target.is_some() {
        Ok(Some(Supplement {
            id,
            label,
            label_automatically: Some(false),
            caption,
            target,
            ..Default::default()
        }))
    } else {
        Ok(None)
    }
}

/// Decode references section
fn decode_references(parser: &Parser, ref_section: &HTMLTag) -> Result<Vec<Reference>> {
    let mut references = Vec::new();

    // Look for ul.ref-list
    for child in ref_section
        .children()
        .top()
        .iter()
        .flat_map(|handle| handle.get(parser))
    {
        if let Some(tag) = child.as_tag() {
            let tag_name = tag.name().as_utf8_str();
            let class = get_class(tag);

            if tag_name.as_ref() == "ul" && class.contains("ref-list") {
                // Process each li as a reference
                for li_child in tag
                    .children()
                    .top()
                    .iter()
                    .flat_map(|handle| handle.get(parser))
                {
                    if let Some(li_tag) = li_child.as_tag()
                        && li_tag.name().as_utf8_str() == "li"
                    {
                        references.push(decode_reference(parser, li_tag));
                    }
                }
            } else if tag_name.as_ref() == "section" {
                // Recurse into sections
                references.extend(decode_references(parser, tag)?);
            }
        }
    }
    Ok(references)
}

/// Decode a single reference
fn decode_reference(parser: &Parser, li_tag: &HTMLTag) -> Reference {
    // Get the reference text from the <cite> element
    let citation_text = if let Some(cite_tag) = li_tag
        .query_selector(parser, "cite")
        .and_then(|mut nodes| nodes.next())
        .and_then(|node| node.get(parser))
        .and_then(|node| node.as_tag())
    {
        get_text(parser, cite_tag)
    } else {
        get_text(parser, li_tag)
    };

    // Parse the structured reference
    let mut reference = text_to_reference(&citation_text);

    // Get the reference id from the li element
    reference.id = get_attr(li_tag, "id");

    reference
}
