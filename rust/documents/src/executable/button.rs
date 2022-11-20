use super::prelude::*;

#[async_trait]
impl Executable for Button {
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let mut draft = self.clone();

        if draft.id.is_none() {
            draft.id = generate_id("bu");
        }

        if draft.name.is_empty() {
            draft.name = draft.id.as_ref().expect("Ensured above").to_string();
        }

        if !draft.code.is_empty()
            && (draft.programming_language.is_empty() || matches!(draft.guess_language, Some(true)))
        {
            draft.programming_language = context.guess_language(&draft.code).to_string();
        }

        let state_digest =
            generate_digest(&["", &draft.name, &draft.code, &draft.programming_language].concat());
        if state_digest == get_state_digest(&draft.compile_digest) {
            return Ok(());
        }

        let semantic_digest;
        if draft.code.is_empty() {
            draft.execution_dependencies = None;
            semantic_digest = 0;
        } else {
            let parse_info = context.parse_code(&draft.programming_language, &draft.code)?;
            draft.execution_dependencies = Some(parse_info.execution_dependencies);
            semantic_digest = parse_info.semantic_digest;
        }

        draft.execution_dependents =
            Some(vec![context.dependent_variable(&draft.name, "Timestamp")]);

        draft.compile_digest = Some(ExecutionDigest {
            state_digest,
            semantic_digest,
            ..Default::default()
        });

        let patch = diff_address(address, self, &draft);
        context.push_patch(patch);

        Ok(())
    }

    #[cfg(ignore)]
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
        if self.code.len() >= 5 {
            self.is_disabled = Some(true);
        }

        // Calculate the current timestamp and set it in the kernel space
        let value = Timestamp::now();
        let kernel_id = kernel_space
            .set(&self.name, Node::Timestamp(value.clone()), kernel_selector)
            .await?;

        // Update both `compile_digest` and `execute_digest` to the compile digest determined
        // during the compile phase
        let digest = resource_info.compile_digest.clone();
        self.compile_digest = digest.clone();
        self.execute_digest = digest;

        // Updated other execution properties
        self.execution_required = Some(ExecutionRequired::No);
        self.execution_ended = Some(Box::new(Timestamp::now()));
        self.execution_kernel = Some(Box::new(kernel_id));
        self.execution_count = Some(self.execution_count.unwrap_or_default() + 1);

        Ok(None)
    }
}
