// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::citation_intent::CitationIntent;
use super::citation_mode::CitationMode;
use super::inline::Inline;
use super::integer_or_string::IntegerOrString;
use super::string::String;

/// A reference to a CreativeWork that is cited in another CreativeWork.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Cite {
    /// The type of this item
    pub r#type: MustBe!("Cite"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The target of the citation (URL or reference ID).
    pub target: String,

    /// Determines how the citation is shown within the surrounding text.
    pub citation_mode: CitationMode,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<CiteOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, TextCodec, Read, Write)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CiteOptions {
    /// The type/s of the citation, both factually and rhetorically.
    pub citation_intent: Option<Vec<CitationIntent>>,

    /// Optional structured content/text of this citation.
    pub content: Option<Vec<Inline>>,

    /// The page on which the work starts; for example "135" or "xiii".
    pub page_start: Option<IntegerOrString>,

    /// The page on which the work ends; for example "138" or "xvi".
    pub page_end: Option<IntegerOrString>,

    /// Any description of pages that is not separated into pageStart and pageEnd;
    /// for example, "1-6, 9, 55".
    pub pagination: Option<String>,

    /// Text to show before the citation.
    pub citation_prefix: Option<String>,

    /// Text to show after the citation.
    pub citation_suffix: Option<String>,
}

impl Cite {
    pub fn new(target: String, citation_mode: CitationMode) -> Self {
        Self {
            target,
            citation_mode,
            ..Default::default()
        }
    }
}
