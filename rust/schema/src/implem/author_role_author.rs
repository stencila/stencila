use crate::{prelude::*, AuthorRoleAuthor};

impl AuthorRoleAuthor {
    pub fn node_type(&self) -> NodeType {
        match self {
            AuthorRoleAuthor::Person(person) => person.node_type(),
            AuthorRoleAuthor::Organization(org) => org.node_type(),
            AuthorRoleAuthor::SoftwareApplication(software) => software.node_type(),
            AuthorRoleAuthor::Thing(thing) => thing.node_type(),
        }
    }

    pub fn node_id(&self) -> NodeId {
        match self {
            AuthorRoleAuthor::Person(person) => person.node_id(),
            AuthorRoleAuthor::Organization(org) => org.node_id(),
            AuthorRoleAuthor::SoftwareApplication(software) => software.node_id(),
            AuthorRoleAuthor::Thing(thing) => thing.node_id(),
        }
    }
}
