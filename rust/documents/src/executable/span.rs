use common::{async_trait::async_trait, eyre::Result, tracing};
use formats::Format;
use graph_triples::{
    resources::{self, ResourceDigest},
    ResourceInfo,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo, TaskResult};
use node_address::Address;

use stencila_schema::{
    CodeError, Cord, Duration, ExecuteAuto, ExecuteRequired, ExecuteStatus, Node, Span, Timestamp,
};

use crate::{assert_id, register_id};

use super::{shared::code_execute_status, AssembleContext, CompileContext, Executable};

#[async_trait]
impl Executable for Span {
    /// Assemble a `Span` node
    ///
    /// Registers the `id` of the span and assembles its `content`.
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        register_id!("sp", self, address, context);

        self.content
            .assemble(&mut address.add_name("content"), context)
            .await?;

        Ok(())
    }

    /// Compile a `Span` node
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        // Infer the language of the expression, falling back to Tailwind
        let lang = match self.programming_language.is_empty() {
            true => Format::Tailwind,
            false => {
                if (matches!(self.guess_language, Some(true))
                    || self.programming_language.to_lowercase() == "unknown")
                {
                    context
                        .kernel_space
                        .guess_language(&self.text, Format::Tailwind, None, None)
                } else {
                    formats::match_name(&self.programming_language)
                }
            }
        };

        // Generate `ResourceInfo` by parsing the code. If there is a passing error still generate resource info
        // but do not generate errors since the user may still be in the process of writing code
        let resource = resources::code(&context.path, id, "Span", lang);
        let mut resource_info = match parsers::parse(resource.clone(), &self.text) {
            Ok(resource_info) => resource_info,
            Err(..) => ResourceInfo::default(resource),
        };

        // Update the resource info (which has (an incomplete) `compile_digest`) with the `execute_digest` from
        // the last time the code chunk was executed
        resource_info.execute_digest = self
            .execute_digest
            .as_ref()
            .map(|cord| ResourceDigest::from_string(&cord.0));
        resource_info.execute_failed = self.execute_status.as_ref().map(|status| {
            // This function can be called while the node is `Scheduled` so this needs to account for that
            // by considering last execution status as well
            matches!(
                status,
                ExecuteStatus::Failed
                    | ExecuteStatus::ScheduledPreviouslyFailed
                    | ExecuteStatus::RunningPreviouslyFailed
            )
        });

        // Assume side-effect free code expression execution semantics
        resource_info.execute_auto = Some(ExecuteAuto::Always);
        resource_info.execute_pure = Some(true);

        // If the language is Tailwind, and it has not relations (i.e. no variable interpolation) then
        // attempt to transpile it to CSS.
        // Fail silently (do not store errors) since the user may still be in the middle
        // of typing during this compile,
        if matches!(lang, Format::Tailwind)
            && resource_info
                .relations
                .as_ref()
                .map_or_else(|| true, |relations| relations.is_empty())
        {
            if let Ok(css) = parser_tailwind::transpile_string(&self.text) {
                self.css = css;
            }
        }

        context.resource_infos.push(resource_info);

        Ok(())
    }

    /// Begin executing a `Span` node
    ///
    /// Starts an async tak in the kernel space
    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!("Executing `Span` `{}`", id);

        let task_info = kernel_space
            .exec(&self.text, resource_info, is_fork, kernel_selector)
            .await?;

        Ok(Some(task_info))
    }

    /// End executing a `Span` node
    ///
    /// Updates various various properties of the node based on the task info and result.
    /// Most importantly, updates the `css` property by transpiling the result of the
    /// evaluation.
    async fn execute_end(&mut self, task_info: TaskInfo, task_result: TaskResult) -> Result<()> {
        let TaskResult {
            outputs,
            messages: mut errors,
        } = task_result;

        // Update both `compile_digest` and `execute_digest` to the compile digest
        let digest = task_info
            .resource_info
            .compile_digest
            .clone()
            .map(|digest| Box::new(Cord(digest.to_string())));
        self.compile_digest = digest.clone();
        self.execute_digest = digest;

        // Update execution status, etc
        let execute_status = code_execute_status(&task_info, &errors);
        self.execute_required = Some(if matches!(execute_status, ExecuteStatus::Succeeded) {
            ExecuteRequired::No
        } else {
            ExecuteRequired::Failed
        });
        self.execute_status = Some(execute_status);
        self.execute_ended = task_info
            .ended()
            .map(|ended| Box::new(Timestamp::from(ended)));
        self.execute_duration = task_info
            .duration()
            .map(|duration| Box::new(Duration::from_micros(duration as i64)));
        self.execute_kernel = task_info.kernel_id.map(Box::new);
        self.execute_count = Some(self.execute_count.unwrap_or_default() + 1);

        // Transpile the returned Tailwind string. To avoid unstyled content, if there is
        // an error we do not reset the CSS
        if let Some(node) = outputs.first() {
            match node {
                Node::String(string) => match parser_tailwind::transpile_string(string) {
                    Ok(css) => self.css = css,
                    Err(error) => {
                        errors.push(CodeError {
                            error_type: Some(Box::new("SyntaxError".to_string())),
                            error_message: error.to_string(),
                            ..Default::default()
                        });
                    }
                },
                _ => errors.push(CodeError {
                    error_type: Some(Box::new("TypeError".to_string())),
                    error_message: format!(
                        "Expected expression to evaluate to a string, got a `{}` instead",
                        node.as_ref()
                    ),
                    ..Default::default()
                }),
            }
        }

        // Update errors
        self.errors = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };

        Ok(())
    }
}
