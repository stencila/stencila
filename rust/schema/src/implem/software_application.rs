use format::Format;

use crate::{AuthorRole, AuthorRoleName, SoftwareApplication, Timestamp};

impl SoftwareApplication {
    /// Create an [`AuthorRole`] for a software application
    pub fn into_author_role(
        self,
        role_name: AuthorRoleName,
        format: Option<Format>,
        last_modified: Option<Timestamp>,
    ) -> AuthorRole {
        let mut role = AuthorRole::software(self, role_name);
        role.format = format.map(|format| format.name().to_string());
        role.last_modified = last_modified;

        role
    }
}
