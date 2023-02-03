use common::{
    eyre::Result,
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
};

pub trait FromJson5: DeserializeOwned {
    /// Deserialize a node from JSON
    fn from_json5(json5: &str) -> Result<Self> {
        Ok(serde_json::from_str(json5)?)
    }
}

impl<T> FromJson5 for T where T: DeserializeOwned {}

pub trait ToJson5: Serialize {
    /// Serialize a node to JSON5
    ///
    /// Note: at the time of writing, the `json5` actually produces
    /// JSON output (which is of course valid JSON5, but less concise).
    fn to_json5(&self) -> Result<String> {
        Ok(serde_json::to_string(self)?)
    }
}

impl<T> ToJson5 for T where T: Serialize {}
