//! Explicit cosmetic classification, applied after matching and difference derivation

use std::collections::BTreeSet;

use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use stencila_node_type::{NodeProperty, NodeType};
use strum::{Display, EnumString};
use unicode_normalization::char::is_combining_mark;

use crate::{Difference, ScalarValue, ValueLocation, ValueState};

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
    /// A run at either end of a `Text.value` is collapsed but never removed, because
    /// it may be the word boundary with an adjacent inline node. Given names have no
    /// such neighbours, so they are also trimmed.
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
pub(crate) fn classify(differences: &mut [Difference], normalize: &BTreeSet<Normalization>) {
    for difference in differences {
        if let Difference::ValueChanged {
            location,
            left,
            right,
            cosmetic,
        } = difference
        {
            *cosmetic = is_cosmetic(location, left, right, normalize);
        }
    }
}

/// Whether the enabled rules explain the entire value difference
fn is_cosmetic(
    location: &ValueLocation,
    left: &ValueState,
    right: &ValueState,
    normalize: &BTreeSet<Normalization>,
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
        ) => normalize_text(left, normalize) == normalize_text(right, normalize),
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
