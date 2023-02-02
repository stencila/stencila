//! Generated file, do not edit

use crate::prelude::*;

use super::organization::Organization;
use super::person::Person;

/// [`Person`] or [`Organization`]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reconcile, Hydrate)]
#[serde(crate = "common::serde")]

pub enum PersonOrOrganization {
    Person(Person),
    Organization(Organization),
}
