//! Selecting which differences a comparison reports
//!
//! A filter is the one deliberately semantic knob in [`crate::CompareOptions`]: it
//! changes which observations a comparison reports, not how the alignment is computed.
//! That separation is the whole point. Matching is decided by the schema-native policy
//! and is never influenced by a filter, so excluding a property can never change which
//! occurrences pair with which — only whether the resulting observation is reported.
//!
//! Filters exist because two documents in different formats legitimately disagree about
//! format-carried bookkeeping. A JATS document declares `jatsRefType` where Markdown has
//! nothing to say, and identifiers are minted independently on each side. Those are real
//! differences, so the comparison is right to derive them, but they are rarely the
//! differences a reader is asking about.
//!
//! # Selectors
//!
//! | Form | Matches |
//! | --- | --- |
//! | `id` | that property, on any node type |
//! | `Link.id` | that property, on that node type only |
//! | `Link` | everything about that node type |
//! | `*`, or `all` | everything |
//!
//! `all` is a synonym for `*`, for the sake of shells in which a bare `*` would be
//! expanded to a list of file names before the command ever sees it.
//!
//! Node types are PascalCase and properties are camelCase, which is what tells a bare
//! `Link` from a bare `id` without any ambiguity.
//!
//! # Precedence
//!
//! The *most specific* matching selector wins, regardless of the order the selectors
//! were given in: `Link.id` beats `id`, which beats `Link`, which beats `*`. Order
//! independence is deliberate, so that a filter means the same thing on a command line,
//! in a configuration file, and through a language binding. An exclusion wins a tie with
//! an inclusion of equal specificity, because hiding too little is easier to notice than
//! hiding too much. Anything no selector matches is reported.
//!
//! So `--exclude id --include Figure.id` reports figure identifiers and no others, and
//! `--exclude '*' --include title` reports nothing but titles.

use std::{fmt, str::FromStr};

use serde::{Deserialize, Serialize};

use stencila_node_type::{NodeProperty, NodeType};

use crate::{alignment::NodeRef, comparison::Difference};

/// What a filter selector matches
///
/// Ordered by specificity, least specific first, which is the order the precedence rule
/// depends on: the derived `Ord` on the discriminant is the specificity ranking.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Selector {
    /// Every difference, of every type, about every property
    All,

    /// Every difference about one node type
    Type(NodeType),

    /// Every difference about one property, on any node type
    Property(NodeProperty),

    /// Every difference about one property of one node type
    TypeProperty(NodeType, NodeProperty),
}

impl Selector {
    /// How specific this selector is
    ///
    /// Higher wins. Taken from the variant order rather than written out again, so that
    /// the ranking cannot drift from the documented one.
    fn specificity(self) -> u8 {
        match self {
            Self::All => 0,
            Self::Type(..) => 1,
            Self::Property(..) => 2,
            Self::TypeProperty(..) => 3,
        }
    }

    /// Whether this selector matches a subject
    fn matches(self, subject: &Subject) -> bool {
        match self {
            Self::All => true,
            Self::Type(node_type) => subject.is_type(node_type),
            Self::Property(property) => subject.property == Some(property),
            Self::TypeProperty(node_type, property) => {
                subject.is_type(node_type) && subject.property == Some(property)
            }
        }
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::All => formatter.write_str("*"),
            Self::Type(node_type) => write!(formatter, "{node_type}"),
            Self::Property(property) => write!(formatter, "{property}"),
            Self::TypeProperty(node_type, property) => write!(formatter, "{node_type}.{property}"),
        }
    }
}

/// Why a selector could not be parsed
///
/// Carries the offending text so that a caller can report it without having to keep the
/// original string alongside the error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SelectorError {
    /// The selector was empty, or had an empty half
    Empty,

    /// The node type is not in the schema
    UnknownType { name: String },

    /// The property is not in the schema
    UnknownProperty { name: String },

    /// Both halves parsed, but the schema does not declare that property on that type
    ///
    /// Rejected rather than silently matching nothing, because a selector that cannot
    /// possibly match is always a mistake.
    UndeclaredProperty {
        node_type: NodeType,
        property: NodeProperty,
    },
}

impl fmt::Display for SelectorError {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str(
                "Expected a selector such as `id`, `Link.id`, `Link` or `*`, but it was empty",
            ),
            Self::UnknownType { name } => write!(
                formatter,
                "`{name}` is not a node type in the Stencila Schema; node types are PascalCase, as in `Link`"
            ),
            Self::UnknownProperty { name } => write!(
                formatter,
                "`{name}` is not a property in the Stencila Schema; properties are camelCase, as in `jatsRefType`"
            ),
            Self::UndeclaredProperty {
                node_type,
                property,
            } => write!(
                formatter,
                "The Stencila Schema does not declare a `{property}` property on `{node_type}`"
            ),
        }
    }
}

