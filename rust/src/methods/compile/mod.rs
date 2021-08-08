use crate::{
    documents::DOCUMENTS,
    graphs::{resources, Relation, Triple, NULL_RANGE},
    traits::ToVecBlockContent,
    utils::path::merge,
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
    node.compile("", &mut context).await?;
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

    /// The path of the document relative to the project it is in
    /// Used to store in relations
    path_within_project: String,

    /// Relations within the document
    pub relations: Vec<Triple>,
}

impl Context {
    pub fn resource_id(&self, address: &str) -> String {
        [&self.path_within_project, address].concat()
    }
}

/// Trait for compiling a node
#[async_trait]
trait Compile {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()>;
}

// The following `impl Compile` for enums try to include all variants so that
// there is a context error if any variant is missing. This
// forces us to have an explicit `impl Compile` for each node type which
// rather than "silently" doing nothing for variants that have been omitted.

/// Compile a `Node`
#[async_trait]
impl Compile for Node {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        match self {
            Node::Array(node) => node.compile(address, context).await,
            Node::Article(node) => node.compile(address, context).await,
            Node::AudioObject(node) => node.compile(address, context).await,
            Node::Boolean(node) => node.compile(address, context).await,
            Node::Cite(node) => node.compile(address, context).await,
            Node::CiteGroup(node) => node.compile(address, context).await,
            Node::Claim(node) => node.compile(address, context).await,
            Node::CodeBlock(node) => node.compile(address, context).await,
            Node::CodeChunk(node) => node.compile(address, context).await,
            Node::CodeExpression(node) => node.compile(address, context).await,
            Node::CodeFragment(node) => node.compile(address, context).await,
            Node::Collection(node) => node.compile(address, context).await,
            Node::Comment(node) => node.compile(address, context).await,
            Node::CreativeWork(node) => node.compile(address, context).await,
            Node::Datatable(node) => node.compile(address, context).await,
            Node::Delete(node) => node.compile(address, context).await,
            Node::Emphasis(node) => node.compile(address, context).await,
            Node::Figure(node) => node.compile(address, context).await,
            Node::Heading(node) => node.compile(address, context).await,
            Node::ImageObject(node) => node.compile(address, context).await,
            Node::Integer(node) => node.compile(address, context).await,
            Node::Link(node) => node.compile(address, context).await,
            Node::List(node) => node.compile(address, context).await,
            Node::MathBlock(node) => node.compile(address, context).await,
            Node::MathFragment(node) => node.compile(address, context).await,
            Node::MediaObject(node) => node.compile(address, context).await,
            Node::NontextualAnnotation(node) => node.compile(address, context).await,
            Node::Note(node) => node.compile(address, context).await,
            Node::Null => Ok(()),
            Node::Number(node) => node.compile(address, context).await,
            Node::Object(node) => node.compile(address, context).await,
            Node::Paragraph(node) => node.compile(address, context).await,
            Node::Periodical(node) => node.compile(address, context).await,
            Node::PublicationIssue(node) => node.compile(address, context).await,
            Node::PublicationVolume(node) => node.compile(address, context).await,
            Node::Quote(node) => node.compile(address, context).await,
            Node::QuoteBlock(node) => node.compile(address, context).await,
            Node::Review(node) => node.compile(address, context).await,
            Node::SoftwareApplication(node) => node.compile(address, context).await,
            Node::SoftwareSourceCode(node) => node.compile(address, context).await,
            Node::String(node) => node.compile(address, context).await,
            Node::Strong(node) => node.compile(address, context).await,
            Node::Subscript(node) => node.compile(address, context).await,
            Node::Superscript(node) => node.compile(address, context).await,
            Node::Table(node) => node.compile(address, context).await,
            Node::ThematicBreak(node) => node.compile(address, context).await,
            Node::VideoObject(node) => node.compile(address, context).await,
            _ => {
                tracing::debug!("Compile is not implemented for {:?}", self);
                Ok(())
            }
        }
    }
}

