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

// Date serialiser necessary for the `Date.value` property.
pub mod date_serializer {

    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(
        date: &DateTime<Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error> {
        date.to_rfc3339().serialize(serializer)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(
        deserializer: D,
    ) -> Result<DateTime<Utc>, D::Error> {
        let s: String = Deserialize::deserialize(deserializer)?;
        Ok(DateTime::parse_from_rfc3339(&s)
            .map_err(serde::de::Error::custom)?
            .into())
    }
}
