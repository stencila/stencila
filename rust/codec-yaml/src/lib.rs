use codec::{
    common::{
        async_trait::async_trait,
        eyre::Result,
        serde::{de::DeserializeOwned, Serialize},
        serde_yaml,
    },
    format::Format,
    schema::Node,
    Codec, DecodeOptions, EncodeOptions,
};

pub struct YamlCodec;

#[async_trait]
impl Codec for YamlCodec {
    fn formats(&self) -> Vec<Format> {
        vec![Format::Yaml]
    }

    async fn from_str(&self, str: &str, _options: Option<DecodeOptions>) -> Result<Node> {
        Node::from_yaml(str)
    }

    async fn to_string(&self, node: &Node, _options: Option<EncodeOptions>) -> Result<String> {
        node.to_yaml()
    }
}

pub trait FromYaml: DeserializeOwned {
    /// Decode a Stencila Schema node from YAML
    fn from_yaml(yaml: &str) -> Result<Self> {
        Ok(serde_yaml::from_str(yaml)?)
    }
}

impl<T> FromYaml for T where T: DeserializeOwned {}

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
