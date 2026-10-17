//! Readable locations derived from immutable document snapshots

use std::collections::{BTreeMap, HashMap};

use eyre::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use stencila_node_compare::{Comparison, NodeRef, Side};
use stencila_node_path::{NodePath, NodeSlot};

use crate::{Snapshot, elide, path};

/// Optional presentation metadata accompanying an unchanged comparison artifact
///
/// Labels describe the supplied snapshots, not persistent node identities. Consumers
/// should use the original node paths for navigation and treat labels as display text.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub locations: LocationLabels,
}

/// Readable locations indexed by side and canonical path
///
/// Keys use `$` for the root and `$/content/0` for a child. Each occurrence is
/// described once per side, even when several differences refer to it. Labels may
/// change as presentation improves; the paths retain their original meaning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocationLabels {
    pub left: BTreeMap<String, String>,
    pub right: BTreeMap<String, String>,
}

impl ReportMetadata {
    /// Describe every occurrence in a comparison using its original snapshots
    pub fn new(comparison: &Comparison, left: Snapshot, right: Snapshot) -> Result<Self> {
        let correspondences = comparison.alignment().correspondences();
        Ok(Self {
            locations: LocationLabels::describe(
                left,
                right,
                correspondences
                    .iter()
                    .flat_map(|entry| entry.nodes(Side::Left)),
                correspondences
                    .iter()
                    .flat_map(|entry| entry.nodes(Side::Right)),
            )?,
        })
    }
}

impl LocationLabels {
    /// Describe only the given occurrences on each side
    pub(crate) fn describe<'a>(
        left: Snapshot,
        right: Snapshot,
        left_nodes: impl Iterator<Item = &'a NodeRef>,
        right_nodes: impl Iterator<Item = &'a NodeRef>,
    ) -> Result<Self> {
        Ok(Self {
            left: Locator::new(left)?.labels(left_nodes),
            right: Locator::new(right)?.labels(right_nodes),
        })
    }
}

/// Describes paths through one serialized snapshot
///
/// Each array is described once, when a path first passes through it, so describing
/// every occurrence is linear in the size of the document rather than quadratic in
/// the length of its longest array.
struct Locator {
    /// The snapshot, serialized once rather than cloning subtrees for every prefix of
    /// every path. This also covers metadata nodes outside the document body.
    document: Value,

    /// The items of each array described so far, keyed by the path of the array
    arrays: HashMap<NodePath, Vec<Item>>,
}

/// The description of one item of an array
struct Item {
    /// The flat headings whose scope the item is in, outermost first
    scope: Vec<String>,

    /// The item's own label
    label: String,
}

impl Locator {
    fn new(snapshot: Snapshot) -> Result<Self> {
        Ok(Self {
            document: serde_json::to_value(snapshot.node)?,
            arrays: HashMap::new(),
        })
    }

    fn labels<'a>(&mut self, nodes: impl Iterator<Item = &'a NodeRef>) -> BTreeMap<String, String> {
        nodes
            .map(|node| (path(&node.path), self.describe(&node.path)))
            .collect()
    }

    /// Describe a path, including flat heading scopes
    fn describe(&mut self, node_path: &NodePath) -> String {
        let Self { document, arrays } = self;
        let mut node = &*document;
        let mut parts = Vec::<String>::new();
        let mut property = None;
        let mut prefix = NodePath::new();

        for (position, slot) in node_path.iter().enumerate() {
            match slot {
                NodeSlot::Property(name) => {
                    let name = name.to_string();
                    let Some(child) = node.get(&name) else {
                        return path(node_path);
                    };
                    if !matches!(name.as_str(), "content" | "items" | "rows" | "cells")
                        && !matches!(node_path.get(position + 1), Some(NodeSlot::Index(_)))
                    {
                        push(&mut parts, property_label(&name).to_string());
                    }
                    property = Some(name);
                    node = child;
                }
                NodeSlot::Index(index) => {
                    let Some(siblings) = node.as_array() else {
                        return path(node_path);
                    };
                    let Some(child) = siblings.get(*index) else {
                        return path(node_path);
                    };
                    let items = arrays
                        .entry(prefix.clone())
                        .or_insert_with(|| items(siblings, property.as_deref()));
                    let Some(item) = items.get(*index) else {
                        return path(node_path);
                    };
                    for label in &item.scope {
                        push(&mut parts, label.clone());
                    }
                    // Preserve a meaningful region even when it directly holds blocks.
                    if let Some(name @ ("abstract" | "caption" | "title")) = property.as_deref() {
                        push(&mut parts, property_label(name).to_string());
                    }
                    push(&mut parts, item.label.clone());
                    node = child;
                    property = None;
                }
            }
            prefix.push_back(slot.clone());
        }

        if parts.is_empty() {
            let node_type = kind(document);
            if node_type.is_empty() {
                "Document".to_string()
            } else {
                node_type.to_string()
            }
        } else {
            parts.join(" › ")
        }
    }
}

