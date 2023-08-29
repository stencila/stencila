use codec::common::{eyre::Result, serde::de::DeserializeOwned, serde_json};

pub trait FromJson: DeserializeOwned {
    /// Decode a Stencila Schema node from JSON
    fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Decode a Stencila Schema node from a [`serde_json::Value`]
    fn from_json_value(json: serde_json::Value) -> Result<Self> {
        Ok(serde_json::from_value::<Self>(json)?)
    }
}

impl<T> FromJson for T where T: DeserializeOwned {}
