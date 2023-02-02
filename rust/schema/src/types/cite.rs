//! Generated file, do not edit

use crate::prelude::*;

use super::citation_intent::CitationIntent;
use super::citation_mode::CitationMode;
use super::inline::Inline;
use super::integer_or_string::IntegerOrString;
use super::string::String;

/// A reference to a CreativeWork that is cited in another CreativeWork.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct Cite {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("Cite"),

    /// The identifier for this item
    id: Option<String>,

    /// The target of the citation (URL or reference ID).
    target: String,

    /// Determines how the citation is shown within the surrounding text.
    citation_mode: CitationMode,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<CiteOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct CiteOptions {
    /// The type/s of the citation, both factually and rhetorically.
    citation_intent: Option<Vec<CitationIntent>>,

    /// Optional structured content/text of this citation.
    content: Option<Vec<Inline>>,

    /// The page on which the work starts; for example "135" or "xiii".
    page_start: Option<IntegerOrString>,

    /// The page on which the work ends; for example "138" or "xvi".
    page_end: Option<IntegerOrString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    pagination: Option<String>,

    /// Text to show before the citation.
    citation_prefix: Option<String>,

    /// Text to show after the citation.
    citation_suffix: Option<String>,
}
