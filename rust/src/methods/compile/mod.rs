use defaults::Defaults;
use eyre::{bail, Result};
use std::{
    collections::{BTreeMap, HashSet},
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
    node.compile(&mut context)?;
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

    /// Files that the document is dependant upon e.g. images, data
    pub input_files: HashSet<PathBuf>,

    /// Files that the document creates e.g. summaries, plots
    pub output_files: HashSet<PathBuf>,
}

/// Trait for compiling a node
trait Compile {
    fn compile(&mut self, context: &mut Context) -> Result<()>;
}

// The following `impl Compile` for enums try to include all variants so that
// there is a context error if any variant is missing. This
// forces us to have an explicit `impl Compile` for each node type which
// rather than "silently" doing nothing for variants that have been omitted.

/// Compile a `Node`
impl Compile for Node {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            Node::Array(node) => node.compile(context),
            Node::Article(node) => node.compile(context),
            Node::AudioObject(node) => node.compile(context),
            Node::Boolean(node) => node.compile(context),
            Node::Cite(node) => node.compile(context),
            Node::CiteGroup(node) => node.compile(context),
            Node::Claim(node) => node.compile(context),
            Node::CodeBlock(node) => node.compile(context),
            Node::CodeChunk(node) => node.compile(context),
            Node::CodeExpression(node) => node.compile(context),
            Node::CodeFragment(node) => node.compile(context),
            Node::Collection(node) => node.compile(context),
            Node::Comment(node) => node.compile(context),
            Node::CreativeWork(node) => node.compile(context),
            Node::Datatable(node) => node.compile(context),
            Node::Delete(node) => node.compile(context),
            Node::Emphasis(node) => node.compile(context),
            Node::Figure(node) => node.compile(context),
            Node::Heading(node) => node.compile(context),
            Node::ImageObject(node) => node.compile(context),
            Node::Integer(node) => node.compile(context),
            Node::Link(node) => node.compile(context),
            Node::List(node) => node.compile(context),
            Node::MathBlock(node) => node.compile(context),
            Node::MathFragment(node) => node.compile(context),
            Node::MediaObject(node) => node.compile(context),
            Node::NontextualAnnotation(node) => node.compile(context),
            Node::Note(node) => node.compile(context),
            Node::Null => Ok(()),
            Node::Number(node) => node.compile(context),
            Node::Object(node) => node.compile(context),
            Node::Paragraph(node) => node.compile(context),
            Node::Periodical(node) => node.compile(context),
            Node::PublicationIssue(node) => node.compile(context),
            Node::PublicationVolume(node) => node.compile(context),
            Node::Quote(node) => node.compile(context),
            Node::QuoteBlock(node) => node.compile(context),
            Node::Review(node) => node.compile(context),
            Node::SoftwareApplication(node) => node.compile(context),
            Node::SoftwareSourceCode(node) => node.compile(context),
            Node::String(node) => node.compile(context),
            Node::Strong(node) => node.compile(context),
            Node::Subscript(node) => node.compile(context),
            Node::Superscript(node) => node.compile(context),
            Node::Table(node) => node.compile(context),
            Node::ThematicBreak(node) => node.compile(context),
            Node::VideoObject(node) => node.compile(context),
            _ => {
                tracing::debug!("Compile is not implemented for {:?}", self);
                Ok(())
            }
        }
    }
}

impl Compile for InlineContent {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            InlineContent::AudioObject(node) => node.compile(context),
            InlineContent::Boolean(node) => node.compile(context),
            InlineContent::Cite(node) => node.compile(context),
            InlineContent::CiteGroup(node) => node.compile(context),
            InlineContent::CodeExpression(node) => node.compile(context),
            InlineContent::CodeFragment(node) => node.compile(context),
            InlineContent::Delete(node) => node.compile(context),
            InlineContent::Emphasis(node) => node.compile(context),
            InlineContent::ImageObject(node) => node.compile(context),
            InlineContent::Integer(node) => node.compile(context),
            InlineContent::Link(node) => node.compile(context),
            InlineContent::MathFragment(node) => node.compile(context),
            InlineContent::NontextualAnnotation(node) => node.compile(context),
            InlineContent::Note(node) => node.compile(context),
            InlineContent::Null => Ok(()),
            InlineContent::Number(node) => node.compile(context),
            InlineContent::Quote(node) => node.compile(context),
            InlineContent::String(node) => node.compile(context),
            InlineContent::Strong(node) => node.compile(context),
            InlineContent::Subscript(node) => node.compile(context),
            InlineContent::Superscript(node) => node.compile(context),
            InlineContent::VideoObject(node) => node.compile(context),
        }
    }
}

impl Compile for BlockContent {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            BlockContent::Claim(node) => node.compile(context),
            BlockContent::CodeBlock(node) => node.compile(context),
            BlockContent::CodeChunk(node) => node.compile(context),
            BlockContent::Collection(node) => node.compile(context),
            BlockContent::Figure(node) => node.compile(context),
            BlockContent::Heading(node) => node.compile(context),
            BlockContent::List(node) => node.compile(context),
            BlockContent::MathBlock(node) => node.compile(context),
            BlockContent::Paragraph(node) => node.compile(context),
            BlockContent::QuoteBlock(node) => node.compile(context),
            BlockContent::Table(node) => node.compile(context),
            BlockContent::ThematicBreak(node) => node.compile(context),
        }
    }
}

