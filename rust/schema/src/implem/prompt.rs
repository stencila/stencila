use crate::{
    prelude::*, AuthorRole, AuthorRoleAuthor, AuthorRoleName, Prompt, SoftwareApplication,
};

impl From<Prompt> for AuthorRole {
    fn from(prompt: Prompt) -> Self {
        AuthorRole {
            role_name: AuthorRoleName::Prompter,
            author: AuthorRoleAuthor::SoftwareApplication(prompt.into()),
            ..Default::default()
        }
    }
}

impl From<Prompt> for SoftwareApplication {
    fn from(prompt: Prompt) -> Self {
        SoftwareApplication {
            id: prompt.id,
            name: prompt.name,
            version: Some(prompt.version),
            ..Default::default()
        }
    }
}

impl MarkdownCodec for Prompt {
    fn to_markdown(&self, context: &mut MarkdownEncodeContext) {
        context.enter_node(self.node_type(), self.node_id());

        if let Some(yaml) = &self.frontmatter {
            if !yaml.is_empty() {
                context.push_prop_fn(NodeProperty::Frontmatter, |context| {
                    context.push_str("---\n");
                    context.push_str(yaml);
                    context.push_str("\n---\n\n");
                });
            }
        }

        context.push_prop_fn(NodeProperty::Content, |context| {
            self.content.to_markdown(context)
        });

        context.append_footnotes();

        context.exit_node_final();
    }
}
