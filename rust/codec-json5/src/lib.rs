use common::{
    eyre::Result,
    json5,
    serde::{de::DeserializeOwned, Serialize},
};

pub trait FromJson5: DeserializeOwned {
    /// Deserialize a node from JSON5
    fn from_json5(json5: &str) -> Result<Self> {
        Ok(json5::from_str(json5)?)
    }
}

impl<T> FromJson5 for T where T: DeserializeOwned {}

pub trait ToJson5: Serialize {
    /// Serialize a node to JSON5
    ///
    /// Note: at the time of writing, the `json5` actually produces
    /// JSON output (which is of course valid JSON5, but less concise).
    fn to_json5(&self) -> Result<String>
    where
        Self: Sized,
    {
        Ok(json5::to_string(self)?)
    }
}

impl<T> ToJson5 for T where T: Serialize {}
