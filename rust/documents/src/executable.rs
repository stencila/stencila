use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
};

use common::{
    async_trait::async_trait,
    eyre::{bail, eyre, Result},
    serde_json,
    tokio::sync::{Mutex, RwLock},
    tracing,
};
use formats::normalize_title;
use graph_triples::{
    relations,
    relations::NULL_RANGE,
    resources::{self, ResourceDigest},
    Relation, ResourceInfo, TagMap,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo, TaskResult};
use node_address::{Address, AddressMap, Slot};
use node_dispatch::{dispatch_block, dispatch_inline, dispatch_node, dispatch_work};
use node_patch::{diff_address, diff_id, produce, Patch};
use node_pointer::Pointer;

use node_query::query;
use node_transform::Transform;
use node_validate::Validator as _;
use path_utils::merge;
use stencila_schema::*;

use crate::document::{CallDocuments, Document};

/// Trait for executable document nodes
///
/// This trait is implemented below for all (or at least most) node types.
#[async_trait]
pub trait Executable {
    async fn assemble(
        &mut self,
        _address: &mut Address,
        _context: &mut AssembleContext,
    ) -> Result<()> {
        Ok(())
    }

    async fn compile(&self, _context: &mut CompileContext) -> Result<()> {
        Ok(())
    }

    async fn execute_begin(
        &mut self,
        _resource_info: &ResourceInfo,
        _kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _is_fork: bool,
        _call_docs: &CallDocuments,
    ) -> Result<Option<TaskInfo>> {
        Ok(None)
    }

    async fn execute_end(&mut self, _task_info: TaskInfo, _task_result: TaskResult) -> Result<()> {
        Ok(())
    }
}

pub struct AssembleContext {
    /// The path of the document being compiled.
    /// Used to resolve relative paths e.g. in `Include` nodes
    pub path: PathBuf,

    /// Counts of the number of node ids with each prefix assigned
    pub ids: HashMap<String, usize>,

    /// A map of node ids to addresses
    pub address_map: AddressMap,

    /// A map of `Call` ids to their `source`
    /// Used so a document can maintain a `Document` for each `Call`
    /// (thereby reducing startup times associated with each execution of the call)
    pub call_docs: Arc<RwLock<CallDocuments>>,

    /// A list of patch operations representing changes to nodes.
    pub patches: Vec<Patch>,
}

impl AssembleContext {
    /// Generate a unique id for a node
    ///
    /// These generated ids use a prefix reflecting the node type (i.g. "cc-" for `CodeChunk` nodes)
    /// which can be used to determine that it was generated (so, for example it is not persisted).
    /// They are deterministic which is also useful (and maybe assumed elsewhere in the code?)
    fn ensure_id(&mut self, prefix: &str) -> String {
        let count = self
            .ids
            .entry(prefix.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        [prefix, "-", &count.to_string()].concat()
    }

    /// Record a node id
    fn record_id(&mut self, id: String, address: Address) {
        self.address_map.insert(id, address);
    }
}

/// Ensure the node has an `id`, generating one if necessary
///
/// This needs to be (?) a macro, rather than a generic function, because
/// it is not possible to define a bound that the type must have the `id` property.
macro_rules! ensure_id {
    ($prefix:expr, $node:expr, $address:expr, $context:expr) => {{
        let id = if let Some(id) = $node.id.as_deref() {
            id.clone()
        } else {
            let id = $context.ensure_id($prefix);
            $node.id = Some(Box::new(id.clone()));
            id
        };
        $context.record_id(id.clone(), $address.clone());
        id
    }};
}

/// Assert that a node has an id
macro_rules! assert_id {
    ($node:expr) => {
        $node
            .id
            .as_deref()
            .ok_or_else(|| eyre!("Node should have `id` assigned in assemble phase"))
    };
}

#[derive(Debug, Default)]
pub struct CompileContext {
    /// The path of the document being compiled.
    /// Used to resolve relative paths e.g. in `ImageObject` nodes
    pub path: PathBuf,

    /// The project that the document is within.
    /// Used to restrict any file links to be within the project
    pub project: PathBuf,

    /// The programming language of the last code node encountered during
    /// compilation
    pub programming_language: Option<String>,

    /// A list of resources compiled from the nodes
    pub resource_infos: Vec<ResourceInfo>,

    /// Any global tags defined in code chunks
    pub global_tags: TagMap,

    /// A map of `Call` ids to their `source`
    /// Used so a document can get the parameters of the called doc
    pub call_docs: Arc<RwLock<CallDocuments>>,

