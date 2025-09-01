use serde::{Serialize, de::DeserializeOwned};
use serde_yaml;

use codec::eyre::Result;

impl<T> YamlCodec for T where T: DeserializeOwned + Serialize {}

pub trait YamlCodec: DeserializeOwned + Serialize {
    /// Decode a Stencila Schema node from YAML
    fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }

    /// Encode a Stencila Schema node to YAML
    fn to_yaml(&self) -> Result<String>
    where
        Self: Sized,
    {
        Ok(serde_yaml::to_string(self)?)
    }

    /// Encode a Stencila Schema node to a YAML value
    fn to_yaml_value(&self) -> Result<serde_yaml::Value>
    where
        Self: Sized,
    {
        Ok(serde_yaml::to_value(self)?)
    }
}
