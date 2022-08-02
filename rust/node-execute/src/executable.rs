use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use common::{async_trait::async_trait, eyre::Result, tracing};
use formats::normalize_title;
use graph_triples::{
    relations,
    relations::NULL_RANGE,
    resources::{self, ResourceDigest},
    Relation, ResourceInfo,
};
use kernels::{KernelSelector, KernelSpace, TaskInfo, TaskResult};
use node_address::{Address, AddressMap, Slot};
use node_dispatch::{dispatch_block, dispatch_inline, dispatch_node, dispatch_work};
use node_validate::Validator as _;
use path_utils::merge;
use stencila_schema::*;

/// The compilation context, used to pass down properties of the
/// root node and to record inputs and outputs etc during compilation
#[derive(Debug, Default)]
pub struct CompileContext {
    /// The path of the document being compiled.
    /// Used to resolve relative paths e.g. in `ImageObject` and `Include` nodes
    path: PathBuf,

    /// The project that the document is within.
    /// Used to restrict any file links to be within the project
    project: PathBuf,

    /// Counts of the number of node ids with each prefix assigned
    ///
    /// Used to generate unique (to a compilation context, usually a document)
    /// but short and meaningful (prefix relates to the node type). An additional
    /// advantage is that the generated ids are deterministic.
    ids: HashMap<String, usize>,

    /// The programming language of the last code node encountered during
    /// compilation
    programming_language: Option<String>,

    /// A map of node ids to addresses
    pub(crate) address_map: AddressMap,

    /// A list of resources compiles from the node
    pub(crate) resource_infos: Vec<ResourceInfo>,
}

impl CompileContext {
    // Create a new compilation context
    pub fn new(path: &Path, project: &Path) -> Self {
        Self {
            path: path.into(),
            project: project.into(),
            ..Default::default()
        }
    }

    /// Generate an id for a given id "family"
    fn identify(&mut self, prefix: &str) -> String {
        let count = self
            .ids
            .entry(prefix.to_string())
            .and_modify(|count| *count += 1)
            .or_insert(1);
        [prefix, "-", &count.to_string()].concat()
    }
}

/// Trait for executable document nodes
///
/// This trait is implemented below for all (or at least most) node types.
#[async_trait]
pub trait Executable {
    fn compile(&mut self, _address: &mut Address, _context: &mut CompileContext) -> Result<()> {
        Ok(())
    }

    async fn execute_begin(
        &mut self,
        _kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _resource_info: &ResourceInfo,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        Ok(None)
    }

    fn execute_end(&mut self, _task_info: TaskInfo, _task_result: TaskResult) -> Result<()> {
        Ok(())
    }
}

/// Identify a node
///
/// If the node does not have an id, generate and assign one.
/// These generated ids use a prefix reflecting the node type (i.g. "cc-" for `CodeChunk` nodes)
/// which can be used to determine that it was generated (so, for example
/// it is not persisted). Returns the node's id.
///
/// This needs to be (?) a macro, rather than a generic function, because
/// it is not possible to define a bound that the type must have the `id` property.
macro_rules! identify {
    ($prefix:expr, $node:expr, $address:expr, $context:expr) => {{
        let id = if let Some(id) = $node.id.as_deref() {
            id.clone()
        } else {
            let id = $context.identify($prefix);
            $node.id = Some(Box::new(id.clone()));
            id
        };
        $context.address_map.insert(id.clone(), $address.clone());
        id
    }};
}

/// Set the programming of a node or of the context
///
/// Ok, bad name but it's like `identify!`: if the node does
/// not have a `programming_language` then we'll use the context's
/// and if it does than we'll set the context's.
macro_rules! langify {
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

// This first set of implementations are for node types that need
// some sort of compilation.

/// Compile a `Link` node
///
/// Adds a `Link` relation
impl Executable for Link {
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = identify!("li", self, address, context);

        let resource = resources::node(&context.path, &id, "Link");

        let target = &self.target;
        let object = if target.starts_with("http://") || target.starts_with("https://") {
            resources::url(target)
        } else {
            resources::file(&merge(&context.path, target))
        };
        let relations = vec![(Relation::Link, object)];

        let resource_info =
            ResourceInfo::new(resource, Some(relations), None, None, None, None, None);
        context.resource_infos.push(resource_info);

        Ok(())
    }
}

