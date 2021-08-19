#[macro_export]
macro_rules! dispatch_inline {
    ($node:expr, $method:ident $( , $arg:ident )*) => {
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
            InlineContent::Null => (),
            InlineContent::Number(node) => node.$method($($arg),*),
            InlineContent::Parameter(node) => node.$method($($arg),*),
            InlineContent::Quote(node) => node.$method($($arg),*),
            InlineContent::String(node) => node.$method($($arg),*),
            InlineContent::Strong(node) => node.$method($($arg),*),
            InlineContent::Subscript(node) => node.$method($($arg),*),
            InlineContent::Superscript(node) => node.$method($($arg),*),
            InlineContent::VideoObject(node) => node.$method($($arg),*),
        }
    };
}