impl std::error::Error for SelectorError {}

impl FromStr for Selector {
    type Err = SelectorError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let text = text.trim();
        if text.is_empty() {
            return Err(SelectorError::Empty);
        }
        // `all` spells the same selector without the shell quoting a bare `*` needs.
        // Checked before the property branch, so it never reads as a property name.
        if text == "*" || text == "all" {
            return Ok(Self::All);
        }

        if let Some((node_type, property)) = text.split_once('.') {
            let node_type = parse_type(node_type)?;
            let property = parse_property(property)?;
            if !node_type.properties().contains(&property) {
                return Err(SelectorError::UndeclaredProperty {
                    node_type,
                    property,
                });
            }
            return Ok(Self::TypeProperty(node_type, property));
        }

        // A leading capital is what distinguishes a node type from a property, so the
        // two halves of the grammar never compete for the same text
        if text.starts_with(char::is_uppercase) {
            Ok(Self::Type(parse_type(text)?))
        } else {
            Ok(Self::Property(parse_property(text)?))
        }
    }
}

/// Parse a node type, without the aliases that `NodeType::from_name` allows
///
/// Aliases are deliberately not accepted: `table` would otherwise be a node type while
/// every other lowercase word in the grammar is a property.
fn parse_type(name: &str) -> Result<NodeType, SelectorError> {
    NodeType::from_str(name).map_err(|_| SelectorError::UnknownType {
        name: name.to_string(),
    })
}

fn parse_property(name: &str) -> Result<NodeProperty, SelectorError> {
    NodeProperty::from_str(name).map_err(|_| SelectorError::UnknownProperty {
        name: name.to_string(),
    })
}

impl Serialize for Selector {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.collect_str(self)
    }
}

impl<'de> Deserialize<'de> for Selector {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        Self::from_str(&text).map_err(serde::de::Error::custom)
    }
}

/// What a filter decision is made about
///
/// A difference is about a pair, whose two sides may be of different node types, so a
/// type selector matches when *either* side is of that type. One-sided structure is
/// about a single occurrence and has no property at all.
struct Subject {
    left_type: NodeType,
    right_type: Option<NodeType>,
    property: Option<NodeProperty>,
}

impl Subject {
    fn is_type(&self, node_type: NodeType) -> bool {
        self.left_type == node_type || self.right_type == Some(node_type)
    }
}

/// Which differences a comparison reports
///
/// An empty filter reports everything, which is what [`Default`] gives, so that adding a
/// filter to an existing call site changes nothing until selectors are supplied.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct DifferenceFilter {
    /// Selectors for differences to report
    ///
    /// Only meaningful against a broader exclusion: with nothing excluded, everything is
    /// already reported.
    pub include: Vec<Selector>,

    /// Selectors for differences not to report
    pub exclude: Vec<Selector>,
}

impl DifferenceFilter {
    /// A filter that reports everything
    pub fn none() -> Self {
        Self::default()
    }

    /// Whether this filter reports everything
    pub fn is_empty(&self) -> bool {
        self.include.is_empty() && self.exclude.is_empty()
    }

    /// Whether a subject survives this filter
    ///
    /// The most specific matching selector decides, and an exclusion wins a tie.
    fn allows(&self, subject: &Subject) -> bool {
        let best = |selectors: &[Selector]| {
            selectors
                .iter()
                .filter(|selector| selector.matches(subject))
                .map(|selector| selector.specificity())
                .max()
        };

        match (best(&self.exclude), best(&self.include)) {
            (Some(exclude), Some(include)) => include > exclude,
            (Some(..), None) => false,
            (None, ..) => true,
        }
    }

    /// Whether a difference is reported
    pub fn allows_difference(&self, difference: &Difference) -> bool {
        self.allows(&Subject {
            left_type: difference.left().node_type,
            right_type: Some(difference.right().node_type),
            property: difference.property(),
        })
    }

    /// Whether a one-sided occurrence is reported
    ///
    /// One-sided structure has no property, so only type selectors and `*` can hide it.
    /// A property selector never does: excluding `id` means "do not tell me about
    /// identifiers", not "pretend this whole missing subtree is not missing".
    pub fn allows_node(&self, node: &NodeRef) -> bool {
        self.allows(&Subject {
            left_type: node.node_type,
            right_type: None,
            property: None,
        })
    }
}
