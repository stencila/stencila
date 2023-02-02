//! Generated file, do not edit

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
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::thing_type::ThingType;

/// An article, including news and scholarly articles.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct Article {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("Article"),

    /// The identifier for this item
    id: Option<String>,

    /// A description of the item.
    description: Option<Vec<Block>>,

    /// The authors of this creative work.
    authors: Option<Vec<PersonOrOrganization>>,

    /// The structured content of this creative work c.f. property `text`.
    content: Vec<Block>,

    /// Date/time of creation.
    date_created: Option<Date>,

    /// Date/time that work was received.
    date_received: Option<Date>,

    /// Date/time of acceptance.
    date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    date_modified: Option<Date>,

    /// Date of first publication.
    date_published: Option<Date>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    keywords: Option<Vec<String>>,

    /// The title of the creative work.
    title: Option<Vec<Inline>>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<ArticleOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct ArticleOptions {
    /// Alternate names (aliases) for the item.
    alternate_names: Option<Vec<String>>,

    /// Any kind of identifier for any kind of Thing.
    identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    name: Option<String>,

    /// The URL of the item.
    url: Option<String>,

    /// The subject matter of the content.
    about: Option<Vec<ThingType>>,

    /// Comments about this creative work.
    comments: Option<Vec<Comment>>,

    /// People who edited the `CreativeWork`.
    editors: Option<Vec<Person>>,

    /// People or organizations that funded the `CreativeWork`.
    funders: Option<Vec<PersonOrOrganization>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    funded_by: Option<Vec<GrantOrMonetaryGrant>>,

    /// Genre of the creative work, broadcast channel or group.
    genre: Option<Vec<String>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    is_part_of: Option<Box<CreativeWorkType>>,

    /// License documents that applies to this content, typically indicated by URL.
    licenses: Option<Vec<CreativeWorkTypeOrString>>,

    /// The people or organizations who maintain this CreativeWork.
    maintainers: Option<Vec<PersonOrOrganization>>,

    /// Elements of the collection which can be a variety of different elements, such as Articles, Datatables, Tables and more.
    parts: Option<Vec<CreativeWorkType>>,

    /// A publisher of the CreativeWork.
    publisher: Option<PersonOrOrganization>,

    /// References to other creative works, such as another publication, web page, scholarly article, etc.
    references: Option<Vec<CreativeWorkTypeOrString>>,

    /// The textual content of this creative work.
    text: Option<String>,

    /// The version of the creative work.
    version: Option<StringOrNumber>,

    /// The page on which the article starts; for example "135" or "xiii".
    page_start: Option<IntegerOrString>,

    /// The page on which the article ends; for example "138" or "xvi".
    page_end: Option<IntegerOrString>,

    /// Any description of pages that is not separated into pageStart and pageEnd; for example, "1-6, 9, 55".
    pagination: Option<String>,
}
