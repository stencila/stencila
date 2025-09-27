use std::fmt::Display;

use crate::{
    Author, AuthorRole, AuthorRoleAuthor, AuthorRoleName, Organization, Person,
    SoftwareApplication, StringOrNumber, Thing, ThingOptions, Timestamp, prelude::*,
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

    /// Get the name of the author of an [`AuthorRole`]
    pub fn name(&self) -> String {
        self.author.name()
    }

    /// Get the short name of the author of an [`AuthorRole`]
    pub fn short_name(&self) -> String {
        self.author.short_name()
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

        if let Some(format) = &self.format {
            context.push_attr("format", format);
        }

        if let Some(last_modified) = &self.last_modified {
            Timestamp::to_dom_attr("last-modified", last_modified, context);
        }

        let (node_type, name) = match &self.author {
            AuthorRoleAuthor::Person(person) => (person.node_type(), person.name()),
            AuthorRoleAuthor::Organization(org) => (org.node_type(), org.name()),
            AuthorRoleAuthor::SoftwareApplication(app) => (app.node_type(), app.name()),
            AuthorRoleAuthor::Thing(thing) => (thing.node_type(), thing.name()),
        };
        context
            .push_attr("type", &node_type.to_string())
            .push_attr("name", &name);

        if let AuthorRoleAuthor::Person(person) = &self.author {
            if let Some(affs) = &person.affiliations.as_ref() {
                let details = affs.iter().filter_map(|org| org.name.clone()).join(", ");
                context.push_attr("details", &details);
            }
        } else if let AuthorRoleAuthor::SoftwareApplication(app) = &self.author {
            if let Some(id) = &app.id {
                context.push_attr("_id", id);
            }

            let mut details = String::new();

            if let Some(version) = &app.options.software_version.clone().or_else(|| {
                app.version.as_ref().map(|version| match version {
                    StringOrNumber::String(string) => string.clone(),
                    StringOrNumber::Number(number) => number.to_string(),
                })
            }) {
                details.push('v');
                details.push_str(version);
            }

            if let Some(url) = &app.options.url {
                if !details.is_empty() {
                    details.push(' ')
                };
                details.push_str(url);
            }

            if !details.is_empty() {
                context.push_attr("details", &details);
            }
        }

        context.exit_node();
    }
}
