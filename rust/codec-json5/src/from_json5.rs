use codec::common::{eyre::Result, serde::de::DeserializeOwned};

pub trait FromJson5: DeserializeOwned {
    /// Decode a Stencila Schema node from JSON5
    fn from_json5(json5: &str) -> Result<Self> {
        Ok(json5::from_str(json5)?)
    }
}

impl<T> FromJson5 for T where T: DeserializeOwned {}
