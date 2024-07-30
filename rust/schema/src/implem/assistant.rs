use crate::{Assistant, AuthorRole, AuthorRoleAuthor, AuthorRoleName, SoftwareApplication};

impl Into<AuthorRole> for Assistant {
    fn into(self) -> AuthorRole {
        AuthorRole {
            role_name: AuthorRoleName::Prompter,
            author: AuthorRoleAuthor::SoftwareApplication(self.into()),
            ..Default::default()
        }
    }
}

impl Into<SoftwareApplication> for Assistant {
    fn into(self) -> SoftwareApplication {
        SoftwareApplication {
            name: self.name,
            version: Some(self.version),
            ..Default::default()
        }
    }
}
