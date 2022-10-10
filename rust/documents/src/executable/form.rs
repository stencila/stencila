use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    tracing,
};
use formats::Format;
use graph_triples::{
    relations::{self, NULL_RANGE},
    resources::{self, ResourceDigest},
    ResourceInfo,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo};
use node_address::Address;

use stencila_schema::{
    CodeError, Cord, ExecuteRequired, Form,
    FormDeriveAction, Node, Timestamp,
};

use crate::{
    assert_id,
    executable::{AssembleContext, CompileContext, Executable},
    register_id,
};

#[async_trait]
impl Executable for Form {
    /// Assemble a `Form` node
    ///
    /// Ensures the form has an `id` (and is registered) and does
    /// the same for any executable nodes (e.g. `Parameter`s) in the form's `content`.
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        let _id = register_id!("fm", self, address, context);

        /*
        context.enter_container(&id);

        self.content
            .assemble(&mut address.add_name("content"), context)
            .await?;

        context.exit_container();
        */

        Ok(())
    }

    /// Compile a `Form` node
    ///
    /// Adds a resource to the context so that the form can be executed.
    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        let resource = resources::code(&context.path, id, "Form", Format::Unknown);
        let relations = if let Some(derive_from) = &self.derive_from {
            Some(vec![(
                relations::uses(NULL_RANGE),
                resources::symbol(&context.path, derive_from, ""),
            )])
        } else {
            None
        };

        let execute_pure = Some(false);
        let compile_digest = Some(ResourceDigest::from_strings("TODO", None));
        let execute_digest = self
            .execute_digest
            .as_deref()
            .map(ResourceDigest::from_cord);
        let execute_failed = self.execute_ended.as_ref().map(|_| false);

        let resource_info = ResourceInfo::new(
            resource,
            relations,
            None,
            execute_pure,
            compile_digest,
            execute_digest,
            execute_failed,
        );
        context.resource_infos.push(resource_info);

        Ok(())
    }

    /// Execute a `Form` node
    ///
    /// If the form is derived then derive its content from the kernel space.
    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _is_fork: bool,
        //_call_docs: &CallDocuments,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!("Executing Form `{id}`");

        // Do any necessary derivation of content
        let mut kernel_id = None;
        let mut errors = Vec::new();
        if let Some(from) = &self.derive_from {
            let action = self
                .derive_action
                .as_ref()
                .unwrap_or(&FormDeriveAction::Create)
                .as_ref()
                .to_lowercase();

            match kernel_space
                .derive(&["form:", &action].concat(), from)
                .await
            {
                Ok((id, nodes)) => {
                    if let Some(Node::Form(form)) = nodes.first() {
                        self.content = form.content.clone();
                        kernel_id = Some(id);
                    } else {
                        // This is not a user error, but a kernel implementation error so bail
                        bail!("Expected to get a form from derive call")
                    }
                }
                Err(error) => errors.push(CodeError {
                    error_type: Some(Box::new("DeriveError".to_string())),
                    error_message: error.to_string(),
                    ..Default::default()
                }),
            }
        }
        self.errors = match errors.is_empty() {
            true => None,
            false => Some(errors),
        };

        // Update both `compile_digest` and `execute_digest` to the compile digest determined
        // during the compile phase
        let digest = resource_info
            .compile_digest
            .clone()
            .map(|digest| Box::new(Cord(digest.to_string())));
        self.compile_digest = digest.clone();
        self.execute_digest = digest;

        // Updated other execution properties
        self.execute_required = Some(ExecuteRequired::No);
        self.execute_ended = Some(Box::new(Timestamp::now()));
        self.execute_kernel = kernel_id.map(Box::new);
        self.execute_count = Some(self.execute_count.unwrap_or_default() + 1);

        Ok(None)
    }
}
