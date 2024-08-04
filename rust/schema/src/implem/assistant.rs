use crate::{Assistant, AuthorRole, AuthorRoleAuthor, AuthorRoleName, SoftwareApplication};

impl From<Assistant> for AuthorRole {
    fn from(val: Assistant) -> Self {
        AuthorRole {
            role_name: AuthorRoleName::Prompter,
            author: AuthorRoleAuthor::SoftwareApplication(val.into()),
            ..Default::default()
        }
    }
}

impl From<Assistant> for SoftwareApplication {
    fn from(val: Assistant) -> Self {
        SoftwareApplication {
            id: val.id,
            name: val.name,
            version: Some(val.version),
            ..Default::default()
        }
    }
}
