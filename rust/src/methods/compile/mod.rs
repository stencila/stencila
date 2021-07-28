use crate::graphs::{Relation, Resource, Triple};
use defaults::Defaults;
use eyre::{bail, Result};
use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};
use stencila_schema::*;

#[cfg(feature = "compile-code")]
pub mod code;

/// Compile a node
pub fn compile(node: &mut Node, path: &Path, project: &Path) -> Result<Context> {
    let mut context = Context {
        path: PathBuf::from(path),
        project: PathBuf::from(project),
        ..Default::default()
    };
    node.compile("", &mut context)?;
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

    /// Relations within the document
    pub relations: Vec<Triple>,
}

/// Trait for compiling a node
trait Compile {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()>;
}

// The following `impl Compile` for enums try to include all variants so that
// there is a context error if any variant is missing. This
// forces us to have an explicit `impl Compile` for each node type which
// rather than "silently" doing nothing for variants that have been omitted.

/// Compile a `Node`
impl Compile for Node {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        match self {
            Node::Array(node) => node.compile(route, context),
            Node::Article(node) => node.compile(route, context),
            Node::AudioObject(node) => node.compile(route, context),
            Node::Boolean(node) => node.compile(route, context),
            Node::Cite(node) => node.compile(route, context),
            Node::CiteGroup(node) => node.compile(route, context),
            Node::Claim(node) => node.compile(route, context),
            Node::CodeBlock(node) => node.compile(route, context),
            Node::CodeChunk(node) => node.compile(route, context),
            Node::CodeExpression(node) => node.compile(route, context),
            Node::CodeFragment(node) => node.compile(route, context),
            Node::Collection(node) => node.compile(route, context),
            Node::Comment(node) => node.compile(route, context),
            Node::CreativeWork(node) => node.compile(route, context),
            Node::Datatable(node) => node.compile(route, context),
            Node::Delete(node) => node.compile(route, context),
            Node::Emphasis(node) => node.compile(route, context),
            Node::Figure(node) => node.compile(route, context),
            Node::Heading(node) => node.compile(route, context),
            Node::ImageObject(node) => node.compile(route, context),
            Node::Integer(node) => node.compile(route, context),
            Node::Link(node) => node.compile(route, context),
            Node::List(node) => node.compile(route, context),
            Node::MathBlock(node) => node.compile(route, context),
            Node::MathFragment(node) => node.compile(route, context),
            Node::MediaObject(node) => node.compile(route, context),
            Node::NontextualAnnotation(node) => node.compile(route, context),
            Node::Note(node) => node.compile(route, context),
            Node::Null => Ok(()),
            Node::Number(node) => node.compile(route, context),
            Node::Object(node) => node.compile(route, context),
            Node::Paragraph(node) => node.compile(route, context),
            Node::Periodical(node) => node.compile(route, context),
            Node::PublicationIssue(node) => node.compile(route, context),
            Node::PublicationVolume(node) => node.compile(route, context),
            Node::Quote(node) => node.compile(route, context),
            Node::QuoteBlock(node) => node.compile(route, context),
            Node::Review(node) => node.compile(route, context),
            Node::SoftwareApplication(node) => node.compile(route, context),
            Node::SoftwareSourceCode(node) => node.compile(route, context),
            Node::String(node) => node.compile(route, context),
            Node::Strong(node) => node.compile(route, context),
            Node::Subscript(node) => node.compile(route, context),
            Node::Superscript(node) => node.compile(route, context),
            Node::Table(node) => node.compile(route, context),
            Node::ThematicBreak(node) => node.compile(route, context),
            Node::VideoObject(node) => node.compile(route, context),
            _ => {
                tracing::debug!("Compile is not implemented for {:?}", self);
                Ok(())
            }
        }
    }
}

