//! Explicit cosmetic classification, applied after matching and difference derivation

use std::collections::{BTreeSet, HashMap};

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use stencila_node_type::{NodeProperty, NodeType};
use strum::{Display, EnumString};
use unicode_normalization::char::is_combining_mark;

use stencila_node_path::NodePath;

use crate::{
    Difference, ScalarValue, ValueLocation, ValueState,
    error::CompareResult,
    projection::{Item, OccurrenceId, Projection},
};

/// A normalization used only to classify differences, never to alter input values
///
/// Whitespace and typography apply to `Text.value` and `Person.givenNames`.
/// Initials apply only to `Person.givenNames`. Other properties, including code,
/// mathematics, identifiers and link targets, retain literal comparison.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Serialize,
    Deserialize,
    Display,
    EnumString,
    ValueEnum,
)]
#[serde(rename_all = "camelCase")]
#[strum(serialize_all = "camelCase")]
pub enum Normalization {
    /// Collapse runs of Unicode whitespace to a single space
    ///
    /// A run at either end of a `Text.value` is collapsed but kept, because it may be
    /// the word boundary with an adjacent inline node. The exception is an end beside
    /// a citation on both sides: whether a citation is set off by a space is a matter
    /// of style, not wording, so whitespace there is ignored. Given names have no
    /// neighbours, so they are trimmed.
    Whitespace,
    /// Fold curly quotes, typographic dashes and ellipses to their ASCII forms
    Typography,
    /// Equate dotted or space-separated single-letter given names with separate initials
    ///
    /// An initial is one letter, with any combining marks that follow it. Preserves
    /// case, accents, order and full names; does not expand initials or split
    /// undelimited names such as `SS`.
    Initials,
}

/// Annotate the raw differences without changing their values or locations
pub(crate) fn classify(
    differences: &mut [Difference],
    left_projection: &Projection,
    right_projection: &Projection,
    normalize: &BTreeSet<Normalization>,
) -> CompareResult<()> {
    if normalize.is_empty() {
        return Ok(());
    }

    let left_texts = Texts::new(left_projection, normalize);
    let right_texts = Texts::new(right_projection, normalize);
    for difference in differences {
        if let Difference::ValueChanged {
            location,
            left,
            right,
            cosmetic,
        } = difference
        {
            let edges = left_texts
                .citation_edges(&location.left.path)?
                .and(right_texts.citation_edges(&location.right.path)?);
            *cosmetic = is_cosmetic(location, left, right, normalize, edges);
        }
    }

    Ok(())
}

/// The ends of a `Text` that are beside a citation
#[derive(Debug, Clone, Copy, Default)]
struct Edges {
    start: bool,
    end: bool,
}

impl Edges {
    /// The ends beside a citation in both of two texts
    fn and(self, other: Self) -> Self {
        Self {
            start: self.start && other.start,
            end: self.end && other.end,
        }
    }
}

/// The `Text` occurrences of one projection, by path
///
/// Empty unless the whitespace rule is enabled, the only rule that looks at neighbours.
struct Texts<'projection> {
    projection: &'projection Projection,
    paths: HashMap<&'projection NodePath, OccurrenceId>,
}

impl<'projection> Texts<'projection> {
    fn new(projection: &'projection Projection, normalize: &BTreeSet<Normalization>) -> Self {
        let paths = if normalize.contains(&Normalization::Whitespace) {
            projection
                .occurrences()
                .iter()
                .filter(|occurrence| occurrence.node_type == NodeType::Text)
                .map(|occurrence| (&occurrence.path, occurrence.id))
                .collect()
        } else {
            HashMap::new()
        };
        Self { projection, paths }
    }

