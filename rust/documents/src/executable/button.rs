use common::{async_trait::async_trait, eyre::Result, tracing};
use formats::Format;
use graph_triples::{
    relations::{self, NULL_RANGE},
    resources::{self, ResourceDigest},
    ResourceInfo,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo};
use node_address::Address;
use stencila_schema::{Button, Cord, ExecuteRequired, Node, Timestamp};

use crate::{assert_id, register_id};

use super::{AssembleContext, CompileContext, Executable};

#[async_trait]
impl Executable for Button {
    /// Assemble a `Button` node
    ///
    /// Simply ensures the button has an `id` and registers it in the context.
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        register_id!("bu", self, address, context);

        Ok(())
    }

    /// Compile a `Button` node
    ///
    /// Adds an `Assign` relation to the compilation context with the name of the button.
    /// As for `Parameter`s, uses `Format::Json` to indicate that the button's `value` will
    /// be set in the "store kernel".
    ///
    /// By definition, a `Button` is always "impure" (has a side effect of setting a variable)
    /// and is assumed to always succeed.
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        // TODO: guess language and parse `text` to determine dependencies

        let resource = resources::code(&context.path, id, "Button", Format::Json);
        let symbol = resources::symbol(&context.path, &self.name, "Timestamp");
        let relations = Some(vec![(relations::assigns(NULL_RANGE), symbol)]);

        let execute_pure = Some(false);
        let compile_digest = Some(ResourceDigest::from_strings(&self.name, None));
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
            None,
            compile_digest,
            execute_digest,
            execute_failed,
        );
        context.resource_infos.push(resource_info);

        Ok(())
    }

    /// Execute a `Button` node
    ///
    /// Sets a timestamp variable in the kernel space and updates execution related
    /// properties of the node itself.
    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!("Execute begin for `{id}`");

        // Determine if the button is enabled
        // TODO: Calculate properly, this is just a placeholder for testing
        if self.text.len() >= 5 {
            self.is_disabled = Some(true);
        }

        // Calculate the current timestamp and set it in the kernel space
        let value = Timestamp::now();
        let kernel_id = kernel_space
            .set(&self.name, Node::Timestamp(value.clone()), kernel_selector)
            .await?;

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
        self.execute_kernel = Some(Box::new(kernel_id));
        self.execute_count = Some(self.execute_count.unwrap_or_default() + 1);

        Ok(None)
    }
}
