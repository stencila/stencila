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
use super::number::Number;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::person_or_organization_or_software_application::PersonOrOrganizationOrSoftwareApplication;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::thing_type::ThingType;

/// An image file.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[html(elem = "img", custom)]
#[jats(elem = "inline-graphic")]
#[markdown(format = "![]({content_url})")]
pub struct ImageObject {
    /// The type of this item
    pub r#type: MustBe!("ImageObject"),

    /// The identifier for this item
    #[strip(id)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// URL for the actual bytes of the media object, for example the image file or video file.
    #[html(attr = "src")]
    #[jats(attr = "xlink:href")]
    pub content_url: String,

    /// IANA media type (MIME type).
    pub media_type: Option<String>,

    /// The caption for this image.
    #[html(attr = "alt")]
    pub caption: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<ImageObjectOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, ReadNode, WriteNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct ImageObjectOptions {
    /// Alternate names (aliases) for the item.
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    pub description: Option<Vec<Block>>,

    /// Any kind of identifier for any kind of Thing.
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    pub images: Option<Vec<ImageObjectOrString>>,

    /// The name of the item.
    pub name: Option<String>,

    /// The URL of the item.
    pub url: Option<String>,

    /// The subject matter of the content.
    pub about: Option<Vec<ThingType>>,

    /// The authors of the `CreativeWork`.
    pub authors: Option<Vec<PersonOrOrganization>>,

    /// A secondary contributor to the `CreativeWork`.
    pub contributors: Option<Vec<PersonOrOrganizationOrSoftwareApplication>>,

    /// People who edited the `CreativeWork`.
    pub editors: Option<Vec<Person>>,

    /// The maintainers of the `CreativeWork`.
    pub maintainers: Option<Vec<PersonOrOrganization>>,

    /// Comments about this creative work.
    pub comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    pub content: Option<Vec<Block>>,

    /// Date/time of creation.
    pub date_created: Option<Date>,

    /// Date/time that work was received.
    pub date_received: Option<Date>,

    /// Date/time of acceptance.
    pub date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    pub date_modified: Option<Date>,

    /// Date of first publication.
    pub date_published: Option<Date>,

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
    pub licenses: Option<Vec<CreativeWorkTypeOrString>>,

    /// Elements of the collection which can be a variety of different elements,
    /// such as Articles, Datatables, Tables and more.
    pub parts: Option<Vec<CreativeWorkType>>,

    /// A publisher of the CreativeWork.
    pub publisher: Option<PersonOrOrganization>,

    /// References to other creative works, such as another publication,
    /// web page, scholarly article, etc.
    pub references: Option<Vec<CreativeWorkTypeOrString>>,

    /// The textual content of this creative work.
    pub text: Option<String>,

    /// The title of the creative work.
    pub title: Option<Vec<Inline>>,

    /// The version of the creative work.
    pub version: Option<StringOrNumber>,

    /// Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
    pub bitrate: Option<Number>,

    /// File size in megabits (Mbit, Mb).
    pub content_size: Option<Number>,

    /// URL that can be used to embed the media on a web page via a specific media player.
    pub embed_url: Option<String>,

    /// Thumbnail image of this image.
    pub thumbnail: Option<ImageObject>,
}

impl ImageObject {
    pub fn new(content_url: String) -> Self {
        Self {
            content_url,
            ..Default::default()
        }
    }
}
