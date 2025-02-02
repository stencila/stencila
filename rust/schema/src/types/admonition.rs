// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::admonition_type::AdmonitionType;
use super::author::Author;
use super::block::Block;
use super::boolean::Boolean;
use super::inline::Inline;
use super::provenance_count::ProvenanceCount;
use super::string::String;

/// A admonition within a document.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, PatchNode, DomCodec, HtmlCodec, JatsCodec, LatexCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display("Admonition")]
#[patch(authors_on = "self")]
#[html(elem = "aside")]
#[jats(elem = "boxed-text")]
pub struct Admonition {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Admonition"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The type of admonition.
    #[serde(alias = "admonition-type", alias = "admonition_type")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"AdmonitionType::Info"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"AdmonitionType::arbitrary()"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"AdmonitionType::arbitrary()"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"AdmonitionType::arbitrary()"#))]
    #[jats(attr = "content-type")]
    pub admonition_type: AdmonitionType,

    /// The title of the admonition.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(vec_inlines_non_recursive(2))"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(vec_inlines_non_recursive(4))"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(vec_inlines_non_recursive(4))"#))]
    #[dom(elem = "p")]
    #[jats(elem = "caption")]
    pub title: Option<Vec<Inline>>,

    /// Whether the admonition is folded.
    #[serde(alias = "is-folded", alias = "is_folded")]
    #[patch(format = "md", format = "smd", format = "myst", format = "ipynb", format = "qmd")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"None"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"option::of(bool::arbitrary())"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"option::of(bool::arbitrary())"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"option::of(bool::arbitrary())"#))]
    #[jats(attr = "is-folded")]
    pub is_folded: Option<Boolean>,

    /// The content within the section.
    #[serde(deserialize_with = "one_or_many")]
    #[walk]
    #[patch(format = "all")]
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"vec![p([t("Admonition content")])]"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_paragraphs(2)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_blocks_non_recursive(4)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_blocks_non_recursive(4)"#))]
    #[dom(elem = "aside")]
    pub content: Vec<Block>,

    /// The authors of the admonition.
    #[serde(alias = "author")]
    #[serde(default, deserialize_with = "option_one_or_many_string_or_object")]
    #[strip(authors)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "div")]
    pub authors: Option<Vec<Author>>,

    /// A summary of the provenance of the content within the admonition.
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(provenance)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[dom(elem = "div")]
    pub provenance: Option<Vec<ProvenanceCount>>,

    /// A unique identifier for a node within a document
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    #[serde(skip)]
    pub uid: NodeUid
}

impl Admonition {
    const NICK: [u8; 3] = [97, 100, 109];
    
    pub fn node_type(&self) -> NodeType {
        NodeType::Admonition
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(&Self::NICK, &self.uid)
    }
    
    pub fn new(admonition_type: AdmonitionType, content: Vec<Block>) -> Self {
        Self {
            admonition_type,
            content,
            ..Default::default()
        }
    }
}
