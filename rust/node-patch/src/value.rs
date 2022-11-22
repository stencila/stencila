use std::{any::Any, fmt::Debug};

use common::strum::Display;

use node_pointer::Pointable;

/// Type for the `value` property of `Add` and `Replace` operations
///
/// Has variants for the most common types of values in patches with
/// a fallback `Any` variant.
#[derive(Debug, Display)]
pub enum Value {
    Any(Box<dyn Any + Send>),
}

impl Value {
    // Construct an any variant
    pub fn any<Type>(value: Type) -> Value
    where
        Type: Send + 'static,
    {
        Self::Any(Box::new(value))
    }

    // Check if the value is of a specific type
    pub fn is<Type>(&self) -> bool
    where
        Type: 'static,
    {
        use Value::*;
        match self {
            Any(any) => any.is::<Type>(),
            _ => false,
        }
    }

    // Downcast the value to a reference of a specific type
    pub fn downcast_ref<Type>(&self) -> Option<&Type>
    where
        Type: 'static,
    {
        use Value::*;
        match self {
            Any(any) => any.downcast_ref(),
            _ => None,
        }
    }

    // Downcast the value to a mutable reference of a specific type
    pub fn downcast_mut<Type>(&mut self) -> Option<&mut Type>
    where
        Type: 'static,
    {
        use Value::*;
        match self {
            Any(any) => any.downcast_mut(),
            _ => None,
        }
    }
}
