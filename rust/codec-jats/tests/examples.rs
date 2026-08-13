use std::{
    collections::{BTreeMap, BTreeSet},
    fs::read_to_string,
    path::PathBuf,
};

use glob::glob;

use stencila_codec::{EncodeOptions, Losses, eyre::Result};

use insta::{assert_json_snapshot, assert_snapshot, assert_yaml_snapshot};
use stencila_codec_jats::{classify, decode, encode};
use stencila_codec_jats_trait::JatsRefType;

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

        // Who wrote and who funded an article, and where they worked, is
        // metadata that a reader relies on as much as the prose
        let original_contributors = contributors_summary(&original)?;
        let roundtrip_contributors = contributors_summary(&jats)?;
        assert_subset(
            "contributors",
            &name,
            &original_contributors.people,
            &roundtrip_contributors.people,
        );
        assert_subset(
            "contributor roles",
            &name,
            &original_contributors.roles,
            &roundtrip_contributors.roles,
        );
        assert_subset(
            "affiliations",
            &name,
            &original_contributors.affiliations,
            &roundtrip_contributors.affiliations,
        );
        assert_subset(
            "funders",
            &name,
            &original_contributors.funders,
            &roundtrip_contributors.funders,
        );
        assert_subset(
            "awards",
            &name,
            &original_contributors.awards,
            &roundtrip_contributors.awards,
        );

        // Every bibliographic field that the schema can represent must survive
        // the round trip, field by field and reference by reference
        let original_references = references_summary(&original)?;
        let roundtrip_references = references_summary(&jats)?;
        assert_eq!(
            original_references.keys().collect::<Vec<_>>(),
            roundtrip_references.keys().collect::<Vec<_>>(),
            "reference ids changed when round-tripping {name}"
        );
        for (reference_id, original_reference) in &original_references {
            let roundtrip_reference = &roundtrip_references[reference_id];
            for (field, original_value) in &original_reference.fields {
                let roundtrip_value = roundtrip_reference.fields.get(field);
                assert_eq!(
                    Some(original_value),
                    roundtrip_value,
                    "{field} of reference {reference_id} changed when round-tripping {name}"
                );
            }
            assert_subset(
                "reference titles",
                &name,
                &original_reference.titles,
                &roundtrip_reference.titles,
            );
            assert_subset(
                "reference identifiers",
                &name,
                &original_reference.identifiers,
                &roundtrip_reference.identifiers,
            );
            assert_subset(
                "reference authors",
                &name,
                &original_reference.authors,
                &roundtrip_reference.authors,
            );
        }

        // Citations and other cross references can only be followed by a reader
        // if they point at something that is still in the document
        let unresolved = unresolved_xref_targets(&jats)?;
        assert!(
            unresolved.is_empty(),
            "unresolved cross reference targets in {name}: {unresolved:?}"
        );

        // Internal navigation belongs in `<xref>`, so no emitted `<ext-link>`
        // should address a fragment of this document
        let local = local_ext_links(&jats)?;
        assert!(
            local.is_empty(),
            "internal targets encoded as ext-link in {name}: {local:?}"
        );

        // A cross reference states the kind of thing it points at, so its
        // `ref-type` has to agree with the element carrying the target id
        let mistyped = mistyped_xrefs(&jats)?;
        assert!(
            mistyped.is_empty(),
            "cross references with the wrong ref-type in {name}: {mistyped:?}"
        );

        // Every cross reference that the source could resolve must still be
        // there; those it could not are dropped rather than left dangling
        assert_subset(
            "cross reference targets",
            &name,
            &resolvable_xref_targets(&original)?,
            &resolvable_xref_targets(&jats)?,
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

/// The people, organizations and awards named in the metadata of an article.
#[derive(Debug, Default)]
struct ContributorsSummary {
    /// `(kind, name, ORCID)` triples for each contributor, where the kind is
    /// either `author` or `editor`
    people: BTreeSet<(String, String, String)>,
    /// The `<role>` of each contributor, with their name
    roles: BTreeSet<(String, String)>,
    /// The institution named by each `<aff>` that a contributor refers to
    affiliations: BTreeSet<String>,
    /// The institutions named as a `<funding-source>`
    funders: BTreeSet<String>,
    /// The `<award-id>` of each award
    awards: BTreeSet<String>,
}

/// Extract the contributor and funding metadata of an article.
///
/// Affiliations are compared by the institution they name rather than by id
/// because the encoder assigns ids of its own; only affiliations that a
/// contributor refers to are compared, because one that nothing refers to is a
/// reported loss rather than something the schema holds.
fn contributors_summary(jats: &str) -> Result<ContributorsSummary> {
    let document = roxmltree::Document::parse_with_options(
        jats,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;

    let text = |node: &roxmltree::Node| {
        let text = node
            .descendants()
            .filter(roxmltree::Node::is_text)
            .filter_map(|node| node.text())
            .collect::<String>();
        normalize_prose(&text)
    };

    // Given names are decoded into separate names, which drops the full stop
    // that a source writes after an initial, so names are compared without one
    let person_name = |node: &roxmltree::Node| text(node).replace('.', "");

    // An organization written as several elements, as JATS allows a department
    // and its institution to be, is decoded into one name, and the punctuation
    // that separated them is normalized away
    let institutions = |node: &roxmltree::Node| {
        let name = node
            .descendants()
            .filter(|node| {
                node.has_tag_name("institution")
                    || (node.has_tag_name("named-content")
                        && node.attribute("content-type") == Some("organisation-division"))
            })
            .map(|node| text(&node).trim_matches([',', '.', ' ']).to_string())
            .filter(|name| !name.is_empty())
            .collect::<Vec<_>>()
            .join(", ");
        (!name.is_empty()).then_some(name)
    };

    let mut summary = ContributorsSummary::default();

    let mut referenced = BTreeSet::new();
    for node in document.descendants().filter(|node| node.is_element()) {
        match node.tag_name().name() {
            "contrib" => {
                // A kind such as `senior_editor` is normalized to `editor`,
                // with the finer distinction kept as a `<role>`
                let contrib_type = node.attribute("contrib-type").unwrap_or("author");
                let contrib_type = if contrib_type.contains("editor") {
                    "editor"
                } else if contrib_type.contains("author") {
                    "author"
                } else {
                    continue;
                }
                .to_string();
                let name = node
                    .children()
                    .find(|child| {
                        child.has_tag_name("name")
                            || child.has_tag_name("string-name")
                            || child.has_tag_name("collab")
                    })
                    .map(|child| person_name(&child))
                    .unwrap_or_default();
                if name.is_empty() {
                    continue;
                }

                let orcid = node
                    .children()
                    .find(|child| {
                        child.has_tag_name("contrib-id")
                            && child.attribute("contrib-id-type") == Some("orcid")
                    })
                    .map(|child| {
                        text(&child)
                            .trim_start_matches("https://orcid.org/")
                            .trim_start_matches("http://orcid.org/")
                            .to_string()
                    })
                    .unwrap_or_default();
                summary.people.insert((contrib_type, name.clone(), orcid));

                if let Some(role) = node.children().find(|child| child.has_tag_name("role")) {
                    summary.roles.insert((name, text(&role)));
                }

                for aff in node.children().filter(|child| child.has_tag_name("aff")) {
                    summary.affiliations.extend(institutions(&aff));
                }
                for rid in node
                    .children()
                    .filter(|child| {
                        child.has_tag_name("xref") && child.attribute("ref-type") == Some("aff")
                    })
                    .filter_map(|child| child.attribute("rid"))
                    .flat_map(str::split_whitespace)
                {
                    referenced.insert(rid.to_string());
                }
            }
            "funding-source" => summary.funders.extend(institutions(&node)),
            "award-id" => {
                let value = text(&node);
                if !value.is_empty() {
                    summary.awards.insert(value);
                }
            }
            _ => {}
        }
    }

    for aff in document.descendants().filter(|node| {
        node.has_tag_name("aff")
            && node
                .attribute("id")
                .is_some_and(|id| referenced.contains(id))
    }) {
        summary.affiliations.extend(institutions(&aff));
    }

    Ok(summary)
}

/// The bibliographic fields of one `<ref>`, independent of how JATS spells them.
#[derive(Debug, Default)]
struct ReferenceSummary {
    /// The title of the referenced work and of any work containing it
    ///
    /// JATS names the title of a whole work `<source>` and that of a part of
    /// one `<article-title>`, a distinction that the schema does not record, so
    /// the titles are compared as a set rather than by which element they were
    /// spelt with.
    titles: BTreeSet<String>,
    /// Single valued fields such as the year and volume
    fields: BTreeMap<&'static str, String>,
    /// `(pub-id-type, value)` pairs, including the electronic location
    identifiers: BTreeSet<(String, String)>,
    /// Author and editor surnames
    authors: BTreeSet<String>,
}

/// Extract the bibliographic fields of every reference, keyed by its ID.
///
/// A `<ref>` may hold a structured citation, a citation rendered as text, or
/// both as alternatives to each other, so the fields of all of them are
/// combined; the point of comparison is what a reader can still recover, not
/// which element it came from.
fn references_summary(jats: &str) -> Result<BTreeMap<String, ReferenceSummary>> {
    let document = roxmltree::Document::parse_with_options(
        jats,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;

    let mut summaries = BTreeMap::new();
    for reference in document
        .descendants()
        .filter(|node| node.has_tag_name("ref"))
    {
        let Some(id) = reference.attribute("id") else {
            continue;
        };

        let mut summary = ReferenceSummary::default();
        for node in reference.descendants().filter(|node| node.is_element()) {
            let text = || {
                let text = node
                    .descendants()
                    .filter(roxmltree::Node::is_text)
                    .filter_map(|node| node.text())
                    .collect::<String>();
                normalize_prose(&text)
            };

            let field = match node.tag_name().name() {
                "article-title" | "chapter-title" | "part-title" | "data-title" | "source" => {
                    let text = text();
                    if !text.is_empty() {
                        summary.titles.insert(text);
                    }
                    continue;
                }
                "year" => "year",
                "volume" => "volume",
                "issue" => "issue",
                "fpage" => "fpage",
                "lpage" => "lpage",
                "page-range" => "page-range",
                "edition" => "edition",
                "publisher-name" => "publisher-name",
                "publisher-loc" => "publisher-loc",
                "surname" => {
                    summary.authors.insert(text());
                    continue;
                }
                "pub-id" => {
                    summary.identifiers.insert((
                        node.attribute("pub-id-type")
                            .unwrap_or_default()
                            .to_string(),
                        text().trim_start_matches("https://doi.org/").to_string(),
                    ));
                    continue;
                }
                "elocation-id" => {
                    summary
                        .identifiers
                        .insert(("elocation-id".to_string(), text()));
                    continue;
                }
                _ => continue,
            };

            let text = text();
            if !text.is_empty() {
                summary.fields.entry(field).or_insert(text);
            }
        }

        summaries.insert(id.to_string(), summary);
    }

    Ok(summaries)
}

/// Collect the id of every element of a document.
fn element_ids<'input>(document: &'input roxmltree::Document) -> BTreeSet<&'input str> {
    document
        .descendants()
        .filter_map(|node| node.attribute("id"))
        .collect()
}

/// Find the targets of cross references, of any kind, that no longer resolve.
fn unresolved_xref_targets(jats: &str) -> Result<BTreeSet<String>> {
    let document = roxmltree::Document::parse(jats)?;
    let ids = element_ids(&document);

    Ok(document
        .descendants()
        .filter(|node| node.has_tag_name("xref"))
        .filter_map(|node| node.attribute("rid"))
        .flat_map(str::split_whitespace)
        .filter(|target| !ids.contains(target))
        .map(String::from)
        .collect())
}

/// Find the targets of cross references that the document is able to resolve.
///
/// Cross references within contributor and funding metadata are excluded
/// because those elements are rebuilt from the people, organizations and grants
/// they belong to, with ids of the encoder's own making, rather than being
/// addressed by an id from the source.
fn resolvable_xref_targets(jats: &str) -> Result<BTreeSet<String>> {
    let document = roxmltree::Document::parse_with_options(
        jats,
        roxmltree::ParsingOptions {
            allow_dtd: true,
            ..Default::default()
        },
    )?;
    let ids = element_ids(&document);

    Ok(document
        .descendants()
        .filter(|node| node.has_tag_name("xref") && !is_contributor_metadata(node))
        .filter_map(|node| node.attribute("rid"))
        .flat_map(str::split_whitespace)
        .filter(|target| ids.contains(target))
        .map(String::from)
        .collect())
}

/// Whether an element belongs to the contributor or funding metadata of an article.
fn is_contributor_metadata(node: &roxmltree::Node) -> bool {
    node.ancestors().any(|ancestor| {
        matches!(
            ancestor.tag_name().name(),
            "aff" | "author-notes" | "contrib" | "contrib-group" | "funding-group"
        )
    })
}

/// Find `<ext-link>` elements that address a fragment of the same document.
fn local_ext_links(jats: &str) -> Result<BTreeSet<String>> {
    let document = roxmltree::Document::parse(jats)?;

    Ok(document
        .descendants()
        .filter(|node| node.has_tag_name("ext-link"))
        .filter_map(|node| node.attribute((XLINK, "href")))
        .filter(|href| href.starts_with('#'))
        .map(String::from)
        .collect())
}

/// Find cross references whose `ref-type` disagrees with the kind of their target.
fn mistyped_xrefs(jats: &str) -> Result<BTreeSet<(String, String, String)>> {
    let document = roxmltree::Document::parse(jats)?;

    let targets = document
        .descendants()
        .filter_map(|node| node.attribute("id").map(|id| (id, node)))
        .collect::<BTreeMap<_, _>>();

    Ok(document
        .descendants()
        .filter(|node| node.has_tag_name("xref"))
        .filter_map(|node| {
            let ref_type = node.attribute("ref-type")?;
            let expected = JatsRefType::from(ref_type).target_elements();
            (!expected.is_empty()).then_some((node, ref_type, expected))
        })
        .flat_map(|(node, ref_type, expected)| {
            node.attribute("rid")
                .into_iter()
                .flat_map(str::split_whitespace)
                .filter_map(|target| {
                    let name = targets.get(target)?.tag_name().name();
                    (!expected.contains(&name))
                        .then(|| (ref_type.to_string(), target.to_string(), name.to_string()))
                })
        })
        .collect())
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
