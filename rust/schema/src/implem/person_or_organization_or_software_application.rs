use std::str::FromStr;

use crate::{prelude::*, Person, PersonOrOrganizationOrSoftwareApplication};

impl FromStr for PersonOrOrganizationOrSoftwareApplication {
    type Err = ErrReport;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Person::from_str(string).map(PersonOrOrganizationOrSoftwareApplication::Person)
    }
}
