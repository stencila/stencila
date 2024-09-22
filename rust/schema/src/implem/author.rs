use std::str::FromStr;

use crate::{prelude::*, Author, AuthorRole, AuthorRoleAuthor, AuthorRoleName, Person};

#[derive(Debug, Clone, Copy)]
pub enum AuthorType {
    Human,
    Machine,
}

impl From<&Author> for AuthorType {
    fn from(value: &Author) -> Self {
        match value {
            Author::SoftwareApplication(..) => Self::Machine,
            _ => Self::Human,
        }
    }
}

impl From<&AuthorRoleAuthor> for AuthorType {
    fn from(value: &AuthorRoleAuthor) -> Self {
        match value {
            AuthorRoleAuthor::SoftwareApplication(..) => Self::Machine,
            _ => Self::Human,
        }
    }
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
     * Use `into_author_role_same` when this is not desired.
     */
    pub fn into_author_role(self, role_name: AuthorRoleName) -> AuthorRole {
        match self {
            Author::Person(person) => AuthorRole::person(person, role_name),
            Author::Organization(org) => AuthorRole::org(org, role_name),
            Author::SoftwareApplication(software) => AuthorRole::software(software, role_name),
            Author::AuthorRole(role) => AuthorRole { role_name, ..role },
        }
    }

    /**
     * Create an [`AuthorRole`] from an author, leaving an existing [`AuthorRole`] unchanged
     */
    pub fn into_author_role_same(self, role_name: AuthorRoleName) -> AuthorRole {
        match self {
            Author::Person(person) => AuthorRole::person(person, role_name),
            Author::Organization(org) => AuthorRole::org(org, role_name),
            Author::SoftwareApplication(software) => AuthorRole::software(software, role_name),
            Author::AuthorRole(role) => role,
        }
    }
}
