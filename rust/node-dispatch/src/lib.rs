///! Macros for dispatching method calls to variants of Stencila document node types

#[macro_export]
macro_rules! dispatch_node {
    ($node:expr, $default:expr, $method:ident $(,$arg:expr)*) => {
        match $node {
            Node::Array(node) => node.$method($($arg),*),
            Node::ArrayValidator(node) => node.$method($($arg),*),
            Node::Article(node) => node.$method($($arg),*),
            Node::AudioObject(node) => node.$method($($arg),*),
            Node::Boolean(node) => node.$method($($arg),*),
            Node::BooleanValidator(node) => node.$method($($arg),*),
            Node::Button(node) => node.$method($($arg),*),
            Node::Call(node) => node.$method($($arg),*),
            Node::Cite(node) => node.$method($($arg),*),
            Node::CiteGroup(node) => node.$method($($arg),*),
            Node::Claim(node) => node.$method($($arg),*),
            Node::CodeBlock(node) => node.$method($($arg),*),
            Node::CodeChunk(node) => node.$method($($arg),*),
            Node::CodeExpression(node) => node.$method($($arg),*),
            Node::CodeFragment(node) => node.$method($($arg),*),
            Node::Collection(node) => node.$method($($arg),*),
            Node::Comment(node) => node.$method($($arg),*),
            Node::ConstantValidator(node) => node.$method($($arg),*),
            Node::CreativeWork(node) => node.$method($($arg),*),
            Node::Datatable(node) => node.$method($($arg),*),
            Node::DatatableColumn(node) => node.$method($($arg),*),
            Node::Date(node) => node.$method($($arg),*),
            Node::DateTime(node) => node.$method($($arg),*),
            Node::Delete(node) => node.$method($($arg),*),
            Node::Directory(node) => node.$method($($arg),*),
            Node::Division(node) => node.$method($($arg),*),
            Node::Duration(node) => node.$method($($arg),*),
            Node::Emphasis(node) => node.$method($($arg),*),
            Node::EnumValidator(node) => node.$method($($arg),*),
            Node::Figure(node) => node.$method($($arg),*),
            Node::File(node) => node.$method($($arg),*),
            Node::For(node) => node.$method($($arg),*),
            Node::Form(node) => node.$method($($arg),*),
            Node::Heading(node) => node.$method($($arg),*),
            Node::If(node) => node.$method($($arg),*),
            Node::ImageObject(node) => node.$method($($arg),*),
            Node::Include(node) => node.$method($($arg),*),
            Node::Integer(node) => node.$method($($arg),*),
            Node::IntegerValidator(node) => node.$method($($arg),*),
            Node::Link(node) => node.$method($($arg),*),
            Node::List(node) => node.$method($($arg),*),
            Node::MathBlock(node) => node.$method($($arg),*),
            Node::MathFragment(node) => node.$method($($arg),*),
            Node::MediaObject(node) => node.$method($($arg),*),
            Node::NontextualAnnotation(node) => node.$method($($arg),*),
            Node::Note(node) => node.$method($($arg),*),
            Node::Null(node) => node.$method($($arg),*),
            Node::Number(node) => node.$method($($arg),*),
            Node::NumberValidator(node) => node.$method($($arg),*),
            Node::Object(node) => node.$method($($arg),*),
            Node::Paragraph(node) => node.$method($($arg),*),
            Node::Parameter(node) => node.$method($($arg),*),
            Node::Periodical(node) => node.$method($($arg),*),
            Node::PublicationIssue(node) => node.$method($($arg),*),
            Node::PublicationVolume(node) => node.$method($($arg),*),
            Node::Quote(node) => node.$method($($arg),*),
            Node::QuoteBlock(node) => node.$method($($arg),*),
            Node::Review(node) => node.$method($($arg),*),
            Node::Span(node) => node.$method($($arg),*),
            Node::SoftwareApplication(node) => node.$method($($arg),*),
            Node::SoftwareSourceCode(node) => node.$method($($arg),*),
            Node::Strikeout(node) => node.$method($($arg),*),
            Node::String(node) => node.$method($($arg),*),
            Node::StringValidator(node) => node.$method($($arg),*),
            Node::Strong(node) => node.$method($($arg),*),
            Node::Subscript(node) => node.$method($($arg),*),
            Node::Superscript(node) => node.$method($($arg),*),
            Node::Table(node) => node.$method($($arg),*),
            Node::ThematicBreak(node) => node.$method($($arg),*),
            Node::Time(node) => node.$method($($arg),*),
            Node::Timestamp(node) => node.$method($($arg),*),
            Node::TupleValidator(node) => node.$method($($arg),*),
            Node::Underline(node) => node.$method($($arg),*),
            Node::Validator(node) => node.$method($($arg),*),
            Node::VideoObject(node) => node.$method($($arg),*),
            _ => $default
        }
    }
}

