use itertools::Itertools;
use serde_json;

use hayagriva::{
    BibliographyDriver, BibliographyItem, BibliographyRequest, CitationItem, CitationRequest,
    ElemChild, Entry, Formatted, Library, RenderedBibliography,
    archive::ArchivedStyle,
    citationberg::{FontStyle, FontWeight, Style, TextDecoration, VerticalAlign},
    io::to_yaml_str,
};
use stencila_codec::{
    eyre::{OptionExt, Result, bail},
    stencila_schema::{
        Block, Inline, Reference,
        shortcuts::{em, lnk, p, stg, sub, sup, t, u},
    },
};
use stencila_codec_markdown_trait::to_markdown;
use stencila_codec_text_trait::to_text;

use crate::conversion::reference_to_entry;

/// Encode a set of Stencila [`Reference`] nodes to Hayagriva YAML
pub fn yaml(references: &[&Reference]) -> Result<String> {
    let entries: Vec<Entry> = references
        .iter()
        .map(|reference| reference_to_entry(reference))
        .try_collect()?;

    let library = Library::from_iter(entries);

    let yaml = to_yaml_str(&library)?;

    Ok(yaml)
}

/// Encode a set of Stencila [`Reference`] nodes to Markdown
pub fn markdown(references: &[&Reference], style: Option<&str>) -> Result<String> {
    Ok(to_markdown(&references_to_blocks(references, style)?))
}

/// Encode a set of Stencila [`Reference`] nodes to plain text
pub fn text(references: &[&Reference], style: Option<&str>) -> Result<String> {
    Ok(to_text(&references_to_blocks(references, style)?))
}

/// Encode a set of Stencila [`Reference`] nodes to Stencila JSON
///
/// Mainly used in testing to check rendering of bibliographies in different styles.
pub fn json(references: &[&Reference], style: Option<&str>) -> Result<String> {
    Ok(serde_json::to_string_pretty(&references_to_blocks(
        references, style,
    )?)?)
}

/// Convert a set of Stencila [`Reference`] nodes to Stencila [`Block`] nodes
fn references_to_blocks(references: &[&Reference], style: Option<&str>) -> Result<Vec<Block>> {
    Ok(bibliography_to_blocks(references_to_bibliography(
        references, style,
    )?))
}

/// Render a set of Stencila [`Reference`] nodes to a Hayagriva [`RenderedBibliography`]
fn references_to_bibliography(
    references: &[&Reference],
    style: Option<&str>,
) -> Result<RenderedBibliography> {
    // Check that style is known
    let style = match style.unwrap_or("apa") {
        "chicago" => "chicago-author-date",
        style => style,
    };
    let Some(style) = ArchivedStyle::by_name(style) else {
        bail!("Unrecognized citation style name: {style}")
    };

    // Convert references to Hayagriva entries
    let entries: Vec<Entry> = references
        .iter()
        .map(|reference| reference_to_entry(reference))
        .try_collect()?;

    // Create a library from entries
    let library = Library::from_iter(entries);

    // Load the CSL style from the archive
    let style = match style.get() {
        Style::Independent(style) => style,
        _ => bail!("Only independent citation styles are supported"),
    };

    // Create bibliography driver
    let mut driver = BibliographyDriver::new();
    for entry in library.iter() {
        let items = vec![CitationItem::with_entry(entry)];
        driver.citation(CitationRequest::from_items(items, &style, &[]));
    }

    // Render bibliography
    let result = driver.finish(BibliographyRequest {
        style: &style,
        locale: None,
        locale_files: &[],
    });

    result.bibliography.ok_or_eyre("No bibliography rendered")
}

/// Convert a Hayagriva [`RenderedBibliography`] to a set of Stencila [`Block`] nodes
fn bibliography_to_blocks(bib: RenderedBibliography) -> Vec<Block> {
    bib.items
        .into_iter()
        .map(bibliography_item_to_block)
        .collect()
}

/// Convert a Hayagriva [`BibliographyItem`] to a Stencila [`Block`] node
fn bibliography_item_to_block(item: BibliographyItem) -> Block {
    let inlines = item
        .content
        .0
        .into_iter()
        .flat_map(elem_child_to_inlines)
        .collect_vec();
    p(inlines)
}

/// Convert a Hayagriva [`ElemChild`] to Stencila [`Inline`] nodes
fn elem_child_to_inlines(child: ElemChild) -> Vec<Inline> {
    match child {
        ElemChild::Text(formatted) => vec![formatted_to_inline(formatted)],
        ElemChild::Link { text, url } => vec![lnk([formatted_to_inline(text)], url)],
        ElemChild::Markup(typst) => vec![t(typst)],
        ElemChild::Transparent { cite_idx, format } => vec![formatted_to_inline(Formatted {
            text: cite_idx.to_string(),
            formatting: format,
        })],
        ElemChild::Elem(elem) => elem
            .children
            .0
            .into_iter()
            .flat_map(elem_child_to_inlines)
            .collect(),
    }
}

/// Convert a Hayagriva [`Formatted`] text to a Stencila [`Inline`] node
fn formatted_to_inline(Formatted { text, formatting }: Formatted) -> Inline {
    let mut inline = t(text);

    if matches!(formatting.font_style, FontStyle::Italic) {
        inline = em([inline])
    }

    if matches!(formatting.font_weight, FontWeight::Bold) {
        inline = stg([inline])
    }

    if matches!(formatting.text_decoration, TextDecoration::Underline) {
        inline = u([inline])
    }

    match formatting.vertical_align {
        VerticalAlign::Sup => inline = sup([inline]),
        VerticalAlign::Sub => inline = sub([inline]),
        _ => {}
    }

    inline
}
