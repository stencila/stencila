use codec::{
    common::{async_trait::async_trait, eyre::Result},
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

    async fn from_str(
        &self,
        _str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, Losses)> {
        let node = Node::Article(Article::default());

        Ok((node, Losses::none()))
    }

    async fn to_string(
        &self,
        _node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, Losses)> {
        let jats = String::new();

        Ok((jats, Losses::none()))
    }
}
