// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::comment::Comment;
use super::creative_work_type::CreativeWorkType;
use super::creative_work_type_or_string::CreativeWorkTypeOrString;
use super::date::Date;
use super::grant_or_monetary_grant::GrantOrMonetaryGrant;
use super::image_object_or_string::ImageObjectOrString;
use super::inline::Inline;
use super::integer_or_string::IntegerOrString;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::person_or_organization_or_software_application::PersonOrOrganizationOrSoftwareApplication;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::thing_type::ThingType;

/// An article, including news and scholarly articles.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[html(elem = "article")]
#[jats(elem = "article", special)]
#[markdown(special)]
pub struct Article {
    /// The type of this item
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("Article"),

    /// The identifier for this item
    #[strip(id)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// A description of the item.
    #[strip(types)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub description: Option<Vec<Block>>,

    /// The authors of the `CreativeWork`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub authors: Option<Vec<PersonOrOrganization>>,

    /// Date/time of creation.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_created: Option<Date>,

    /// Date/time that work was received.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_received: Option<Date>,

    /// Date/time of acceptance.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_modified: Option<Date>,

    /// Date of first publication.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_published: Option<Date>,

    /// Keywords or tags used to describe this content.
    /// Multiple entries in a keywords list are typically delimited by commas.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub keywords: Option<Vec<String>>,

    /// The title of the creative work.
    #[strip(types)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub title: Option<Vec<Inline>>,

    /// The content of the article.
    #[strip(types)]
    #[cfg_attr(feature = "proptest-min", proptest(strategy = r#"vec_blocks(1)"#))]
    #[cfg_attr(feature = "proptest-low", proptest(strategy = r#"vec_blocks(2)"#))]
    #[cfg_attr(feature = "proptest-high", proptest(strategy = r#"vec_blocks(4)"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"vec_blocks(8)"#))]
    pub content: Vec<Block>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<ArticleOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct ArticleOptions {
    /// Alternate names (aliases) for the item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub alternate_names: Option<Vec<String>>,

    /// Any kind of identifier for any kind of Thing.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub name: Option<String>,

    /// The URL of the item.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub url: Option<String>,

    /// The subject matter of the content.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub about: Option<Vec<ThingType>>,

    /// A secondary contributor to the `CreativeWork`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub contributors: Option<Vec<PersonOrOrganizationOrSoftwareApplication>>,

    /// People who edited the `CreativeWork`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub editors: Option<Vec<Person>>,

    /// The maintainers of the `CreativeWork`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub maintainers: Option<Vec<PersonOrOrganization>>,

    /// Comments about this creative work.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub comments: Option<Vec<Comment>>,

    /// People or organizations that funded the `CreativeWork`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub funders: Option<Vec<PersonOrOrganization>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub funded_by: Option<Vec<GrantOrMonetaryGrant>>,

    /// Genre of the creative work, broadcast channel or group.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub genre: Option<Vec<String>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub is_part_of: Option<CreativeWorkType>,

    /// License documents that applies to this content, typically indicated by URL.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub licenses: Option<Vec<CreativeWorkTypeOrString>>,

    /// Elements of the collection which can be a variety of different elements,
    /// such as Articles, Datatables, Tables and more.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub parts: Option<Vec<CreativeWorkType>>,

    /// A publisher of the CreativeWork.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub publisher: Option<PersonOrOrganization>,

    /// References to other creative works, such as another publication,
    /// web page, scholarly article, etc.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub references: Option<Vec<CreativeWorkTypeOrString>>,

    /// The textual content of this creative work.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub text: Option<String>,

    /// The version of the creative work.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub version: Option<StringOrNumber>,

    /// The page on which the article starts; for example "135" or "xiii".
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub page_start: Option<IntegerOrString>,

    /// The page on which the article ends; for example "138" or "xvi".
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub page_end: Option<IntegerOrString>,

    /// Any description of pages that is not separated into pageStart and pageEnd;
    /// for example, "1-6, 9, 55".
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub pagination: Option<String>,
}

impl Article {
    pub fn new(content: Vec<Block>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }
}
