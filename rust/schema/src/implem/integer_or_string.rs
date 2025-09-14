use stencila_node_store::ReadNode;

use crate::{IntegerOrString, prelude::*};

impl<S> From<S> for IntegerOrString
where
    S: AsRef<str>,
{
    fn from(value: S) -> Self {
        match value.as_ref().parse::<i64>() {
            Ok(int) => IntegerOrString::Integer(int),
            Err(..) => IntegerOrString::String(value.as_ref().into()),
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
