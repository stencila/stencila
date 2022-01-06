use async_trait::async_trait;
use eyre::{eyre, Result};
use formats::normalize_title;
use graph_triples::{relations, relations::NULL_RANGE, resources, Relation, Relations, ResourceId};
use kernels::{KernelSelector, KernelSpace, TaskResult};
use node_address::{Address, AddressMap, Slot};
use node_dispatch::{dispatch_block, dispatch_inline, dispatch_node, dispatch_work};
use parsers::{ParseInfo, ParseMap};
use path_utils::merge;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};
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

    /// Relations with other resources for each compiled resource
    /// in the document.
    pub(crate) relations: Relations,

    /// Parse results from parsing code during compilation
    pub(crate) parse_map: ParseMap,
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

#[derive(Debug, Default)]
pub struct ExecuteContext {
    /// Parse results from parsing code during compilation
    pub(crate) parse_info: BTreeMap<ResourceId, ParseInfo>,
}

/// Trait for executable document nodes
///
/// This trait is implemented below for all (or at least most) node types.
#[async_trait]
pub trait Executable {
    fn compile(&mut self, _address: &mut Address, _context: &mut CompileContext) -> Result<()> {
        Ok(())
    }

    async fn execute(
        &mut self,
        _kernel_space: &KernelSpace,
        _kernel_selector: &KernelSelector,
        _parse_info: Option<&ParseInfo>,
        _is_fork: bool,
    ) -> Result<()> {
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
        let subject = resources::node(&context.path, &id, "Link");
        let target = &self.target;
        let object = if target.starts_with("http://") || target.starts_with("https://") {
            resources::url(target)
        } else {
            resources::file(&merge(&context.path, target))
        };

        context
            .relations
            .push((subject, vec![(Relation::Link, object)]));

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
                let subject = resources::node(&context.path, &id, stringify!($type));
                let url = executable_content_url(&self.content_url, context);
                let object = if url.starts_with("http") || url.starts_with("data:") {
                    resources::url(&url)
                } else {
                    let url = url.strip_prefix("file://").unwrap_or(&url);
                    resources::file(&Path::new(&url))
                };
                context
                    .relations
                    .push((subject, vec![(Relation::Embed, object)]));

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

        let subject = resources::code(
            &context.path,
            &id,
            "Parameter",
            context.programming_language.clone(),
        );
        let resource_id = subject.id();

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
        context.relations.push((subject, relations.clone()));

        let value = self
            .value
            .as_deref()
            .or_else(|| self.default.as_deref())
            .map(|node| format!("{:?}", node))
            .unwrap_or_else(|| "".to_string());
        let parse_info = ParseInfo {
            relations,
            semantic_hash: ParseInfo::hash(&value),
            ..Default::default()
        };
        context.parse_map.insert(resource_id, parse_info);

        Ok(())
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        _parse_info: Option<&ParseInfo>,
        _is_fork: bool,
    ) -> Result<()> {
        tracing::debug!("Executing `Parameter`");

        if let Some(value) = self.value.as_deref().or_else(|| self.default.as_deref()) {
            kernel_space
                .set(&self.name, value.clone(), kernel_selector)
                .await?;
        }

        Ok(())
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

        let parse_info = match parsers::parse(&context.path, &self.text, &lang) {
            Ok(parse_info) => parse_info,
            Err(error) => {
                tracing::debug!("While parsing code chunk `{}`: {}", id, error);
                ParseInfo::default()
            }
        };

        let subject = resources::code(&context.path, &id, "CodeChunk", Some(lang));
        let resource_id = subject.id();

        context
            .relations
            .push((subject, parse_info.relations.clone()));

        context.parse_map.insert(resource_id, parse_info);

        Ok(())
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        parse_info: Option<&ParseInfo>,
        is_fork: bool,
    ) -> Result<()> {
        tracing::debug!("Executing `CodeChunk`");

        let parse_info =
            parse_info.ok_or_else(|| eyre!("Parse info is required to execute CodeChunk"))?;

        let mut task = kernel_space
            .exec(&self.text, parse_info, is_fork, kernel_selector)
            .await?;

        let TaskResult {
            outputs,
            messages: errors,
        } = task.result().await?;

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

        let parse_info = match parsers::parse(&context.path, &self.text, &lang) {
            Ok(parse_info) => parse_info,
            Err(error) => {
                tracing::debug!("While parsing code expression `{}`: {}", id, error);
                ParseInfo::default()
            }
        };

        let subject = resources::code(
            &context.path,
            &id,
            "CodeExpression",
            Some(normalize_title(&lang)),
        );
        let resource_id = subject.id();

        context
            .relations
            .push((subject, parse_info.relations.clone()));

        context.parse_map.insert(resource_id, parse_info);

        Ok(())
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        parse_info: Option<&ParseInfo>,
        is_fork: bool,
    ) -> Result<()> {
        tracing::debug!("Executing `CodeExpression`");

        let parse_info =
            parse_info.ok_or_else(|| eyre!("Parse info is required to execute CodeExpression"))?;

        let mut task = kernel_space
            .exec(&self.text, parse_info, is_fork, kernel_selector)
            .await?;

        let TaskResult {
            outputs,
            messages: errors,
        } = task.result().await?;

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
        identify!("sc", self, address, context);
        if let (Some(text), Some(programming_language)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            let subject = resources::file(&context.path);
            let parse_info = parsers::parse(&context.path, text, programming_language)?;
            context.relations.push((subject, parse_info.relations));
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
        let subject = resources::node(&context.path, &id, "Include");
        let path = merge(&context.path, &self.source);
        let object = resources::file(&path);

        context
            .relations
            .push((subject, vec![(Relation::Include, object)]));

        Ok(())
    }
}

// Nodes types that simply need an `id` assigned so that custom web component events to have a target

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

// The following are simple "dispatching" implementations of `compile`.
// They implement the depth first walk across a node tree by calling `compile`
// on child nodes and where necessary pushing slots onto the address.

#[async_trait]
impl Executable for Node {
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        dispatch_node!(self, Ok(()), compile, address, context)
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        parse_info: Option<&ParseInfo>,
        is_fork: bool,
    ) -> Result<()> {
        dispatch_node!(
            self,
            Box::pin(async { Ok(()) }),
            execute,
            kernel_space,
            kernel_selector,
            parse_info,
            is_fork
        )
        .await
    }
}