impl Compile for InlineContent {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        match self {
            InlineContent::AudioObject(node) => node.compile(route, context),
            InlineContent::Boolean(node) => node.compile(route, context),
            InlineContent::Cite(node) => node.compile(route, context),
            InlineContent::CiteGroup(node) => node.compile(route, context),
            InlineContent::CodeExpression(node) => node.compile(route, context),
            InlineContent::CodeFragment(node) => node.compile(route, context),
            InlineContent::Delete(node) => node.compile(route, context),
            InlineContent::Emphasis(node) => node.compile(route, context),
            InlineContent::ImageObject(node) => node.compile(route, context),
            InlineContent::Integer(node) => node.compile(route, context),
            InlineContent::Link(node) => node.compile(route, context),
            InlineContent::MathFragment(node) => node.compile(route, context),
            InlineContent::NontextualAnnotation(node) => node.compile(route, context),
            InlineContent::Note(node) => node.compile(route, context),
            InlineContent::Null => Ok(()),
            InlineContent::Number(node) => node.compile(route, context),
            InlineContent::Quote(node) => node.compile(route, context),
            InlineContent::String(node) => node.compile(route, context),
            InlineContent::Strong(node) => node.compile(route, context),
            InlineContent::Subscript(node) => node.compile(route, context),
            InlineContent::Superscript(node) => node.compile(route, context),
            InlineContent::VideoObject(node) => node.compile(route, context),
        }
    }
}

impl Compile for BlockContent {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        match self {
            BlockContent::Claim(node) => node.compile(route, context),
            BlockContent::CodeBlock(node) => node.compile(route, context),
            BlockContent::CodeChunk(node) => node.compile(route, context),
            BlockContent::Collection(node) => node.compile(route, context),
            BlockContent::Figure(node) => node.compile(route, context),
            BlockContent::Heading(node) => node.compile(route, context),
            BlockContent::List(node) => node.compile(route, context),
            BlockContent::MathBlock(node) => node.compile(route, context),
            BlockContent::Paragraph(node) => node.compile(route, context),
            BlockContent::QuoteBlock(node) => node.compile(route, context),
            BlockContent::Table(node) => node.compile(route, context),
            BlockContent::ThematicBreak(node) => node.compile(route, context),
        }
    }
}

impl Compile for CreativeWorkTypes {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        match self {
            CreativeWorkTypes::Article(node) => node.compile(route, context),
            CreativeWorkTypes::AudioObject(node) => node.compile(route, context),
            CreativeWorkTypes::Claim(node) => node.compile(route, context),
            CreativeWorkTypes::Collection(node) => node.compile(route, context),
            CreativeWorkTypes::Comment(node) => node.compile(route, context),
            CreativeWorkTypes::CreativeWork(node) => node.compile(route, context),
            CreativeWorkTypes::Datatable(node) => node.compile(route, context),
            CreativeWorkTypes::Figure(node) => node.compile(route, context),
            CreativeWorkTypes::ImageObject(node) => node.compile(route, context),
            CreativeWorkTypes::MediaObject(node) => node.compile(route, context),
            CreativeWorkTypes::Periodical(node) => node.compile(route, context),
            CreativeWorkTypes::PublicationIssue(node) => node.compile(route, context),
            CreativeWorkTypes::PublicationVolume(node) => node.compile(route, context),
            CreativeWorkTypes::Review(node) => node.compile(route, context),
            CreativeWorkTypes::SoftwareApplication(node) => node.compile(route, context),
            CreativeWorkTypes::SoftwareSourceCode(node) => node.compile(route, context),
            CreativeWorkTypes::Table(node) => node.compile(route, context),
            CreativeWorkTypes::VideoObject(node) => node.compile(route, context),
        }
    }
}

// Implementations for `Option<Box>`, `Vec` etc that are often used in properties

impl<T> Compile for Option<T>
where
    T: Compile,
{
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        if let Some(value) = self {
            value.compile(route, context)
        } else {
            Ok(())
        }
    }
}

impl<T> Compile for Box<T>
where
    T: Compile,
{
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        (**self).compile(route, context)
    }
}

