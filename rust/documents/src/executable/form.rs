use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    tracing,
};
use formats::Format;
use graph_triples::{
    execution_digest_from_content,
    relations::{self},
    resources::{self},
    ResourceInfo,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo};
use node_address::Address;
use stencila_schema::{CodeError, ExecutionRequired, Form, FormDeriveAction, Node, Timestamp};

use crate::executable::{CompileContext, Executable};

#[async_trait]
impl Executable for Form {
    /// Compile a `Form` node
    ///
    /// Adds a resource to the context so that the form can be executed.
    #[cfg(ignore)]
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = ensure_id!(self, "fm", context);

        // If the form is derived, then do the derivation and add any nodes that
        // are missing from the content.
        if let Some(from) = &self.derive_from {
            // Create a draft to make changes to and generate a patch from
            let mut draft = self.clone();

            let action = self
                .derive_action
                .as_ref()
                .unwrap_or(&FormDeriveAction::Create)
                .as_ref()
                .to_lowercase();

            match context
                .kernel_space
                .derive(&["form:", &action].concat(), from)
                .await
            {
                Ok((.., nodes)) => {
                    if let Some(Node::Form(form)) = nodes.first() {
                        draft.content = form.content.clone();

                        // Compile the new content to ensure executable nodes in the patch have an id
                        draft.content.compile(context).await?;

                        draft.errors = None;
                    } else {
                        // This is not a user error, but a kernel implementation error so bail
                        bail!("Expected to get a form from derive call")
                    }
                }
                Err(error) => {
                    draft.errors = Some(vec![CodeError {
                        error_type: Some(Box::new("DeriveError".to_string())),
                        error_message: error.to_string(),
                        ..Default::default()
                    }]);
                }
            }
        }

        // Compile the content of the form
        self.content.compile(context).await?;

        let resource = resources::code(&context.path, id, "Form", Format::Unknown);
        let relations = if let Some(derive_from) = &self.derive_from {
            Some(vec![(
                relations::derives(),
                resources::symbol(&context.path, derive_from, ""),
            )])
        } else {
            None
        };

        let execute_pure = Some(false);
        let compile_digest = Some(execution_digest_from_content("TODO"));
        let execute_digest = self.execute_digest.clone();
        let execute_failed = self.execution_ended.as_ref().map(|_| false);

        let resource_info = ResourceInfo::new(
            resource,
            relations,
            None,
            execute_pure,
            None,
            compile_digest,
            execute_digest,
            execute_failed,
        );
        context.resource_infos.push(resource_info);

        self.content.compile(context).await?;

        Ok(())
    }

    /// Execute a `Form` node
    ///
    /// If the form is derived then derive its content from the kernel space.
    #[cfg(ignore)]
    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        _kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!("Executing Form `{id}`");

        // Do any necessary derivation of content
        let kernel_id = None;

        // Update both `compile_digest` and `execute_digest` to the compile digest determined
        // during the compile phase
        let digest = resource_info.compile_digest.clone();
        self.compile_digest = digest.clone();
        self.execute_digest = digest;

        // Updated other execution properties
        self.execution_required = Some(ExecutionRequired::No);
        self.execution_ended = Some(Box::new(Timestamp::now()));
        self.execution_kernel = kernel_id.map(Box::new);
        self.execution_count = Some(self.execution_count.unwrap_or_default() + 1);

        Ok(None)
    }
}
