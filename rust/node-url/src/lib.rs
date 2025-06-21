use std::{
    fmt::Display,
    io::{Read, Write},
    str::FromStr,
};

use base64::{prelude::BASE64_URL_SAFE_NO_PAD, Engine as _};
use flate2::{read::ZlibDecoder, write::ZlibEncoder, Compression};
use url::Url;

use common::{
    eyre::{bail, Report, Result},
    serde::{de::DeserializeOwned, Serialize},
    serde_json,
    strum::{Display, EnumString},
};
use node_id::NodeId;
use node_path::NodePath;
use node_type::NodeType;

const DOMAIN: &str = "stencila.io";
const PATH: &str = "/node";

/// A URL describing a Stencila Schema node
///
/// Note that all fields, equivalent to URL query parameters, are optional.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct NodeUrl {
    /// The type of node
    pub r#type: Option<NodeType>,

    /// The id of the node
    pub id: Option<NodeId>,

    /// The path of the node within the document
    pub path: Option<NodePath>,

    /// The position of the link within the node
    pub position: Option<NodePosition>,

    /// The node as JSON, compressed using ZLib, and Base64 encoded
    ///
    /// This is useful for formats, such as Google Docs, where it is not possible to embed
    /// a cache of the root node in the document.
    pub jzb64: Option<String>,
}

/// The position in the node that the URL relates to
#[derive(Debug, EnumString, Display, PartialEq, Eq)]
#[strum(
    ascii_case_insensitive,
    serialize_all = "lowercase",
    crate = "common::strum"
)]
pub enum NodePosition {
    Begin,
    End,
}

impl NodeUrl {
    /// Convert a node to the the `jzb64` field of the URL
    ///
    /// This uses ZLib encoding to reduce the length of the encoded JSON and Base64 encodes it to
    /// ensure that it is URL safe. The overhead of compression is small. For example, the URL for
    /// an empty string (the smallest possible node to encode) is:
    ///
    /// https://stencila.io/node?jzb64=eNpTUgIAAGgARQ
    pub fn to_jzb64<T>(node: T) -> Result<String>
    where
        T: Serialize,
    {
        let json = serde_json::to_string(&node)?;

        let mut encoder = ZlibEncoder::new(Vec::new(), Compression::best());
        encoder.write_all(json.as_bytes())?;

        let compressed = encoder.finish()?;
        let base64 = BASE64_URL_SAFE_NO_PAD.encode(&compressed);

        Ok(base64)
    }

    /// Create a node from the `jzb64` field of the URL
    pub fn from_jzb64<T>(jzb64: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let compressed = BASE64_URL_SAFE_NO_PAD.decode(jzb64)?;

        let mut decoder = ZlibDecoder::new(compressed.as_slice());
        let mut json = String::new();
        decoder.read_to_string(&mut json)?;

        let node: T = serde_json::from_str(&json)?;
        Ok(node)
    }
}

impl FromStr for NodeUrl {
    type Err = Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let url = Url::from_str(s)?;

        if !matches!(url.domain(), Some(DOMAIN)) {
            bail!("Domain is invalid for a Stencila node URL")
        }

        if !matches!(url.path(), PATH) {
            bail!("Path is invalid for a Stencila node URL")
        }

        let mut node_url = NodeUrl::default();

        for (name, value) in url.query_pairs() {
            match name.as_ref() {
                "type" => node_url.r#type = value.parse().ok(),
                "id" => node_url.id = value.parse().ok(),
                "path" => node_url.path = value.parse().ok(),
                "position" => node_url.position = value.parse().ok(),
                "jzb64" => node_url.jzb64 = Some(value.to_string()),
                _ => {}
            };
        }

        Ok(node_url)
    }
}

impl Display for NodeUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "https://{DOMAIN}{PATH}")?;
        let mut pairs = Vec::new();
        if let Some(t) = &self.r#type {
            pairs.push(format!("type={}", t));
        }
        if let Some(id) = &self.id {
            pairs.push(format!("id={}", id));
        }
        if let Some(path) = &self.path {
            pairs.push(format!("path={}", path));
        }
        if let Some(pos) = &self.position {
            pairs.push(format!("position={}", pos));
        }
        if let Some(jzb64) = &self.jzb64 {
            pairs.push(format!("jzb64={}", jzb64));
        }
        if !pairs.is_empty() {
            write!(f, "?")?;
            write!(f, "{}", pairs.join("&"))?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use common::eyre::Result;

    use super::*;

    #[test]
    fn roundtrip_empty() -> Result<()> {
        let url = NodeUrl::default();

        let s = url.to_string();
        assert_eq!(s, format!("https://{DOMAIN}{PATH}"));
        assert_eq!(NodeUrl::from_str(&s)?, url);

        Ok(())
    }

    #[test]
    fn roundtrip_fields() -> Result<()> {
        let url = NodeUrl {
            r#type: Some(NodeType::CodeChunk),
            id: Some(NodeId::from_str("cdc_123456")?),
            path: Some(NodePath::from_str("content/1/item/4")?),
            position: Some(NodePosition::End),
            ..Default::default()
        };

        let s = url.to_string();
        let parsed = NodeUrl::from_str(&s)?;
        assert_eq!(parsed, url);

        Ok(())
    }

    #[test]
    fn roundtrip_jzb64() -> Result<()> {
        let node = "Hello world!";

        let mut url = NodeUrl::default();
        url.jzb64 = Some(NodeUrl::to_jzb64(node)?);
        let url = url.to_string();

        let url = NodeUrl::from_str(&url)?;
        let round_tripped: String = NodeUrl::from_jzb64(&url.jzb64.unwrap())?;
        assert_eq!(node, round_tripped);

        Ok(())
    }

    #[test]
    fn parse_full_url() -> Result<()> {
        let s = "https://stencila.io/node?type=CodeChunk&id=cdc_abc123&path=content/1/item/4&position=begin";
        let url = NodeUrl::from_str(s)?;
        assert_eq!(url.r#type, Some(NodeType::CodeChunk));
        assert_eq!(url.id, Some(NodeId::from_str("cdc_abc123")?));
        assert_eq!(url.path, Some(NodePath::from_str("content/1/item/4")?));
        assert_eq!(url.position, Some(NodePosition::Begin));

        Ok(())
    }
}