    /// A list of patch operations representing changes to nodes.
    pub patches: Vec<Patch>,
}

/// Set the programming of a node or of the context
///
/// Ok, bad name but it's like `ensure_id!`: if the node does
/// not have a `programming_language` then we'll use the context's
/// and if it does than we'll set the context's.
macro_rules! ensure_lang {
    ($node:expr, $context:expr) => {
        if $node.programming_language.is_empty() {
            $context.programming_language.clone().unwrap_or_default()
        } else {
            let lang = normalize_title(&$node.programming_language);
            $context.programming_language = Some(lang.clone());
            lang
        }
    };
}

#[async_trait]
impl Executable for Link {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        ensure_id!("li", self, address, context);
        Ok(())
    }

    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;
        let resource = resources::node(&context.path, id, "Link");

        let target = &self.target;
        let object = if target.starts_with("http://") || target.starts_with("https://") {
            resources::url(target)
        } else {
            resources::file(&merge(&context.path, target))
        };
        let relations = vec![(Relation::Links, object)];

        let resource_info =
            ResourceInfo::new(resource, Some(relations), None, None, None, None, None);
        context.resource_infos.push(resource_info);

        Ok(())
    }
}

/// Compile the `content_url` property of `MediaObject` node types
///
/// If the `content_url` property is  a `file://` URL (implicitly
/// or explicitly) then resolves the file path, records it as
/// a file dependency, and returns an absolute `file://` URL.
fn compile_content_url(content_url: &str, context: &mut CompileContext) -> String {
    if content_url.starts_with("http://")
        || content_url.starts_with("https://")
        || content_url.starts_with("data:")
    {
        return content_url.into();
    }

    // Extract the path
    let path = if let Some(path) = content_url.strip_prefix("file://") {
        path
    } else {
        content_url
    };

    // If necessary make the path absolute
    let path = Path::new(path);
    let path = if path.is_relative() {
        match context.path.parent() {
            Some(dir) => dir.join(path),
            None => path.to_path_buf(),
        }
    } else {
        path.to_path_buf()
    };

    // Assert that the path is within the project
    if path.strip_prefix(&context.project).is_err() {
        tracing::warn!(
            "Document contains a link to a file outside of its project: '{}'. Resolved path '{}' is outside of project path '{}'",
            content_url,
            path.display(),
            &context.project.display()
        )
    }

    format!("file://{}", path.display())
}

/// Compile a `MediaObject` node type
/// 
/// Note that this patches the `content_url` property so that any absolute file path
/// that is resolved in `compile_content_url()` is available, for example
/// for encoding to other formats.
macro_rules! executable_media_object {
    ($type:ty, $prefix:expr) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(
                &mut self,
                address: &mut Address,
                context: &mut AssembleContext,
            ) -> Result<()> {
                ensure_id!($prefix, self, address, context);
                Ok(())
            }

            async fn compile(&self, context: &mut CompileContext) -> Result<()> {
                let id = assert_id!(self)?;
                let resource = resources::node(&context.path, &id, stringify!($type));

                let url = compile_content_url(&self.content_url, context);
                let object = if url.starts_with("http") || url.starts_with("data:") {
                    resources::url(&url)
                } else {
                    let url = url.strip_prefix("file://").unwrap_or(&url);
                    resources::file(&Path::new(&url))
                };
                let relations = vec![(Relation::Embed, object)];

                let resource_info =
                    ResourceInfo::new(resource, Some(relations), None, None, None, None, None);
                context.resource_infos.push(resource_info);

                let patch = produce(self, Some(id.clone()), None, |draft| {
                    draft.content_url = url.clone();
                });
                context.patches.push(patch);

                Ok(())
            }
        }
    };
}

executable_media_object!(MediaObject, "me");
executable_media_object!(AudioObject, "au");
executable_media_object!(AudioObjectSimple, "au");
executable_media_object!(ImageObject, "im");
executable_media_object!(ImageObjectSimple, "im");
executable_media_object!(VideoObject, "vi");
executable_media_object!(VideoObjectSimple, "vi");

/// Get the value for a parameter
///
/// Uses the parameters `value`, falling back to it's `default`, falling back
/// to a default based on the validator.
fn parameter_value(param: &Parameter) -> Node {
    param
        .value
        .as_deref()
        .or(param.default.as_deref())
        .cloned()
        .or_else(|| {
            param
                .validator
                .as_ref()
                .map(|validator| validator.default_())
        })
        .unwrap_or_else(|| Node::String(String::new()))
}