/// Compile to `content_url` property of `MediaObject` node types
///
/// If the `content_url` property is  a `file://` URL (implicitly
/// or explicitly) then resolves the file path, records it as
/// a file dependency, and returns an absolute `file://` URL.
fn executable_content_url(content_url: &str, context: &mut CompileContext) -> String {
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
macro_rules! executable_media_object {
    ($type:ty, $prefix:expr) => {
        impl Executable for $type {
            fn compile(
                &mut self,
                address: &mut Address,
                context: &mut CompileContext,
            ) -> Result<()> {
                let id = identify!($prefix, self, address, context);

                let resource = resources::node(&context.path, &id, stringify!($type));

                let url = executable_content_url(&self.content_url, context);
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

                self.content_url = url;

                Ok(())
            }
        }
    };
}

executable_media_object!(AudioObject, "au");
executable_media_object!(AudioObjectSimple, "au");
executable_media_object!(ImageObject, "im");
executable_media_object!(ImageObjectSimple, "im");
executable_media_object!(MediaObject, "me");
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

#[async_trait]
impl Executable for Parameter {
    /// Compile a `Parameter` node
    ///
    /// Adds an `Assign` relation to the compilation context with the name and kind of value.
    /// If the language of the parameter is not defined (currently schema does not allow for this
    /// anyway), then use the language of the last code node. By definition, a `Parameter` is always
    /// "impure" (has a side effect).
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = identify!("pa", self, address, context);

        let resource = resources::code(
            &context.path,
            &id,
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

        // For `ResourceDigest`, content string is `Debug` representation of the parameter value
        // and semantic string adds the parameter name
        let value = parameter_value(self);
        let content_str = format!("{:?}", value);
        let semantic_str = [self.name.as_str(), content_str.as_str()].concat();

        let resource_info = ResourceInfo::new(
            resource,
            Some(relations),
            None,
            None,
            Some(ResourceDigest::from_strings(
                &content_str,
                Some(&semantic_str),
            )),
            None,
            None,
        );
        context.resource_infos.push(resource_info);

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        _resource_info: &ResourceInfo,
        _is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        tracing::trace!("Executing `Parameter`");

        let value = parameter_value(self);

        kernel_space
            .set(&self.name, value.clone(), kernel_selector)
            .await?;

        if self.value.is_none() {
            self.value = Some(Box::new(value));
        }

        Ok(None)
    }
}

/// Determine the status of an executable code node from kernel `TaskInfo` and list of messages
fn code_execute_status(task_info: &TaskInfo, errors: &[CodeError]) -> CodeExecutableExecuteStatus {
    if task_info.was_finished() {
        if errors.is_empty() {
            CodeExecutableExecuteStatus::Succeeded
        } else {
            CodeExecutableExecuteStatus::Failed
        }
    } else if task_info.was_cancelled() {
        CodeExecutableExecuteStatus::Cancelled
    } else if task_info.was_started() {
        CodeExecutableExecuteStatus::Running
    } else {
        CodeExecutableExecuteStatus::Scheduled
    }
}

#[async_trait]
impl Executable for CodeChunk {
    /// Compile a `CodeChunk` node
    ///
    /// Performs semantic analysis of the code (if language is supported) and adds the resulting
    /// relations to the compilation context. If the `programming_language` is an empty string
    /// then use the current language of the context.
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = identify!("cc", self, address, context);
        let lang = langify!(self, context);

        // Generate `ResourceInfo` by parsing the code
        let resource = resources::code(&context.path, &id, "CodeChunk", Some(lang));
        let mut resource_info = match parsers::parse(resource, &self.text) {
            Ok(resource_info) => resource_info,
            Err(error) => {
                tracing::debug!("While parsing code chunk `{}`: {}", id, error);
                return Ok(());
            }
        };

        // Update the node with properties from resource info derived from parsing it's code
        self.execute_auto = resource_info.execute_auto.clone();
        self.execute_pure = resource_info.execute_pure;

        // Update the resource info with properties from the node
        resource_info.execute_digest = self
            .execute_digest
            .as_ref()
            .map(|cord| ResourceDigest::from_string(&cord.0));
        resource_info.execute_status = self.execute_status.clone();

        context.resource_infos.push(resource_info);

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        resource_info: &ResourceInfo,
        is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        tracing::trace!("Executing `CodeChunk` `{:?}`", self.id);

        let task_info = kernel_space
            .exec(&self.text, resource_info, is_fork, kernel_selector)
            .await?;

        Ok(Some(task_info))
    }

    fn execute_end(&mut self, task_info: TaskInfo, task_result: TaskResult) -> Result<()> {
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
        self.execute_required = Some(
            if matches!(execute_status, CodeExecutableExecuteStatus::Succeeded) {
                CodeExecutableExecuteRequired::No
            } else {
                CodeExecutableExecuteRequired::Failed
            },
        );
        self.execute_status = Some(execute_status);
        self.execute_ended = task_info.ended().map(|date| Box::new(Date::from(date)));
        self.execute_duration = task_info.duration().map(Number);

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
    /// Compile a `CodeExpression` node
    ///
    /// Performs semantic analysis of the code (if necessary) and adds the resulting
    /// relations to the compilation context. If the `programming_language` is an empty string
    /// then use the current language of the context.
    ///
    /// A `CodeExpression` is assumed to be pure (i.e. have no side effects and can be executed
    /// in a fork).
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = identify!("ce", self, address, context);
        let lang = langify!(self, context);

        // Generate `ResourceInfo` by parsing the code
        let resource = resources::code(
            &context.path,
            &id,
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
        resource_info.execute_status = self.execute_status.clone();

        // Force code expression execution semantics (in case `@impure` or `@autorun` tags
        // where inadvertently used in code) by setting to `None`
        resource_info.execute_auto = None;
        resource_info.execute_pure = None;

        context.resource_infos.push(resource_info);

        Ok(())
    }

    async fn execute_begin(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        resource_info: &ResourceInfo,
        is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        tracing::trace!("Executing `CodeExpression` `{:?}`", self.id);

        let task_info = kernel_space
            .exec(&self.text, resource_info, is_fork, kernel_selector)
            .await?;

        Ok(Some(task_info))
    }

    fn execute_end(&mut self, task_info: TaskInfo, task_result: TaskResult) -> Result<()> {
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
        self.execute_required = Some(
            if matches!(execute_status, CodeExecutableExecuteStatus::Succeeded) {
                CodeExecutableExecuteRequired::No
            } else {
                CodeExecutableExecuteRequired::Failed
            },
        );
        self.execute_status = Some(execute_status);
        self.execute_ended = task_info.ended().map(|date| Box::new(Date::from(date)));
        self.execute_duration = task_info.duration().map(Number);

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
impl Executable for SoftwareSourceCode {
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = identify!("sc", self, address, context);

        if let (Some(code), Some(language)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            let resource = resources::code(
                &context.path,
                &id,
                "SoftwareSourceCode",
                Some(language.clone()),
            );
            let resource_info = parsers::parse(resource, code)?;
            context.resource_infos.push(resource_info);
        }

        Ok(())
    }
}

/// Compile an `Include` node
///
/// Adds an `Include` relation
impl Executable for Include {
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        let id = identify!("in", self, address, context);

        let resource = resources::node(&context.path, &id, "Include");

        let path = merge(&context.path, &self.source);
        let object = resources::file(&path);
        let relations = vec![(Relation::Include, object)];

        let resource_info =
            ResourceInfo::new(resource, Some(relations), None, None, None, None, None);
        context.resource_infos.push(resource_info);

        Ok(())
    }
}

// Nodes types that simply need an `id` assigned so that custom web component patch events have a target

macro_rules! executable_identify_only {
    ($type:ty, $prefix:expr) => {
        #[async_trait]
        impl Executable for $type {
            fn compile(
                &mut self,
                address: &mut Address,
                context: &mut CompileContext,
            ) -> Result<()> {
                identify!($prefix, self, address, context);
                Ok(())
            }
        }
    };
}

executable_identify_only!(CodeBlock, "cb");
executable_identify_only!(CodeFragment, "cf");
executable_identify_only!(MathBlock, "mb");
executable_identify_only!(MathFragment, "mf");

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
    TupleValidator
);

