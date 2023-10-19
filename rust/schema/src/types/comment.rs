// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::creative_work_type::CreativeWorkType;
use super::creative_work_type_or_text::CreativeWorkTypeOrText;
use super::date::Date;
use super::grant_or_monetary_grant::GrantOrMonetaryGrant;
use super::image_object::ImageObject;
use super::inline::Inline;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::person_or_organization_or_software_application::PersonOrOrganizationOrSoftwareApplication;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::text::Text;
use super::thing_type::ThingType;

/// A comment on an item, e.g on a `Article` or `SoftwareSourceCode`.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct Comment {
    /// The type of this item.
    pub r#type: MustBe!("Comment"),

    /// The identifier for this item.
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// The authors of the `CreativeWork`.
    pub authors: Option<Vec<PersonOrOrganization>>,

    /// Date of first publication.
    pub date_published: Option<Date>,

    /// Content of the comment, usually one or more paragraphs.
    #[strip(types)]
    pub content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<CommentOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct CommentOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Text>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObject>>,

    /// The name of the item.
    pub name: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The subject matter of the content.
    pub about: Option<Vec<ThingType>>,

    /// A a short description that summarizes a `CreativeWork`.
    #[strip(types)]
    pub r#abstract: Option<Vec<Block>>,

    /// A secondary contributor to the `CreativeWork`.
    pub contributors: Option<Vec<PersonOrOrganizationOrSoftwareApplication>>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// The maintainers of the `CreativeWork`.
    pub maintainers: Option<Vec<PersonOrOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// People or organizations that funded the `CreativeWork`.
    pub funders: Option<Vec<PersonOrOrganization>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    pub funded_by: Option<Vec<GrantOrMonetaryGrant>>,

    /// Genre of the creative work, broadcast channel or group.
    pub genre: Option<Vec<String>>,

    /// Keywords or tags used to describe this content.
    /// Multiple entries in a keywords list are typically delimited by commas.
    pub keywords: Option<Vec<String>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    pub is_part_of: Option<CreativeWorkType>,

    /// License documents that applies to this content, typically indicated by URL.
    pub licenses: Option<Vec<CreativeWorkTypeOrText>>,

    /// Elements of the collection which can be a variety of different elements,
    /// such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkType>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrOrganization>,

    /// References to other creative works, such as another publication,
    /// web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypeOrText>>,

    /// The textual content of this creative work.
    pub text: Option<Text>,

    /// The title of the creative work.
    #[strip(types)]
    pub title: Option<Vec<Inline>>,

    /// The version of the creative work.
    pub version: Option<StringOrNumber>,

    /// The parent comment of this comment.
    pub parent_item: Option<Comment>,

    /// The part or facet of the item that is being commented on.
    pub comment_aspect: Option<String>,
}

impl Comment {
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
