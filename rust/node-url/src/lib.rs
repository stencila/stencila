use std::{fmt::Display, str::FromStr};

use common::{
    eyre::{bail, Report},
    strum::{Display, EnumString},
};
use node_id::NodeId;
use node_path::NodePath;
use node_type::NodeType;
use url::Url;

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
    fn roundtrip_all_fields() -> Result<()> {
        let url = NodeUrl {
            r#type: Some(NodeType::CodeChunk),
            id: Some(NodeId::from_str("cdc_123456")?),
            path: Some(NodePath::from_str("content/1/item/4")?),
            position: Some(NodePosition::End),
        };

        let s = url.to_string();
        let parsed = NodeUrl::from_str(&s)?;
        assert_eq!(parsed, url);

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
