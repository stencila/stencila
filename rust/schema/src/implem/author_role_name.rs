use crate::AuthorRoleName;

/// Normalize a CRediT term for lenient comparison
///
/// Lowercases and joins the alphanumeric words of the term, dropping "and"
/// so that, for example, "Writing — review and editing" (em dash), the
/// canonical "Writing – review & editing" (en dash), the URI slug
/// "writing-review-editing", and the variant name "WritingReviewEditing" all
/// normalize to the same string. Publishers vary in dash character, casing,
/// and use of "&" versus "and", so all of those differences are ignored.
fn normalize(term: &str) -> String {
    term.to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|word| !word.is_empty() && *word != "and")
        .collect()
}

impl AuthorRoleName {
    /// Get the CRediT role matching a term, if any
    ///
    /// The term may be the canonical role name (e.g. "Writing – original
    /// draft"), the URI slug (e.g. "writing-original-draft"), or the variant
    /// name (e.g. "WritingOriginalDraft"), compared leniently with respect to
    /// case, dash characters, and "&" versus "and".
    pub fn from_credit(term: &str) -> Option<Self> {
        let term = normalize(term);
        if term.is_empty() {
            return None;
        }

        Self::CREDIT_METADATA
            .iter()
            .find(|(.., jid, name)| {
                normalize(jid.strip_prefix("credit:").unwrap_or(jid)) == term
                    || normalize(name) == term
            })
            .map(|(role, ..)| *role)
    }

    /// Get the CRediT role matching a role URI, if any
    ///
    /// Accepts both `https://` and `http://` forms of
    /// `credit.niso.org/contributor-roles/{slug}/`, with or without the
    /// trailing slash.
    pub fn from_credit_uri(uri: &str) -> Option<Self> {
        let uri = uri.trim();
        let lower = uri.to_ascii_lowercase();
        let prefix = [
            "http://credit.niso.org/contributor-roles/",
            "https://credit.niso.org/contributor-roles/",
        ]
        .into_iter()
        .find(|prefix| lower.starts_with(prefix))?;
        Self::from_credit(uri[prefix.len()..].trim_end_matches('/'))
    }

    /// Whether this role is one of the fourteen CRediT contributor roles
    pub fn is_credit(&self) -> bool {
        Self::CREDIT_METADATA.iter().any(|(role, ..)| role == self)
    }

    /// The canonical CRediT name of this role (e.g. "Writing – original
    /// draft"), if it is a CRediT role
    pub fn credit_name(&self) -> Option<&'static str> {
        Self::CREDIT_METADATA
            .iter()
            .find(|(role, ..)| role == self)
            .map(|(.., name)| *name)
    }

    /// The canonical CRediT role URI (e.g.
    /// `https://credit.niso.org/contributor-roles/conceptualization/`), if
    /// this is a CRediT role
    pub fn credit_uri(&self) -> Option<String> {
        Self::CREDIT_METADATA
            .iter()
            .find(|(role, ..)| role == self)
            .and_then(|(_, jid, ..)| jid.strip_prefix("credit:"))
            .map(|slug| ["https://credit.niso.org/contributor-roles/", slug, "/"].concat())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_credit_is_lenient() -> eyre::Result<()> {
        for (role, jid, name) in AuthorRoleName::CREDIT_METADATA {
            let slug = jid
                .strip_prefix("credit:")
                .ok_or_else(|| eyre::eyre!("not a CRediT id: {jid}"))?;
            assert_eq!(AuthorRoleName::from_credit(slug), Some(*role));
            assert_eq!(AuthorRoleName::from_credit(name), Some(*role));
            assert_eq!(AuthorRoleName::from_credit(&role.to_string()), Some(*role));
        }

        // Dash and ampersand variations
        for term in [
            "Writing – review & editing", // en dash (canonical)
            "Writing — review & editing", // em dash
            "Writing - review & editing", // hyphen
            "Writing – review and editing",
            "writing review editing",
        ] {
            assert_eq!(
                AuthorRoleName::from_credit(term),
                Some(AuthorRoleName::WritingReviewEditing),
                "term: {term}"
            );
        }

        assert_eq!(AuthorRoleName::from_credit("Senior Editor"), None);
        assert_eq!(AuthorRoleName::from_credit(""), None);
        Ok(())
    }

    #[test]
    fn from_credit_uri_accepts_both_schemes() {
        for uri in [
            "https://credit.niso.org/contributor-roles/conceptualization/",
            "http://credit.niso.org/contributor-roles/conceptualization/",
            "https://credit.niso.org/contributor-roles/conceptualization",
        ] {
            assert_eq!(
                AuthorRoleName::from_credit_uri(uri),
                Some(AuthorRoleName::Conceptualization),
                "uri: {uri}"
            );
        }

        assert_eq!(
            AuthorRoleName::from_credit_uri("https://example.org/conceptualization"),
            None
        );
    }

    #[test]
    fn credit_uri_round_trips() -> eyre::Result<()> {
        for (role, ..) in AuthorRoleName::CREDIT_METADATA {
            let uri = role
                .credit_uri()
                .ok_or_else(|| eyre::eyre!("missing CRediT URI for {role}"))?;
            assert_eq!(AuthorRoleName::from_credit_uri(&uri), Some(*role));
        }

        assert!(!AuthorRoleName::Writer.is_credit());
        assert_eq!(AuthorRoleName::Writer.credit_uri(), None);
        Ok(())
    }
}
