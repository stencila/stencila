use crate::{
    dispatch_block, dispatch_inline, dispatch_node, dispatch_work,
    patches::{Address, Slot},
    utils::hash::str_sha256_hex,
};
use async_trait::async_trait;
use eyre::Result;
use graph_triples::{relations, relations::NULL_RANGE, resources, Relation, Resource};
use kernels::{KernelSelector, KernelSpace};
use path_utils::merge;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};
use stencila_schema::*;

type Addresses = HashMap<String, Address>;
type Relations = HashMap<Resource, Vec<(Relation, Resource)>>;

/// Compile a node
///
/// Compiling a document involves walking over the node tree and compiling each
/// individual node so that it is ready to be built & executed. This includes
/// (but is not limited to):
///
/// - for those node types needing to be accesses directly (e.g. executable nodes) ensuring
///   they have an `id` and recording their address
/// - for executable nodes (e.g. `CodeChunk`) performing semantic analysis of the code
/// - determining dependencies within and between documents and other resources
pub fn compile(node: &mut Node, path: &Path, project: &Path) -> Result<(Addresses, Relations)> {
    let mut address = Address::default();
    let mut context = Context {
        path: PathBuf::from(path),
        project: PathBuf::from(project),
        ..Default::default()
    };
    node.compile(&mut address, &mut context)?;

    let addresses = context.addresses;
    let relations = context.relations.into_iter().collect();
    Ok((addresses, relations))
}

pub async fn execute<Type>(node: &mut Type, kernels: &mut KernelSpace) -> Result<()>
where
    Type: Compile + Send,
{
    node.execute(kernels).await
}

/// The compilation context, used to pass down properties of the
/// root node and to record inputs and outputs etc during compilation
#[derive(Debug, Default)]
pub struct Context {
    /// The path of the document being compiled.
    /// Used to resolve relative paths e.g. in `ImageObject` and `Include` nodes
    path: PathBuf,

    /// The project that the document is within.
    /// Used to restrict any file links to be within the project
    project: PathBuf,

    /// A map of node ids to addresses
    pub addresses: HashMap<String, Address>,

    /// Relations with other resources for each compiled resource
    /// in the document.
    pub relations: Vec<(Resource, Vec<(Relation, Resource)>)>,
}

/// Trait for compiling a node
///
/// This trait is implemented below for all (or at least most)
/// node types.
#[async_trait]
pub trait Compile {
    fn compile(&mut self, _address: &mut Address, _context: &mut Context) -> Result<()> {
        Ok(())
    }

    async fn execute(&mut self, _kernels: &mut KernelSpace) -> Result<()> {
        Ok(())
    }
}

/// Identify a node
///
/// If the node does not have an id, generate and assign one.
/// These generated id belong to the `Node` family (i.e. have a leading "no-")
/// which can be used to determine that it was generated (so, for example
/// it is not persisted. Return the node's id.
///
/// This needs to be (?) a macro, rather than a generic function, because
/// it is not possible to define a bound that the type must have the `id` property.
macro_rules! identify {
    ($node:expr, $address:expr, $context:expr) => {{
        let id = if let Some(id) = $node.id.as_deref() {
            id.clone()
        } else {
            let id = uuids::generate("no").to_string();
            $node.id = Some(Box::new(id.clone()));
            id
        };
        $context.addresses.insert(id.clone(), $address.clone());
        id
    }};
}

// This first set of implementations are for node types that need
// some sort of compilation.

/// Compile a `Link` node
///
/// Adds a `Link` relation
impl Compile for Link {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self, address, context);
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
fn compile_content_url(content_url: &str, context: &mut Context) -> String {
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
macro_rules! compile_media_object {
    ( $( $type:ty ),* ) => {
        $(
            impl Compile for $type {
                fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
                    let id = identify!(self, address, context);
                    let subject = resources::node(&context.path, &id, stringify!($type));
                    let url = compile_content_url(&self.content_url, context);
                    let object = if url.starts_with("http") || url.starts_with("data:") {
                        resources::url(&url)
                    } else {
                        let url = url.strip_prefix("file://").unwrap_or(&url);
                        resources::file(&Path::new(&url))
                    };
                    context.relations.push((subject, vec![(Relation::Embed, object)]));

                    self.content_url = url;

                    Ok(())
                }
            }
        )*
    };
}

