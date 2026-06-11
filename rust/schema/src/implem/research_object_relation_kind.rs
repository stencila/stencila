use crate::ResearchObjectRelationKind;

impl ResearchObjectRelationKind {
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
}

#[cfg(test)]
mod tests {
    use super::*;

    const ALL: [ResearchObjectRelationKind; 11] = [
        ResearchObjectRelationKind::Supports,
        ResearchObjectRelationKind::SupportedBy,
        ResearchObjectRelationKind::Opposes,
        ResearchObjectRelationKind::OpposedBy,
        ResearchObjectRelationKind::Addresses,
        ResearchObjectRelationKind::AddressedBy,
        ResearchObjectRelationKind::Follows,
        ResearchObjectRelationKind::Grounds,
        ResearchObjectRelationKind::IsGroundedIn,
        ResearchObjectRelationKind::RequestFor,
        ResearchObjectRelationKind::RequestTarget,
    ];

    #[test]
    fn authored_keys_round_trip() {
        for kind in ALL {
            assert_eq!(
                ResearchObjectRelationKind::from_authored_key(kind.authored_key()),
                Some(kind)
            );
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
