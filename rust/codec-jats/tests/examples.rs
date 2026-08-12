use std::{collections::BTreeMap, fs::read_to_string, path::PathBuf};

use glob::glob;

use stencila_codec::{EncodeOptions, Losses, eyre::Result};

use insta::{assert_json_snapshot, assert_snapshot, assert_yaml_snapshot};
use stencila_codec_jats::{classify, decode, encode};

/// Decode each example of a JATS article and create JSON and JATS snapshots
/// including for losses
#[test]
fn examples() -> Result<()> {
    let pattern = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/examples")
        .canonicalize()?
        .to_string_lossy()
        .to_string()
        + "/**/*.jats.xml";

    let mut categories: BTreeMap<String, BTreeMap<String, usize>> = BTreeMap::new();

    for path in glob(&pattern)?.flatten() {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".jats.xml").map(String::from))
            .expect("should have .jats.xml suffix");

        let original = read_to_string(path)?;

        let (article, info) = decode(&original, None)?;

        assert_json_snapshot!(format!("{name}.json"), article);
        assert_yaml_snapshot!(format!("{name}.decode.losses"), info.losses);
        categories.insert(format!("{name} decode"), losses_by_category(&info.losses));

        let (jats, info) = encode(
            &article,
            Some(EncodeOptions {
                compact: Some(false),
                ..Default::default()
            }),
        )?;

        let (compact_jats, ..) = encode(
            &article,
            Some(EncodeOptions {
                compact: Some(true),
                ..Default::default()
            }),
        )?;
        assert_eq!(
            parsed_prose(&compact_jats)?,
            parsed_prose(&jats)?,
            "compact and pretty encodings have different text content for {name}"
        );
        assert_eq!(
            semantic_summary(&compact_jats)?,
            semantic_summary(&jats)?,
            "compact and pretty encodings have different semantics for {name}"
        );

        assert_snapshot!(format!("{name}.jats"), jats);
        assert_yaml_snapshot!(format!("{name}.encode.losses"), info.losses);
        categories.insert(format!("{name} encode"), losses_by_category(&info.losses));
    }

    // A single view of how much of each fixture's loss total is semantic, so
    // that content and metadata regressions are not hidden by the much larger
    // number of source-system details
    assert_yaml_snapshot!("loss-categories", categories);

    Ok(())
}

/// Total the occurrences of losses in each category
fn losses_by_category(losses: &Losses) -> BTreeMap<String, usize> {
    let mut totals = BTreeMap::new();
    for (label, count) in losses.iter() {
        *totals.entry(classify(label).to_string()).or_default() += count;
    }
    totals
}

/// Text and structural features used to compare JATS independently of formatting.
#[derive(Debug, Eq, PartialEq)]
struct SemanticSummary {
    prose: Vec<(String, String)>,
    structures: BTreeMap<&'static str, usize>,
}

/// Extract the exact parsed text for each logical prose block.
fn parsed_prose(jats: &str) -> Result<Vec<(String, String)>> {
    let document = roxmltree::Document::parse(jats)?;
    Ok(prose_blocks(&document))
}

/// Extract the exact text for each logical prose block of a parsed document.
fn prose_blocks(document: &roxmltree::Document) -> Vec<(String, String)> {
    document
        .descendants()
        .filter(|node| node.is_element() && is_logical_prose_block(node.tag_name().name()))
        .map(|node| {
            let text = node
                .descendants()
                .filter_map(|descendant| descendant.text())
                .collect::<String>();
            (node.tag_name().name().to_string(), text)
        })
        .collect()
}

/// Extract normalized prose and counts of content-bearing JATS structures.
fn semantic_summary(jats: &str) -> Result<SemanticSummary> {
    let document = roxmltree::Document::parse(jats)?;
    let prose = prose_blocks(&document)
        .into_iter()
        .map(|(element, text)| (element, normalize_prose(&text)))
        .collect();
    let structures = [
        "abstract",
        "sub-article",
        "sec",
        "p",
        "list",
        "fn",
        "fig",
        "table-wrap",
        "supplementary-material",
        "disp-formula",
        "inline-formula",
    ]
    .into_iter()
    .map(|element| {
        let count = document
            .descendants()
            .filter(|node| node.has_tag_name(element))
            .count();
        (element, count)
    })
    .collect();

    Ok(SemanticSummary { prose, structures })
}

/// Normalize layout whitespace without adding boundaries absent from parsed text.
fn normalize_prose(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Return whether an element is a logical unit of prose in JATS.
fn is_logical_prose_block(name: &str) -> bool {
    matches!(
        name,
        "article-title"
            | "subtitle"
            | "title"
            | "p"
            | "label"
            | "kwd"
            | "license-p"
            | "td"
            | "th"
            | "mixed-citation"
    )
}

#[test]
fn semantic_summary_detects_changed_word_boundaries_and_descendants() -> Result<()> {
    let intact = semantic_summary(
        "<article><body><p>text <italic>inline</italic> text</p></body></article>",
    )?;
    let joined = semantic_summary(
        "<article><body><p>text<italic>inline</italic> text</p></body></article>",
    )?;
    let dropped = semantic_summary("<article><body><p>text  text</p></body></article>")?;

    assert_ne!(intact, joined);
    assert_ne!(intact, dropped);
    Ok(())
}
