use common::{async_trait::async_trait, eyre::Result, tracing};
use graph_triples::ResourceInfo;
use kernels::{KernelSelector, KernelSpace, TaskInfo};
use path_utils::merge;
use stencila_schema::Call;

use super::{CompileContext, Executable};

#[async_trait]
impl Executable for Call {
    async fn compile(&mut self, context: &mut CompileContext) -> Result<()> {
        let id = ensure_id!(self, "ca", context);

        // Open the called document and register in `call_docs`
        let path = merge(&context.path, &self.source);
        let _format = self.media_type.as_deref().cloned();
        tracing::trace!("Opening doc `{}` for call `{}`", path.display(), id);
        //let doc = Document::open(path, format).await?;
        //context.call_docs.write().await.insert(id, Mutex::new(doc));

        /*
        // Get the parameters of the called document
        let doc = context.call_docs.read().await;
        let mut doc = doc
            .get(id)
            .ok_or_else(|| eyre!("No document open for call `{}`", id))?
            .lock()
            .await;
        let params = doc.params().await?;

        // Update the call arguments to include all parameters and with `name`,
        // `validator` and `default` inherited from the parameter
        let arguments = if !params.is_empty() {
            Some(
                params
                    .values()
                    .map(|(.., param)| {
                        let arg = self
                            .arguments
                            .iter()
                            .flatten()
                            .find(|arg| arg.name == param.name)
                            .cloned()
                            .unwrap_or_default();
                        CallArgument {
                            name: param.name.clone(),
                            validator: param.validator.clone(),
                            default: param.default.clone(),
                            ..arg
                        }
                    })
                    .collect::<Vec<CallArgument>>(),
            )
        } else {
            None
        };

        // Calculate content for `compile_digest` based on concatenating properties affecting execution of the
        // call, including properties of the call args (in following loop)
        let mut content_string = self.source.clone();
        if let Some(media_type) = self.media_type.as_deref() {
            content_string.push_str(media_type);
        }
        if let Some(select) = self.select.as_deref() {
            content_string.push_str(select);
        }

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

        // Calculate compile digest
        let compile_digest = ResourceDigest::from_strings(&content_string, None);

        // Make execute digest the correct type for `ResourceInfo`
        let execute_digest = self
            .execute_digest
            .as_deref()
            .map(ResourceDigest::from_cord);

        // Add resource info to the compile context
        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            self.execute_auto.clone(),
            Some(true), // Never has side effects
            Some(compile_digest),
            execute_digest,
            None,
        );
        context.resource_infos.push(resource_info);

        // Generate a patch for updates to `arguments`
        let patch = diff_id(
            id,
            self,
            &Call {
                arguments,
                ..self.clone()
            },
        );
        context.patches.push(patch);
        */

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        _resource_info: &ResourceInfo,
        _kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!("Executing `Call` `{}`", id);

        /*
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
        let (execute_required, execute_status) = if succeeded {
            (ExecuteRequired::No, ExecuteStatus::Succeeded)
        } else {
            (ExecuteRequired::No, ExecuteStatus::Failed)
        };
        self.execute_required = Some(execute_required);
        self.execute_status = Some(execute_status);
        */

        Ok(None)
    }
}
