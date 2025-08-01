use codec::{
    Codec, DecodeInfo, DecodeOptions,
    common::{
        async_trait::async_trait,
        eyre::{Result, bail},
        serde::de::DeserializeOwned,
        serde_json,
    },
    schema::Node,
    status::Status,
};

pub mod client;
pub mod responses;
pub mod search_code;

pub use client::{request, search_url};
pub use responses::SearchCodeResponse;
pub use search_code::CodeSearchItem;

/// A codec for decoding GitHub REST API responses to Stencila Schema nodes
///
/// Not exposed as a standalone codec but used by sibling crates that
/// make use of the GitHub API.
///
/// See https://docs.github.com/en/rest for details.
pub struct GithubCodec;

#[async_trait]
impl Codec for GithubCodec {
    fn name(&self) -> &str {
        "github"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    async fn from_str(
        &self,
        json: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        Ok((from_str_any(json)?, DecodeInfo::none()))
    }
}

/// Decode a Stencila [`Node`] from a GitHub response JSON of known type
pub fn from_str<T>(json: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    Ok(serde_json::from_str(json)?)
}

/// Decode a Stencila [`Node`] from a GitHub response JSON of unknown type
pub fn from_str_any(json: &str) -> Result<Node> {
    let value: serde_json::Value = serde_json::from_str(json)?;

    let node = if let Some(items) = value.get("items") {
        if let Some(first_item_value) = items.as_array().and_then(|arr| arr.first()) {
            from_value_any(first_item_value)?
        } else {
            bail!("Empty GitHub search response")
        }
    } else {
        from_value_any(&value)?
    };

    Ok(node)
}

/// Decode a Stencila [`Node`] from a [`serde_json::Value`] in a GitHub response JSON
pub fn from_value_any(value: &serde_json::Value) -> Result<Node> {
    // For now, assume it's a code search item since that's what we support
    // In the future, this could be extended to handle other GitHub API responses
    if value.get("name").is_some()
        && value.get("path").is_some()
        && value.get("repository").is_some()
    {
        let code_item: CodeSearchItem = serde_json::from_value(value.clone())?;
        Ok(Node::SoftwareSourceCode(code_item.into()))
    } else {
        bail!("Unsupported GitHub API response format")
    }
}
