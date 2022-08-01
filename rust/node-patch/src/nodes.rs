use common::serde_json;
use stencila_schema::{Node, Number};

use super::prelude::*;

/// Override of macro to implement `from_value` for all node types
macro_rules! patchable_node_variants {
    ($( $variant:path )*) => {
        impl Patchable for Node {
            patchable_variants_is_equal!($( $variant )*);
            patchable_variants_hash!($( $variant )*);
            patchable_variants_apply_add!($( $variant )*);
            patchable_variants_apply_remove!($( $variant )*);
            patchable_variants_apply_replace!($( $variant )*);
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
                        if !self.is_equal(other).is_ok() {
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

            fn from_value(value: &Value) -> Result<Self>
            where
                Self: Clone + Sized + 'static,
            {
                if let Some(value) = value.downcast_ref::<Self>() {
                    return Ok(value.clone());
                } else if let Some(value) = value.downcast_ref::<serde_json::Value>() {
                    if let Some(string) = value.as_str() {
                        return Ok(Node::String(string.to_string()));
                    }
                    if let Some(number) = value.as_f64() {
                        return Ok(Node::Number(Number(number)));
                    }
                    if let Some(integer) = value.as_i64() {
                        return Ok(Node::Integer(integer));
                    }
                    if let Some(boolean) = value.as_bool() {
                        return Ok(Node::Boolean(boolean));
                    }
                }
                bail!(invalid_patch_value::<Self>())
            }
        }
    };
}

patchable_node_variants!(
    Node::Array
    Node::Article
    Node::AudioObject
    Node::Boolean
    Node::Cite
    Node::CiteGroup
    Node::Claim
    Node::CodeBlock
    Node::CodeChunk
    Node::CodeExpression
    Node::CodeFragment
    Node::Datatable
    Node::DatatableColumn
    Node::Delete
    Node::Emphasis
    Node::Figure
    Node::Heading
    Node::ImageObject
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
    Node::Strikeout
    Node::String
    Node::Strong
    Node::Subscript
    Node::Superscript
    Node::Table
    Node::ThematicBreak
    Node::Underline
    Node::VideoObject
);
