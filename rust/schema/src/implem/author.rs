use std::str::FromStr;

use crate::{prelude::*, Author, AuthorRole, AuthorRoleName, Person};

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

impl Author {
    /**
     * Create an [`AuthorRole`] from an author
     *
     * Note that this will intentionally change the `role_name` of any existing [`AuthorRole`].
     */
    pub fn into_author_role(self, role_name: AuthorRoleName) -> AuthorRole {
        match self {
            Author::Person(person) => AuthorRole::person(person, role_name),
            Author::Organization(org) => AuthorRole::org(org, role_name),
            Author::SoftwareApplication(software) => AuthorRole::software(software, role_name),
            Author::AuthorRole(role) => AuthorRole { role_name, ..role },
        }
    }
}