impl Compile for CreativeWorkTypes {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            CreativeWorkTypes::Article(node) => node.compile(context),
            CreativeWorkTypes::AudioObject(node) => node.compile(context),
            CreativeWorkTypes::Claim(node) => node.compile(context),
            CreativeWorkTypes::Collection(node) => node.compile(context),
            CreativeWorkTypes::Comment(node) => node.compile(context),
            CreativeWorkTypes::CreativeWork(node) => node.compile(context),
            CreativeWorkTypes::Datatable(node) => node.compile(context),
            CreativeWorkTypes::Figure(node) => node.compile(context),
            CreativeWorkTypes::ImageObject(node) => node.compile(context),
            CreativeWorkTypes::MediaObject(node) => node.compile(context),
            CreativeWorkTypes::Periodical(node) => node.compile(context),
            CreativeWorkTypes::PublicationIssue(node) => node.compile(context),
            CreativeWorkTypes::PublicationVolume(node) => node.compile(context),
            CreativeWorkTypes::Review(node) => node.compile(context),
            CreativeWorkTypes::SoftwareApplication(node) => node.compile(context),
            CreativeWorkTypes::SoftwareSourceCode(node) => node.compile(context),
            CreativeWorkTypes::Table(node) => node.compile(context),
            CreativeWorkTypes::VideoObject(node) => node.compile(context),
        }
    }
}

// Implementations for `Option<Box>`, `Vec` etc that are often used in properties

impl<T> Compile for Option<T>
where
    T: Compile,
{
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        if let Some(value) = self {
            value.compile(context)
        } else {
            Ok(())
        }
    }
}

impl<T> Compile for Box<T>
where
    T: Compile,
{
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        (**self).compile(context)
    }
}

impl<T> Compile for Vec<T>
where
    T: Compile,
{
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        for item in self {
            item.compile(context)?
        }
        Ok(())
    }
}

/// An implementation where `compile` is a no-op
macro_rules! compile_nothing {
    ( $( $type:ty ),* ) => {
        $(
            impl Compile for $type {
                fn compile(&mut self, _compilation: &mut Context) -> Result<()> {Ok(())}
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
                fn compile(&mut self, context: &mut Context) -> Result<()> {
                    self.content.compile(context)
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
    Link,
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
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            CreativeWorkContent::String(node) => node.compile(context),
            CreativeWorkContent::VecNode(nodes) => nodes.compile(context),
        }
    }
}

impl Compile for ListItemContent {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        match self {
            ListItemContent::VecInlineContent(nodes) => nodes.compile(context),
            ListItemContent::VecBlockContent(nodes) => nodes.compile(context),
        }
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

    // Add the media file to the file_inputs
    context.input_files.insert(path.clone());

    Ok(format!("file://{}", path.display()))
}

/// A `Compile` implementation for `MediaObject` node types
macro_rules! compile_media_object {
    ( $( $type:ty ),* ) => {
        $(
            impl Compile for $type {
                fn compile(&mut self, context: &mut Context) -> Result<()> {
                    self.content_url = compile_content_url(&self.content_url, context)?;
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

/// A `Compile` implementation for executable code node types having
/// `text` and `programming_language`
fn compile_code(text: &str, lang: &str, context: &mut Context) -> Result<()> {
    let analysis = code::compile(text, lang)?;

    // Add any files read and written to inputs and outputs
    for file in analysis.reads_files.unwrap_or_default() {
        context.input_files.insert(file.into());
    }
    for file in analysis.writes_files.unwrap_or_default() {
        context.output_files.insert(file.into());
    }

    Ok(())
}

impl Compile for CodeChunk {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        if let Some(lang) = self.programming_language.as_deref() {
            compile_code(&self.text, lang, context)
        } else {
            Ok(())
        }
    }
}

impl Compile for CodeExpression {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        if let Some(lang) = self.programming_language.as_deref() {
            compile_code(&self.text, lang, context)
        } else {
            Ok(())
        }
    }
}

impl Compile for SoftwareSourceCode {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        if let (Some(text), Some(lang)) =
            (self.text.as_deref(), self.programming_language.as_deref())
        {
            compile_code(text, lang, context)
        } else {
            Ok(())
        }
    }
}

// Custom implementations where necessary for other types

impl Compile for CiteGroup {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.items.compile(context)
    }
}

impl Compile for Collection {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.parts.compile(context)
    }
}

impl Compile for CollectionSimple {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.parts.compile(context)
    }
}

impl Compile for List {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.items.compile(context)
    }
}

impl Compile for ListItem {
    fn compile(&mut self, context: &mut Context) -> Result<()> {
        self.item.compile(context)?;
        self.content.compile(context)?;
        Ok(())
    }
}
