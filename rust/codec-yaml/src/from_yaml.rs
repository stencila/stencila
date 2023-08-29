use codec::common::{eyre::Result, serde::de::DeserializeOwned, serde_yaml};

pub trait FromYaml: DeserializeOwned {
    /// Decode a Stencila Schema node from YAML
    fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }
}

impl<T> FromYaml for T where T: DeserializeOwned {}