/// Describe the items of an array, which a property holds, in one pass
fn items(siblings: &[Value], property: Option<&str>) -> Vec<Item> {
    // Some codecs emit flat headings rather than Section nodes. Follow their levels so
    // a later peer heading closes its scope, and number other nodes within the scope
    // of the nearest heading, as a reader would count them.
    let flat = property == Some("content");
    let mut headings = Vec::<(u64, String)>::new();
    let mut ordinals = HashMap::<&str, usize>::new();

    let mut items = Vec::with_capacity(siblings.len());
    for (index, sibling) in siblings.iter().enumerate() {
        let node_type = kind(sibling);
        let heading = flat && node_type == "Heading";
        if heading {
            // The heading replaces prior headings at its own level or deeper
            let level = sibling.get("level").and_then(Value::as_u64).unwrap_or(1);
            while headings
                .last()
                .is_some_and(|(previous, _)| *previous >= level)
            {
                headings.pop();
            }
            ordinals.retain(|node_type, _| *node_type == "Heading");
        }
        let ordinal = ordinals.entry(node_type).or_default();
        *ordinal += 1;
        let ordinal = *ordinal;

        let label = match property {
            Some("authors") => format!("Author {}", index + 1),
            Some("references") => format!("Reference {}", index + 1),
            _ => match node_type {
                "Section" => sibling
                    .get("content")
                    .and_then(Value::as_array)
                    .and_then(|content| content.iter().find(|node| kind(node) == "Heading"))
                    .and_then(heading_label)
                    .unwrap_or_else(|| format!("Section {ordinal}")),
                "Heading" => heading_label(sibling).unwrap_or_else(|| format!("Heading {ordinal}")),
                "Figure" | "Table" => {
                    let label = sibling.get("label").map(text).unwrap_or_default();
                    let label = clean(&label);
                    if label.is_empty() {
                        format!("{node_type} {ordinal}")
                    } else {
                        format!("{node_type} {label}")
                    }
                }
                "" => format!(
                    "{} {}",
                    property_label(property.unwrap_or("Item")),
                    index + 1
                ),
                node_type => format!("{node_type} {ordinal}"),
            },
        };
        items.push(Item {
            scope: headings.iter().map(|(_, label)| label.clone()).collect(),
            label,
        });

        if heading && let Some(label) = heading_label(sibling) {
            let level = sibling.get("level").and_then(Value::as_u64).unwrap_or(1);
            headings.push((level, label));
        }
    }
    items
}

fn kind(node: &Value) -> &str {
    node.get("type").and_then(Value::as_str).unwrap_or_default()
}

fn property_label(property: &str) -> &str {
    match property {
        "title" => "Title",
        "abstract" => "Abstract",
        "caption" => "Caption",
        "authors" => "Authors",
        "references" => "References",
        _ => property,
    }
}

fn push(parts: &mut Vec<String>, label: String) {
    // A Section and its first Heading normally supply the same title.
    if parts.last() != Some(&label) {
        parts.push(label);
    }
}

fn heading_label(node: &Value) -> Option<String> {
    let label = clean(&text(node));
    (!label.is_empty()).then(|| format!("“{label}”"))
}

fn clean(text: &str) -> String {
    elide(&text.split_whitespace().collect::<Vec<_>>().join(" "), 60)
}

