use common::{async_trait::async_trait, eyre::Result, tracing};
use formats::Format;
use graph_triples::{
    resources::{self, ResourceDigest},
    ResourceInfo,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo, TaskResult};
use node_address::Address;
use node_patch::produce;

use stencila_schema::{
    CodeError, Cord, Division, Duration, ExecuteRequired, ExecuteStatus, Node, Timestamp,
};

use crate::{assert_id, register_id};

use super::{shared::code_execute_status, AssembleContext, CompileContext, Executable};

#[async_trait]
impl Executable for Division {
    /// Compile a `Division`
    ///
    /// Ensures that the division has an `id`, and that it is registered, and
    /// assembles the `content`.
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        register_id!("di", self, address, context);

        self.content
            .assemble(&mut address.add_name("content"), context)
            .await?;

        Ok(())
    }

    /// Compile a `Division`
    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        let lang = match self.programming_language.is_empty() {
            true => Format::Tailwind,
            false => formats::match_name(&self.programming_language),
        };

        // Generate `ResourceInfo` by parsing the code. If there is a passing error
        // still generate resource info but do not generate errors since the user may
        // still be in the process of writing code
        let resource = resources::code(&context.path, id, "Division", lang);
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

        // Force code expression execution semantics (in case `@impure` or `@autorun` tags
        // where inadvertently used in code) by setting to `None`
        resource_info.execute_auto = None;
        resource_info.execute_pure = None;

        // If the language is Tailwind, and there are no dependencies, then transpile the to CSS now so that
        // the content is styled prior to execution
        let no_dependencies = resource_info.dependencies.is_none();
        if lang == Format::Tailwind && no_dependencies {
            let patch = match parser_tailwind::transpile_string(&self.text) {
                Ok(css) => {
                    // On success, update both `css` and `errors`
                    produce(self, Some(id.clone()), None, |draft| {
                        draft.css = css.clone();
                        draft.errors = None;
                    })
                }
                Err(error) => {
                    // On error, update the `error` property but do not alter any CSS
                    produce(self, Some(id.clone()), None, |draft| {
                        draft.errors = Some(vec![CodeError {
                            error_type: Some(Box::new("SyntaxError".to_string())),
                            error_message: error.to_string(),
                            ..Default::default()
                        }]);
                    })
                }
            };
            context.patches.push(patch);
        }

        context.resource_infos.push(resource_info);

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        tracing::trace!("Executing `Division` `{:?}`", self.id);

        let task_info = kernel_space
            .exec(&self.text, resource_info, is_fork, kernel_selector)
            .await?;

        Ok(Some(task_info))
    }

    async fn execute_end(&mut self, task_info: TaskInfo, task_result: TaskResult) -> Result<()> {
        let TaskResult {
            outputs,
            messages: errors,
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

        // Update output and errors
        self.errors = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };

        if let Some(tailwind) = outputs.last() {
            let tailwind = match tailwind {
                Node::String(string) => string,
                //Node::Array(array) => arr
                _ => "",
            };
            match parser_tailwind::transpile_string(tailwind) {
                Ok(css) => self.css = css,
                Err(error) => {
                    self.errors = Some(vec![CodeError {
                        error_type: Some(Box::new("SyntaxError".to_string())),
                        error_message: error.to_string(),
                        ..Default::default()
                    }]);
                }
            };
        }

        Ok(())
    }
}
