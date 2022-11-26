//! Patching for [`Node`]s

use stencila_schema::Node;

use super::prelude::*;

/// Override of macro to implement `from_value` for all node types
macro_rules! patchable_node_variants {
    ($( $variant:path )*) => {
        impl Patchable for Node {
            patchable_variants_apply_add!($( $variant )*);
            patchable_variants_apply_add_many!($( $variant )*);

            patchable_variants_apply_remove!($( $variant )*);
            patchable_variants_apply_remove_many!($( $variant )*);

            patchable_variants_apply_replace!($( $variant )*);
            patchable_variants_apply_replace_many!($( $variant )*);

            patchable_variants_apply_move!($( $variant )*);

            patchable_variants_apply_transform!($( $variant )*);

            fn diff(&self, other: &Self, differ: &mut Differ) {
                #[allow(unreachable_patterns)]
                match (self, other) {
                    // For the atomic primitives, do replacement at this level,
                    // so that the `Replace` operation has a `value` of type
                    // `Node::Number` not a `f64` etc.
                    (Node::Boolean(..), Node::Boolean(..)) |
                    (Node::Integer(..), Node::Integer(..)) |
                    (Node::Number(..), Node::Number(..)) => {
                        if self != other {
                            differ.replace(other)
                        }
                    },
                    // For other matching pairs of other variants do diffing
                    $(
                        ($variant(me), $variant(other)) => me.diff(other, differ),
                    )*
                    // Usual fallback to replacement for unmatched variants
                    _ => differ.replace(other)
                }
            }
        }
    };
}

patchable_node_variants!(
    Node::Array
    Node::Article
    Node::AudioObject
    Node::Boolean
    Node::Button
    Node::Call
    Node::Cite
    Node::CiteGroup
    Node::Claim
    Node::CodeBlock
    Node::CodeChunk
    Node::CodeExpression
    Node::CodeFragment
    Node::Division
    Node::Datatable
    Node::DatatableColumn
    Node::Date
    Node::DateTime
    Node::Delete
    Node::Duration
    Node::Emphasis
    Node::Figure
    Node::For
    Node::Form
    Node::Heading
    Node::ImageObject
    Node::If
    Node::Include
    Node::Integer
    Node::Link
    Node::List
    Node::MathBlock
    Node::MathFragment
    Node::NontextualAnnotation
    Node::Note
    Node::Null
    Node::Number
    Node::Object
    Node::Paragraph
    Node::Parameter
    Node::Quote
    Node::QuoteBlock
    Node::Span
    Node::Strikeout
    Node::String
    Node::Strong
    Node::Subscript
    Node::Superscript
    Node::Table
    Node::ThematicBreak
    Node::Time
    Node::Timestamp
    Node::Underline
    Node::VideoObject
);
