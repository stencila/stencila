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
use super::number::Number;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::thing_type::ThingType;

/// An audio file
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct AudioObject {
    /// The type of this item
    r#type: MustBe!("AudioObject"),

    /// The identifier for this item
    id: String,

    /// URL for the actual bytes of the media object, for example the image file or video file.
    content_url: String,

    /// IANA media type (MIME type).
    media_type: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<AudioObjectOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct AudioObjectOptions {
    /// Alternate names (aliases) for the item.
    alternate_names: Option<Vec<String>>,

    /// A description of the item.
    description: Option<Vec<Block>>,

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

    /// The authors of this creative work.
    authors: Option<Vec<PersonOrOrganization>>,

    /// Comments about this creative work.
    comments: Option<Vec<Comment>>,

    /// The structured content of this creative work c.f. property `text`.
    content: Option<Vec<Block>>,

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

    /// People who edited the `CreativeWork`.
    editors: Option<Vec<Person>>,

    /// People or organizations that funded the `CreativeWork`.
    funders: Option<Vec<PersonOrOrganization>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    funded_by: Option<Vec<GrantOrMonetaryGrant>>,

    /// Genre of the creative work, broadcast channel or group.
    genre: Option<Vec<String>>,

    /// Keywords or tags used to describe this content. Multiple entries in a keywords list are typically delimited by commas.
    keywords: Option<Vec<String>>,

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

    /// The title of the creative work.
    title: Option<Vec<Inline>>,

    /// The version of the creative work.
    version: Option<StringOrNumber>,

    /// Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
    bitrate: Option<Number>,

    /// File size in megabits (Mbit, Mb).
    content_size: Option<Number>,

    /// URL that can be used to embed the media on a web page via a specific media player.
    embed_url: Option<String>,

    /// The caption for this audio recording.
    caption: Option<String>,

    /// The transcript of this audio recording.
    transcript: Option<String>,
}
