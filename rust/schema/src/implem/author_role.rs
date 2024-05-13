use std::fmt::Display;

use crate::{
    prelude::*, Author, AuthorRole, AuthorRoleAuthor, AuthorRoleName, Organization, Person,
    SoftwareApplication, StringOrNumber, Thing, ThingOptions, Timestamp,
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

    /// Create an `Author` from this author role if possible
    pub fn to_author(&self) -> Option<Author> {
        Some(match &self.author {
            AuthorRoleAuthor::Person(author) => Author::Person(author.clone()),
            AuthorRoleAuthor::Organization(author) => Author::Organization(author.clone()),
            AuthorRoleAuthor::SoftwareApplication(author) => {
                Author::SoftwareApplication(author.clone())
            }
            AuthorRoleAuthor::Thing(..) => return None,
        })
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

impl DomCodec for AuthorRole {
    fn to_dom(&self, context: &mut DomEncodeContext) {
        // Custom implementation to normalize with the
        // other types of authors: Person, Organization and SoftwareApplication
        // to ease front-end implementation of display

        context
            .enter_node(self.node_type(), self.node_id())
            .push_attr("role-name", &self.role_name.to_string());

        if let Some(last_modified) = &self.last_modified {
            Timestamp::to_dom_attr("last-modified", last_modified, context);
        }

        let (node_type, name) = match &self.author {
            AuthorRoleAuthor::Person(person) => {
                let mut name = person
                    .given_names
                    .iter()
                    .flatten()
                    .chain(person.family_names.iter().flatten())
                    .join(" ");
                if name.is_empty() {
                    if let Some(opt_name) = &person.options.name {
                        name = opt_name.clone();
                    }
                }
                if name.is_empty() {
                    name = "Anonymous".to_string();
                }

                (person.node_type(), name)
            }
            AuthorRoleAuthor::Organization(org) => (
                org.node_type(),
                org.options.name.clone().unwrap_or_default(),
            ),
            AuthorRoleAuthor::SoftwareApplication(app) => (app.node_type(), app.name.clone()),
            AuthorRoleAuthor::Thing(thing) => (
                thing.node_type(),
                thing.options.name.clone().unwrap_or_default(),
            ),
        };
        context
            .push_attr("type", &node_type.to_string())
            .push_attr("name", &name);

        if let AuthorRoleAuthor::Person(person) = &self.author {
            if let Some(affs) = &person.affiliations.as_ref() {
                let details = affs
                    .iter()
                    .filter_map(|org| org.options.name.clone())
                    .join(", ");
                context.push_attr("details", &details);
            }
        } else if let AuthorRoleAuthor::SoftwareApplication(app) = &self.author {
            if let Some(id) = &app.id {
                context.push_attr("_id", id);
            }

            if let Some(version) = &app.options.software_version.clone().or_else(|| {
                app.options.version.as_ref().map(|version| match version {
                    StringOrNumber::String(string) => string.clone(),
                    StringOrNumber::Number(number) => number.to_string(),
                })
            }) {
                context.push_attr("details", &["v", version].concat());
            }
        }

        context
            .push_slot_fn("div", "author", |context| self.author.to_dom(context))
            .exit_node();
    }
}
