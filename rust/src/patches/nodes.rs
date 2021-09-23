use super::prelude::*;
use stencila_schema::Node;

/// Override of macro to implement `from_value` for all node types
macro_rules! patchable_node {
    ($( $variant:path )*) => {
        impl Patchable for Node {
            patchable_is_same!();
            patchable_variants_is_equal!($( $variant )*);
            patchable_variants_hash!($( $variant )*);

            patchable_diff!();
            patchable_variants_diff_same!($( $variant )*);

            patchable_variants_apply_maybe!($( $variant )*);
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

patchable_node!(Node::Article);
