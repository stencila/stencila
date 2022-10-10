use std::path::Path;

use common::{async_trait::async_trait, eyre::Result};
use graph_triples::{
    resources::{self, ResourceDigest},
    Relation, ResourceInfo,
};

use node_address::Address;
use node_patch::diff_address;
use node_transform::Transform;
use path_utils::merge;
use stencila_schema::{BlockContent, Include, InlineContent, Paragraph};

use crate::{
    assert_id,
    executable::{AssembleContext, CompileContext, Executable},
    register_id,
};

fn include_digest(include: &Include, path: &Path) -> ResourceDigest {
    let mut content_string = include.source.clone();
    if let Some(media_type) = include.media_type.as_deref() {
        content_string.push_str(media_type);
    }
    if let Some(select) = include.select.as_deref() {
        content_string.push_str(select);
    }

    let mut digest = ResourceDigest::from_strings(&content_string, None);
    let dependency_digest =
        ResourceDigest::from_path(path, include.media_type.as_ref().map(|str| str.as_str()));
    digest.dependencies_digest = dependency_digest.content_digest;

    digest
}

#[async_trait]
impl Executable for Include {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        register_id!("in", self, address, context);

        let path = merge(&context.path, &self.source);
        let digest = include_digest(self, &path);

        let should_decode = self.content.is_none()
            || match self.compile_digest.as_deref() {
                Some(compile_digest) => digest.to_cord() != *compile_digest,
                None => true,
            };
        if should_decode {
            // Decode block content from the source
            let mut content = match codecs::from_path(
                &path,
                self.media_type
                    .as_ref()
                    .map(|media_type| media_type.as_str()),
                None,
            )
            .await
            {
                Ok(content) => Some(content.to_blocks()),
                Err(error) => Some(vec![BlockContent::Paragraph(Paragraph {
                    content: vec![InlineContent::String(error.to_string())],
                    ..Default::default()
                })]),
            };

            // Assemble the content as well
            content
                .assemble(&mut address.add_name("content"), context)
                .await?;

            // Generate a patch for changes to self
            let mut patch = diff_address(
                address.clone(),
                self,
                &Include {
                    content,
                    ..self.to_owned()
                },
            );
            patch.remove_overwrite_derived();
            context.patches.push(patch)
        }

        Ok(())
    }

    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        // Add resource relations to the context
        let path = merge(&context.path, &self.source);
        let digest = include_digest(self, &path);
        let resource = resources::node(&context.path, id, "Include");
        let object = resources::file(&path);
        let relations = vec![(Relation::Includes, object)];
        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            self.execute_auto.clone(),
            Some(true), // Never has side effects
            Some(digest),
            None,
            None,
        );
        context.resource_infos.push(resource_info);

        // Compile any included content
        if let Some(content) = &self.content {
            content.compile(context).await?
        }

        Ok(())
    }
}
