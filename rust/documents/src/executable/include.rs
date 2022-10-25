use common::{async_trait::async_trait, eyre::Result};
use graph_triples::{
    resources::{self, ResourceDigest},
    Relation, ResourceInfo,
};

use node_address::Address;
use node_patch::diff_id;
use node_transform::Transform;
use path_utils::merge;
use stencila_schema::{CodeError, Include};

use crate::{
    assert_id,
    executable::{AssembleContext, CompileContext, Executable},
    register_id,
};

#[async_trait]
impl Executable for Include {
    /// Assemble an `Include` node
    ///
    /// Registers the node `id`, and decodes the content of `source` into `content`.
    /// The `compile_digest` is used to ensure that this is not done unnecessarily.
    ///
    /// Generates a patch for the change in content (unlike for `compile()` and `execute()`
    /// this is not done automatically for `assemble()` for efficiency reasons).
    ///
    /// Because reading the `source` from disk and calculating a hash is relatively
    /// resource intensive, this is only done during `assemble`, not during `compile`.
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        let id = register_id!("in", self, address, context);

        // Calculate a resource digest using the `source`, `media_type` and `select` properties
        let mut resource_digest = digest_from_properties(self);

        // Add the digest of the content of the source file as the dependencies digest
        let path = merge(&context.path, &self.source);
        let dependency_digest =
            ResourceDigest::from_path(&path, self.media_type.as_ref().map(|str| str.as_str()));
        resource_digest.dependencies_digest = dependency_digest.content_digest;

        // Determine if it is necessary to update content (i.e. has the `compile_digest` changed since last time)
        let should_decode = self.content.is_none()
            || match self.compile_digest.as_deref() {
                Some(compile_digest) => resource_digest.to_cord() != *compile_digest,
                None => true,
            };
        if should_decode {
            // Clone self for patch
            let before = self.clone();

            // Decode block content from the source
            let result = codecs::from_path(
                &path,
                self.media_type
                    .as_ref()
                    .map(|media_type| media_type.as_str()),
                None,
            )
            .await;
            match result {
                Ok(content) => {
                    // Decoding was OK so assemble to content and clear errors
                    let mut content = content.to_blocks();
                    content
                        .assemble(&mut address.add_name("content"), context)
                        .await?;
                    self.content = Some(content);
                    self.errors = None;
                }
                Err(error) => {
                    // Decoding failed so clear content and populate errors
                    self.content = None;
                    self.errors = Some(vec![CodeError {
                        error_message: error.to_string(),
                        ..Default::default()
                    }]);
                }
            };

            // Generate a patch for changes to self
            let mut patch = diff_id(&id, &before, self);
            patch.remove_overwrite_derived();
            context.patches.push(patch)
        }

        Ok(())
    }

    /// Compile an `Include` node
    ///
    /// Declares this node as a resource with a relation to the source file.
    /// Note that there is no need to compile the `content` since any executable
    /// nodes within it should have been registered in `assemble`.
    ///
    /// TODO: If there is a change in the compile digest (ie. `source`, `media_type`
    /// or `select` have changed) then trigger an `assemble` of the document.
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        let path = merge(&context.path, &self.source);
        let resource = resources::node(&context.path, id, "Include");
        let object = resources::file(&path);
        let relations = vec![(Relation::Includes, object)];
        let execute_pure = Some(true); // Never has side effects
        let compile_digest = self
            .compile_digest
            .as_ref()
            .map(|cord| ResourceDigest::from_cord(cord));
        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            self.execute_auto.clone(),
            execute_pure,
            None,
            compile_digest,
            None,
            None,
        );
        context.resource_infos.push(resource_info);

        Ok(())
    }
}

/// Calculate a resource digest based on the `source`, `media_type` and `select` properties
/// of an `Include` node
fn digest_from_properties(include: &Include) -> ResourceDigest {
    let mut content_str = include.source.clone();
    if let Some(media_type) = include.media_type.as_deref() {
        content_str.push_str(media_type);
    }
    if let Some(select) = include.select.as_deref() {
        content_str.push_str(select);
    }
    ResourceDigest::from_strings(&content_str, None)
}
