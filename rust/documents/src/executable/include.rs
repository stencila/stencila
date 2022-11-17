use common::{async_trait::async_trait, eyre::Result};
use graph_triples::{
    execution_digest_from_content, execution_digest_from_path,
    resources::{self},
    Relation, ResourceInfo,
};

use node_transform::Transform;
use path_utils::merge;
use stencila_schema::{CodeError, ExecutionDigest, Include};

use crate::executable::{CompileContext, Executable};

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

    /// Compile an `Include` node
    ///
    /// Declares this node as a resource with a relation to the source file.
    /// Note that there is no need to compile the `content` since any executable
    /// nodes within it should have been registered in `assemble`.
    ///
    /// TODO: If there is a change in the compile digest (ie. `source`, `media_type`
    /// or `select` have changed) then trigger an `assemble` of the document.
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        let id = ensure_id!(self, "in", context);

        // Calculate a resource digest using the `source`, `media_type` and `select` properties
        let mut execution_digest = digest_from_properties(self);

        // Add the digest of the content of the source file as the dependencies digest
        let path = merge(&context.path, &self.source);
        let dependency_digest =
            execution_digest_from_path(&path, self.media_type.as_ref().map(|str| str.as_str()));
        execution_digest.dependencies_digest = dependency_digest.content_digest;

        // Determine if it is necessary to update content (i.e. has the `compile_digest` changed since last time)
        let should_decode = self.content.is_none()
            || match &self.compile_digest {
                Some(compile_digest) => execution_digest != *compile_digest,
                None => true,
            };
        if should_decode {
            // Clone self for patch
            let _before = self.clone();

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
                    // Decoding was OK so compile content and clear errors
                    let mut content = content.to_blocks();
                    content.compile(context).await?;
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
        }

        let path = merge(&context.path, &self.source);
        let resource = resources::node(&context.path, id, "Include");
        let object = resources::file(&path);
        let relations = vec![(Relation::Includes, object)];
        let execute_pure = Some(true); // Never has side effects
        let compile_digest = self.compile_digest.clone();
        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            self.execution_auto.clone(),
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
fn digest_from_properties(include: &Include) -> ExecutionDigest {
    let mut content_str = include.source.clone();
    if let Some(media_type) = include.media_type.as_deref() {
        content_str.push_str(media_type);
    }
    if let Some(select) = include.select.as_deref() {
        content_str.push_str(select);
    }
    execution_digest_from_content(&content_str)
}
