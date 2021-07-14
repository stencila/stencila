pub use defaults::Defaults;
pub use enum_dispatch::enum_dispatch;
pub use serde::{de, Deserialize, Deserializer, Serialize};
pub use serde_json::Value;
pub use serde_with::skip_serializing_none;
use std::collections::BTreeMap;
pub use std::sync::Arc;

/// A trait for methods that can be called on all types of nodes
#[enum_dispatch]
pub trait NodeTrait {
    /// Retrieve the `type` of an entity
    /// Needs to be called `type_name` because `type` is a reserved word
    fn type_name(&self) -> String;

    /// Retrieve the `id` of an entity
    fn id(&self) -> Option<String>;
}

macro_rules! impl_primitive {
    ($type:ident) => {
        impl NodeTrait for $type {
            fn type_name(&self) -> String {
                stringify!($type).into()
            }

            fn id(&self) -> Option<String> {
                None
            }
        }
    };
}

/// The set of primitive (non-Entity) node types
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Primitive {
    Null,
    Boolean(Boolean),
    Integer(Integer),
    Number(Number),
    String(String),
    Object(Object),
    Array(Array),
}

/// A boolean value
pub type Boolean = bool;
impl_primitive!(Boolean);

/// An integer value
///
/// Uses `i64` for maximum precision.
pub type Integer = i64;
impl_primitive!(Integer);

/// A floating point value (a.k.a real number)
///
/// Uses `i64` for maximum precision.
pub type Number = f64;
impl_primitive!(Number);

/// An array value (a.k.a. vector)
pub type Array = Vec<Primitive>;
impl_primitive!(Array);

/// An object value (a.k.a map, dictionary)
///
/// Uses `BTreeMap` to preserve order.
pub type Object = BTreeMap<String, Primitive>;
impl_primitive!(Object);

/// Macro to implement functions for struct schemas
#[macro_export]
macro_rules! impl_struct {
    ($type:ident) => {
        impl NodeTrait for $type {
            fn type_name(&self) -> String {
                stringify!($type).into()
            }

            fn id(&self) -> Option<String> {
                self.id.as_ref().map(|id| *id.clone())
            }
        }
    };
}

/// Macro to implement functions for enum schemas
#[macro_export]
macro_rules! impl_enum {
    ($type:ident) => {
        impl NodeTrait for $type {
            fn type_name(&self) -> String {
                stringify!($type).into()
            }

            fn id(&self) -> Option<String> {
                None
            }
        }
    };
}
