use std::str::FromStr;

use crate::{prelude::*, Person, PersonOrOrganization};

impl FromStr for PersonOrOrganization {
    type Err = ErrReport;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Person::from_str(string).map(PersonOrOrganization::Person)
    }
}
