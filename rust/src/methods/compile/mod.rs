use crate::{
    documents::DOCUMENTS,
    graphs::{relations, resources, Relation, Resource, NULL_RANGE},
    traits::ToVecBlockContent,
    utils::{hash::str_sha256_bytes, path::merge, uuids},
};
use async_trait::async_trait;
use defaults::Defaults;
use eyre::Result;
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use stencila_schema::*;

#[cfg(feature = "compile-code")]
pub mod code;

/// Compile a node
pub async fn compile(node: &mut Node, path: &Path, project: &Path) -> Result<Context> {
    let mut context = Context {
        path: PathBuf::from(path),
        project: PathBuf::from(project),
        ..Default::default()
    };
    node.compile(&mut context).await?;
    Ok(context)
}

/// The compilation context, used to pass down properties of the
/// root node and to record inputs and outputs etc during compilation
#[derive(Defaults)]
pub struct Context {
    /// The path of the document being compiled.
    /// Used to resolve relative paths e.g. in `ImageObject` and `Include` nodes
    path: PathBuf,

    /// The project that the document is within.
    /// Used to restrict any file links to be within the project
    project: PathBuf,

    /// Relations with other resources for each compiled resource
    /// in the document.
    pub relations: Vec<(Resource, Vec<(Relation, Resource)>)>,
}

/// Trait for compiling a node
#[async_trait]
trait Compile {
    async fn compile(&mut self, context: &mut Context) -> Result<()>;
}

/// Identify a node
///
/// If the node does not have an id, generate and assign one with a
/// leading "_" to indicate it is generated.
/// Return the node's id.
macro_rules! identify {
    ($node:expr) => {
        if let Some(id) = $node.id.as_deref() {
            id.clone()
        } else {
            let id = ["_", &uuids::generate_chars(20)].concat();
            $node.id = Some(Box::new(id.clone()));
            id
        }
    };
}

// The following `impl Compile` for enums try to include all variants so that
// there is a context error if any variant is missing. This
// forces us to have an explicit `impl Compile` for each node type which
// rather than "silently" doing nothing for variants that have been omitted.

/// Compile a `Node`
#[async_trait]
impl Compile for Node {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            Node::Array(node) => node.compile(context).await,
            Node::Article(node) => node.compile(context).await,
            Node::AudioObject(node) => node.compile(context).await,
            Node::Boolean(node) => node.compile(context).await,
            Node::Cite(node) => node.compile(context).await,
            Node::CiteGroup(node) => node.compile(context).await,
            Node::Claim(node) => node.compile(context).await,
            Node::CodeBlock(node) => node.compile(context).await,
            Node::CodeChunk(node) => node.compile(context).await,
            Node::CodeExpression(node) => node.compile(context).await,
            Node::CodeFragment(node) => node.compile(context).await,
            Node::Collection(node) => node.compile(context).await,
            Node::Comment(node) => node.compile(context).await,
            Node::CreativeWork(node) => node.compile(context).await,
            Node::Datatable(node) => node.compile(context).await,
            Node::Delete(node) => node.compile(context).await,
            Node::Emphasis(node) => node.compile(context).await,
            Node::Figure(node) => node.compile(context).await,
            Node::Heading(node) => node.compile(context).await,
            Node::ImageObject(node) => node.compile(context).await,
            Node::Integer(node) => node.compile(context).await,
            Node::Link(node) => node.compile(context).await,
            Node::List(node) => node.compile(context).await,
            Node::MathBlock(node) => node.compile(context).await,
            Node::MathFragment(node) => node.compile(context).await,
            Node::MediaObject(node) => node.compile(context).await,
            Node::NontextualAnnotation(node) => node.compile(context).await,
            Node::Note(node) => node.compile(context).await,
            Node::Null => Ok(()),
            Node::Number(node) => node.compile(context).await,
            Node::Object(node) => node.compile(context).await,
            Node::Paragraph(node) => node.compile(context).await,
            Node::Periodical(node) => node.compile(context).await,
            Node::PublicationIssue(node) => node.compile(context).await,
            Node::PublicationVolume(node) => node.compile(context).await,
            Node::Quote(node) => node.compile(context).await,
            Node::QuoteBlock(node) => node.compile(context).await,
            Node::Review(node) => node.compile(context).await,
            Node::SoftwareApplication(node) => node.compile(context).await,
            Node::SoftwareSourceCode(node) => node.compile(context).await,
            Node::String(node) => node.compile(context).await,
            Node::Strong(node) => node.compile(context).await,
            Node::Subscript(node) => node.compile(context).await,
            Node::Superscript(node) => node.compile(context).await,
            Node::Table(node) => node.compile(context).await,
            Node::ThematicBreak(node) => node.compile(context).await,
            Node::VideoObject(node) => node.compile(context).await,
            _ => {
                tracing::debug!("Compile is not implemented for {:?}", self);
                Ok(())
            }
        }
    }
}