impl<T> Compile for Vec<T>
where
    T: Compile,
{
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        for (index, item) in self.iter_mut().enumerate() {
            item.compile(&[route, ".", &index.to_string()].concat(), context)?
        }
        Ok(())
    }
}

/// An implementation where `compile` is a no-op
macro_rules! compile_nothing {
    ( $( $type:ty ),* ) => {
        $(
            impl Compile for $type {
                fn compile(&mut self, _route: &str, _compilation: &mut Context) -> Result<()> {Ok(())}
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
            impl Compile for $type {
                fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
                    self.content.compile(&[route, ".content"].concat(), context)
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

impl Compile for CreativeWorkContent {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        match self {
            CreativeWorkContent::String(node) => node.compile(route, context),
            CreativeWorkContent::VecNode(nodes) => nodes.compile(route, context),
        }
    }
}

impl Compile for ListItemContent {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        match self {
            ListItemContent::VecInlineContent(nodes) => nodes.compile(route, context),
            ListItemContent::VecBlockContent(nodes) => nodes.compile(route, context),
        }
    }
}

/// Compile a `Link` to add its `target` to the list of included files
impl Compile for Link {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        let target = self.target.clone();
        let resource = if target.starts_with("http://") || target.starts_with("https://") {
            Resource::Url(target)
        } else {
            Resource::File(target)
        };
        context
            .relations
            .push((Resource::Link(route.to_string()), Relation::Links, resource));

        Ok(())
    }
}

/// Compile to `content_url` property of `MediaObject` node types
///
/// If the `content_url` property is  a `file://` URL (implicitly
/// or explicitly) then resolves the file path, records it as
/// a file dependency, and returns an absolute `file://` URL.
fn compile_content_url(content_url: &str, context: &mut Context) -> Result<String> {
    if content_url.starts_with("http://") || content_url.starts_with("https://") {
        return Ok(content_url.into());
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
        bail!(
            "Document contains a link to a file outside of its project: '{}'. Resolved path '{}' is outside of project path '{}'",
            content_url,
            path.display(),
            &context.project.display()
        )
    }

    Ok(format!("file://{}", path.display()))
}

/// A `Compile` implementation for `MediaObject` node types
macro_rules! compile_media_object {
    ( $( $type:ty ),* ) => {
        $(
            impl Compile for $type {
                fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
                    let url = compile_content_url(&self.content_url, context)?;

                    context.relations.push((
                        Resource::Embed(route.to_string()),
                        Relation::Embeds,
                        Resource::File(url.clone()),
                    ));

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

impl Compile for CodeChunk {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        if let Some(lang) = self.programming_language.as_deref() {
            let mut relations =
                code::compile(&Resource::CodeChunk(route.to_string()), &self.text, lang);
            context.relations.append(&mut relations)
        }
        Ok(())
    }
}

impl Compile for CodeExpression {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        if let Some(lang) = self.programming_language.as_deref() {
            let mut relations = code::compile(
                &Resource::CodeExpression(route.to_string()),
                &self.text,
                lang,
            );
            context.relations.append(&mut relations)
        }
        Ok(())
    }
}

impl Compile for SoftwareSourceCode {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        if let (Some(text), Some(lang)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            let mut relations =
                code::compile(&Resource::SoftwareSourceCode(route.to_string()), text, lang);
            context.relations.append(&mut relations)
        }
        Ok(())
    }
}

// Custom implementations where necessary for other types

impl Compile for CiteGroup {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        self.items.compile(route, context)
    }
}

impl Compile for Collection {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        self.parts.compile(route, context)
    }
}

impl Compile for CollectionSimple {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        self.parts.compile(route, context)
    }
}

impl Compile for List {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        self.items.compile(route, context)
    }
}

impl Compile for ListItem {
    fn compile(&mut self, route: &str, context: &mut Context) -> Result<()> {
        self.item.compile(route, context)?;
        self.content.compile(route, context)?;
        Ok(())
    }
}