compile_media_object!(
    AudioObject,
    AudioObjectSimple,
    ImageObject,
    ImageObjectSimple,
    MediaObject,
    VideoObject,
    VideoObjectSimple
);

/// Compile a `Parameter` node
///
/// Adds an `Assign` relation.
#[async_trait]
impl Compile for Parameter {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self, address, context);
        let subject = resources::node(&context.path, &id, "Parameter");
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

        context.relations.push((subject, relations));

        Ok(())
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        tracing::debug!("Executing `Parameter`");
        if let Some(value) = self.value.as_deref() {
            kernels.set(&self.name, value.clone(), "").await?;
        }
        Ok(())
    }
}

/// Compile a `CodeChunk` node
///
/// Performs semantic analysis of the code (if necessary) and adds the resulting
/// relations.
#[async_trait]
impl Compile for CodeChunk {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self, address, context);
        let digest =
            str_sha256_hex(&[self.text.as_str(), self.programming_language.as_str()].concat());
        if Some(digest.clone()) != self.compile_digest {
            match parsers::parse(&context.path, &self.text, &self.programming_language) {
                Ok(relations) => {
                    let subject = resources::node(&context.path, &id, "CodeChunk");
                    context.relations.push((subject, relations));
                    self.compile_digest = Some(digest);
                }
                Err(error) => tracing::warn!("While parsing code chunk `{}`: {}", id, error),
            };
        }

        Ok(())
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        tracing::debug!("Executing `CodeChunk`");

        // TODO: Pass relations hashmap in context for lookup instead of re-compiling
        let relations = parsers::parse("", &self.text, &self.programming_language)?;
        let selector = KernelSelector::new(None, Some(self.programming_language.clone()), None);
        let (outputs, errors) = kernels.exec(&self.text, &selector, Some(relations)).await?;

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

/// Compile a `CodeExpression` node
///
/// Performs semantic analysis of the code (if necessary) and adds the resulting
/// relations.
#[async_trait]
impl Compile for CodeExpression {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self, address, context);
        let digest =
            str_sha256_hex(&[self.text.as_str(), self.programming_language.as_str()].concat());
        if Some(digest.clone()) != self.compile_digest {
            match parsers::parse(&context.path, &self.text, &self.programming_language) {
                Ok(relations) => {
                    let subject = resources::node(&context.path, &id, "CodeExpression");
                    context.relations.push((subject, relations));
                    self.compile_digest = Some(digest);
                }
                Err(error) => tracing::warn!("While parsing code expression `{}`: {}", id, error),
            };
        }

        Ok(())
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        tracing::debug!("Executing `CodeExpression`");

        // TODO: Pass relations hashmap in context for lookup instead of re-compiling
        let relations = parsers::parse("", &self.text, &self.programming_language)?;
        let selector = KernelSelector::new(None, Some(self.programming_language.clone()), None);
        let (outputs, errors) = kernels.exec(&self.text, &selector, Some(relations)).await?;

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
impl Compile for SoftwareSourceCode {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        identify!(self, address, context);
        if let (Some(text), Some(programming_language)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            let subject = resources::file(&context.path);
            let relations = parsers::parse(&context.path, text, programming_language)?;
            context.relations.push((subject, relations));
        }

        Ok(())
    }
}

/// Compile an `Include` node
///
/// Adds an `Include` relation
impl Compile for Include {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self, address, context);
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

macro_rules! compile_identify_only {
    ( $( $type:ty ),* ) => {
        $(
            #[async_trait]
            impl Compile for $type {
                fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
                    identify!(self, address, context);
                    Ok(())
                }
            }
        )*
    };
}

compile_identify_only!(CodeBlock, CodeFragment, MathBlock, MathFragment);

// Node types that do not need anything done

