use std::fmt::Display;

use crate::{
    AuthorRole, AuthorRoleAuthor, AuthorRoleName, Organization, Person, SoftwareApplication, Thing,
    ThingOptions, Timestamp,
};

impl AuthorRole {
    /// Create an author role for an anonymous author
    pub fn anon(role_name: AuthorRoleName) -> Self {
        Self {
            author: AuthorRoleAuthor::Thing(Thing {
                options: Box::new(ThingOptions {
                    name: Some("anon".to_string()),
                    ..Default::default()
                }),
                ..Default::default()
            }),
            role_name,
            ..Default::default()
        }
    }

    /// Create an author role for a person
    pub fn person(person: Person, role_name: AuthorRoleName) -> Self {
        Self {
            author: AuthorRoleAuthor::Person(person),
            role_name,
            ..Default::default()
        }
    }

    /// Create an author role for an organization
    pub fn org(org: Organization, role_name: AuthorRoleName) -> Self {
        Self {
            author: AuthorRoleAuthor::Organization(org),
            role_name,
            ..Default::default()
        }
    }

    /// Create an author role for a software application
    pub fn software(software: SoftwareApplication, role_name: AuthorRoleName) -> Self {
        Self {
            author: AuthorRoleAuthor::SoftwareApplication(software),
            role_name,
            ..Default::default()
        }
    }

    /// Set the format of the author role
    pub fn format<F>(&mut self, format: F)
    where
        F: Display,
    {
        self.format = Some(format.to_string());
    }

    /// Set the last modified timestamp of the author role to now
    pub fn now(&mut self) {
        self.last_modified = Some(Timestamp::now());
    }
}
