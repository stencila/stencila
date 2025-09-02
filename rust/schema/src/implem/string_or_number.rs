use stencila_node_store::ReadNode;

use crate::{StringOrNumber, prelude::*};

impl From<&str> for StringOrNumber {
    fn from(value: &str) -> Self {
        match value.parse::<f64>() {
            Ok(num) => StringOrNumber::Number(num),
            Err(..) => StringOrNumber::String(value.into()),
        }
    }
}

impl ReadNode for StringOrNumber {
    fn load_int(value: &i64) -> Result<Self> {
        Ok(StringOrNumber::Number(*value as f64))
    }

    fn load_uint(value: &u64) -> Result<Self> {
        Ok(StringOrNumber::Number(*value as f64))
    }

    fn load_f64(value: &f64) -> Result<Self> {
        Ok(StringOrNumber::Number(*value))
    }

    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(StringOrNumber::String(value.to_string()))
    }
}
