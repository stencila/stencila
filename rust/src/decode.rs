use crate::nodes::Node;
use eyre::Result;

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub fn decode(content: String, format: &str) -> Result<Node> {
    let node = match format {
        #[cfg(feature = "format-json")]
        "json" => serde_json::from_str::<Node>(content.as_str())?,
        #[cfg(feature = "format-yaml")]
        "yaml" => serde_yaml::from_str::<Node>(content.as_str())?,
        _ => {
            #[cfg(feature = "request")]
            return crate::delegate::delegate(
                crate::methods::Method::Decode,
                serde_json::json!({
                    "content": content,
                    "format": format
                }),
            );

            #[cfg(not(feature = "request"))]
            eyre::bail!("Unable to decode a node from format \"{}\"", from)
        }
    };
    Ok(node)
}
#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        pub content: String,

        pub format: Option<String>,
    }

    pub fn decode(params: Params) -> Result<Node> {
        let Params { content, format } = params;
        super::decode(content, &format.unwrap_or_else(|| "json".to_string()))
    }
}
