//! Snapshot tests of decoding the TEI fixtures in `tests/examples`, and a
//! comparison of the structure of each GROBID fixture with that of the JATS of
//! the same article

use std::{fs::read_to_string, path::PathBuf};

use glob::glob;
use insta::{assert_json_snapshot, assert_yaml_snapshot};
use serde_json::Value;

use stencila_codec::{eyre::Result, stencila_schema::Node};
use stencila_codec_tei::decode;

/// The directory of the fixtures
fn examples() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/examples")
}

/// Decode each TEI fixture and snapshot the decoded article and its losses,
/// and a summary of its structure
#[test]
fn examples_snapshots() -> Result<()> {
    let pattern = examples().to_string_lossy().to_string() + "/*.tei.xml";

    let mut count = 0;
    for path in glob(&pattern)?.flatten() {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".tei.xml").map(String::from))
            .unwrap_or_default();

        let tei = read_to_string(&path)?;
        let (article, info) = decode(&tei, None)?;

        assert_json_snapshot!(format!("{name}.json"), article);
        assert_yaml_snapshot!(format!("{name}.losses"), info.losses);
        assert_yaml_snapshot!(format!("{name}.summary"), Summary::of(&article)?);

        count += 1;
    }
    assert!(count >= 6, "expected at least six fixtures, found {count}");

    Ok(())
}

/// Compare the structure of each GROBID fixture with that of the JATS of the
/// same article
///
/// The comparison is snapshotted, rather than asserted to be equal, because
/// GROBID is extracting structure from a PDF and does not always find what
/// the publisher's JATS states (see the README of the fixtures for the
/// differences and their causes). What is asserted is the agreement that is
/// expected of GROBID for these articles: the same number of references, the
/// same number of authors give or take one, and, where the JATS has more than
/// a couple of section headings, at least 85% of them in the TEI.
#[test]
fn compare_with_jats() -> Result<()> {
    let pattern = examples().to_string_lossy().to_string() + "/*.jats.xml";

    let mut count = 0;
    for path in glob(&pattern)?.flatten() {
        let name = path
            .file_name()
            .map(|name| name.to_string_lossy())
            .and_then(|name| name.strip_suffix(".jats.xml").map(String::from))
            .unwrap_or_default();

        let jats = read_to_string(&path)?;
        let tei = read_to_string(examples().join(format!("{name}.tei.xml")))?;

        let (jats, ..) = stencila_codec_jats::decode(&jats, None)?;
        let (tei, ..) = decode(&tei, None)?;

        let jats = Summary::of(&jats)?;
        let tei = Summary::of(&tei)?;

        let comparison = Comparison::new(&tei, &jats);
        assert_yaml_snapshot!(format!("{name}.comparison"), comparison);

        assert_eq!(
            tei.references, jats.references,
            "different numbers of references for {name}"
        );
        assert!(
            tei.authors.len().abs_diff(jats.authors.len()) <= 1,
            "different numbers of authors for {name}"
        );
        if jats.headings.len() > 2 {
            let found = comparison.headings_in_both.len() as f64;
            let total = found + comparison.headings_only_in_jats.len() as f64;
            assert!(
                found / total >= 0.85,
                "only {found} of {total} section headings found for {name}"
            );
        }

        count += 1;
    }
    assert!(
        count >= 4,
        "expected at least four JATS counterparts, found {count}"
    );

    Ok(())
}

/// A summary of the structure of a decoded article
#[derive(serde::Serialize)]
struct Summary {
    title: String,
    authors: Vec<String>,
    abstract_paragraphs: usize,
    /// Headings, indented by their level
    headings: Vec<String>,
    figures: usize,
    figure_labels: Vec<String>,
    tables: usize,
    table_labels: Vec<String>,
    math_blocks: usize,
    citations: usize,
    references: usize,
    references_with_doi: usize,
}

impl Summary {
    /// Summarize an article from its JSON so that articles decoded from JATS
    /// and from TEI are summarized in exactly the same way
    fn of(node: &Node) -> Result<Self> {
        let json = serde_json::to_value(node)?;

        let title = json.get("title").map(text).unwrap_or_default();

        let authors = json
            .get("authors")
            .and_then(Value::as_array)
            .map(|authors| {
                authors
                    .iter()
                    .map(|author| {
                        // An author with roles is an `AuthorRole` wrapping the author
                        let author = author.get("author").unwrap_or(author);
                        let name = [
                            strings(author.get("givenNames")),
                            strings(author.get("familyNames")),
                        ]
                        .concat()
                        .join(" ");
                        let name = if name.is_empty() {
                            author
                                .get("name")
                                .and_then(Value::as_str)
                                .unwrap_or_default()
                                .to_string()
                        } else {
                            name
                        };
                        let affiliations = author
                            .get("affiliations")
                            .and_then(Value::as_array)
                            .map(Vec::len)
                            .unwrap_or_default();
                        format!("{name} ({affiliations} affiliations)")
                    })
                    // JATS contributors with CRediT roles are an `AuthorRole`
                    // for each role, so are deduplicated to count people
                    .fold(Vec::new(), |mut authors, author| {
                        if !authors.contains(&author) {
                            authors.push(author);
                        }
                        authors
                    })
            })
            .unwrap_or_default();

        let abstract_paragraphs = json
            .get("abstract")
            .map(|value| count(value, "Paragraph"))
            .unwrap_or_default();

        let content = json.get("content").cloned().unwrap_or_default();

        let mut headings = Vec::new();
        section_headings(&content, &mut headings);

        let labels = |node_type: &str| {
            let mut labels = Vec::new();
            visit(&content, &mut |node| {
                if node.get("type").and_then(Value::as_str) == Some(node_type) {
                    labels.push(
                        node.get("label")
                            .and_then(Value::as_str)
                            .unwrap_or("-")
                            .to_string(),
                    );
                }
            });
            labels
        };
        let figure_labels = labels("Figure");
        let table_labels = labels("Table");

        let references = json.get("references").and_then(Value::as_array);

        Ok(Self {
            title,
            authors,
            abstract_paragraphs,
            headings,
            figures: figure_labels.len(),
            figure_labels,
            tables: table_labels.len(),
            table_labels,
            math_blocks: count(&content, "MathBlock"),
            citations: count(&content, "Citation"),
            references: references.map(Vec::len).unwrap_or_default(),
            references_with_doi: references
                .map(|references| {
                    references
                        .iter()
                        .filter(|reference| reference.get("doi").is_some())
                        .count()
                })
                .unwrap_or_default(),
        })
    }
}

