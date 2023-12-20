use common::smol_str::SmolStr;
use node_store::ReadNode;

use crate::{prelude::*, IntegerOrString};

impl ReadNode for IntegerOrString {
    fn load_int(value: &i64) -> Result<Self> {
        Ok(IntegerOrString::Integer(*value))
    }

    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(IntegerOrString::String(value.to_string()))
    }
}