/// Generate a `ResourceDigest` for a parameter
///
/// The content string is the JSON representation of the parameter value
/// and semantic string adds the parameter name
fn parameter_digest(param: &Parameter, value: &Node) -> ResourceDigest {
    let content_str = serde_json::to_string(&value).unwrap_or_default();
    let semantic_str = [param.name.as_str(), content_str.as_str()].concat();
    ResourceDigest::from_strings(&content_str, Some(&semantic_str))
}

#[async_trait]
impl Executable for Parameter {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        ensure_id!("pa", self, address, context);
        Ok(())
    }

    /// Compile a `Parameter` node
    ///
    /// Adds an `Assign` relation to the compilation context with the name and kind of value.
    /// If the language of the parameter is not defined (currently schema does not allow for this
    /// anyway), then use the language of the last code node. By definition, a `Parameter` is always
    /// "impure" (has a side effect).
    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        let resource = resources::code(
            &context.path,
            id,
            "Parameter",
            context.programming_language.clone(),
        );

        let kind = match self.validator.as_deref() {
            Some(ValidatorTypes::BooleanValidator(..)) => "Boolean",
            Some(ValidatorTypes::IntegerValidator(..)) => "Integer",
            Some(ValidatorTypes::NumberValidator(..)) => "Number",
            Some(ValidatorTypes::StringValidator(..)) => "String",
            Some(ValidatorTypes::TupleValidator(..)) => "Tuple",
            Some(ValidatorTypes::ArrayValidator(..)) => "Array",
            _ => "",
        };
        let object = resources::symbol(&context.path, &self.name, kind);
        let relations = vec![(relations::assigns(NULL_RANGE), object)];

        let value = parameter_value(self);
        let compile_digest = parameter_digest(self, &value);

        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            None,
            Some(false), // Always impure because affects the kernel space
            Some(compile_digest),
            self.execute_digest
                .as_ref()
                .map(|cord| ResourceDigest::from_string(&cord.0)),
            None,
        );
        context.resource_infos.push(resource_info);

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        _resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        _is_fork: bool,
        _call_docs: &CallDocuments,
    ) -> Result<Option<TaskInfo>> {
        tracing::trace!("Executing `Parameter`");

        let value = parameter_value(self);

        let kernel_id = kernel_space
            .set(&self.name, value.clone(), kernel_selector)
            .await?;

        let digest = Box::new(Cord(parameter_digest(self, &value).to_string()));

        self.value = Some(Box::new(value));
        self.compile_digest = Some(digest.clone());
        self.execute_digest = Some(digest);
        self.execute_required = Some(ExecuteRequired::No);
        self.execute_kernel = Some(Box::new(kernel_id));

        Ok(None)
    }
}

/// Determine the status of an executable code node from kernel `TaskInfo` and list of messages
fn code_execute_status(task_info: &TaskInfo, errors: &[CodeError]) -> ExecuteStatus {
    if task_info.was_finished() {
        if errors.is_empty() {
            ExecuteStatus::Succeeded
        } else {
            ExecuteStatus::Failed
        }
    } else if task_info.was_interrupted() {
        ExecuteStatus::Cancelled
    } else if task_info.was_started() {
        ExecuteStatus::Running
    } else {
        ExecuteStatus::Scheduled
    }
}

#[async_trait]
impl Executable for CodeChunk {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        ensure_id!("cc", self, address, context);
        Ok(())
    }

    /// Compile a `CodeChunk` node
    ///
    /// Performs semantic analysis of the code (if language is supported) and adds the resulting
    /// relations to the compilation context. If the `programming_language` is an empty string
    /// then use the current language of the context.
    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;
        let lang = ensure_lang!(self, context);

        // Generate `ResourceInfo` by parsing the code
        let resource = resources::code(&context.path, id, "CodeChunk", Some(lang));
        let mut resource_info = match parsers::parse(resource, &self.text) {
            Ok(resource_info) => resource_info,
            Err(error) => {
                tracing::debug!("While parsing code chunk `{}`: {}", id, error);
                return Ok(());
            }
        };

        // Update the resource info with properties from the node
        resource_info.execute_digest = self
            .execute_digest
            .as_deref()
            .map(ResourceDigest::from_cord);
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

        context.global_tags.insert_globals(&resource_info.tags);
        context.resource_infos.push(resource_info);

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        is_fork: bool,
        _call_docs: &CallDocuments,
    ) -> Result<Option<TaskInfo>> {
        let id = assert_id!(self)?;
        tracing::trace!(
            "Executing `CodeChunk` `{}` with kernel selector: {}",
            id,
            kernel_selector
        );

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
        self.execute_ended = task_info.ended().map(|date| Box::new(Date::from(date)));
        self.execute_duration = task_info.duration().map(Number);
        self.execute_kernel = task_info.kernel_id.map(Box::new);

        // Update outputs and errors
        self.outputs = if outputs.is_empty() {
            None
        } else {
            Some(outputs)
        };
        self.errors = if errors.is_empty() {
            None
        } else {
            Some(errors)
        };

        Ok(())
    }
}

