use std::{
    fmt::Display,
    io::{Read, Write},
    str::FromStr,
};

use base64::{Engine as _, prelude::BASE64_URL_SAFE_NO_PAD};
use eyre::{Report, Result};
use flate2::{Compression, read::ZlibDecoder, write::ZlibEncoder};
use percent_encoding::{AsciiSet, CONTROLS, percent_encode};
use serde::{Serialize, de::DeserializeOwned};
use strum::{Display, EnumString};
use url::Url;

use stencila_node_id::NodeId;
use stencila_node_path::NodePath;
use stencila_node_type::NodeType;

const BASE_URL: &str = "https://stencila.link";

/// Characters to percent-encode in URL query parameters
const QUERY_ENCODE_SET: &AsciiSet = &CONTROLS
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'&')
    .add(b'=');

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

    /// A file path for file link URLs
    ///
    /// Used to encode local file paths as stencila.link URLs for cloud document formats.
    pub file: Option<String>,

    /// The repository URL for file links
    ///
    /// Used with `file` and `commit` to enable redirects to GitHub.
    pub repo: Option<String>,

    /// The commit hash for file links
    ///
    /// Used with `file` and `repo` to enable permalinks to GitHub.
    pub commit: Option<String>,
}

/// The position in the node that the URL relates to
#[derive(Debug, Clone, Copy, EnumString, Display, PartialEq, Eq)]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
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
    /// https://stencila.link?jzb64=eNpTUgIAAGgARQ
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

        let mut node_url = NodeUrl::default();

        for (name, value) in url.query_pairs() {
            match name.as_ref() {
                "type" => node_url.r#type = value.parse().ok(),
                "id" => node_url.id = value.parse().ok(),
                "path" => node_url.path = value.parse().ok(),
                "position" => node_url.position = value.parse().ok(),
                "jzb64" => node_url.jzb64 = Some(value.to_string()),
                "file" => node_url.file = Some(value.to_string()),
                "repo" => node_url.repo = Some(value.to_string()),
                "commit" => node_url.commit = Some(value.to_string()),
                _ => {}
            };
        }

        Ok(node_url)
    }
}

impl Display for NodeUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{BASE_URL}")?;
        let mut pairs = Vec::new();
        if let Some(t) = &self.r#type {
            pairs.push(format!("type={t}"));
        }
        if let Some(id) = &self.id {
            pairs.push(format!("id={id}"));
        }
        if let Some(path) = &self.path {
            pairs.push(format!("path={path}"));
        }
        if let Some(pos) = &self.position {
            pairs.push(format!("position={pos}"));
        }
        if let Some(jzb64) = &self.jzb64 {
            pairs.push(format!("jzb64={jzb64}"));
        }
        if let Some(file) = &self.file {
            let encoded = percent_encode(file.as_bytes(), QUERY_ENCODE_SET);
            pairs.push(format!("file={encoded}"));
        }
        if let Some(repo) = &self.repo {
            let encoded = percent_encode(repo.as_bytes(), QUERY_ENCODE_SET);
            pairs.push(format!("repo={encoded}"));
        }
        if let Some(commit) = &self.commit
            && !matches!(commit.as_str(), "untracked" | "dirty")
        {
            pairs.push(format!("commit={commit}"));
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

    use eyre::{OptionExt, Result};

    use super::*;

    #[test]
    fn roundtrip_empty() -> Result<()> {
        let url = NodeUrl::default();

        let s = url.to_string();
        assert_eq!(s, BASE_URL);
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

        let url = NodeUrl {
            jzb64: Some(NodeUrl::to_jzb64(node)?),
            ..Default::default()
        }
        .to_string();

        let url = NodeUrl::from_str(&url)?;
        let round_tripped: String =
            NodeUrl::from_jzb64(&url.jzb64.ok_or_eyre("should have jzb64")?)?;
        assert_eq!(node, round_tripped);

        Ok(())
    }

    #[test]
    fn parse_full_url() -> Result<()> {
        let s =
            format!("{BASE_URL}?type=CodeChunk&id=cdc_abc123&path=content/1/item/4&position=begin");
        let url = NodeUrl::from_str(&s)?;
        assert_eq!(url.r#type, Some(NodeType::CodeChunk));
        assert_eq!(url.id, Some(NodeId::from_str("cdc_abc123")?));
        assert_eq!(url.path, Some(NodePath::from_str("content/1/item/4")?));
        assert_eq!(url.position, Some(NodePosition::Begin));

        Ok(())
    }

    #[test]
    fn roundtrip_file_link() -> Result<()> {
        let url = NodeUrl {
            file: Some("docs/README.md".to_string()),
            repo: Some("https://github.com/stencila/stencila".to_string()),
            commit: Some("abc123".to_string()),
            ..Default::default()
        };

        let s = url.to_string();
        assert!(s.starts_with(BASE_URL));
        assert!(s.contains("file=docs/README.md"));
        assert!(s.contains("repo=https://github.com/stencila/stencila"));
        assert!(s.contains("commit=abc123"));

        let parsed = NodeUrl::from_str(&s)?;
        assert_eq!(parsed.file, url.file);
        assert_eq!(parsed.repo, url.repo);
        assert_eq!(parsed.commit, url.commit);

        Ok(())
    }

    #[test]
    fn file_link_with_special_chars() -> Result<()> {
        let url = NodeUrl {
            file: Some("path with spaces/file#1.md".to_string()),
            ..Default::default()
        };

        let s = url.to_string();
        // Spaces and # should be encoded
        assert!(s.contains("file=path%20with%20spaces/file%231.md"));

        let parsed = NodeUrl::from_str(&s)?;
        assert_eq!(parsed.file, url.file);

        Ok(())
    }
}
