use codec::common::{
    eyre::Result,
    serde::{de::DeserializeOwned, Serialize},
    serde_yaml,
};

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
}