#[async_trait]
impl Compile for InlineContent {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            InlineContent::AudioObject(node) => node.compile(context).await,
            InlineContent::Boolean(node) => node.compile(context).await,
            InlineContent::Cite(node) => node.compile(context).await,
            InlineContent::CiteGroup(node) => node.compile(context).await,
            InlineContent::CodeExpression(node) => node.compile(context).await,
            InlineContent::CodeFragment(node) => node.compile(context).await,
            InlineContent::Delete(node) => node.compile(context).await,
            InlineContent::Emphasis(node) => node.compile(context).await,
            InlineContent::ImageObject(node) => node.compile(context).await,
            InlineContent::Integer(node) => node.compile(context).await,
            InlineContent::Link(node) => node.compile(context).await,
            InlineContent::MathFragment(node) => node.compile(context).await,
            InlineContent::NontextualAnnotation(node) => node.compile(context).await,
            InlineContent::Note(node) => node.compile(context).await,
            InlineContent::Null => Ok(()),
            InlineContent::Number(node) => node.compile(context).await,
            InlineContent::Parameter(node) => node.compile(context).await,
            InlineContent::Quote(node) => node.compile(context).await,
            InlineContent::String(node) => node.compile(context).await,
            InlineContent::Strong(node) => node.compile(context).await,
            InlineContent::Subscript(node) => node.compile(context).await,
            InlineContent::Superscript(node) => node.compile(context).await,
            InlineContent::VideoObject(node) => node.compile(context).await,
        }
    }
}

#[async_trait]
impl Compile for BlockContent {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            BlockContent::Claim(node) => node.compile(context).await,
            BlockContent::CodeBlock(node) => node.compile(context).await,
            BlockContent::CodeChunk(node) => node.compile(context).await,
            BlockContent::Collection(node) => node.compile(context).await,
            BlockContent::Figure(node) => node.compile(context).await,
            BlockContent::Heading(node) => node.compile(context).await,
            BlockContent::Include(node) => node.compile(context).await,
            BlockContent::List(node) => node.compile(context).await,
            BlockContent::MathBlock(node) => node.compile(context).await,
            BlockContent::Paragraph(node) => node.compile(context).await,
            BlockContent::QuoteBlock(node) => node.compile(context).await,
            BlockContent::Table(node) => node.compile(context).await,
            BlockContent::ThematicBreak(node) => node.compile(context).await,
        }
    }
}

#[async_trait]
impl Compile for CreativeWorkTypes {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            CreativeWorkTypes::Article(node) => node.compile(context).await,
            CreativeWorkTypes::AudioObject(node) => node.compile(context).await,
            CreativeWorkTypes::Claim(node) => node.compile(context).await,
            CreativeWorkTypes::Collection(node) => node.compile(context).await,
            CreativeWorkTypes::Comment(node) => node.compile(context).await,
            CreativeWorkTypes::CreativeWork(node) => node.compile(context).await,
            CreativeWorkTypes::Datatable(node) => node.compile(context).await,
            CreativeWorkTypes::Figure(node) => node.compile(context).await,
            CreativeWorkTypes::ImageObject(node) => node.compile(context).await,
            CreativeWorkTypes::MediaObject(node) => node.compile(context).await,
            CreativeWorkTypes::Periodical(node) => node.compile(context).await,
            CreativeWorkTypes::PublicationIssue(node) => node.compile(context).await,
            CreativeWorkTypes::PublicationVolume(node) => node.compile(context).await,
            CreativeWorkTypes::Review(node) => node.compile(context).await,
            CreativeWorkTypes::SoftwareApplication(node) => node.compile(context).await,
            CreativeWorkTypes::SoftwareSourceCode(node) => node.compile(context).await,
            CreativeWorkTypes::Table(node) => node.compile(context).await,
            CreativeWorkTypes::VideoObject(node) => node.compile(context).await,
        }
    }
}

// Implementations for `Option<Box>`, `Vec` etc that are often used in properties

#[async_trait]
impl<T> Compile for Option<T>
where
    T: Compile + Send,
{
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        if let Some(value) = self {
            value.compile(context).await
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
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        (**self).compile(context).await
    }
}

#[async_trait]
impl<T> Compile for Vec<T>
where
    T: Compile + Send,
{
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        for item in self.iter_mut() {
            item.compile(context).await?
        }
        Ok(())
    }
}

/// An implementation where `compile` is a no-op
macro_rules! compile_nothing {
    ( $( $type:ty ),* ) => {
        $(
            #[async_trait]
            impl Compile for $type {
                async fn compile(&mut self, _context: &mut Context) -> Result<()> {Ok(())}
            }
        )*
    };
}

