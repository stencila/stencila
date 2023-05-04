use codec::{
    common::{async_trait::async_trait, eyre::Result},
    format::Format,
    schema::{Article, Node},
    status::Status,
    Codec, DecodeOptions, EncodeOptions, Losses,
};

/// A codec for JATS XML
pub struct JatsCodec;

#[async_trait]
impl Codec for JatsCodec {
    fn name(&self) -> &str {
        "jats"
    }

    fn status(&self) -> Status {
        Status::UnderDevelopment
    }

    fn supported_formats(&self) -> Vec<Format> {
        vec![Format::Jats]
    }

    async fn from_str(
        &self,
        _str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, Losses)> {
        let mut losses = Losses::new();

        let node = Node::Article(Article::default());
        losses.register("Node", "*", "Under development");

        Ok((node, losses))
    }

    async fn to_string(
        &self,
        _node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let mut losses = Losses::new();

        let jats = String::new();
        losses.register("Node", "*", "Under development");

        Ok((jats, losses))
    }
}
