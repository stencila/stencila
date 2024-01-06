use std::str::FromStr;

use crate::{prelude::*, Author, Person};

impl FromStr for Author {
    type Err = ErrReport;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Person::from_str(string).map(Author::Person)
    }
}
