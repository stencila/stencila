//! Generated file, do not edit

use crate::prelude::*;

use super::block::Block;
use super::image_object_or_string::ImageObjectOrString;
use super::number::Number;
use super::person_or_organization::PersonOrOrganization;
use super::property_value_or_string::PropertyValueOrString;
use super::string::String;
use super::thing::Thing;

/// A monetary grant.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct MonetaryGrant {
    /// The type of this item
    #[autosurgeon(with = "autosurgeon_must_be")]
    pub r#type: MustBe!("MonetaryGrant"),

    /// The identifier for this item
    #[key]
    pub id: Option<String>,

    /// Non-core optional fields
    #[serde(flatten)]
    pub options: Box<MonetaryGrantOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(rename_all = "camelCase", crate = "common::serde")]
pub struct MonetaryGrantOptions {
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

    /// Indicates an item funded or sponsored through a Grant.
    pub funded_items: Option<Vec<Thing>>,

    /// A person or organization that supports a thing through a pledge, promise, or financial contribution.
    pub sponsors: Option<Vec<PersonOrOrganization>>,

    /// The amount of money.
    pub amounts: Option<Number>,

    /// A person or organization that supports (sponsors) something through some kind of financial contribution.
    pub funders: Option<Vec<PersonOrOrganization>>,
}
