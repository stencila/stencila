use serde::{de::DeserializeOwned, Serialize};
use serde_json::{from_value, to_value, Value};

use crate::{prelude::*, ModifyOperation, StringPatchOrPrimitive};

impl ModifyOperation {
    /// Apply one or more modification operations to one or more nodes
    pub fn apply_many<T>(ops: &[ModifyOperation], nodes: &[T]) -> Result<Vec<T>>
    where
        T: Serialize + DeserializeOwned,
    {
        let mut values = Vec::new();

        for old in nodes {
            let mut new = to_value(old)?;
            for op in ops {
                op.apply_to(&mut new)?;
            }
            values.push(new)
        }

        let new = from_value(Value::Array(values))?;

        Ok(new)
    }

    /// Apply a modification operation to a JSON value
    fn apply_to(&self, old: &mut Value) -> Result<()> {
        if let StringPatchOrPrimitive::StringPatch(patch) = &*self.value {
            if let Some(existing) = old.get_mut(&self.target) {
                if let Some(string) = existing.as_str() {
                    *existing = Value::String(patch.apply(string));
                } else {
                    bail!(
                        "Unable to apply patch to property `{}` because it is not a string",
                        self.target
                    )
                }
            }
        } else if let StringPatchOrPrimitive::Primitive(value) = &*self.value {
            let value = to_value(value)?;
            if let Some(existing) = old.get_mut(&self.target) {
                *existing = value;
            } else if let Some(object) = old.as_object_mut() {
                object.insert(self.target.to_string(), value);
            } else {
                bail!(
                    "Unable to apply patch to property `{}` because it is not an object",
                    self.target
                )
            }
        }

        Ok(())
    }
}
