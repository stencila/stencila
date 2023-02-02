//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A physical mailing address.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct PostalAddress {
    /// The type of this item
    r#type: MustBe!("PostalAddress"),

    /// The identifier for this item
    id: String,

    /// Email address for correspondence.
    emails: Option<Vec<String>>,

    /// Telephone numbers for the contact point.
    telephone_numbers: Option<Vec<String>>,

    /// The street address.
    street_address: Option<String>,

    /// The locality in which the street address is, and which is in the region.
    address_locality: Option<String>,

    /// The region in which the locality is, and which is in the country.
    address_region: Option<String>,

    /// The postal code.
    postal_code: Option<String>,

    /// The country.
    address_country: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<PostalAddressOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct PostalAddressOptions {
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

    /// Languages (human not programming) in which it is possible to communicate with the organization/department etc.
    available_languages: Option<Vec<String>>,

    /// The post office box number.
    post_office_box_number: Option<String>,
}
