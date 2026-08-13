use std::{
    collections::{BTreeMap, BTreeSet},
    fs::read_to_string,
    path::PathBuf,
};

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

        // Content structures recovered by the decoder must survive another
        // decode and encode cycle rather than degrading each time. Only the
        // structure counts are compared: prose within some of them is still
        // altered by media, citation and inline fidelity gaps addressed later.
        let (article, ..) = decode(&jats, None)?;
        let (jats_again, ..) = encode(
            &article,
            Some(EncodeOptions {
                compact: Some(false),
                ..Default::default()
            }),
        )?;
        assert_eq!(
            semantic_summary(&jats_again)?.structures,
            semantic_summary(&jats)?.structures,
            "another decode and encode cycle changed the structures of {name}"
        );

        // Publication metadata that the schema can represent must survive the
        // round trip. Subsets are compared because encoding also recovers
        // metadata that is only implicit in the original, such as a license URL
        // that the source states only within its license prose.
        let original_metadata = metadata_summary(&original)?;
        let roundtrip_metadata = metadata_summary(&jats)?;
        assert_subset(
            "article identifiers",
            &name,
            &original_metadata.article_identifiers,
            &roundtrip_metadata.article_identifiers,
        );
        assert_subset(
            "journal identifiers",
            &name,
            &original_metadata.journal_identifiers,
            &roundtrip_metadata.journal_identifiers,
        );
        assert_subset(
            "dates",
            &name,
            &original_metadata.dates,
            &roundtrip_metadata.dates,
        );
        assert_subset(
            "licenses",
            &name,
            &original_metadata.licenses,
            &roundtrip_metadata.licenses,
        );
        assert_subset(
            "portable resources",
            &name,
            &original_metadata.resources,
            &roundtrip_metadata.resources,
        );

        // Links into a publishing system's own file system are deliberately
        // filtered out; see `decode_self_uri`
        let unfiltered = roundtrip_metadata
            .resources
            .intersection(&original_metadata.internal_resources)
            .collect::<Vec<_>>();
        assert!(
            unfiltered.is_empty(),
            "non-portable resources retained for {name}: {unfiltered:?}"
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

/// Assert that none of the metadata of an original fixture was dropped.
fn assert_subset<T: Ord + std::fmt::Debug>(
    what: &str,
    name: &str,
    original: &BTreeSet<T>,
    roundtrip: &BTreeSet<T>,
) {
    let missing = original.difference(roundtrip).collect::<Vec<_>>();
    assert!(
        missing.is_empty(),
        "{what} dropped when round-tripping {name}: {missing:?}"
    );
}

/// Publication metadata of an article, independent of how JATS spells it.
#[derive(Debug, Default)]
struct MetadataSummary {
    /// `(pub-id-type, value)` pairs from `<article-id>` and `<elocation-id>`
    article_identifiers: BTreeSet<(String, String)>,
    /// `(kind, value)` pairs from `<journal-id>`, `<issn>` and `<journal-title>`
    journal_identifiers: BTreeSet<(String, String)>,
    /// `(role, ISO 8601 date)` pairs from every kind of date element
    dates: BTreeSet<(String, String)>,
    /// License URLs
    licenses: BTreeSet<String>,
    /// Links to representations of the article that are usable elsewhere
    resources: BTreeSet<String>,
    /// Links to representations of the article within a publishing system
    internal_resources: BTreeSet<String>,
}

const XLINK: &str = "http://www.w3.org/1999/xlink";

/// Extract the publication metadata of the `<front>` of the outermost `<article>`.
fn metadata_summary(jats: &str) -> Result<MetadataSummary> {
    let document = roxmltree::Document::parse_with_options(
        jats,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;

    let mut summary = MetadataSummary::default();

    let Some(front) = document
        .descendants()
        .find(|node| node.has_tag_name("article"))
        .and_then(|article| {
            article
                .children()
                .find(|child| child.has_tag_name("front") || child.has_tag_name("front-stub"))
        })
    else {
        return Ok(summary);
    };

    for node in front.descendants().filter(|node| node.is_element()) {
        let text = || {
            let text = node
                .descendants()
                .filter(roxmltree::Node::is_text)
                .filter_map(|node| node.text())
                .collect::<String>();
            text.split_whitespace().collect::<Vec<_>>().join(" ")
        };

        match node.tag_name().name() {
            "article-id" => {
                summary.article_identifiers.insert((
                    node.attribute("pub-id-type")
                        .unwrap_or_default()
                        .to_string(),
                    text(),
                ));
            }
            "elocation-id" => {
                summary
                    .article_identifiers
                    .insert(("elocation-id".to_string(), text()));
            }
            "journal-id" => {
                summary.journal_identifiers.insert((
                    node.attribute("journal-id-type")
                        .unwrap_or_default()
                        .to_string(),
                    text(),
                ));
            }
            "issn" => {
                summary.journal_identifiers.insert((
                    ["issn-", &publication_format(&node).unwrap_or_default()].concat(),
                    text(),
                ));
            }
            "pub-date" | "date" => {
                if let Some(date) = iso_date(&node) {
                    summary.dates.insert((date_role(&node), date));
                }
            }
            "license" => {
                if let Some(url) = node.attribute((XLINK, "href")) {
                    summary.licenses.insert(url.to_string());
                }
            }
            "license_ref" => {
                summary.licenses.insert(text());
            }
            "self-uri" => {
                if let Some(url) = node.attribute((XLINK, "href")) {
                    if url.starts_with("file:/") || url.starts_with('/') {
                        summary.internal_resources.insert(url.to_string());
                    } else {
                        summary.resources.insert(url.to_string());
                    }
                }
            }
            _ => {}
        }
    }

    Ok(summary)
}

/// Normalize the two ways that JATS spells a publication format.
fn publication_format(node: &roxmltree::Node) -> Option<String> {
    let format = node
        .attribute("pub-type")
        .or_else(|| node.attribute("publication-format"))?;

    Some(
        match format {
            "electronic" => "epub",
            "print" => "ppub",
            format => format,
        }
        .to_string(),
    )
}

/// The kind of date that a date element represents.
fn date_role(node: &roxmltree::Node) -> String {
    node.attribute("pub-type")
        .or_else(|| node.attribute("date-type"))
        .map(String::from)
        .or_else(|| publication_format(node))
        .or_else(|| {
            node.parent()
                .and_then(|parent| parent.attribute("event-type"))
                .map(String::from)
        })
        .unwrap_or_default()
}

/// The date that a date element represents, zero padded so that dates written
/// differently still compare equal.
fn iso_date(node: &roxmltree::Node) -> Option<String> {
    let part = |name: &str| {
        node.children()
            .find(|child| child.has_tag_name(name))
            .and_then(|child| child.text())
            .map(str::trim)
            .map(String::from)
    };

    let year = part("year")?;
    if year.len() != 4 {
        return None;
    }

    let mut date = year;
    let Some(month) = part("month").and_then(|month| month.parse::<u32>().ok()) else {
        return Some(date);
    };
    date.push_str(&format!("-{month:02}"));

    if let Some(day) = part("day").and_then(|day| day.parse::<u32>().ok()) {
        date.push_str(&format!("-{day:02}"));
    }

    Some(date)
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
