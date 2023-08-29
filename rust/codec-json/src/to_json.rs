use codec::common::{eyre::Result, serde::Serialize, serde_json};
pub trait ToJson: Serialize {
    /// Encode a Stencila Schema node to JSON
    fn to_json(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }

    /// Encode a Stencila Schema node to indented JSON
    fn to_json_pretty(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }

    /// Encode a Stencila Schema node to a [`serde_json::Value`]
    fn to_json_value(&self) -> Result<serde_json::Value> {
        Ok(serde_json::to_value(self)?)
    }
}

impl<T> ToJson for T where T: Serialize {}