#[async_trait]
impl Compile for InlineContent {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        match self {
            InlineContent::AudioObject(node) => node.compile(address, context).await,
            InlineContent::Boolean(node) => node.compile(address, context).await,
            InlineContent::Cite(node) => node.compile(address, context).await,
            InlineContent::CiteGroup(node) => node.compile(address, context).await,
            InlineContent::CodeExpression(node) => node.compile(address, context).await,
            InlineContent::CodeFragment(node) => node.compile(address, context).await,
            InlineContent::Delete(node) => node.compile(address, context).await,
            InlineContent::Emphasis(node) => node.compile(address, context).await,
            InlineContent::ImageObject(node) => node.compile(address, context).await,
            InlineContent::Integer(node) => node.compile(address, context).await,
            InlineContent::Link(node) => node.compile(address, context).await,
            InlineContent::MathFragment(node) => node.compile(address, context).await,
            InlineContent::NontextualAnnotation(node) => node.compile(address, context).await,
            InlineContent::Note(node) => node.compile(address, context).await,
            InlineContent::Null => Ok(()),
            InlineContent::Number(node) => node.compile(address, context).await,
            InlineContent::Parameter(node) => node.compile(address, context).await,
            InlineContent::Quote(node) => node.compile(address, context).await,
            InlineContent::String(node) => node.compile(address, context).await,
            InlineContent::Strong(node) => node.compile(address, context).await,
            InlineContent::Subscript(node) => node.compile(address, context).await,
            InlineContent::Superscript(node) => node.compile(address, context).await,
            InlineContent::VideoObject(node) => node.compile(address, context).await,
        }
    }
}

#[async_trait]
impl Compile for BlockContent {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        match self {
            BlockContent::Claim(node) => node.compile(address, context).await,
            BlockContent::CodeBlock(node) => node.compile(address, context).await,
            BlockContent::CodeChunk(node) => node.compile(address, context).await,
            BlockContent::Collection(node) => node.compile(address, context).await,
            BlockContent::Figure(node) => node.compile(address, context).await,
            BlockContent::Heading(node) => node.compile(address, context).await,
            BlockContent::Include(node) => node.compile(address, context).await,
            BlockContent::List(node) => node.compile(address, context).await,
            BlockContent::MathBlock(node) => node.compile(address, context).await,
            BlockContent::Paragraph(node) => node.compile(address, context).await,
            BlockContent::QuoteBlock(node) => node.compile(address, context).await,
            BlockContent::Table(node) => node.compile(address, context).await,
            BlockContent::ThematicBreak(node) => node.compile(address, context).await,
        }
    }
}

#[async_trait]
impl Compile for CreativeWorkTypes {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        match self {
            CreativeWorkTypes::Article(node) => node.compile(address, context).await,
            CreativeWorkTypes::AudioObject(node) => node.compile(address, context).await,
            CreativeWorkTypes::Claim(node) => node.compile(address, context).await,
            CreativeWorkTypes::Collection(node) => node.compile(address, context).await,
            CreativeWorkTypes::Comment(node) => node.compile(address, context).await,
            CreativeWorkTypes::CreativeWork(node) => node.compile(address, context).await,
            CreativeWorkTypes::Datatable(node) => node.compile(address, context).await,
            CreativeWorkTypes::Figure(node) => node.compile(address, context).await,
            CreativeWorkTypes::ImageObject(node) => node.compile(address, context).await,
            CreativeWorkTypes::MediaObject(node) => node.compile(address, context).await,
            CreativeWorkTypes::Periodical(node) => node.compile(address, context).await,
            CreativeWorkTypes::PublicationIssue(node) => node.compile(address, context).await,
            CreativeWorkTypes::PublicationVolume(node) => node.compile(address, context).await,
            CreativeWorkTypes::Review(node) => node.compile(address, context).await,
            CreativeWorkTypes::SoftwareApplication(node) => node.compile(address, context).await,
            CreativeWorkTypes::SoftwareSourceCode(node) => node.compile(address, context).await,
            CreativeWorkTypes::Table(node) => node.compile(address, context).await,
            CreativeWorkTypes::VideoObject(node) => node.compile(address, context).await,
        }
    }
}

// Implementations for `Option<Box>`, `Vec` etc that are often used in properties

#[async_trait]
impl<T> Compile for Option<T>
where
    T: Compile + Send,
{
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        if let Some(value) = self {
            value.compile(address, context).await
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
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        (**self).compile(address, context).await
    }
}

