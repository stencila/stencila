use common::{
    async_trait::async_trait,
    eyre::{bail, Result},
    tracing,
};
use formats::Format;
use graph_triples::{
    resources::{self},
    ResourceInfo,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo};
use node_transform::Transform;
use stencila_schema::{CodeError, For, Node};

use crate::executable::Executable;

use super::{CompileContext, ExecuteContext};

#[async_trait]
impl Executable for For {
    /// Compile a `For` node
    ///
    /// Defines a resource for the node itself with relations to its variables etc
    /// used in `text`. No relation is necessary between the `For` and its `otherwise` content.
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        let id = ensure_id!(self, "fo", context);

        // Compile `otherwise` but do not compile `content` or `iterations` since these are
        // not part of the document's dependency graph.
        self.otherwise.compile(context).await?;

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

        // TODO: Define relation to expression
        // TODO: Consider "inheriting" `Uses` relations from nodes in `content` so that the loop
        // reactively updates when something that its content depends upon updates
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
            Ok(items) => {
                self.errors = None;
                Some(match items {
                    Node::Array(array) => array,
                    // TODO: handle objects and datatables
                    _ => bail!("Expected an array, got {:?}", items),
                })
            }
            Err(errors) => {
                self.errors = Some(errors);
                None
            }
        };

        /*

        // Inherit errors and output from expression
        let (items, errors) = {
            if let Some(errors) = &self.errors {
                (None, Some(errors.clone()))
            } else if let Some(output) = &self.output {
                match output.as_ref() {
                    Node::Array(array) => (Some(array), None),
                    _ => (
                        None,
                        Some(vec![CodeError {
                            error_message: "Expected expression to evaluate to an array"
                                .to_string(),
                            ..Default::default()
                        }]),
                    ),
                }
            } else {
                (
                    None,
                    Some(vec![CodeError {
                        error_message: "Expected expression to have an output".to_string(),
                        ..Default::default()
                    }]),
                )
            }
        };
        self.errors = errors;
        */

        // Execute the content for each item
        if let Some(items) = items {
            let mut context = ExecuteContext { kernel_space };

            let mut iterations = Vec::new();
            for _item in items {
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