#[macro_export]
macro_rules! dispatch_primitive {
    ($node:expr, $method:ident $(,$arg:expr)*) => {
        match $node {
            Primitive::Null(node) => node.$method($($arg),*),
            Primitive::Boolean(node) => node.$method($($arg),*),
            Primitive::Integer(node) => node.$method($($arg),*),
            Primitive::Number(node) => node.$method($($arg),*),
            Primitive::String(node) => node.$method($($arg),*),
            Primitive::Date(node) => node.$method($($arg),*),
            Primitive::Time(node) => node.$method($($arg),*),
            Primitive::DateTime(node) => node.$method($($arg),*),
            Primitive::Timestamp(node) => node.$method($($arg),*),
            Primitive::Duration(node) => node.$method($($arg),*),
            Primitive::Array(node) => node.$method($($arg),*),
            Primitive::Object(node) => node.$method($($arg),*),
        }
    }
}

#[macro_export]
macro_rules! dispatch_primitive_pair {
    ($node:expr, $other:expr, $default:expr, $method:ident $(,$arg:expr)*) => {
        match ($node, $other) {
            (Primitive::Null(node), Primitive::Null(other)) => node.$method(other, $($arg),*),
            (Primitive::Boolean(node), Primitive::Boolean(other)) => node.$method(other, $($arg),*),
            (Primitive::Integer(node), Primitive::Integer(other)) => node.$method(other, $($arg),*),
            (Primitive::Number(node), Primitive::Number(other)) => node.$method(other, $($arg),*),
            (Primitive::String(node), Primitive::String(other)) => node.$method(other, $($arg),*),
            (Primitive::Date(node), Primitive::Date(other)) => node.$method(other, $($arg),*),
            (Primitive::Time(node), Primitive::Time(other)) => node.$method(other, $($arg),*),
            (Primitive::DateTime(node), Primitive::DateTime(other)) => node.$method(other, $($arg),*),
            (Primitive::Timestamp(node), Primitive::Timestamp(other)) => node.$method(other, $($arg),*),
            (Primitive::Duration(node), Primitive::Duration(other)) => node.$method(other, $($arg),*),
            (Primitive::Array(node), Primitive::Array(other)) => node.$method(other, $($arg),*),
            (Primitive::Object(node), Primitive::Object(other)) => node.$method(other, $($arg),*),
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
            InlineContent::Button(node) => node.$method($($arg),*),
            InlineContent::Cite(node) => node.$method($($arg),*),
            InlineContent::CiteGroup(node) => node.$method($($arg),*),
            InlineContent::CodeExpression(node) => node.$method($($arg),*),
            InlineContent::CodeFragment(node) => node.$method($($arg),*),
            InlineContent::Date(node) => node.$method($($arg),*),
            InlineContent::DateTime(node) => node.$method($($arg),*),
            InlineContent::Delete(node) => node.$method($($arg),*),
            InlineContent::Duration(node) => node.$method($($arg),*),
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
            InlineContent::Span(node) => node.$method($($arg),*),
            InlineContent::Strikeout(node) => node.$method($($arg),*),
            InlineContent::String(node) => node.$method($($arg),*),
            InlineContent::Strong(node) => node.$method($($arg),*),
            InlineContent::Subscript(node) => node.$method($($arg),*),
            InlineContent::Superscript(node) => node.$method($($arg),*),
            InlineContent::Time(node) => node.$method($($arg),*),
            InlineContent::Timestamp(node) => node.$method($($arg),*),
            InlineContent::Underline(node) => node.$method($($arg),*),
            InlineContent::VideoObject(node) => node.$method($($arg),*),
        }
    }
}