// The following are "enum variant dispatching" implementations of `Executable` for
// the types that are also `Pointable`

#[async_trait]
impl Executable for Node {
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        dispatch_node!(self, Ok(()), compile, address, context)
    }

    async fn execute_begin(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        resource_info: &ResourceInfo,
        is_fork: bool,
    ) -> Result<Option<TaskInfo>> {
        dispatch_node!(
            self,
            Box::pin(async { Ok(None) }),
            execute_begin,
            kernel_space,
            kernel_selector,
            resource_info,
            is_fork
        )
        .await
    }

    fn execute_end(&mut self, task_info: TaskInfo, task_result: TaskResult) -> Result<()> {
        dispatch_node!(self, Ok(()), execute_end, task_info, task_result)
    }
}

macro_rules! executable_enum {
    ($type: ty, $dispatch_macro: ident) => {
        #[async_trait]
        impl Executable for $type {
            fn compile(
                &mut self,
                address: &mut Address,
                context: &mut CompileContext,
            ) -> Result<()> {
                $dispatch_macro!(self, compile, address, context)
            }

            async fn execute_begin(
                &mut self,
                kernel_space: &KernelSpace,
                kernel_selector: &KernelSelector,
                resource_info: &ResourceInfo,
                is_fork: bool,
            ) -> Result<Option<TaskInfo>> {
                $dispatch_macro!(
                    self,
                    execute_begin,
                    kernel_space,
                    kernel_selector,
                    resource_info,
                    is_fork
                )
                .await
            }

            fn execute_end(&mut self, task_info: TaskInfo, task_result: TaskResult) -> Result<()> {
                $dispatch_macro!(self, execute_end, task_info, task_result)
            }
        }
    };
}