macro_rules! compile_nothing {
    ( $( $type:ty ),* ) => {
        $(
            impl Compile for $type {}
        )*
    };
}
compile_nothing!(
    // Primitives
    Null,
    Boolean,
    Integer,
    Number,
    String,
    Array,
    Object,
    // Entity types that are unlikely to need anything else done to them
    ThematicBreak,
    // Entity types that may need to be compiled but are here for now
    Datatable,
    Periodical,
    PublicationIssue,
    PublicationVolume,
    Review,
    SoftwareApplication
);

// The following are simple "dispatching" implementations of `compile`.
// They implement the depth first walk across a node tree by calling `compile`
// on child nodes and where necessary pushing slots onto the address.

#[async_trait]
impl Compile for Node {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        dispatch_node!(self, Ok(()), compile, address, context)
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        dispatch_node!(self, Box::pin(async { Ok(()) }), execute, kernels).await
    }
}

#[async_trait]
impl Compile for CreativeWorkTypes {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        dispatch_work!(self, compile, address, context)
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        dispatch_work!(self, execute, kernels).await
    }
}

#[async_trait]
impl Compile for BlockContent {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        dispatch_block!(self, compile, address, context)
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        dispatch_block!(self, execute, kernels).await
    }
}

#[async_trait]
impl Compile for InlineContent {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        dispatch_inline!(self, compile, address, context)
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        dispatch_inline!(self, execute, kernels).await
    }
}

#[async_trait]
impl<T> Compile for Option<T>
where
    T: Compile + Send,
{
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        if let Some(value) = self {
            value.compile(address, context)
        } else {
            Ok(())
        }
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        if let Some(value) = self {
            value.execute(kernels).await
        } else {
            Ok(())
        }
    }
}

#[async_trait]
impl<T> Compile for Box<T>
where
    T: Compile + Send,
{
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        (**self).compile(address, context)
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        (**self).execute(kernels).await
    }
}

#[async_trait]
impl<T> Compile for Vec<T>
where
    T: Compile + Send,
{
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        for (index, item) in self.iter_mut().enumerate() {
            address.push_back(Slot::Index(index));
            item.compile(address, context)?;
            address.pop_back();
        }
        Ok(())
    }

    async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
        for item in self.iter_mut() {
            item.execute(kernels).await?;
        }
        Ok(())
    }
}

/// Compile fields of a struct
macro_rules! compile_fields {
    ($type:ty $(, $field:ident)* ) => {
        #[async_trait]
        impl Compile for $type {
            fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
                $(
                    address.push_back(Slot::Name(stringify!($field).to_string()));
                    self.$field.compile(address, context)?;
                    address.pop_back();
                )*
                Ok(())
            }

            async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
                $(
                    self.$field.execute(kernels).await?;
                )*
                Ok(())
            }
        }
    };
}

compile_fields!(CiteGroup, items);
compile_fields!(Collection, parts);
compile_fields!(CollectionSimple, parts);
compile_fields!(List, items);
compile_fields!(ListItem, item, content);

/// Compile the content field of a struct only
macro_rules! compile_content {
    ($type:ty) => {
        compile_fields!($type, content);
    };
}

/// Compile content for several types
macro_rules! compile_content_for {
    ( $( $type:ty ),* ) => {
        $(
            compile_content!($type);
        )*
    };
}

compile_content_for!(
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
macro_rules! compile_variants {
    ( $type:ty $(, $variant:path )* ) => {
        #[async_trait]
        impl Compile for $type {
            fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
                match self {
                    $(
                        $variant(node) => node.compile(address, context),
                    )*
                }
            }

            async fn execute(&mut self, kernels: &mut KernelSpace) -> Result<()> {
                match self {
                    $(
                        $variant(node) => node.execute(kernels).await,
                    )*
                }
            }
        }
    };
}

compile_variants!(
    CreativeWorkContent,
    CreativeWorkContent::String,
    CreativeWorkContent::VecNode
);

compile_variants!(
    ListItemContent,
    ListItemContent::VecInlineContent,
    ListItemContent::VecBlockContent
);
