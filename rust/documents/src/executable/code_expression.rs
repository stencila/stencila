use common::{async_trait::async_trait, eyre::Result, tracing};
use formats::Format;
use graph_triples::{
    resources::{self},
    ResourceInfo,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo, TaskResult};

use stencila_schema::{CodeExpression, Duration, ExecutionRequired, ExecutionStatus, Timestamp};

use super::{shared::code_execution_status, CompileContext, Executable};

#[async_trait]
impl Executable for CodeExpression {
    /// Compile a `CodeExpression` node
    ///
    /// Performs semantic analysis of the code (if necessary) and adds the resulting
    /// relations to the compilation context. If the `programming_language` is an empty string
    /// then use the current language of the context.
    ///
    /// A `CodeExpression` is assumed to be pure (i.e. have no side effects and can be executed
    /// in a fork).
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        let id = ensure_id!(self, "ce", context);

        // Guess language if specified or necessary
        if matches!(self.guess_language, Some(true)) || self.programming_language.is_empty() {
            self.programming_language = context
                .kernel_space
                .guess_language(&self.text, Format::Unknown, None, None)
                .to_string();
        };

        // Generate `ResourceInfo` by parsing the code. If there is a passing error
        // still generate resource info but do not generate errors since the user may
        // still be in the process of writing code
        let resource = resources::code(
            &context.path,
            id,
            "CodeExpression",
            formats::match_name(&self.programming_language),
        );
        let mut resource_info = match parsers::parse(resource.clone(), &self.text) {
            Ok(resource_info) => resource_info,
            Err(..) => ResourceInfo::default(resource),
        };

        // Update the resource info (which has (an incomplete) `compile_digest`) with the `execute_digest` from
        // the last time the code chunk was executed
        resource_info.execute_digest = self.execute_digest.clone();
        resource_info.execute_failed = self.execution_status.as_ref().map(|status| {
            // This function can be called while the node is `Scheduled` so this needs to account for that
            // by considering last execution status as well
            matches!(
                status,
                ExecutionStatus::Failed
                    | ExecutionStatus::ScheduledPreviouslyFailed
                    | ExecutionStatus::RunningPreviouslyFailed
            )
        });

        // Force code expression execution semantics (in case `@impure` or `@autorun` tags
        // where inadvertently used in code) by setting to `None`
        resource_info.execute_auto = None;
        resource_info.execute_pure = None;

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
        tracing::trace!("Executing `CodeExpression` `{:?}`", self.id);

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
        let digest = task_info.resource_info.compile_digest.clone();
        self.compile_digest = digest.clone();
        self.execute_digest = digest;

        // Update execution status, etc
        let execution_status = code_execution_status(&task_info, &errors);
        self.execution_required = Some(if matches!(execution_status, ExecutionStatus::Succeeded) {
            ExecutionRequired::No
        } else {
            ExecutionRequired::Failed
        });
        self.execution_status = Some(execution_status);
        self.execution_ended = task_info
            .ended()
            .map(|ended| Box::new(Timestamp::from(ended)));
        self.execution_duration = task_info
            .duration()
            .map(|duration| Box::new(Duration::from_micros(duration as i64)));
        self.execution_kernel = task_info.kernel_id.map(Box::new);
        self.execution_count = Some(self.execution_count.unwrap_or_default() + 1);

        // Update output and errors
        self.output = outputs.get(0).map(|output| Box::new(output.clone()));
        self.errors = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };

        Ok(())
    }
}