#[async_trait]
impl Executable for CodeExpression {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        ensure_id!("ce", self, address, context);
        Ok(())
    }

    /// Compile a `CodeExpression` node
    ///
    /// Performs semantic analysis of the code (if necessary) and adds the resulting
    /// relations to the compilation context. If the `programming_language` is an empty string
    /// then use the current language of the context.
    ///
    /// A `CodeExpression` is assumed to be pure (i.e. have no side effects and can be executed
    /// in a fork).
    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;
        let lang = ensure_lang!(self, context);

        // Generate `ResourceInfo` by parsing the code
        let resource = resources::code(
            &context.path,
            id,
            "CodeExpression",
            Some(normalize_title(&lang)),
        );
        let mut resource_info = match parsers::parse(resource, &self.text) {
            Ok(resource_info) => resource_info,
            Err(error) => {
                tracing::debug!("While parsing code expression `{}`: {}", id, error);
                return Ok(());
            }
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

        context.resource_infos.push(resource_info);

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        is_fork: bool,
        _call_docs: &CallDocuments,
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
        self.execute_ended = task_info.ended().map(|date| Box::new(Date::from(date)));
        self.execute_duration = task_info.duration().map(Number);
        self.execute_kernel = task_info.kernel_id.map(Box::new);

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

/// Compile a `SoftwareSourceCode` node
///
/// Performs semantic analysis of the code (if necessary) and adds the resulting
/// relations.
#[async_trait]
impl Executable for SoftwareSourceCode {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        ensure_id!("sc", self, address, context);
        Ok(())
    }

    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        if let (Some(code), Some(language)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            let resource = resources::code(
                &context.path,
                id,
                "SoftwareSourceCode",
                Some(language.clone()),
            );
            let resource_info = parsers::parse(resource, code)?;
            context.resource_infos.push(resource_info);
        }

        Ok(())
    }
}

fn include_digest(include: &Include, path: &Path) -> ResourceDigest {
    let mut content_string = include.source.clone();
    if let Some(media_type) = include.media_type.as_deref() {
        content_string.push_str(media_type);
    }
    if let Some(select) = include.select.as_deref() {
        content_string.push_str(select);
    }

    let mut digest = ResourceDigest::from_strings(&content_string, None);
    let dependency_digest =
        ResourceDigest::from_path(path, include.media_type.as_ref().map(|str| str.as_str()));
    digest.dependencies_digest = dependency_digest.content_digest;

    digest
}

#[async_trait]
impl Executable for Include {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        ensure_id!("in", self, address, context);

        let path = merge(&context.path, &self.source);
        let digest = include_digest(self, &path);

        let should_decode = self.content.is_none()
            || match self.compile_digest.as_deref() {
                Some(compile_digest) => digest.to_cord() != *compile_digest,
                None => true,
            };
        if should_decode {
            // Decode block content from the source
            let mut content = match codecs::from_path(
                &path,
                self.media_type
                    .as_ref()
                    .map(|media_type| media_type.as_str()),
                None,
            )
            .await
            {
                Ok(content) => Some(content.to_blocks()),
                Err(error) => Some(vec![BlockContent::Paragraph(Paragraph {
                    content: vec![InlineContent::String(error.to_string())],
                    ..Default::default()
                })]),
            };

            // Assemble the content as well
            content
                .assemble(&mut address.add_name("content"), context)
                .await?;

            // Generate a patch for changes to self
            let mut patch = diff_address(
                address.clone(),
                self,
                &Include {
                    content,
                    ..self.to_owned()
                },
            );
            patch.remove_overwrite_derived();
            context.patches.push(patch)
        }