#[async_trait]
impl Executable for CreativeWorkTypes {
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        dispatch_work!(self, compile, address, context)
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        parse_info: Option<&ParseInfo>,
        is_fork: bool,
    ) -> Result<()> {
        dispatch_work!(
            self,
            execute,
            kernel_space,
            kernel_selector,
            parse_info,
            is_fork
        )
        .await
    }
}

#[async_trait]
impl Executable for BlockContent {
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        dispatch_block!(self, compile, address, context)
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        parse_info: Option<&ParseInfo>,
        is_fork: bool,
    ) -> Result<()> {
        dispatch_block!(
            self,
            execute,
            kernel_space,
            kernel_selector,
            parse_info,
            is_fork
        )
        .await
    }
}

#[async_trait]
impl Executable for InlineContent {
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        dispatch_inline!(self, compile, address, context)
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        parse_info: Option<&ParseInfo>,
        is_fork: bool,
    ) -> Result<()> {
        dispatch_inline!(
            self,
            execute,
            kernel_space,
            kernel_selector,
            parse_info,
            is_fork
        )
        .await
    }
}

#[async_trait]
impl<T> Executable for Option<T>
where
    T: Executable + Send,
{
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        if let Some(value) = self {
            value.compile(address, context)
        } else {
            Ok(())
        }
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        parse_info: Option<&ParseInfo>,
        is_fork: bool,
    ) -> Result<()> {
        if let Some(value) = self {
            value
                .execute(kernel_space, kernel_selector, parse_info, is_fork)
                .await
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<T> Executable for Box<T>
where
    T: Executable + Send,
{
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        (**self).compile(address, context)
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        parse_info: Option<&ParseInfo>,
        is_fork: bool,
    ) -> Result<()> {
        (**self)
            .execute(kernel_space, kernel_selector, parse_info, is_fork)
            .await
    }
}

#[async_trait]
impl<T> Executable for Vec<T>
where
    T: Executable + Send,
{
    fn compile(&mut self, address: &mut Address, context: &mut CompileContext) -> Result<()> {
        for (index, item) in self.iter_mut().enumerate() {
            address.push_back(Slot::Index(index));
            item.compile(address, context)?;
            address.pop_back();
        }
        Ok(())
    }

    async fn execute(
        &mut self,
        kernel_space: &KernelSpace,
        kernel_selector: &KernelSelector,
        parse_info: Option<&ParseInfo>,
        is_fork: bool,
    ) -> Result<()> {
        for item in self.iter_mut() {
            item.execute(kernel_space, kernel_selector, parse_info, is_fork)
                .await?;
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

            async fn execute(
                &mut self,
                kernel_space: &KernelSpace,
                kernel_selector: &KernelSelector,
                parse_info: Option<&ParseInfo>,
                is_fork: bool,
            ) -> Result<()> {
                $(
                    self.$field.execute(kernel_space, kernel_selector, parse_info, is_fork).await?;
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
    Strong,
    Subscript,
    Superscript,
    Table,
    TableSimple
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

            async fn execute(
                &mut self,
                kernel_space: &KernelSpace,
                kernel_selector: &KernelSelector,
                parse_info: Option<&ParseInfo>,
                is_fork: bool,
            ) -> Result<()> {
                match self {
                    $(
                        $variant(node) => node.execute(kernel_space, kernel_selector, parse_info, is_fork).await,
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
