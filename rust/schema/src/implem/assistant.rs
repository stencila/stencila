use common::inflector::Inflector;

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
        // As done for models, use `name` as the `id` and
        // use title-cased name as the `name`.
        let name = self
            .name
            .rsplit_once('/')
            .map(|(.., name)| name.split_once('-').map_or(name, |(name, ..)| name))
            .unwrap_or(&self.name);
        let name = name.to_title_case();

        SoftwareApplication {
            id: Some(self.name),
            name,
            version: Some(self.version),
            ..Default::default()
        }
    }
}
