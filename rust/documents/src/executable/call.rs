use common::itertools::Itertools;

use crate::{
    document::Document,
    messages::{CompileRequest, Request},
    DOCUMENTS,
};

use super::prelude::*;

#[async_trait]
impl Executable for Call {
    async fn compile(&self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let mut draft = self.clone();

        if draft.id.is_none() {
            draft.id = generate_id("ca");
        }

        let document = match DOCUMENTS
            .open(
                draft.source.trim(),
                draft.media_type.as_ref().map(|mt| mt.to_string()),
            )
            .await
        {
            Ok(document) => {
                draft.errors = None;

                document
            }
            Err(error) => {
                draft.errors = Some(vec![CodeError {
                    error_message: format!(
                        "While attempting to open document `{}`: {}",
                        draft.source, error
                    ),
                    ..Default::default()
                }]);

                let patch = diff_address(address, self, &draft);
                context.push_patch(patch);

                return Ok(());
            }
        };
        let document_version = document.version().to_string();

        let state_string = &[
            draft.source.as_str(),
            draft.media_type.as_ref().map_or("", |mt| mt.as_str()),
            draft.select.as_ref().map_or("", |select| select.as_str()),
            draft
                .arguments
                .iter()
                .map(|arg| {
                    [
                        arg.name.as_str(),
                        arg.code.as_str(),
                        arg.programming_language.as_str(),
                    ]
                    .concat()
                })
                .join("")
                .as_str(),
            document_version.as_str(),
        ]
        .concat();

        let state_digest = generate_digest(state_string);

        if state_digest == get_state_digest(&draft.compile_digest) {
            return Ok(());
        }

        let semantic_digest = 0;

        let params = document.params().await?;
        draft.arguments = params
            .into_values()
            .map(|(.., param)| {
                let mut arg = draft
                    .arguments
                    .iter()
                    .find(|arg| arg.name == param.name)
                    .cloned()
                    .unwrap_or_default();

                if arg.id.is_none() {
                    arg.id = generate_id("ar");
                }
                arg.name = param.name;
                arg.validator = param.validator;
                arg.default = param.default;

                arg
            })
            .collect();

        draft.compile_digest = Some(ExecutionDigest {
            state_digest,
            semantic_digest,
            ..Default::default()
        });

        let patch = diff_address(address, self, &draft);
        context.push_patch(patch);

        let call_id = draft
            .id
            .as_ref()
            .expect("Should have id ensured above")
            .to_string();
        context.push_event_listener(
            call_id,
            ["documents:", &document.id, ":patched"].concat(),
            |_topic, _detail| Request::Compile(CompileRequest::now()),
        );

        Ok(())
        /*
        // Create relations between this resource and the `source` file and any `symbol`s
        // used by arguments. Add to `compile_digest` in same loop.
        let resource = resources::code(&context.path, id, "Call", Format::Unknown);
        let mut relations = vec![(
            Relation::Calls,
            resources::file(&merge(&context.path, &self.source)),
        )];
        if let Some(args) = &arguments {
            for arg in args {
                content_string.push_str(&arg.name);
                if let Some(symbol) = &arg.symbol {
                    content_string.push('s');
                    content_string.push_str(symbol);
                    relations.push((
                        relations::uses(NULL_RANGE),
                        resources::symbol(&context.path, symbol, ""),
                    ));
                } else if let Some(value) = &arg.value {
                    content_string.push('v');
                    content_string.push_str(&serde_json::to_string(value).unwrap_or_default());
                }
            }
        }
        */
    }

    #[cfg(ignore)]
    async fn execute_begin(
        &mut self,
        _resource_info: &ResourceInfo,
        _kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!("Executing `Call` `{}`", id);

        // Get the doc that was opened in assemble
        let doc = call_docs
            .get(id)
            .ok_or_else(|| eyre!("No document registered for call `{}`", id))?;

        // Collect any errors when resolving args, etc
        let mut errors = Vec::new();

        // Resolve each of the call's arguments into a `Node`: either fetching its `symbol` from the kernel space,
        // or using its `value`. If the argument has neither then do not include it in the args map
        let mut args = HashMap::new();
        for arg in self.arguments.iter().flatten() {
            if let Some(symbol) = &arg.symbol {
                let value = match kernel_space.get(symbol).await {
                    Ok(value) => value,
                    Err(error) => {
                        errors.push(CodeError {
                            error_type: Some(Box::new("ArgumentError".to_string())),
                            error_message: error.to_string(),
                            ..Default::default()
                        });
                        continue;
                    }
                };
                args.insert(arg.name.clone(), value);
            } else if let Some(value) = &arg.value {
                args.insert(arg.name.clone(), value.as_ref().clone());
            }
        }

        // Call the document with the resolved args map
        let mut doc = doc.lock().await;
        if let Err(error) = doc.call(args).await {
            errors.push(CodeError {
                error_type: Some(Box::new("CallError".to_string())),
                error_message: error.to_string(),
                ..Default::default()
            })
        }

        // TODO: The following could be done in `execute_end` but needs us to
        // return a TaskInfo.

        // If succeeded, select the content wanted and convert to static block content
        self.content = if !errors.is_empty() {
            None
        } else {
            let root = &*doc.root.read().await;
            if let Some(select) = self.select.as_deref() {
                match query(root, select, None) {
                    Ok(nodes) => Some(match nodes.is_array() {
                        true => serde_json::from_value::<Vec<Node>>(nodes)?.to_static_blocks(),
                        false => serde_json::from_value::<Node>(nodes)?.to_static_blocks(),
                    }),
                    Err(error) => {
                        errors.push(CodeError {
                            error_type: Some(Box::new("SelectQueryError".to_string())),
                            error_message: error.to_string(),
                            ..Default::default()
                        });
                        None
                    }
                }
            } else {
                Some(root.to_static_blocks())
            }
        };

        // Set errors and determine success
        let succeeded = if errors.is_empty() {
            self.errors = None;
            true
        } else {
            self.errors = Some(errors);
            false
        };

        // Update both `compile_digest` and `execute_digest` to the compile digest
        let digest = resource_info
            .compile_digest
            .clone()
            .map(|digest| Box::new(Cord(digest.to_string())));
        self.compile_digest = digest.clone();
        self.execute_digest = digest;

        // Update execution status, etc
        let (execution_required, execution_status) = if succeeded {
            (ExecutionRequired::No, ExecutionStatus::Succeeded)
        } else {
            (ExecutionRequired::No, ExecutionStatus::Failed)
        };
        self.execution_required = Some(execution_required);
        self.execution_status = Some(execution_status);

        Ok(None)
    }
}
