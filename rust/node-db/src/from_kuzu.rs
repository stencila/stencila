use kuzu::Value;

use schema::*;

/// A trait for converting Kuzu values to Stencila nodes
pub trait FromKuzu {
    /// Create from a Kuzu value
    fn from_kuzu_value(value: Value) -> Self;
}

impl FromKuzu for Primitive {
    fn from_kuzu_value(value: Value) -> Self {
        Primitive::String(value.to_string())
    }
}
