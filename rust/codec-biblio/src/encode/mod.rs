use std::{collections::HashMap, sync::LazyLock};

use itertools::Itertools;
use serde_json;

use hayagriva::{
    BibliographyDriver, BibliographyRequest, CitationItem, CitationRequest, CitePurpose, ElemChild,
    ElemChildren, Entry, Formatted, Library,
    archive::locales,
    citationberg::{
        FontStyle, FontWeight, Locale, SortKey, TextDecoration, VerticalAlign,
        taxonomy::{NumberVariable, Variable},
    },
    io::to_yaml_str,
};

/// CSL locales loaded once per process (expensive to load)
static LOCALES: LazyLock<Vec<Locale>> = LazyLock::new(locales);
use stencila_codec::{
    eyre::{OptionExt, Result, bail},
    stencila_schema::{
        Block, CitationGroup, CitationMode, Inline, Reference,
        shortcuts::{em, lnk, p, stg, sub, sup, t, u},
    },
};
use stencila_codec_markdown_trait::to_markdown;
use stencila_codec_text_trait::to_text;

use crate::{
    conversion::{reference_key, reference_to_entry},
    encode::styles::get_style,
};

mod styles;

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
pub async fn markdown(references: &[&Reference], style: Option<&str>) -> Result<String> {
    Ok(to_markdown(&render_references(references, style).await?))
}

/// Encode a set of Stencila [`Reference`] nodes to plain text
pub async fn text(references: &[&Reference], style: Option<&str>) -> Result<String> {
    Ok(to_text(&render_references(references, style).await?))
}

/// Encode a set of Stencila [`Reference`] nodes to Stencila JSON
///
/// Mainly used in testing to check rendering of bibliographies in different styles.
pub async fn json(references: &[&Reference], style: Option<&str>) -> Result<String> {
    Ok(serde_json::to_string_pretty(
        &render_references(references, style).await?,
    )?)
}

/// Render a set of Stencila [`Reference`] nodes to Stencila `Paragraphs`
async fn render_references(references: &[&Reference], style: Option<&str>) -> Result<Vec<Block>> {
    let style_name = style.unwrap_or("ama");

    // Load the style
    let style = get_style(style_name).await?;

    // Convert references to Hayagriva entries
    let entries: Vec<Entry> = references
        .iter()
        .map(|reference| reference_to_entry(reference))
        .try_collect()?;

    // Create a library from entries
    let library = Library::from_iter(entries);

    // Create bibliography driver
    let mut driver = BibliographyDriver::new();
    for entry in library.iter() {
        let items = vec![CitationItem::with_entry(entry)];
        driver.citation(CitationRequest::from_items(items, &style, &LOCALES));
    }

    // Render bibliography
    let result = driver.finish(BibliographyRequest {
        style: &style,
        locale: None,
        locale_files: &LOCALES,
    });

    let bibliography = result.bibliography.ok_or_eyre("No references rendered")?;
    let blocks = bibliography
        .items
        .into_iter()
        .map(|reference| p(elem_children_to_inlines(reference.content)))
        .collect_vec();

    Ok(blocks)
}

/// Render a set of citations and their references
#[tracing::instrument(skip(citations))]
pub async fn render_citations(
    citations: Vec<&CitationGroup>,
    style: Option<&str>,
) -> Result<(Vec<Vec<Inline>>, Vec<Reference>)> {
    let style_name = style.unwrap_or("ama");

    tracing::trace!("Rendering citations");

    // Load the style
    let style = get_style(style_name).await?;

    // Get the set of unique references from the citations
    let mut references = HashMap::new();
    for citation_group in &citations {
        for citation in &citation_group.items {
            if let Some(reference) = &citation.options.cites
                && let Some(key) = reference_key(reference)
                && !references.contains_key(key)
            {
                references.insert(key, reference.clone());
            }
        }
    }

    // Convert references to Hayagriva entries
    let entries: Vec<Entry> = references.values().map(reference_to_entry).try_collect()?;

    // Create a library from entries
    let library = Library::from_iter(entries);

    // Create bibliography driver
    let mut driver = BibliographyDriver::new();

    // Create a "citation request" for each citation group
    for citation_group in citations {
        let citation_items = citation_group
            .items
            .iter()
            .filter_map(|citation| {
                let key = citation.options.cites.as_ref().and_then(reference_key)?;
                let entry = library.get(key)?;

                let mut citation_item = CitationItem::with_entry(entry);
                match citation.citation_mode {
                    Some(CitationMode::Narrative) => {
                        citation_item = citation_item.kind(CitePurpose::Prose)
                    }
                    Some(CitationMode::NarrativeAuthor) => {
                        citation_item = citation_item.kind(CitePurpose::Author)
                    }
                    _ => {}
                };

                Some(citation_item)
            })
            .collect_vec();

        let citation_request = CitationRequest::from_items(citation_items, &style, &LOCALES);

        driver.citation(citation_request);
    }

    // Render bibliography
    let result = driver.finish(BibliographyRequest {
        style: &style,
        locale: None,
        locale_files: &LOCALES,
    });

    // Convert rendered citations to inlines
    let citation_contents = result
        .citations
        .into_iter()
        .map(|citation| elem_children_to_inlines(citation.citation))
        .collect();

    // Determine if citations are ordered by appearance
    let appearance_order = style
        .bibliography
        .map(|bibliography| match bibliography.sort {
            Some(sort) => {
                sort.keys.len() == 1
                    && matches!(
                        sort.keys.first(),
                        Some(SortKey::Variable {
                            variable: Variable::Number(NumberVariable::CitationNumber),
                            ..
                        })
                    )
            }
            None => true,
        })
        .unwrap_or(true);

    // Add rendered inlines to the content of the references. References will be
    // ordered according to the citation style (appearance order or alphabetic)
    let mut ordered_references = Vec::new();
    if let Some(bibliography) = result.bibliography {
        for (index, item) in bibliography.items.into_iter().enumerate() {
            let Some(mut reference) = references.remove(item.key.as_str()) else {
                continue;
            };

            let mut content = Vec::with_capacity(1);

            // If references are in appearance order then add a numeric prefix (Hayagriva does not do that)
            if appearance_order {
                content.push(t(format!("{}. ", index + 1)));
            }

            if let Some(text) = &reference.options.text {
                // If the reference has `text` (and therefore probably lacking
                // title and other details then use the text)
                content.push(t(text));
            } else {
                // Otherwise use Hayagriva's rendered content
                content.extend(elem_children_to_inlines(item.content));
            };

            reference.options.content = Some(content);

            ordered_references.push(reference);
        }
    } else {
        // For some CSL files (e.g. agora) no bibliography or citation content is generated.
        // This may be related to how we are using those files but for now just advise the user
        // to use another style
        bail!(
            "No references rendered for citation style `{style_name}`, please try using a different style"
        )
    }

    Ok((citation_contents, ordered_references))
}

/// Convert a Hayagriva [`ElemChildren`] to Stencila [`Inline`] nodes
fn elem_children_to_inlines(children: ElemChildren) -> Vec<Inline> {
    children.0.into_iter().flat_map(elem_child_to_inlines).fold(
        Vec::new(),
        |mut inlines, inline| {
            // Merge adjacent text nodes
            match (inlines.last_mut(), &inline) {
                (Some(Inline::Text(last)), Inline::Text(curr)) => {
                    last.value.push_str(&curr.value);
                }
                _ => inlines.push(inline),
            };
            inlines
        },
    )
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
        VerticalAlign::Baseline | VerticalAlign::None => {}
    }

    inline
}
