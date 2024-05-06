use std::str::FromStr;

use crate::{prelude::*, Author, Person};

#[derive(Clone, Copy)]
pub enum AuthorType {
    Human,
    Machine,
}

impl FromStr for Author {
    type Err = ErrReport;

    fn from_str(string: &str) -> Result<Self, Self::Err> {
        Person::from_str(string).map(Author::Person)
    }
}
