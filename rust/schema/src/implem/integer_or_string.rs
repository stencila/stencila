use stencila_node_store::ReadNode;

use crate::{IntegerOrString, prelude::*};

impl From<&str> for IntegerOrString {
    fn from(value: &str) -> Self {
        match value.parse::<i64>() {
            Ok(int) => IntegerOrString::Integer(int),
            Err(..) => IntegerOrString::String(value.into()),
        }
    }
}

impl ReadNode for IntegerOrString {
    fn load_int(value: &i64) -> Result<Self> {
        Ok(IntegerOrString::Integer(*value))
    }

    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(IntegerOrString::String(value.to_string()))
    }
}