executable_enum!(CreativeWorkTypes, dispatch_work);
executable_enum!(BlockContent, dispatch_block);
executable_enum!(InlineContent, dispatch_inline);

// The following implementations of `Executable` for generic types, only implement `compile`
// to "collect up" executable children into the `CompileContext` when they are walked over.

#[async_trait]
impl<T> Executable for Option<T>
where
    T: Executable,
{
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        match self {
            Some(value) => value.compile(address, context),
            None => Ok(()),
        }
    }
}

#[async_trait]
impl<T> Executable for Box<T>
where
    T: Executable,
{
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        (**self).compile(address, context)
    }
}

#[async_trait]
impl<T> Executable for Vec<T>
where
    T: Executable,
{
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        for (index, item) in self.iter_mut().enumerate() {
            address.push_back(Slot::Index(index));
            item.compile(address, context)?;
            address.pop_back();
        }
        Ok(())
    }
}

/// Compile fields of a struct
macro_rules! executable_fields {
    ($type:ty $(, $field:ident)* ) => {
        #[async_trait]
        impl Executable for $type {
            fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
                $(
                    address.push_back(Slot::Name(stringify!($field).to_string()));
                    self.$field.compile(address, context)?;
                    address.pop_back();
                )*
                Ok(())
            }
        }
    };
}

executable_fields!(CiteGroup, items);

executable_fields!(Collection, parts);
executable_fields!(CollectionSimple, parts);

executable_fields!(List, items);
executable_fields!(ListItem, item, content);

executable_fields!(Table, rows, caption);
executable_fields!(TableSimple, rows, caption);
executable_fields!(TableRow, cells);
executable_fields!(TableCell, content);

/// Compile the content field of a struct only
macro_rules! executable_content {
    ($type:ty) => {
        executable_fields!($type, content);
    };
}

/// Compile content for several types
macro_rules! executable_content_for {
    ( $( $type:ty ),* ) => {
        $(
            executable_content!($type);
        )*
    };
}

executable_content_for!(
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

/// Compile variants of an enum
macro_rules! executable_variants {
    ( $type:ty $(, $variant:path )* ) => {
        #[async_trait]
        impl Executable for $type {
            fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
                match self {
                    $(
                        $variant(node) => node.compile(address, context),
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
