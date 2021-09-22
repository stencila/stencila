use crate::{
    dispatch_block, dispatch_inline, dispatch_node, dispatch_work,
    graphs::{relations, resources, Relation, Resource, NULL_RANGE},
    patches::{Address, Slot},
    utils::{hash::str_sha256_hex, path::merge, uuids},
};
use eyre::Result;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};
use stencila_schema::*;

#[cfg(feature = "compile-code")]
pub mod code;

#[cfg(not(feature = "compile-code"))]
pub mod code {
    use super::*;
    pub fn compile(path: &Path, code: &str, language: &str) -> Vec<(Relation, Resource)> {
        Vec::new()
    }
}

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
trait Compile {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()>;
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
    ($node:expr) => {
        if let Some(id) = $node.id.as_deref() {
            id.clone()
        } else {
            let id = uuids::generate(uuids::Family::Node);
            $node.id = Some(Box::new(id.clone()));
            id
        }
    };
}

// This first set of implementations are for node types that need
// some sort of compilation.

/// Compile a `Link` node
///
/// Adds a `Link` relation
impl Compile for Link {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self);
        context.addresses.insert(id.clone(), address.clone());

        let subject = resources::node(&context.path, &id, &self.type_name());
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
    if content_url.starts_with("http://") || content_url.starts_with("https://") {
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
                    let id = identify!(self);
                    context.addresses.insert(id.clone(), address.clone());

                    let subject = resources::node(&context.path, &id, &self.type_name());
                    let url = compile_content_url(&self.content_url, context);
                    let object = if url.starts_with("http") {
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
impl Compile for Parameter {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self);
        context.addresses.insert(id.clone(), address.clone());

        let subject = resources::node(&context.path, &id, &self.type_name());
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
}

/// Compile a `CodeChunk` node
///
/// Performs semantic analysis of the code (if necessary) and adds the resulting
/// relations.
impl Compile for CodeChunk {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self);
        context.addresses.insert(id.clone(), address.clone());

        let digest =
            str_sha256_hex(&[self.text.as_str(), self.programming_language.as_str()].concat());
        if Some(digest.clone()) != self.compile_digest {
            let subject = resources::node(&context.path, &id, &self.type_name());
            let relations = code::compile(&context.path, &self.text, &self.programming_language);
            context.relations.push((subject, relations));
            self.compile_digest = Some(digest)
        }

        Ok(())
    }
}

/// Compile a `CodeExpression` node
///
/// Performs semantic analysis of the code (if necessary) and adds the resulting
/// relations.
impl Compile for CodeExpression {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self);
        context.addresses.insert(id.clone(), address.clone());

        let digest =
            str_sha256_hex(&[self.text.as_str(), self.programming_language.as_str()].concat());
        if Some(digest.clone()) != self.compile_digest {
            let subject = resources::node(&context.path, &id, &self.type_name());
            let relations = code::compile(&context.path, &self.text, &self.programming_language);
            context.relations.push((subject, relations));
            self.compile_digest = Some(digest);
        }

        Ok(())
    }
}

/// Compile a `SoftwareSourceCode` node
///
/// Performs semantic analysis of the code (if necessary) and adds the resulting
/// relations.
impl Compile for SoftwareSourceCode {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        let id = identify!(self);
        context.addresses.insert(id, address.clone());

        if let (Some(text), Some(programming_language)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            let subject = resources::file(&context.path);
            let relations = code::compile(&context.path, text, programming_language);
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
        let id = identify!(self);
        context.addresses.insert(id.clone(), address.clone());

        let subject = resources::node(&context.path, &id, &self.type_name());
        let path = merge(&context.path, &self.source);
        let object = resources::file(&path);

        context
            .relations
            .push((subject, vec![(Relation::Include, object)]));

        Ok(())
    }
}

// The following are simple "dispatching" implementations of `compile`.
// They implement the depth first walk across a node tree by calling `compile`
// on child nodes and where necessary pushing slots onto the address.

impl Compile for Node {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        dispatch_node!(self, Ok(()), compile, address, context)
    }
}

impl Compile for CreativeWorkTypes {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        dispatch_work!(self, compile, address, context)
    }
}

impl Compile for BlockContent {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        dispatch_block!(self, compile, address, context)
    }
}

impl Compile for InlineContent {
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        dispatch_inline!(self, Ok(()), compile, address, context)
    }
}

impl<T> Compile for Option<T>
where
    T: Compile,
{
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        if let Some(value) = self {
            value.compile(address, context)
        } else {
            Ok(())
        }
    }
}

impl<T> Compile for Box<T>
where
    T: Compile,
{
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        (**self).compile(address, context)
    }
}

impl<T> Compile for Vec<T>
where
    T: Compile,
{
    fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
        for (index, item) in self.iter_mut().enumerate() {
            address.push_back(Slot::Index(index));
            item.compile(address, context)?;
            address.pop_back();
        }
        Ok(())
    }
}

/// Compile nothing
macro_rules! compile_nothing {
    ($type:ty) => {
        impl Compile for $type {
            fn compile(&mut self, _address: &mut Address, _context: &mut Context) -> Result<()> {
                Ok(())
            }
        }
    };
}

/// Compile nothing for several types
macro_rules! compile_nothing_for {
    ( $( $type:ty ),* ) => {
        $(
            compile_nothing!($type);
        )*
    };
}

compile_nothing_for!(
    // Primitives
    bool, // Boolean
    i64, // Integer
    f64, // Number
    String,
    Vec<Primitive>, // Array
    BTreeMap<String, Primitive>, // Object

    // Entity types that are unlikely to need anything else done to them
    ThematicBreak,

    // Entity types that may need to be compiled but are here for now
    CodeBlock,
    CodeFragment,
    Datatable,
    MathBlock,
    MathFragment,
    Periodical,
    PublicationIssue,
    PublicationVolume,
    Review,
    SoftwareApplication
);

/// Compile fields of a struct
macro_rules! compile_fields {
    ($type:ty $(, $field:ident)* ) => {
        impl Compile for $type {
            fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
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
        impl Compile for $type {
            fn compile(&mut self, address: &mut Address, context: &mut Context) -> Result<()> {
                match self {
                    $(
                        $variant(node) => node.compile(address, context),
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
