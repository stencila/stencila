// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::image_object::ImageObject;
use super::number::Number;
use super::person_or_organization::PersonOrOrganization;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::text::Text;
use super::thing::Thing;

/// A monetary grant.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "MonetaryGrant")]
pub struct MonetaryGrant {
    /// The type of this item.
    pub r#type: MustBe!("MonetaryGrant"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    #[markdown(flatten)]
    pub options: Box<MonetaryGrantOptions>,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec, WriteNode, ReadNode)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct MonetaryGrantOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "optional_csv_or_array")]
    #[strip(metadata)]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    pub description: Option<Text>,

    /// Any kind of identifier for any kind of Thing.
    #[serde(alias = "identifier")]
    #[serde_as(deserialize_as = "Option<OneOrMany<_, PreferMany>>")]
    #[strip(metadata)]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[serde(alias = "image")]
    #[serde_as(deserialize_as = "Option<OneOrMany<_, PreferMany>>")]
    #[strip(metadata)]
    pub images: Option<Vec<ImageObject>>,

    /// The name of the item.
    #[strip(metadata)]
    pub name: Option<String>,

    /// The URL of the item.
    #[strip(metadata)]
    pub url: Option<String>,

    /// Indicates an item funded or sponsored through a Grant.
    #[serde(alias = "funded-items", alias = "funded_items", alias = "fundedItem", alias = "funded-item", alias = "funded_item")]
    #[serde_as(deserialize_as = "Option<OneOrMany<_, PreferMany>>")]
    pub funded_items: Option<Vec<Thing>>,

    /// A person or organization that supports a thing through a pledge, promise, or financial contribution.
    #[serde(alias = "sponsor")]
    #[serde_as(deserialize_as = "Option<OneOrMany<_, PreferMany>>")]
    pub sponsors: Option<Vec<PersonOrOrganization>>,

    /// The amount of money.
    pub amounts: Option<Number>,

    /// A person or organization that supports (sponsors) something through some kind of financial contribution.
    #[serde(alias = "funder")]
    #[serde_as(deserialize_as = "Option<OneOrMany<_, PreferMany>>")]
    pub funders: Option<Vec<PersonOrOrganization>>,
}

impl MonetaryGrant {
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