/// A comparison of the summaries of the TEI and JATS decodes of an article
#[derive(serde::Serialize)]
struct Comparison {
    /// Headings in the TEI, at any level, that are not headings in the JATS
    headings_only_in_tei: Vec<String>,
    /// Headings in the JATS, at any level, that are not headings in the TEI
    headings_only_in_jats: Vec<String>,
    /// Section headings in both, in the order they are in the TEI, with the
    /// level in each (TEI level, JATS level)
    headings_in_both: Vec<String>,
    /// Counts in the TEI and the JATS
    figures: (usize, usize),
    tables: (usize, usize),
    math_blocks: (usize, usize),
    citations: (usize, usize),
    references: (usize, usize),
    authors: (usize, usize),
}

impl Comparison {
    fn new(tei: &Summary, jats: &Summary) -> Self {
        // Text extracted from a PDF differs from the publisher's in spacing,
        // hyphenation and punctuation (e.g. "G 2 /M" for "G2/M") so only
        // letters and digits are compared
        let normalize = |heading: &str| {
            heading
                .chars()
                .filter(|char| char.is_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        };
        let level = |heading: &String| heading.len() - heading.trim_start().len();

        let find = |heading: &String, others: &[String]| {
            others
                .iter()
                .find(|other| normalize(other) == normalize(heading))
                .cloned()
        };

        let mut headings_in_both = Vec::new();
        let mut headings_only_in_tei = Vec::new();
        for heading in &tei.headings {
            match find(heading, &jats.headings) {
                Some(other) => headings_in_both.push(format!(
                    "{} ({}, {})",
                    heading.trim(),
                    level(heading) / 2 + 1,
                    level(&other) / 2 + 1
                )),
                None => headings_only_in_tei.push(heading.trim().to_string()),
            }
        }
        let headings_only_in_jats = jats
            .headings
            .iter()
            .filter(|heading| find(heading, &tei.headings).is_none())
            .map(|heading| heading.trim().to_string())
            .collect();

        Self {
            headings_only_in_tei,
            headings_only_in_jats,
            headings_in_both,
            figures: (tei.figures, jats.figures),
            tables: (tei.tables, jats.tables),
            math_blocks: (tei.math_blocks, jats.math_blocks),
            citations: (tei.citations, jats.citations),
            references: (tei.references, jats.references),
            authors: (tei.authors.len(), jats.authors.len()),
        }
    }
}

/// Collect the headings of the sections within some blocks, indented by level
///
/// Only headings in sections, or at the top level, are collected so that
/// headings within captions and supplementary material are not.
fn section_headings(blocks: &Value, headings: &mut Vec<String>) {
    for block in blocks.as_array().into_iter().flatten() {
        match block.get("type").and_then(Value::as_str) {
            Some("Heading") => {
                let level = block.get("level").and_then(Value::as_i64).unwrap_or(1);
                let indent = "  ".repeat((level.max(1) - 1) as usize);
                headings.push(format!("{indent}{}", text(&block["content"])));
            }
            Some("Section") => section_headings(&block["content"], headings),
            _ => {}
        }
    }
}

/// Visit every object within a JSON value
fn visit(value: &Value, visitor: &mut impl FnMut(&Value)) {
    match value {
        Value::Object(object) => {
            visitor(value);
            for value in object.values() {
                visit(value, visitor);
            }
        }
        Value::Array(array) => {
            for value in array {
                visit(value, visitor);
            }
        }
        _ => {}
    }
}

/// Count the nodes of a type within a JSON value
fn count(value: &Value, node_type: &str) -> usize {
    let mut count = 0;
    visit(value, &mut |node| {
        if node.get("type").and_then(Value::as_str) == Some(node_type) {
            count += 1;
        }
    });
    count
}

/// Get the text within a JSON value
fn text(value: &Value) -> String {
    let mut text = String::new();
    visit(value, &mut |node| {
        // The value of text is a cord, which is serialized as an object
        if node.get("type").and_then(Value::as_str) == Some("Text")
            && let Some(value) = node.get("value")
            && let Some(value) = value
                .as_str()
                .or_else(|| value.get("string").and_then(Value::as_str))
        {
            text.push_str(value);
        }
    });
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Get a list of strings from a JSON value
fn strings(value: Option<&Value>) -> Vec<String> {
    value
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(String::from)
                .collect()
        })
        .unwrap_or_default()
}