    /// Which ends of the `Text` at a path have a citation as their sibling
    fn citation_edges(&self, path: &NodePath) -> CompareResult<Edges> {
        let Some(&id) = self.paths.get(path) else {
            return Ok(Edges::default());
        };
        let occurrence = self.projection.occurrence(id)?;
        let (Some(parent), Some(property)) = (occurrence.parent, occurrence.parent_property) else {
            return Ok(Edges::default());
        };
        let Some(siblings) = self
            .projection
            .occurrence(parent)?
            .properties
            .iter()
            .find(|candidate| candidate.decl.property == property)
            .map(|property| &property.items)
        else {
            return Ok(Edges::default());
        };
        let Some(position) = siblings
            .iter()
            .position(|item| matches!(item, Item::Structured(sibling) if *sibling == id))
        else {
            return Ok(Edges::default());
        };

        let is_citation = |item: Option<&Item>| -> CompareResult<bool> {
            Ok(match item {
                Some(Item::Structured(sibling)) => matches!(
                    self.projection.occurrence(*sibling)?.node_type,
                    NodeType::Citation | NodeType::CitationGroup
                ),
                _ => false,
            })
        };
        Ok(Edges {
            start: is_citation(
                position
                    .checked_sub(1)
                    .and_then(|before| siblings.get(before)),
            )?,
            end: is_citation(siblings.get(position + 1))?,
        })
    }
}

/// Whether the enabled rules explain the entire value difference
fn is_cosmetic(
    location: &ValueLocation,
    left: &ValueState,
    right: &ValueState,
    normalize: &BTreeSet<Normalization>,
    edges: Edges,
) -> bool {
    if normalize.is_empty() || left == right || location.left.node_type != location.right.node_type
    {
        return false;
    }
    match (location.left.node_type, location.property, left, right) {
        (
            NodeType::Text,
            Some(NodeProperty::Value),
            ValueState::One {
                value: ScalarValue::String { value: left },
            },
            ValueState::One {
                value: ScalarValue::String { value: right },
            },
        ) => {
            let fold = |value: &str| {
                let value = normalize_text(value, normalize);
                let value = if edges.start {
                    value.trim_start()
                } else {
                    &value
                };
                let value = if edges.end { value.trim_end() } else { value };
                value.to_string()
            };
            fold(left) == fold(right)
        }
        (
            NodeType::Person,
            Some(NodeProperty::GivenNames),
            ValueState::Many { values: left },
            ValueState::Many { values: right },
        ) => match (
            normalize_names(left, normalize),
            normalize_names(right, normalize),
        ) {
            (Some(left), Some(right)) => left == right,
            _ => false,
        },
        _ => false,
    }
}

fn normalize_text(value: &str, normalize: &BTreeSet<Normalization>) -> String {
    let value = if normalize.contains(&Normalization::Typography) {
        value.chars().fold(String::new(), |mut text, character| {
            match character {
                '‘' | '’' | '‚' | '‛' => text.push('\''),
                '“' | '”' | '„' | '‟' => text.push('"'),
                '‐' | '‑' | '–' | '—' => text.push('-'),
                '…' => text.push_str("..."),
                _ => text.push(character),
            }
            text
        })
    } else {
        value.to_string()
    };
    if normalize.contains(&Normalization::Whitespace) {
        collapse_whitespace(&value)
    } else {
        value
    }
}

/// Collapse each run of whitespace, including one at either end, to a single space
///
/// Unlike trimming, this keeps the word boundary that edge whitespace in a `Text`
/// may provide between it and an adjacent inline node.
fn collapse_whitespace(value: &str) -> String {
    let mut collapsed = String::with_capacity(value.len());
    let mut in_whitespace = false;
    for character in value.chars() {
        if character.is_whitespace() {
            if !in_whitespace {
                collapsed.push(' ');
            }
            in_whitespace = true;
        } else {
            collapsed.push(character);
            in_whitespace = false;
        }
    }
    collapsed
}

fn normalize_names(
    values: &[ScalarValue],
    normalize: &BTreeSet<Normalization>,
) -> Option<Vec<String>> {
    let mut names = Vec::new();
    for value in values {
        let ScalarValue::String { value } = value else {
            return None;
        };
        let mut value = normalize_text(value, normalize);
        if normalize.contains(&Normalization::Whitespace) {
            value = value.trim().to_string();
        }
        if normalize.contains(&Normalization::Initials) {
            let initials: Vec<_> = value
                .split(|character: char| character == '.' || character.is_whitespace())
                .filter(|part| !part.is_empty())
                .collect();
            if !initials.is_empty()
                && initials.iter().all(|part| {
                    let mut characters = part.chars();
                    characters.next().is_some_and(char::is_alphabetic)
                        && characters.all(is_combining_mark)
                })
            {
                names.extend(initials.into_iter().map(String::from));
                continue;
            }
        }
        names.push(value);
    }
    Some(names)
}