compile_nothing!(
    // Primitives (will never need to be compiled)
    bool, // Boolean
    i64, // Integer
    f64, // Number
    String,
    Vec<Primitive>, // Array
    BTreeMap<String, Primitive>, // Object

    // Nodes that are unlikely to need to be compiled
    ThematicBreak,

    // Nodes that may need to be compiled but are here to
    // have an explicit no-op implementation
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

/// A default implementation for node types that have a `content` property
macro_rules! compile_content {
    ( $( $type:ty ),* ) => {
        $(
            #[async_trait]
            impl Compile for $type {
                async fn compile(&mut self, context: &mut Context) -> Result<()> {
                    self.content.compile(context).await
                }
            }
        )*
    };
}

compile_content!(
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

// Implementations for `content` property enums

#[async_trait]
impl Compile for CreativeWorkContent {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            CreativeWorkContent::String(node) => node.compile(context).await,
            CreativeWorkContent::VecNode(nodes) => nodes.compile(context).await,
        }
    }
}

#[async_trait]
impl Compile for ListItemContent {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            ListItemContent::VecInlineContent(nodes) => nodes.compile(context).await,
            ListItemContent::VecBlockContent(nodes) => nodes.compile(context).await,
        }
    }
}

/// Compile a `Link` to add its `target` to the list of included files
#[async_trait]
impl Compile for Link {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        let id = identify!(self);
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

/// A `Compile` implementation for `MediaObject` node types
macro_rules! compile_media_object {
    ( $( $type:ty ),* ) => {
        $(
            #[async_trait]
            impl Compile for $type {
                async fn compile(&mut self, context: &mut Context) -> Result<()> {
                    let id = identify!(self);
                    let subject = resources::node(&context.path, &id, &self.type_name());

                    let url = compile_content_url(&self.content_url, context);
                    let object = if url.starts_with("http") {
                        resources::url(&url)
                    } else {
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

#[async_trait]
impl Compile for Parameter {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        let id = identify!(self);
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

        context
            .relations
            .push((subject, vec![(relations::assigns(NULL_RANGE), object)]));
        Ok(())
    }
}

#[async_trait]
impl Compile for CodeChunk {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        let digest =
            str_sha256_bytes(&[self.text.as_str(), self.programming_language.as_str()].concat());

        if Some(digest) != self.compile_digest {
            let id = identify!(self);
            let subject = resources::node(&context.path, &id, &self.type_name());
            let relations = code::compile(&context.path, &self.text, &self.programming_language);
            context.relations.push((subject, relations));
            self.compile_digest = Some(digest)
        }

        Ok(())
    }
}

#[async_trait]
impl Compile for CodeExpression {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        let digest =
            str_sha256_bytes(&[self.text.as_str(), self.programming_language.as_str()].concat());

        if Some(digest) != self.compile_digest {
            let id = identify!(self);
            let subject = resources::node(&context.path, &id, &self.type_name());
            let relations = code::compile(&context.path, &self.text, &self.programming_language);
            context.relations.push((subject, relations));
            self.compile_digest = Some(digest);
        }

        Ok(())
    }
}

#[async_trait]
impl Compile for SoftwareSourceCode {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        if let (Some(text), Some(programming_language)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            let _digest =
                str_sha256_bytes(&[text.as_str(), programming_language.as_str()].concat());

            let subject = resources::file(&context.path);
            let relations = code::compile(&context.path, text, programming_language);
            context.relations.push((subject, relations));
        }
        Ok(())
    }
}

#[async_trait]
impl Compile for Include {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        let id = identify!(self);
        let subject = resources::node(&context.path, &id, &self.type_name());

        let path = merge(&context.path, &self.source);
        let format = self.media_type.as_deref().cloned();
        let document = DOCUMENTS.open(&path, format).await?;
        self.content = document
            .root
            .as_ref()
            .map(|root| root.to_vec_block_content());
        //self.sha256 = Some(Box::new(document.sha256()?));

        let object = resources::file(&path);

        context
            .relations
            .push((subject, vec![(Relation::Include, object)]));

        Ok(())
    }
}

// Custom implementations where necessary for other types

#[async_trait]
impl Compile for CiteGroup {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.items.compile(context).await
    }
}

#[async_trait]
impl Compile for Collection {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.parts.compile(context).await
    }
}

#[async_trait]
impl Compile for CollectionSimple {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.parts.compile(context).await
    }
}

#[async_trait]
impl Compile for List {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.items.compile(context).await
    }
}

#[async_trait]
impl Compile for ListItem {
    async fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.item.compile(context).await?;
        self.content.compile(context).await?;
        Ok(())
    }
}
