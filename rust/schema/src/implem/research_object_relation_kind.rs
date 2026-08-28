use crate::{GraphEdgeKind, Primitive, ResearchObjectRelationKind};

/// Parse relation targets from authored string or array metadata.
pub fn research_relation_targets(value: &Primitive) -> Vec<String> {
    match value {
        Primitive::String(value) => value
            .split(|character: char| character.is_whitespace() || character == ',')
            .filter(|target| !target.is_empty())
            .map(str::to_string)
            .collect(),
        Primitive::Array(values) => values.iter().flat_map(research_relation_targets).collect(),
        _ => Vec::new(),
    }
}

impl ResearchObjectRelationKind {
    /// Return every authored relation kind in schema order.
    pub const fn all() -> &'static [Self] {
        &[
            Self::Supports,
            Self::SupportedBy,
            Self::Opposes,
            Self::OpposedBy,
            Self::Addresses,
            Self::AddressedBy,
            Self::Follows,
            Self::Grounds,
            Self::IsGroundedIn,
            Self::RequestFor,
            Self::RequestTarget,
        ]
    }

    /// The kebab-case attribute key used when authoring this relation kind
    /// in Markdown-based formats, e.g. `supported-by="#e1"`.
    pub fn authored_key(&self) -> &'static str {
        use ResearchObjectRelationKind::*;
        match self {
            Supports => "supports",
            SupportedBy => "supported-by",
            Opposes => "opposes",
            OpposedBy => "opposed-by",
            Addresses => "addresses",
            AddressedBy => "addressed-by",
            Follows => "follows",
            Grounds => "grounds",
            IsGroundedIn => "is-grounded-in",
            RequestFor => "request-for",
            RequestTarget => "request-target",
        }
    }

    /// Parse an authored attribute key into a relation kind.
    ///
    /// Accepts kebab-case, snake_case, and camelCase variants of the keys
    /// produced by [`Self::authored_key`], case insensitively, plus the
    /// legacy `grounded-in` alias for `IsGroundedIn`.
    pub fn from_authored_key(key: &str) -> Option<Self> {
        use ResearchObjectRelationKind::*;

        let normalized = key
            .chars()
            .filter(|char| *char != '-' && *char != '_')
            .collect::<String>()
            .to_lowercase();

        Some(match normalized.as_str() {
            "supports" => Supports,
            "supportedby" => SupportedBy,
            "opposes" => Opposes,
            "opposedby" => OpposedBy,
            "addresses" => Addresses,
            "addressedby" => AddressedBy,
            "follows" => Follows,
            "grounds" => Grounds,
            "isgroundedin" | "groundedin" => IsGroundedIn,
            "requestfor" => RequestFor,
            "requesttarget" => RequestTarget,
            _ => return None,
        })
    }

    /// The local name of this relation in the MIRA vocabulary.
    pub const fn mira_name(self) -> &'static str {
        match self {
            Self::Supports => "supports",
            Self::SupportedBy => "supportedBy",
            Self::Opposes => "opposes",
            Self::OpposedBy => "opposedBy",
            Self::Addresses => "addresses",
            Self::AddressedBy => "addressedBy",
            Self::Follows => "follows",
            Self::Grounds => "grounds",
            Self::IsGroundedIn => "is_grounded_in",
            Self::RequestFor => "request_for",
            Self::RequestTarget => "request_target",
        }
    }
}

impl From<ResearchObjectRelationKind> for GraphEdgeKind {
    fn from(kind: ResearchObjectRelationKind) -> Self {
        match kind {
            ResearchObjectRelationKind::Supports => Self::Supports,
            ResearchObjectRelationKind::SupportedBy => Self::SupportedBy,
            ResearchObjectRelationKind::Opposes => Self::Opposes,
            ResearchObjectRelationKind::OpposedBy => Self::OpposedBy,
            ResearchObjectRelationKind::Addresses => Self::Addresses,
            ResearchObjectRelationKind::AddressedBy => Self::AddressedBy,
            ResearchObjectRelationKind::Follows => Self::Follows,
            ResearchObjectRelationKind::Grounds => Self::Grounds,
            ResearchObjectRelationKind::IsGroundedIn => Self::IsGroundedIn,
            ResearchObjectRelationKind::RequestFor => Self::RequestFor,
            ResearchObjectRelationKind::RequestTarget => Self::RequestTarget,
        }
    }
}

impl TryFrom<GraphEdgeKind> for ResearchObjectRelationKind {
    type Error = ();

    fn try_from(kind: GraphEdgeKind) -> Result<Self, Self::Error> {
        Ok(match kind {
            GraphEdgeKind::Supports => Self::Supports,
            GraphEdgeKind::SupportedBy => Self::SupportedBy,
            GraphEdgeKind::Opposes => Self::Opposes,
            GraphEdgeKind::OpposedBy => Self::OpposedBy,
            GraphEdgeKind::Addresses => Self::Addresses,
            GraphEdgeKind::AddressedBy => Self::AddressedBy,
            GraphEdgeKind::Follows => Self::Follows,
            GraphEdgeKind::Grounds => Self::Grounds,
            GraphEdgeKind::IsGroundedIn => Self::IsGroundedIn,
            GraphEdgeKind::RequestFor => Self::RequestFor,
            GraphEdgeKind::RequestTarget => Self::RequestTarget,
            _ => return Err(()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authored_keys_round_trip() {
        for kind in ResearchObjectRelationKind::all() {
            assert_eq!(
                ResearchObjectRelationKind::from_authored_key(kind.authored_key()),
                Some(*kind)
            );
        }
    }

    #[test]
    fn graph_edge_kinds_and_mira_names_round_trip() {
        for kind in ResearchObjectRelationKind::all() {
            assert_eq!(
                ResearchObjectRelationKind::try_from(GraphEdgeKind::from(*kind)),
                Ok(*kind)
            );
            assert!(!kind.mira_name().is_empty());
        }
    }

    #[test]
    fn authored_key_variants() {
        use ResearchObjectRelationKind::*;

        for (key, kind) in [
            ("supportedBy", SupportedBy),
            ("supported_by", SupportedBy),
            ("Supported-By", SupportedBy),
            ("isGroundedIn", IsGroundedIn),
            ("is_grounded_in", IsGroundedIn),
            ("grounded-in", IsGroundedIn),
            ("groundedIn", IsGroundedIn),
            ("requestFor", RequestFor),
            ("request_target", RequestTarget),
        ] {
            assert_eq!(
                ResearchObjectRelationKind::from_authored_key(key),
                Some(kind),
                "{key}"
            );
        }
    }

    #[test]
    fn unknown_keys_are_rejected() {
        for key in ["", "supportz", "cites", "label", "id"] {
            assert_eq!(ResearchObjectRelationKind::from_authored_key(key), None);
        }
    }
}
