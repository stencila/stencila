pub use defaults::Defaults;
pub use serde::{de, Deserialize, Deserializer, Serialize};
pub use serde_json::Value;
pub use serde_with::skip_serializing_none;
use std::collections::BTreeMap;
pub use std::sync::Arc;

/// The set of primitive (non-Entity) node types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Primitive {
    Null,
    Bool(Bool),
    Integer(Integer),
    Number(Number),
    String(String),
    Object(Object),
    Array(Array),
}

/// A boolean value
pub type Bool = bool;

/// An integer value
///
/// Uses `i64` for maximum precision.
pub type Integer = i64;

/// A floating point value (a.k.a real number)
///
/// Uses `i64` for maximum precision.
pub type Number = f64;

/// An array value (a.k.a. vector)
pub type Array = Vec<Primitive>;

/// An object value (a.k.a map, dictionary)
///
/// Uses `BTreeMap` to preserve order.
pub type Object = BTreeMap<String, Primitive>;

// Checks the `type` property during deserialization.
// See notes in TypesScript function `interfaceSchemaToEnum`
// and https://github.com/serde-rs/serde/issues/760.
#[macro_export]
macro_rules! impl_type {
    ($type:ident) => {
        impl $type {
            pub fn deserialize_type<'de, D>(d: D) -> Result<String, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(d)?;
                if value != stringify!($type) {
                    return Err(de::Error::invalid_value(
                        de::Unexpected::Str(value.as_str()),
                        &stringify!($type),
                    ));
                }
                Ok(value)
            }
        }
    };
}
