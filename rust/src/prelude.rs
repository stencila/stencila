pub use defaults::Defaults;
pub use serde::{de, Deserialize, Deserializer, Serialize};
pub use serde_json::Value;
pub use serde_with::skip_serializing_none;
use std::collections::HashMap;
pub use std::sync::Arc;

pub type Null = Value;
pub type Bool = bool;
pub type Integer = i32;
pub type Number = f32;
pub type Array = Vec<Value>;
pub type Object = HashMap<String, Value>;

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