#[macro_export]
macro_rules! dispatch_inline_pair {
    ($node:expr, $other:expr, $default:expr, $method:ident $(,$arg:expr)*) => {
        match ($node, $other) {
            (InlineContent::AudioObject(node), InlineContent::AudioObject(other)) => node.$method(other, $($arg),*),
            (InlineContent::Boolean(node), InlineContent::Boolean(other)) => node.$method(other, $($arg),*),
            (InlineContent::Button(node), InlineContent::Button(other)) => node.$method(other, $($arg),*),
            (InlineContent::Cite(node), InlineContent::Cite(other)) => node.$method(other, $($arg),*),
            (InlineContent::CiteGroup(node), InlineContent::CiteGroup(other)) => node.$method(other, $($arg),*),
            (InlineContent::CodeExpression(node), InlineContent::CodeExpression(other)) => node.$method(other, $($arg),*),
            (InlineContent::CodeFragment(node), InlineContent::CodeFragment(other)) => node.$method(other, $($arg),*),
            (InlineContent::Date(node), InlineContent::Date(other)) => node.$method(other, $($arg),*),
            (InlineContent::DateTime(node), InlineContent::DateTime(other)) => node.$method(other, $($arg),*),
            (InlineContent::Delete(node), InlineContent::Delete(other)) => node.$method(other, $($arg),*),
            (InlineContent::Duration(node), InlineContent::Duration(other)) => node.$method(other, $($arg),*),
            (InlineContent::Emphasis(node), InlineContent::Emphasis(other)) => node.$method(other, $($arg),*),
            (InlineContent::ImageObject(node), InlineContent::ImageObject(other)) => node.$method(other, $($arg),*),
            (InlineContent::Integer(node), InlineContent::Integer(other)) => node.$method(other, $($arg),*),
            (InlineContent::Link(node), InlineContent::Link(other)) => node.$method(other, $($arg),*),
            (InlineContent::MathFragment(node), InlineContent::MathFragment(other)) => node.$method(other, $($arg),*),
            (InlineContent::NontextualAnnotation(node), InlineContent::NontextualAnnotation(other)) => node.$method(other, $($arg),*),
            (InlineContent::Note(node), InlineContent::Note(other)) => node.$method(other, $($arg),*),
            (InlineContent::Null(node), InlineContent::Null(other)) => node.$method(other, $($arg),*),
            (InlineContent::Number(node), InlineContent::Number(other)) => node.$method(other, $($arg),*),
            (InlineContent::Parameter(node), InlineContent::Parameter(other)) => node.$method(other, $($arg),*),
            (InlineContent::Quote(node), InlineContent::Quote(other)) => node.$method(other, $($arg),*),
            (InlineContent::Span(node), InlineContent::Span(other)) => node.$method(other, $($arg),*),
            (InlineContent::Strikeout(node), InlineContent::Strikeout(other)) => node.$method(other, $($arg),*),
            (InlineContent::String(node), InlineContent::String(other)) => node.$method(other, $($arg),*),
            (InlineContent::Strong(node), InlineContent::Strong(other)) => node.$method(other, $($arg),*),
            (InlineContent::Subscript(node), InlineContent::Subscript(other)) => node.$method(other, $($arg),*),
            (InlineContent::Superscript(node), InlineContent::Superscript(other)) => node.$method(other, $($arg),*),
            (InlineContent::Time(node), InlineContent::Time(other)) => node.$method(other, $($arg),*),
            (InlineContent::Timestamp(node), InlineContent::Timestamp(other)) => node.$method(other, $($arg),*),
            (InlineContent::Underline(node), InlineContent::Underline(other)) => node.$method(other, $($arg),*),
            (InlineContent::VideoObject(node), InlineContent::VideoObject(other)) => node.$method(other, $($arg),*),
            _ => $default
        }
    }
}

