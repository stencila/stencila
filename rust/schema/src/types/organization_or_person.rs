//! Generated file, do not edit

use crate::prelude::*;

use super::organization::Organization;
use super::person::Person;

/// [`Organization`] or [`Person`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(untagged, crate = "common::serde")]

pub enum OrganizationOrPerson {
    Organization(Organization),
    Person(Person),
}