/// Only read visible inline content, never arbitrary metadata or identifiers
fn text(node: &Value) -> String {
    match node {
        Value::String(text) => text.clone(),
        Value::Array(nodes) => nodes.iter().map(text).collect(),
        Value::Object(object) => object
            .get(match kind(node) {
                "Text" => "value",
                "CodeInline" | "MathInline" => "code",
                // Cords serialize as objects even when they have no authorship.
                "" => "string",
                _ => "content",
            })
            .map(text)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;
    use stencila_node_compare::compare;
    use stencila_schema::Node;

    use super::*;

    #[test]
    fn describes_nested_sections_and_metadata() -> Result<()> {
        let document = json!({
            "type": "Article",
            "content": [{"type": "Section", "content": [
                {"type": "Heading", "content": [{"type": "Text", "value": "Results"}]},
                {"type": "Section", "content": [
                    {"type": "Heading", "content": [{"type": "Text", "value": "Auxin signals"}]},
                    {"type": "Paragraph", "content": []},
                    {"type": "Paragraph", "content": []},
                    {"type": "Figure", "label": "4", "caption": [{"type": "Paragraph", "content": []}]}
                ]}
            ]}],
            "references": [{"type": "Reference", "authors": [{"type": "Person", "givenNames": ["Ada"]}]}],
            "abstract": [{"type": "Paragraph", "content": []}]
        });
        let mut locator = Locator {
            document,
            arrays: HashMap::new(),
        };
        for (path, expected) in [
            (
                "content/0/content/1/content/2",
                "“Results” › “Auxin signals” › Paragraph 2",
            ),
            (
                "content/0/content/1/content/3",
                "“Results” › “Auxin signals” › Figure 4",
            ),
            (
                "content/0/content/1/content/3/caption/0",
                "“Results” › “Auxin signals” › Figure 4 › Caption › Paragraph 1",
            ),
            (
                "references/0/authors/0/givenNames",
                "Reference 1 › Author 1 › givenNames",
            ),
            ("abstract/0", "Abstract › Paragraph 1"),
            ("", "Article"),
        ] {
            assert_eq!(locator.describe(&path.parse()?), expected);
        }
        Ok(())
    }

    #[test]
    fn flat_headings_close_at_their_level_and_keep_inline_text() -> Result<()> {
        let document = json!({"type": "Article", "content": [
            {"type": "Heading", "level": 1, "content": [{"type": "Text", "value": "Results"}]},
            {"type": "Heading", "level": 2, "content": [
                {"type": "Text", "value": "Auxin "},
                {"type": "Emphasis", "content": [{"type": "Text", "value": "signals"}]}
            ]},
            {"type": "Paragraph", "content": []},
            {"type": "Heading", "level": 1, "content": [{"type": "Text", "value": "Discussion"}]},
            {"type": "Paragraph", "content": []},
            {"type": "Paragraph", "content": []}
        ]});
        let mut locator = Locator {
            document,
            arrays: HashMap::new(),
        };
        assert_eq!(
            locator.describe(&"content/2".parse()?),
            "“Results” › “Auxin signals” › Paragraph 1"
        );
        assert_eq!(locator.describe(&"content/3".parse()?), "“Discussion”");
        // Numbering restarts within each heading's scope
        assert_eq!(
            locator.describe(&"content/4".parse()?),
            "“Discussion” › Paragraph 1"
        );
        assert_eq!(
            locator.describe(&"content/5".parse()?),
            "“Discussion” › Paragraph 2"
        );
        assert_eq!(locator.describe(&"content/99".parse()?), "$/content/99");
        Ok(())
    }

    #[test]
    fn metadata_covers_both_sides_and_html_escapes_labels() -> Result<()> {
        let left: Node = serde_json::from_value(json!({"type": "Article", "content": []}))?;
        let right: Node = serde_json::from_value(json!({"type": "Article", "content": [
            {"type": "Section", "content": [
                {"type": "Heading", "level": 1, "content": [{"type": "Text", "value": "<script>& résumé 日本語"}]},
                {"type": "Paragraph", "content": [{"type": "Text", "value": "Hello"}]}
            ]}
        ]}))?;
        let comparison = compare(&left, &right)?;
        let left = Snapshot {
            node: &left,
            label: "left",
        };
        let right = Snapshot {
            node: &right,
            label: "right",
        };
        let metadata = ReportMetadata::new(&comparison, left, right)?;
        for entry in comparison.alignment().correspondences() {
            if let Some(node) = entry.left() {
                assert!(metadata.locations.left.contains_key(&path(&node.path)));
            }
            if let Some(node) = entry.right() {
                assert!(metadata.locations.right.contains_key(&path(&node.path)));
            }
        }
        assert_eq!(
            metadata.locations.right.get("$/content/0"),
            Some(&"“<script>& résumé 日本語”".to_string())
        );
        let html = crate::html_report(&comparison, left, right, false)?;
        assert!(html.contains("“&lt;script&gt;&amp; résumé 日本語”"));
        assert!(!html.contains("<script>"));
        Ok(())
    }

    #[test]
    fn metadata_covers_every_paragraph_of_a_group() -> Result<()> {
        let paragraph =
            |text: &str| json!({"type": "Paragraph", "content": [{"type": "Text", "value": text}]});
        let left: Node = serde_json::from_value(json!({"type": "Article", "content": [
            paragraph("First."),
            paragraph("Second.")
        ]}))?;
        let right: Node = serde_json::from_value(json!({"type": "Article", "content": [
            paragraph("First. Second.")
        ]}))?;
        let comparison = compare(&left, &right)?;
        let metadata = ReportMetadata::new(
            &comparison,
            Snapshot {
                node: &left,
                label: "left",
            },
            Snapshot {
                node: &right,
                label: "right",
            },
        )?;
        assert_eq!(
            metadata.locations.left.get("$/content/1"),
            Some(&"Paragraph 2".to_string())
        );
        assert!(metadata.locations.right.contains_key("$/content/0"));
        Ok(())
    }

    #[test]
    fn html_uses_supplied_locations_rather_than_deriving_them() -> Result<()> {
        let left: Node = serde_json::from_value(json!({"type": "Article", "content": []}))?;
        let right: Node = serde_json::from_value(json!({"type": "Article", "content": [
            {"type": "Paragraph", "content": [{"type": "Text", "value": "Hello"}]}
        ]}))?;
        let comparison = compare(&left, &right)?;
        let left = Snapshot {
            node: &left,
            label: "left",
        };
        let right = Snapshot {
            node: &right,
            label: "right",
        };
        let locations = LocationLabels {
            left: BTreeMap::new(),
            right: BTreeMap::from([("$/content/0".to_string(), "Supplied".to_string())]),
        };
        let overlay = Node::Null(stencila_schema::Null);
        let html = crate::html_report_with_overlay(
            &comparison,
            left,
            right,
            false,
            &overlay,
            Some(&locations),
        )?;
        assert!(html.contains("<span class=\"location\">Supplied</span>"));
        Ok(())
    }
}