        Ok(())
    }

    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

        // Add resource relations to the context
        let path = merge(&context.path, &self.source);
        let digest = include_digest(self, &path);
        let resource = resources::node(&context.path, id, "Include");
        let object = resources::file(&path);
        let relations = vec![(Relation::Includes, object)];
        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            self.execute_auto.clone(),
            Some(true), // Never has side effects
            Some(digest),
            None,
            None,
        );
        context.resource_infos.push(resource_info);

        // Compile any included content
        if let Some(content) = &self.content {
            content.compile(context).await?
        }

        Ok(())
    }
}

#[async_trait]
impl Executable for Call {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        let id = ensure_id!("ca", self, address, context);

        // Open the called document and register in `call_docs`
        let path = merge(&context.path, &self.source);
        let format = self.media_type.as_deref().cloned();
        tracing::trace!("Opening doc `{}` for call `{}`", path.display(), id);
        let doc = Document::open(path, format).await?;
        context.call_docs.write().await.insert(id, Mutex::new(doc));

        Ok(())
    }

    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        let id = assert_id!(self)?;

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
        let resource = resources::code(&context.path, id, "Call", None);
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

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _is_fork: bool,
        call_docs: &CallDocuments,
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
        let (execute_required, execute_status) = if succeeded {
            (ExecuteRequired::No, ExecuteStatus::Succeeded)
        } else {
            (ExecuteRequired::No, ExecuteStatus::Failed)
        };
        self.execute_required = Some(execute_required);
        self.execute_status = Some(execute_status);

        Ok(None)
    }
}

// Nodes types that simply need an `id` assigned so that custom web component patch events have a target

macro_rules! executable_assemble_id_only {
    ($type:ty, $prefix:expr) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(
                &mut self,
                address: &mut Address,
                context: &mut AssembleContext,
            ) -> Result<()> {
                ensure_id!($prefix, self, address, context);
                Ok(())
            }
        }
    };
}

executable_assemble_id_only!(CodeBlock, "cb");
executable_assemble_id_only!(CodeFragment, "cf");
executable_assemble_id_only!(MathBlock, "mb");
executable_assemble_id_only!(MathFragment, "mf");

// Node types that do not need anything done

macro_rules! executable_nothing {
    ( $( $type:ty ),* ) => {
        $(
            impl Executable for $type {}
        )*
    };
}
executable_nothing!(
    // Primitives
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
    // Entity types that are unlikely to ever need to be executable
    ThematicBreak,
    // Entity types that may need to be executable in the future
    Datatable,
    DatatableColumn,
    Periodical,
    PublicationIssue,
    PublicationVolume,
    Review,
    SoftwareApplication,
    Validator,
    ArrayValidator,
    BooleanValidator,
    ConstantValidator,
    EnumValidator,
    IntegerValidator,
    NumberValidator,
    StringValidator,
    TupleValidator,
    // External resources
    File
);

// The following are "enum variant dispatching" implementations of `Executable` for
// the types that are also `Pointable`

#[async_trait]
impl Executable for Node {
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        dispatch_node!(self, Box::pin(async { Ok(()) }), assemble, address, context).await
    }

    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        dispatch_node!(self, Box::pin(async { Ok(()) }), compile, context).await
    }

    async fn execute_begin(
        &mut self,
        resource_info: &ResourceInfo,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        is_fork: bool,
        call_docs: &CallDocuments,
    ) -> Result<Option<TaskInfo>> {
        dispatch_node!(
            self,
            Box::pin(async { Ok(None) }),
            execute_begin,
            resource_info,
            kernel_space,
            kernel_selector,
            is_fork,
            call_docs
        )
        .await
    }

    async fn execute_end(&mut self, task_info: TaskInfo, task_result: TaskResult) -> Result<()> {
        dispatch_node!(
            self,
            Box::pin(async { Ok(()) }),
            execute_end,
            task_info,
            task_result
        )
        .await
    }
}

macro_rules! executable_enum {
    ($type: ty, $dispatch_macro: ident) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(
                &mut self,
                address: &mut Address,
                context: &mut AssembleContext,
            ) -> Result<()> {
                $dispatch_macro!(self, assemble, address, context).await
            }

            async fn compile(&self, context: &mut CompileContext) -> Result<()> {
                $dispatch_macro!(self, compile, context).await
            }

            async fn execute_begin(
                &mut self,
                resource_info: &ResourceInfo,
                kernel_space: &KernelSpace,
                kernel_selector: &KernelSelector,
                is_fork: bool,
                call_docs: &CallDocuments,
            ) -> Result<Option<TaskInfo>> {
                $dispatch_macro!(
                    self,
                    execute_begin,
                    resource_info,
                    kernel_space,
                    kernel_selector,
                    is_fork,
                    call_docs
                )
                .await
            }

            async fn execute_end(
                &mut self,
                task_info: TaskInfo,
                task_result: TaskResult,
            ) -> Result<()> {
                $dispatch_macro!(self, execute_end, task_info, task_result).await
            }
        }
    };
}