#[macro_export]
macro_rules! dispatch_block {
    ($node:expr, $method:ident $(,$arg:expr)*) => {
        match $node {
            BlockContent::Call(node) => node.$method($($arg),*),
            BlockContent::Claim(node) => node.$method($($arg),*),
            BlockContent::CodeBlock(node) => node.$method($($arg),*),
            BlockContent::CodeChunk(node) => node.$method($($arg),*),
            BlockContent::Division(node) => node.$method($($arg),*),
            BlockContent::Figure(node) => node.$method($($arg),*),
            BlockContent::For(node) => node.$method($($arg),*),
            BlockContent::Form(node) => node.$method($($arg),*),
            BlockContent::Heading(node) => node.$method($($arg),*),
            BlockContent::If(node) => node.$method($($arg),*),
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
macro_rules! dispatch_block_pair {
    ($node:expr, $other:expr, $default:expr, $method:ident $(,$arg:expr)*) => {
        match ($node, $other) {
            (BlockContent::Call(node), BlockContent::Call(other)) => node.$method(other, $($arg),*),
            (BlockContent::Claim(node), BlockContent::Claim(other)) => node.$method(other, $($arg),*),
            (BlockContent::CodeBlock(node), BlockContent::CodeBlock(other)) => node.$method(other, $($arg),*),
            (BlockContent::CodeChunk(node), BlockContent::CodeChunk(other)) => node.$method(other, $($arg),*),
            (BlockContent::Division(node), BlockContent::Division(other)) => node.$method(other, $($arg),*),
            (BlockContent::Figure(node), BlockContent::Figure(other)) => node.$method(other, $($arg),*),
            (BlockContent::For(node), BlockContent::For(other)) => node.$method(other, $($arg),*),
            (BlockContent::Form(node), BlockContent::Form(other)) => node.$method(other, $($arg),*),
            (BlockContent::Heading(node), BlockContent::Heading(other)) => node.$method(other, $($arg),*),
            (BlockContent::If(node), BlockContent::If(other)) => node.$method(other, $($arg),*),
            (BlockContent::Include(node), BlockContent::Include(other)) => node.$method(other, $($arg),*),
            (BlockContent::List(node), BlockContent::List(other)) => node.$method(other, $($arg),*),
            (BlockContent::MathBlock(node), BlockContent::MathBlock(other)) => node.$method(other, $($arg),*),
            (BlockContent::Paragraph(node), BlockContent::Paragraph(other)) => node.$method(other, $($arg),*),
            (BlockContent::QuoteBlock(node), BlockContent::QuoteBlock(other)) => node.$method(other, $($arg),*),
            (BlockContent::Table(node), BlockContent::Table(other)) => node.$method(other, $($arg),*),
            (BlockContent::ThematicBreak(node), BlockContent::ThematicBreak(other)) => node.$method(other, $($arg),*),
            _ => $default
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
            CreativeWorkTypes::Directory(node) => node.$method($($arg),*),
            CreativeWorkTypes::Figure(node) => node.$method($($arg),*),
            CreativeWorkTypes::File(node) => node.$method($($arg),*),
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

#[macro_export]
macro_rules! dispatch_validator {
    ($node:expr, $method:ident $(,$arg:expr)*) => {
        match $node {
            ValidatorTypes::ArrayValidator(node) => node.$method($($arg),*),
            ValidatorTypes::BooleanValidator(node) => node.$method($($arg),*),
            ValidatorTypes::ConstantValidator(node) => node.$method($($arg),*),
            ValidatorTypes::DateTimeValidator(node) => node.$method($($arg),*),
            ValidatorTypes::DateValidator(node) => node.$method($($arg),*),
            ValidatorTypes::DurationValidator(node) => node.$method($($arg),*),
            ValidatorTypes::EnumValidator(node) => node.$method($($arg),*),
            ValidatorTypes::IntegerValidator(node) => node.$method($($arg),*),
            ValidatorTypes::NumberValidator(node) => node.$method($($arg),*),
            ValidatorTypes::StringValidator(node) => node.$method($($arg),*),
            ValidatorTypes::TimestampValidator(node) => node.$method($($arg),*),
            ValidatorTypes::TimeValidator(node) => node.$method($($arg),*),
            ValidatorTypes::TupleValidator(node) => node.$method($($arg),*),
            ValidatorTypes::Validator(node) => node.$method($($arg),*),
        }
    }
}

#[macro_export]
macro_rules! dispatch_validator_pair {
    ($node:expr, $other:expr, $default:expr, $method:ident $(,$arg:expr)*) => {
        match ($node, $other) {
            (ValidatorTypes::ArrayValidator(node), ValidatorTypes::ArrayValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::BooleanValidator(node), ValidatorTypes::BooleanValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::ConstantValidator(node), ValidatorTypes::ConstantValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::DateTimeValidator(node), ValidatorTypes::DateTime(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::DateValidator(node), DateTypes::Date(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::DurationValidator(node), ValidatorTypes::DurationValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::EnumValidator(node), ValidatorTypes::EnumValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::IntegerValidator(node), ValidatorTypes::IntegerValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::NumberValidator(node), ValidatorTypes::NumberValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::StringValidator(node), ValidatorTypes::StringValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::TimestampValidator(node), ValidatorTypes::TimestampValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::TimeValidator(node), ValidatorTypes::Time(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::TupleValidator(node), ValidatorTypes::TupleValidator(other)) => node.$method(other, $($arg),*),
            (ValidatorTypes::Validator(node), ValidatorTypes::Validator(other)) => node.$method(other, $($arg),*),
            _ => $default
        }
    }
}
