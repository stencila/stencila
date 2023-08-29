use codec::common::{eyre::Result, serde::Serialize, serde_yaml};

pub trait ToYaml: Serialize {
    /// Encode a Stencila Schema node to YAML
    fn to_yaml(&self) -> Result<String>
    where
        Self: Sized,
    {
        Ok(serde_yaml::to_string(self)?)
    }
}

impl<T> ToYaml for T where T: Serialize {}
