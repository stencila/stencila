///! Macros for dispatching a method call based on `Node` enum

#[macro_export]
macro_rules! dispatch_node {
    ($node:expr, $default:expr, $method:ident $(,$arg:expr)*) => {
        match $node {
            Node::Array(node) => node.$method($($arg),*),
            Node::Article(node) => node.$method($($arg),*),
            Node::AudioObject(node) => node.$method($($arg),*),
            Node::Boolean(node) => node.$method($($arg),*),
            Node::Cite(node) => node.$method($($arg),*),
            Node::CiteGroup(node) => node.$method($($arg),*),
            Node::Claim(node) => node.$method($($arg),*),
            Node::CodeBlock(node) => node.$method($($arg),*),
            Node::CodeChunk(node) => node.$method($($arg),*),
            Node::CodeExpression(node) => node.$method($($arg),*),
            Node::CodeFragment(node) => node.$method($($arg),*),
            Node::Collection(node) => node.$method($($arg),*),
            Node::Comment(node) => node.$method($($arg),*),
            Node::CreativeWork(node) => node.$method($($arg),*),
            Node::Datatable(node) => node.$method($($arg),*),
            Node::Delete(node) => node.$method($($arg),*),
            Node::Emphasis(node) => node.$method($($arg),*),
            Node::Figure(node) => node.$method($($arg),*),
            Node::Heading(node) => node.$method($($arg),*),
            Node::ImageObject(node) => node.$method($($arg),*),
            Node::Integer(node) => node.$method($($arg),*),
            Node::Link(node) => node.$method($($arg),*),
            Node::List(node) => node.$method($($arg),*),
            Node::MathBlock(node) => node.$method($($arg),*),
            Node::MathFragment(node) => node.$method($($arg),*),
            Node::MediaObject(node) => node.$method($($arg),*),
            Node::NontextualAnnotation(node) => node.$method($($arg),*),
            Node::Note(node) => node.$method($($arg),*),
            Node::Number(node) => node.$method($($arg),*),
            Node::Object(node) => node.$method($($arg),*),
            Node::Paragraph(node) => node.$method($($arg),*),
            Node::Periodical(node) => node.$method($($arg),*),
            Node::PublicationIssue(node) => node.$method($($arg),*),
            Node::PublicationVolume(node) => node.$method($($arg),*),
            Node::Quote(node) => node.$method($($arg),*),
            Node::QuoteBlock(node) => node.$method($($arg),*),
            Node::Review(node) => node.$method($($arg),*),
            Node::SoftwareApplication(node) => node.$method($($arg),*),
            Node::SoftwareSourceCode(node) => node.$method($($arg),*),
            Node::String(node) => node.$method($($arg),*),
            Node::Strong(node) => node.$method($($arg),*),
            Node::Subscript(node) => node.$method($($arg),*),
            Node::Superscript(node) => node.$method($($arg),*),
            Node::Table(node) => node.$method($($arg),*),
            Node::ThematicBreak(node) => node.$method($($arg),*),
            Node::VideoObject(node) => node.$method($($arg),*),
            _ => $default
        }
    }
}

#[macro_export]
macro_rules! dispatch_inline {
    ($node:expr, $method:ident $(,$arg:expr)*) => {
        match $node {
            InlineContent::AudioObject(node) => node.$method($($arg),*),
            InlineContent::Boolean(node) => node.$method($($arg),*),
            InlineContent::Cite(node) => node.$method($($arg),*),
            InlineContent::CiteGroup(node) => node.$method($($arg),*),
            InlineContent::CodeExpression(node) => node.$method($($arg),*),
            InlineContent::CodeFragment(node) => node.$method($($arg),*),
            InlineContent::Delete(node) => node.$method($($arg),*),
            InlineContent::Emphasis(node) => node.$method($($arg),*),
            InlineContent::ImageObject(node) => node.$method($($arg),*),
            InlineContent::Integer(node) => node.$method($($arg),*),
            InlineContent::Link(node) => node.$method($($arg),*),
            InlineContent::MathFragment(node) => node.$method($($arg),*),
            InlineContent::NontextualAnnotation(node) => node.$method($($arg),*),
            InlineContent::Note(node) => node.$method($($arg),*),
            InlineContent::Null(node) => node.$method($($arg),*),
            InlineContent::Number(node) => node.$method($($arg),*),
            InlineContent::Parameter(node) => node.$method($($arg),*),
            InlineContent::Quote(node) => node.$method($($arg),*),
            InlineContent::String(node) => node.$method($($arg),*),
            InlineContent::Strong(node) => node.$method($($arg),*),
            InlineContent::Subscript(node) => node.$method($($arg),*),
            InlineContent::Superscript(node) => node.$method($($arg),*),
            InlineContent::VideoObject(node) => node.$method($($arg),*),
        }
    }
}

#[macro_export]
macro_rules! dispatch_block {
    ($node:expr, $method:ident $(,$arg:expr)*) => {
        match $node {
            BlockContent::Claim(node) => node.$method($($arg),*),
            BlockContent::CodeBlock(node) => node.$method($($arg),*),
            BlockContent::CodeChunk(node) => node.$method($($arg),*),
            BlockContent::Collection(node) => node.$method($($arg),*),
            BlockContent::Figure(node) => node.$method($($arg),*),
            BlockContent::Heading(node) => node.$method($($arg),*),
            BlockContent::Include(node) => node.$method($($arg),*),
            BlockContent::List(node) => node.$method($($arg),*),
            BlockContent::MathBlock(node) => node.$method($($arg),*),
            BlockContent::Paragraph(node) => node.$method($($arg),*),
            BlockContent::QuoteBlock(node) => node.$method($($arg),*),
            BlockContent::Table(node) => node.$method($($arg),*),
            BlockContent::ThematicBreak(node) => node.$method($($arg),*),
        }
    }
}

#[macro_export]
macro_rules! dispatch_work {
    ($node:expr, $method:ident $(,$arg:expr)*) => {
        match $node {
            CreativeWorkTypes::Article(node) => node.$method($($arg),*),
            CreativeWorkTypes::AudioObject(node) => node.$method($($arg),*),
            CreativeWorkTypes::Claim(node) => node.$method($($arg),*),
            CreativeWorkTypes::Collection(node) => node.$method($($arg),*),
            CreativeWorkTypes::Comment(node) => node.$method($($arg),*),
            CreativeWorkTypes::CreativeWork(node) => node.$method($($arg),*),
            CreativeWorkTypes::Datatable(node) => node.$method($($arg),*),
            CreativeWorkTypes::Figure(node) => node.$method($($arg),*),
            CreativeWorkTypes::ImageObject(node) => node.$method($($arg),*),
            CreativeWorkTypes::MediaObject(node) => node.$method($($arg),*),
            CreativeWorkTypes::Periodical(node) => node.$method($($arg),*),
            CreativeWorkTypes::PublicationIssue(node) => node.$method($($arg),*),
            CreativeWorkTypes::PublicationVolume(node) => node.$method($($arg),*),
            CreativeWorkTypes::Review(node) => node.$method($($arg),*),
            CreativeWorkTypes::SoftwareApplication(node) => node.$method($($arg),*),
            CreativeWorkTypes::SoftwareSourceCode(node) => node.$method($($arg),*),
            CreativeWorkTypes::Table(node) => node.$method($($arg),*),
            CreativeWorkTypes::VideoObject(node) => node.$method($($arg),*),
        }
    }
}
