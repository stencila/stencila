use common::smol_str::SmolStr;
use node_store::{automerge::ObjId, ReadCrdt, ReadNode};

use crate::{prelude::*, PropertyValue, PropertyValueOrString};

impl ReadNode for PropertyValueOrString {
    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(PropertyValueOrString::String(value.to_string()))
    }

    fn load_map<C: ReadCrdt>(crdt: &C, obj: &ObjId) -> Result<Self> {
        Ok(PropertyValueOrString::PropertyValue(
            PropertyValue::load_map(crdt, obj)?,
        ))
    }
}
