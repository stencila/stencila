//! Generated file, do not edit

use crate::prelude::*;

use super::number::Number;
use super::person_or_organization::PersonOrOrganization;
use super::thing::Thing;

/// A monetary grant.
#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct MonetaryGrant {
    /// Non-core optional fields
    #[serde(flatten)]
    options: Box<MonetaryGrantOptions>,
}

#[derive(Debug, Defaults, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct MonetaryGrantOptions {
    /// Indicates an item funded or sponsored through a Grant.
    funded_items: Option<Vec<Thing>>,

    /// A person or organization that supports a thing through a pledge, promise, or financial contribution.
    sponsors: Option<Vec<PersonOrOrganization>>,

    /// The amount of money.
    amounts: Option<Number>,

    /// A person or organization that supports (sponsors) something through some kind of financial contribution.
    funders: Option<Vec<PersonOrOrganization>>,
}
