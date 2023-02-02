//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;

/// A contact point, usually within an organization.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct ContactPoint {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    r#type: MustBe!("ContactPoint"),

    /// The identifier for this item
    id: Option<String>,

    /// Email address for correspondence.
    emails: Option<Vec<String>>,

    /// Telephone numbers for the contact point.
    telephone_numbers: Option<Vec<String>>,

    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<ContactPointOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]
pub struct ContactPointOptions {
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
}
