use codec::common::{
    eyre::Result,
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
};

impl<T> JsonCodec for T where T: DeserializeOwned + Serialize {}

pub trait JsonCodec: DeserializeOwned + Serialize {
    /// Decode a Stencila Schema node from JSON
    fn from_json(json: &str) -> Result<Self> {
        Ok(serde_json::from_str(json)?)
    }

    /// Decode a Stencila Schema node from a [`serde_json::Value`]
    fn from_json_value(json: serde_json::Value) -> Result<Self> {
        Ok(serde_json::from_value::<Self>(json)?)
    }

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
