use crate::prelude::*;

use super::organization::Organization;
use super::person::Person;

/// [`Organization`] or [`Person`]
#[derive(Debug, Clone, PartialEq, Display, Serialize, Deserialize, Read, Write, ToHtml)]
#[serde(untagged, crate = "common::serde")]

pub enum OrganizationOrPerson {
    Organization(Organization),
    Person(Person),
}
