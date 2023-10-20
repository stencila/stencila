// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::block::Block;
use super::comment::Comment;
use super::creative_work_type::CreativeWorkType;
use super::creative_work_type_or_text::CreativeWorkTypeOrText;
use super::date::Date;
use super::grant_or_monetary_grant::GrantOrMonetaryGrant;
use super::inline::Inline;
use super::number::Number;
use super::person::Person;
use super::person_or_organization::PersonOrOrganization;
use super::person_or_organization_or_software_application::PersonOrOrganizationOrSoftwareApplication;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::string_or_number::StringOrNumber;
use super::text::Text;
use super::thing_type::ThingType;

/// An image file.
#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
#[derive(derive_more::Display)]
#[display(fmt = "ImageObject")]
#[html(elem = "img", custom)]
#[jats(elem = "inline-graphic", special)]
#[markdown(format = "![]({content_url})")]
pub struct ImageObject {
    /// The type of this item.
    #[cfg_attr(feature = "proptest", proptest(value = "Default::default()"))]
    pub r#type: MustBe!("ImageObject"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// URL for the actual bytes of the media object, for example the image file or video file.
    #[cfg_attr(feature = "proptest-min", proptest(value = r#"String::from("https://example.org/image.png")"#))]
    #[cfg_attr(feature = "proptest-low", proptest(regex = r#"https://\w+\.\w+/\w+\.png"#))]
    #[cfg_attr(feature = "proptest-high", proptest(regex = r#"[a-zA-Z0-9]{1,100}"#))]
    #[cfg_attr(feature = "proptest-max", proptest(strategy = r#"String::arbitrary()"#))]
    #[html(attr = "src")]
    pub content_url: String,

    /// IANA media type (MIME type).
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub media_type: Option<String>,

    /// The caption for this image.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    #[html(attr = "alt")]
    pub caption: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<ImageObjectOptions>,
}

#[skip_serializing_none]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[cfg_attr(feature = "proptest", derive(Arbitrary))]
pub struct ImageObjectOptions {
    /// Alternate names (aliases) for the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub description: Option<Text>,

    /// Any kind of identifier for any kind of Thing.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub images: Option<Vec<ImageObject>>,

    /// The name of the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub name: Option<String>,

    /// The URL of the item.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub url: Option<String>,

    /// The subject matter of the content.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub about: Option<Vec<ThingType>>,

    /// A a short description that summarizes a `CreativeWork`.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub r#abstract: Option<Vec<Block>>,

    /// The authors of the `CreativeWork`.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub authors: Option<Vec<PersonOrOrganization>>,

    /// A secondary contributor to the `CreativeWork`.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub contributors: Option<Vec<PersonOrOrganizationOrSoftwareApplication>>,

    /// People who edited the `CreativeWork`.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub editors: Option<Vec<Person>>,

    /// The maintainers of the `CreativeWork`.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub maintainers: Option<Vec<PersonOrOrganization>>,

    /// Comments about this creative work.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub comments: Option<Vec<Comment>>,

    /// Date/time of creation.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_created: Option<Date>,

    /// Date/time that work was received.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_received: Option<Date>,

    /// Date/time of acceptance.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_accepted: Option<Date>,

    /// Date/time of most recent modification.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_modified: Option<Date>,

    /// Date of first publication.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub date_published: Option<Date>,

    /// People or organizations that funded the `CreativeWork`.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub funders: Option<Vec<PersonOrOrganization>>,

    /// Grants that funded the `CreativeWork`; reverse of `fundedItems`.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub funded_by: Option<Vec<GrantOrMonetaryGrant>>,

    /// Genre of the creative work, broadcast channel or group.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub genre: Option<Vec<String>>,

    /// Keywords or tags used to describe this content.
    /// Multiple entries in a keywords list are typically delimited by commas.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub keywords: Option<Vec<String>>,

    /// An item or other CreativeWork that this CreativeWork is a part of.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub is_part_of: Option<CreativeWorkType>,

    /// License documents that applies to this content, typically indicated by URL.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub licenses: Option<Vec<CreativeWorkTypeOrText>>,

    /// Elements of the collection which can be a variety of different elements,
    /// such as Articles, Datatables, Tables and more.
    #[strip(content)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub parts: Option<Vec<CreativeWorkType>>,

    /// A publisher of the CreativeWork.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub publisher: Option<PersonOrOrganization>,

    /// References to other creative works, such as another publication,
    /// web page, scholarly article, etc.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub references: Option<Vec<CreativeWorkTypeOrText>>,

    /// The textual content of this creative work.
    #[strip(content)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub text: Option<Text>,

    /// The title of the creative work.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub title: Option<Vec<Inline>>,

    /// The version of the creative work.
    #[strip(metadata)]
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub version: Option<StringOrNumber>,

    /// Bitrate in megabits per second (Mbit/s, Mb/s, Mbps).
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub bitrate: Option<Number>,

    /// File size in megabits (Mbit, Mb).
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub content_size: Option<Number>,

    /// URL that can be used to embed the media on a web page via a specific media player.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
    pub embed_url: Option<String>,

    /// Thumbnail image of this image.
    #[cfg_attr(feature = "proptest", proptest(value = "None"))]
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
