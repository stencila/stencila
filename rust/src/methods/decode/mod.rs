use eyre::Result;
use maplit::hashmap;
use stencila_schema::Node;

#[cfg(feature = "decode-json")]
pub mod json;

#[cfg(feature = "decode-html")]
pub mod html;

#[cfg(feature = "decode-md")]
pub mod md;

#[cfg(feature = "decode-toml")]
pub mod toml;

#[cfg(feature = "decode-yaml")]
pub mod yaml;

/// Decode a `Node` from string content.
///
/// # Arguments
///
/// - `content`: the content to decode
/// - `format`: the format of the content e.g. `json`, `md`
pub async fn decode(content: &str, format: &str) -> Result<Node> {
    // Allow these for when no features are enabled
    #[allow(unused_variables, unreachable_code)]
    Ok(match format {
        #[cfg(feature = "decode-html")]
        "html" => html::decode(content, html::Options::default())?,

        #[cfg(feature = "decode-json")]
        "json" => json::decode(content)?,

        #[cfg(feature = "decode-md")]
        "md" => md::decode(content)?,

        #[cfg(feature = "decode-toml")]
        "toml" => toml::decode(content)?,

        #[cfg(feature = "decode-yaml")]
        "yaml" => yaml::decode(content)?,

        _ => {
            #[cfg(feature = "request")]
            return crate::plugins::delegate(
                super::Method::Decode,
                hashmap! {
                    "content".to_string() => serde_json::to_value(content)?,
                    "format".to_string() => serde_json::to_value(format)?
                },
            )
            .await;

            #[cfg(not(feature = "request"))]
            eyre::bail!("Unable to decode format \"{}\"", format)
        }
    })
}

#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        pub content: String,

        pub format: String,
    }

    pub async fn decode(params: Params) -> Result<Node> {
        let Params { content, format } = params;
        super::decode(&content, &format).await
    }
}
