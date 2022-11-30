use common::async_trait::async_trait;

use stencila_schema::If;

use crate::executable::Executable;

#[async_trait]
impl Executable for If {
    /// Compile an `If` node
    #[cfg(ignore)]
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = ensure_id!(self, "if", context);

        // Guess clause language if specified or necessary
        let mut format = Format::Unknown;
        for clause in self.clauses.iter_mut() {
            if (matches!(clause.guess_language, Some(true))
                || clause.programming_language.is_empty()
                || clause.programming_language.to_lowercase() == "unknown")
            {
                format = context.kernel_space.guess_language(
                    &clause.text,
                    format,
                    None,
                    Some(&[Format::Tailwind]),
                );
                clause.programming_language = format.to_string();
            };
        }

        // Add a resource for the `If` itself so it can be included in an execution plan
        // TODO Add relations based on the `clauses` expressions
        context.resource_infos.push(ResourceInfo::new(
            resources::code(&context.path, id, "If", Format::Unknown),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
        ));

        Ok(())
    }

    #[cfg(ignore)]
    async fn execute_begin(
        &mut self,
        _resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!("Executing If `{}`", id);

        let clauses_count = self.clauses.len();
        let mut activated = false;
        for (index, clause) in self.clauses.iter_mut().enumerate() {
            // If this is the last clause, the expression is empty (i.e. an "else" clause)
            // and no other clauses have been made active then make active
            if !activated && index == clauses_count - 1 && clause.code.trim().is_empty() {
                clause.is_active = Some(true);
                break;
            }

            // Skip evaluation if there is another clause that is already activated
            if activated {
                clause.is_active = Some(false);
                continue;
            }

            // Evaluate the clause expression to a value
            let format = formats::match_name(&clause.programming_language);
            let value = {
                // TODO: This needs to be a proper resource info for the expression, including symbols used etc
                let resource_info =
                    ResourceInfo::default(resources::code(&PathBuf::new(), "", "", format));
                let kernel_selector = KernelSelector::from_format_and_tags(format, None);
                let mut task_info = kernel_space
                    .exec(&clause.code, &resource_info, false, &kernel_selector)
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

            // Transform the value into a boolean
            let condition = match value {
                Ok(condition) => {
                    clause.errors = None;
                    Some(match condition {
                        Node::Null(..) => false,
                        Node::Boolean(bool) => bool,
                        Node::Integer(int) => int == 0,
                        Node::Number(num) => num.0 == 0.,
                        Node::String(str) => !str.is_empty(),
                        Node::Array(array) => !array.is_empty(),
                        Node::Object(map) => !map.is_empty(),
                        Node::Datatable(dt) => !dt
                            .columns
                            .first()
                            .map(|col| !col.values.is_empty())
                            .unwrap_or(false),
                        _ => true,
                    })
                }
                Err(errors) => {
                    clause.errors = Some(errors);
                    None
                }
            };

            // Activate, or not, the clause based on the condition
            if matches!(condition, Some(true)) {
                activated = true;
            }
            clause.is_active = condition;
        }

        Ok(None)
    }
}