#[async_trait]
impl<T> Compile for Vec<T>
where
    T: Compile + Send,
{
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        for (index, item) in self.iter_mut().enumerate() {
            item.compile(&[address, ".", &index.to_string()].concat(), context)
                .await?
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
                async fn compile(&mut self, _address: &str, _compilation: &mut Context) -> Result<()> {Ok(())}
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
                async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
                    self.content.compile(&[address, ".content"].concat(), context).await
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
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        match self {
            CreativeWorkContent::String(node) => node.compile(address, context).await,
            CreativeWorkContent::VecNode(nodes) => nodes.compile(address, context).await,
        }
    }
}

#[async_trait]
impl Compile for ListItemContent {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        match self {
            ListItemContent::VecInlineContent(nodes) => nodes.compile(address, context).await,
            ListItemContent::VecBlockContent(nodes) => nodes.compile(address, context).await,
        }
    }
}

/// Compile a `Link` to add its `target` to the list of included files
#[async_trait]
impl Compile for Link {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        let subject = resources::node(&context.path, address, &self.type_name());

        let target = &self.target;
        let object = if target.starts_with("http://") || target.starts_with("https://") {
            resources::url(target)
        } else {
            resources::file(&merge(&context.path, target))
        };

        context.relations.push((subject, Relation::Link, object));

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
                async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
                    let subject = resources::node(&context.path, address, &self.type_name());

                    let url = compile_content_url(&self.content_url, context);
                    let object = if url.starts_with("http") {
                        resources::url(&url)
                    } else {
                        resources::file(&Path::new(&url))
                    };

                    context.relations.push((subject, Relation::Embed, object));

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
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        let subject = resources::node(&context.path, address, &self.type_name());
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
            .push((subject, Relation::Assign(NULL_RANGE), object));
        Ok(())
    }
}

#[async_trait]
impl Compile for CodeChunk {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        if let Some(lang) = self.programming_language.as_deref() {
            let subject = resources::node(&context.path, address, &self.type_name());
            let mut relations = code::compile(&context.path, &subject, &self.text, lang);
            context.relations.append(&mut relations)
        }
        Ok(())
    }
}

#[async_trait]
impl Compile for CodeExpression {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        if let Some(lang) = self.programming_language.as_deref() {
            let subject = resources::node(&context.path, address, &self.type_name());
            let mut relations = code::compile(&context.path, &subject, &self.text, lang);
            context.relations.append(&mut relations)
        }
        Ok(())
    }
}

#[async_trait]
impl Compile for SoftwareSourceCode {
    async fn compile(&mut self, _address: &str, context: &mut Context) -> Result<()> {
        if let (Some(text), Some(lang)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            let subject = resources::file(&context.path);
            let mut relations = code::compile(&context.path, &subject, text, lang);
            context.relations.append(&mut relations)
        }
        Ok(())
    }
}

#[async_trait]
impl Compile for Include {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        let subject = resources::node(&context.path, address, &self.type_name());

        let path = merge(&context.path, &self.source);
        let format = self.media_type.as_deref().cloned();
        let document = DOCUMENTS.open(&path, format).await?;
        self.content = document
            .root
            .as_ref()
            .map(|root| root.to_vec_block_content());
        //self.sha256 = Some(Box::new(document.sha256()?));

        let object = resources::file(&path);

        context.relations.push((subject, Relation::Include, object));

        Ok(())
    }
}

// Custom implementations where necessary for other types

#[async_trait]
impl Compile for CiteGroup {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        self.items.compile(address, context).await
    }
}

#[async_trait]
impl Compile for Collection {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        self.parts.compile(address, context).await
    }
}

#[async_trait]
impl Compile for CollectionSimple {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        self.parts.compile(address, context).await
    }
}

#[async_trait]
impl Compile for List {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        self.items.compile(address, context).await
    }
}

#[async_trait]
impl Compile for ListItem {
    async fn compile(&mut self, address: &str, context: &mut Context) -> Result<()> {
        self.item.compile(address, context).await?;
        self.content.compile(address, context).await?;
        Ok(())
    }
}
