use crate::{Prompt, AuthorRole, AuthorRoleAuthor, AuthorRoleName, SoftwareApplication};

impl From<Prompt> for AuthorRole {
    fn from(val: Prompt) -> Self {
        AuthorRole {
            role_name: AuthorRoleName::Prompter,
            author: AuthorRoleAuthor::SoftwareApplication(val.into()),
            ..Default::default()
        }
    }
}

impl From<Prompt> for SoftwareApplication {
    fn from(val: Prompt) -> Self {
        SoftwareApplication {
            id: val.id,
            name: val.name,
            version: Some(val.version),
            ..Default::default()
        }
    }
}
