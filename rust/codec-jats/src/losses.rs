//! Classification of JATS decode and encode losses
//!
//! Raw loss counts do not indicate semantic impact: a single dropped
//! `<sub-article>` matters far more than several hundred source-system
//! attributes. This module groups loss labels so that tests and audit tooling
//! can distinguish losses that change the meaning of a document from those that
//! only change its serialization.

use std::fmt::{self, Display};

/// The kind of information that a loss label represents
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LossCategory {
    /// Content-bearing structures and prose
    Content,
    /// Bibliographic, publication, contributor, and funding metadata
    Metadata,
    /// Links and identifiers, including cross reference targets
    LinkOrIdentifier,
    /// Presentation and source-system detail
    Presentation,
}

impl Display for LossCategory {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(match self {
            LossCategory::Content => "content",
            LossCategory::Metadata => "metadata",
            LossCategory::LinkOrIdentifier => "link-or-identifier",
            LossCategory::Presentation => "presentation",
        })
    }
}

/// Names, of either XML nodes or schema properties, that denote a link or an
/// identifier
const LINK_OR_IDENTIFIER: &[&str] = &[
    "article-id",
    "contrib-id",
    "doi",
    "elocation-id",
    "ext-link",
    "href",
    "id",
    "identifiers",
    "institution-id",
    "isbn",
    "issn",
    "issue-id",
    "journal-id",
    "orcid",
    "pub-id",
    "pub-id-type",
    "ref-type",
    "rid",
    "self-uri",
    "target",
    "uri",
    "url",
    "xlink:href",
    "xref",
];

/// Names that denote presentation or source-system detail rather than meaning
const PRESENTATION: &[&str] = &[
    "align",
    "border",
    "cellpadding",
    "cellspacing",
    "char",
    "charoff",
    "colspan",
    "colwidth",
    "dtd-version",
    "frame",
    "height",
    "orientation",
    "position",
    "rowspan",
    "rules",
    "sortable",
    "style",
    "toggle",
    "valign",
    "width",
];

/// Names that denote content-bearing structures or prose
const CONTENT: &[&str] = &[
    "abstract",
    "alt-text",
    "boxed-text",
    "caption",
    "chem-struct",
    "code",
    "content",
    "def-list",
    "disp-formula",
    "disp-quote",
    "fig",
    "fig-group",
    "fn",
    "fn-group",
    "graphic",
    "inline-formula",
    "inline-graphic",
    "label",
    "list",
    "media",
    "mml:math",
    "named-content",
    "notes",
    "p",
    "preformat",
    "sec",
    "statement",
    "sub-article",
    "supplementary-material",
    "table",
    "table-wrap",
    "text()",
    "title",
    "verse-group",
];

/// Classify a decode or encode loss label
///
/// Handles both XPath-like decode labels such as
/// `//article/front/article-meta/@specific-use` and schema property encode
/// labels such as `Article.identifiers`.
pub fn classify(label: &str) -> LossCategory {
    let name = leaf_name(label);
    let is_attribute = label
        .rsplit(['/', '.'])
        .next()
        .is_some_and(|segment| segment.starts_with('@'));

    if LINK_OR_IDENTIFIER.contains(&name) {
        return LossCategory::LinkOrIdentifier;
    }
    if PRESENTATION.contains(&name) {
        return LossCategory::Presentation;
    }
    if CONTENT.contains(&name) {
        return LossCategory::Content;
    }

    // Unrecognized attributes are overwhelmingly source-system detail, whereas
    // unrecognized elements and schema properties carry document metadata.
    if is_attribute {
        LossCategory::Presentation
    } else {
        LossCategory::Metadata
    }
}

/// Extract the name that a loss label is ultimately about
///
/// Drops any predicate (e.g. `[@pub-id-type='pmid']`), takes the final path or
/// property segment, and removes any attribute marker.
fn leaf_name(label: &str) -> &str {
    let label = label.split('[').next().unwrap_or(label);
    let segment = label.rsplit(['/', '.']).next().unwrap_or(label);
    segment.trim_start_matches('@')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_decode_paths() {
        assert_eq!(
            classify("//article/back/ref-list/ref/element-citation/pub-id[@pub-id-type='pmid']"),
            LossCategory::LinkOrIdentifier
        );
        assert_eq!(
            classify("//article/body/sec/p/named-content"),
            LossCategory::Content
        );
        assert_eq!(
            classify("//article/body/sec/p/text()"),
            LossCategory::Content
        );
        assert_eq!(
            classify("//article/front/article-meta/@specific-use"),
            LossCategory::Presentation
        );
        assert_eq!(
            classify("//article/front/article-meta/permissions"),
            LossCategory::Metadata
        );
        assert_eq!(
            classify("//article/back/ref-list/ref/@id"),
            LossCategory::LinkOrIdentifier
        );
    }

    #[test]
    fn classifies_encode_properties() {
        assert_eq!(
            classify("Article.identifiers"),
            LossCategory::LinkOrIdentifier
        );
        assert_eq!(classify("Article.licenses"), LossCategory::Metadata);
        assert_eq!(
            classify("Reference.isPartOf.doi"),
            LossCategory::LinkOrIdentifier
        );
        assert_eq!(classify("Table.provenance"), LossCategory::Metadata);
        assert_eq!(classify("Supplement.caption"), LossCategory::Content);
    }
}