executable_enum!(CreativeWorkTypes, dispatch_work);
executable_enum!(BlockContent, dispatch_block);
executable_enum!(InlineContent, dispatch_inline);

// The following implementations of `Executable` for generic types, only implement `assemble`
// to "collect up" `Executable` children into the `AssembleContext`.

#[async_trait]
impl<T> Executable for Option<T>
where
    T: Executable + Send + Sync,
{
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        match self {
            Some(value) => value.assemble(address, context).await,
            None => Ok(()),
        }
    }
}

#[async_trait]
impl<T> Executable for Box<T>
where
    T: Executable + Send + Sync,
{
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        (**self).assemble(address, context).await
    }
}

#[async_trait]
impl<T> Executable for Vec<T>
where
    T: Executable + Send + Sync,
{
    async fn assemble(
        &mut self,
        address: &mut Address,
        context: &mut AssembleContext,
    ) -> Result<()> {
        for (index, item) in self.iter_mut().enumerate() {
            address.push_back(Slot::Index(index));
            item.assemble(address, context).await?;
            address.pop_back();
        }
        Ok(())
    }
}

/// Implementation of `Executable` for `Pointer` allowing us to execute nodes in an address map
#[async_trait]
impl<'lt> Executable for Pointer<'lt> {
    async fn compile(&self, context: &mut CompileContext) -> Result<()> {
        match self {
            Pointer::Inline(inline) => inline.compile(context).await,
            Pointer::Block(block) => block.compile(context).await,
            Pointer::Node(node) => node.compile(context).await,
            Pointer::Work(work) => work.compile(context).await,
            _ => bail!("Unhandled pointer variant {:?}", self),
        }
    }
}

/// Implementation of `Executable` for various fields of a struct
macro_rules! executable_fields {
    ($type:ty $(, $field:ident)* ) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(&mut self, address: &mut Address, context: &mut AssembleContext) -> Result<()> {
                $(
                    address.push_back(Slot::Name(stringify!($field).to_string()));
                    self.$field.assemble(address, context).await?;
                    address.pop_back();
                )*
                Ok(())
            }
        }
    };
}

executable_fields!(CiteGroup, items);

executable_fields!(Collection, parts);
executable_fields!(Directory, parts);

executable_fields!(List, items);
executable_fields!(ListItem, item, content);

executable_fields!(Table, rows, caption);
executable_fields!(TableSimple, rows, caption);
executable_fields!(TableRow, cells);
executable_fields!(TableCell, content);

/// Implementation of `Executable` for only the `content` field of a struct
macro_rules! executable_content {
    ($type:ty) => {
        executable_fields!($type, content);
    };
    ( $( $type:ty ),* ) => {
        $(
            executable_content!($type);
        )*
    };
}

executable_content!(
    Article,
    Cite,
    Claim,
    ClaimSimple,
    Comment,
    CreativeWork,
    Delete,
    Emphasis,
    Figure,
    FigureSimple,
    Heading,
    NontextualAnnotation,
    Note,
    Paragraph,
    Quote,
    QuoteBlock,
    Strikeout,
    Strong,
    Subscript,
    Superscript,
    Underline
);

/// Implementation of `Executable` for enum variants
macro_rules! executable_variants {
    ( $type:ty $(, $variant:path )* ) => {
        #[async_trait]
        impl Executable for $type {
            async fn assemble(&mut self, address: &mut Address, context: &mut AssembleContext) -> Result<()> {
                match self {
                    $(
                        $variant(node) => node.assemble(address, context).await,
                    )*
                }
            }
        }
    };
}

executable_variants!(
    CreativeWorkContent,
    CreativeWorkContent::String,
    CreativeWorkContent::VecNode
);

executable_variants!(
    DirectoryParts,
    DirectoryParts::File,
    DirectoryParts::Directory
);

executable_variants!(
    ListItemContent,
    ListItemContent::VecInlineContent,
    ListItemContent::VecBlockContent
);

executable_variants!(
    TableCaption,
    TableCaption::String,
    TableCaption::VecBlockContent
);

executable_variants!(
    TableCellContent,
    TableCellContent::VecInlineContent,
    TableCellContent::VecBlockContent
);
