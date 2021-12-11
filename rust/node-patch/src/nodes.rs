use super::prelude::*;
use stencila_schema::Node;

/// Override of macro to implement `from_value` for all node types
macro_rules! patchable_node {
    ($( $variant:path )*) => {
        impl Patchable for Node {
            patchable_variants_is_equal!($( $variant )*);
            patchable_variants_hash!($( $variant )*);


            patchable_variants_diff!($( $variant )*);

            patchable_variants_apply_add!($( $variant )*);
            patchable_variants_apply_remove!($( $variant )*);
            patchable_variants_apply_replace!($( $variant )*);
            patchable_variants_apply_move!($( $variant )*);
            patchable_variants_apply_transform!($( $variant )*);

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
                        return Ok(Node::Number(number));
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

// TODO: Commented out node types need methods implemented to be
// able to be included as variants here
patchable_node!(
    //Node::Array
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
    //Node::Object
    Node::Paragraph
    Node::Quote
    Node::QuoteBlock
    Node::String
    Node::Strong
    Node::Subscript
    Node::Superscript
    Node::Table
    Node::ThematicBreak
    Node::VideoObject
);
