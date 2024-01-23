// Generated file; do not edit. See `schema-gen` crate.

use crate::prelude::*;

use super::image_object::ImageObject;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::text::Text;

/// A physical mailing address.
#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
#[derive(derive_more::Display)]
#[display(fmt = "PostalAddress")]
#[jats(elem = "address")]
pub struct PostalAddress {
    /// The type of this item.
    pub r#type: MustBe!("PostalAddress"),

    /// The identifier for this item.
    #[strip(metadata)]
    #[html(attr = "id")]
    pub id: Option<String>,

    /// Email address for correspondence.
    #[serde(alias = "email")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    pub emails: Option<Vec<String>>,

    /// Telephone numbers for the contact point.
    #[serde(alias = "telephone", alias = "telephone-numbers", alias = "telephone_numbers", alias = "telephoneNumber", alias = "telephone-number", alias = "telephone_number")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    pub telephone_numbers: Option<Vec<String>>,

    /// The street address.
    #[serde(alias = "street-address", alias = "street_address")]
    pub street_address: Option<String>,

    /// The locality in which the street address is, and which is in the region.
    #[serde(alias = "address-locality", alias = "address_locality")]
    pub address_locality: Option<String>,

    /// The region in which the locality is, and which is in the country.
    #[serde(alias = "address-region", alias = "address_region")]
    pub address_region: Option<String>,

    /// The postal code.
    #[serde(alias = "postal-code", alias = "postal_code")]
    pub postal_code: Option<String>,

    /// The country.
    #[serde(alias = "address-country", alias = "address_country")]
    pub address_country: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    #[html(flatten)]
    #[jats(flatten)]
    pub options: Box<PostalAddressOptions>,

    /// A unique identifier for a node within a document
    
    #[serde(skip)]
    pub uid: NodeUid
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, SmartDefault, Clone, PartialEq, Serialize, Deserialize, StripNode, WalkNode, WriteNode, ReadNode, HtmlCodec, JatsCodec, MarkdownCodec, TextCodec)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct PostalAddressOptions {
    /// Alternate names (aliases) for the item.
    #[serde(alias = "alternate-names", alias = "alternate_names", alias = "alternateName", alias = "alternate-name", alias = "alternate_name")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    #[strip(metadata)]
    pub alternate_names: Option<Vec<String>>,

    /// A description of the item.
    #[strip(metadata)]
    pub description: Option<Text>,

    /// Any kind of identifier for any kind of Thing.
    #[serde(alias = "identifier")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    pub identifiers: Option<Vec<PropertyValueOrString>>,

    /// Images of the item.
    #[serde(alias = "image")]
    #[serde(default, deserialize_with = "option_one_or_many")]
    #[strip(metadata)]
    pub images: Option<Vec<ImageObject>>,

    /// The name of the item.
    #[strip(metadata)]
    pub name: Option<String>,

    /// The URL of the item.
    #[strip(metadata)]
    pub url: Option<String>,

    /// Languages (human not programming) in which it is possible to communicate with the organization/department etc.
    #[serde(alias = "available-languages", alias = "available_languages", alias = "availableLanguage", alias = "available-language", alias = "available_language")]
    #[serde(default, deserialize_with = "option_csv_or_array")]
    pub available_languages: Option<Vec<String>>,

    /// The post office box number.
    #[serde(alias = "post-office-box-number", alias = "post_office_box_number")]
    pub post_office_box_number: Option<String>,
}

impl PostalAddress {
    const NICK: &'static str = "pos";
    
    pub fn node_type(&self) -> NodeType {
        NodeType::PostalAddress
    }

    pub fn node_id(&self) -> NodeId {
        NodeId::new(Self::NICK, &self.uid)
    }
    
    pub fn new() -> Self {
        Self {
            ..Default::default()
        }
    }
}
