use common::{
    eyre::Result,
    serde::{de::DeserializeOwned, Serialize},
    serde_yaml,
};

pub trait FromYaml: DeserializeOwned {
    /// Deserialize a node from YAML
    fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }
}

impl<T> FromYaml for T where T: DeserializeOwned {}

pub trait ToYaml: Serialize {
    /// Serialize a node to YAML
    fn to_yaml(&self) -> Result<String>
    where
        Self: Sized,
    {
        Ok(serde_yaml::to_string(self)?)
    }
}

impl<T> ToYaml for T where T: Serialize {}
