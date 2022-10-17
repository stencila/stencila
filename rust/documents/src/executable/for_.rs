use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    serde_json, tracing,
};
use formats::Format;
use graph_triples::{
    relations::{self, NULL_RANGE},
    resources::{self, ResourceDigest},
    Relation, ResourceInfo,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo};
use node_address::Address;
use node_transform::Transform;
use stencila_schema::{CodeError, For, Node};

use crate::executable::Executable;

use super::{AssembleContext, CompileContext, ExecuteContext};

#[async_trait]
impl Executable for For {
    /// Assemble a `For` node
    ///
    /// Just registers the address of the node.
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        register_id!("fo", self, address, context);

        Ok(())
    }

    /// Compile a `For` node
    ///
    /// Compiles the for's `CodeExpression` thereby creating a relation between the
    /// expression
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;
        tracing::trace!("Compiling `{id}`");

        // Guess clause language if specified or necessary
        let format = if (matches!(self.guess_language, Some(true))
            || self.programming_language.is_empty()
            || self.programming_language.to_lowercase() == "unknown")
        {
            context.kernel_space.guess_language(
                &self.text,
                Format::Unknown,
                None,
                Some(&[Format::Tailwind]),
            )
        } else {
            formats::match_name(&self.programming_language)
        };

        // Add a resource for the `For` based on parsing the code
        // TODO Add relations based on the `content` and `otherwise` so that this
        // for loop reactively updates
        let resource = resources::code(&context.path, id, "For", format);
        let resource_info = match parsers::parse(resource.clone(), &self.text) {
            Ok(resource_info) => resource_info,
            Err(..) => ResourceInfo::default(resource),
        };
        context.resource_infos.push(resource_info);

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!("Executing `{id}`");

        // Evaluate the expression to a value
        let value = {
            let mut task_info = kernel_space
                .exec(&self.text, resource_info, false, kernel_selector)
                .await?;
            let mut task_result = task_info.result().await?;

            if task_result.has_errors() {
                Err(task_result.messages)
            } else if task_result.outputs.len() == 1 {
                Ok(task_result.outputs.remove(0))
            } else {
                Err(vec![CodeError {
                    error_message: format!(
                        "Expected one output from expression, got {} outputs",
                        task_result.outputs.len()
                    ),
                    ..Default::default()
                }])
            }
        };

        // Transform the value to a set of items
        let items = match value {
            Ok(items) => Some(match items {
                Node::Array(array) => array,
                // TODO: handle objects and datatables
                _ => bail!("Expected an array, got {:?}", items),
            }),
            Err(errors) => {
                self.errors = Some(errors);
                None
            }
        };

        // Execute the content for each item
        if let Some(items) = items {
            let mut context = ExecuteContext { kernel_space };

            let mut iterations = Vec::new();
            for item in items {
                // Clone the content for the item
                let mut content = self.content.clone();
                // TODO: Create a fork of kernels to execute content in with item
                // set as variable
                content.execute(&mut context).await?;
                let content = content.to_static_blocks();
                iterations.push(content);
            }
            self.iterations = Some(iterations);
        } else {
            self.iterations = None;
        }

        Ok(None)
    }
}
