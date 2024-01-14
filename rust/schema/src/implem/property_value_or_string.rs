use node_store::{automerge::ObjId, ReadNode, ReadStore};

use crate::{prelude::*, PropertyValue, PropertyValueOrString};

impl ReadNode for PropertyValueOrString {
    fn load_str(value: &SmolStr) -> Result<Self> {
        Ok(PropertyValueOrString::String(value.to_string()))
    }

    fn load_map<S: ReadStore>(store: &S, obj: &ObjId) -> Result<Self> {
        Ok(PropertyValueOrString::PropertyValue(
            PropertyValue::load_map(store, obj)?,
        ))
    }
}
