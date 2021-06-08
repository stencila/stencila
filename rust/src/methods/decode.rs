use crate::plugins;
use eyre::Result;
use maplit::hashmap;
use stencila_schema::Node;

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub async fn decode(content: &str, format: &str) -> Result<Node> {
    let node = match format {
        #[cfg(feature = "format-json")]
        "json" => serde_json::from_str::<Node>(content)?,
        #[cfg(feature = "format-yaml")]
        "yaml" => serde_yaml::from_str::<Node>(content)?,
        _ => {
            #[cfg(feature = "request")]
            return plugins::delegate(
                super::Method::Decode,
                hashmap! {
                    "content".to_string() => serde_json::to_value(content)?,
                    "format".to_string() => serde_json::to_value(format)?
                },
            )
            .await;

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

    pub async fn decode(params: Params) -> Result<Node> {
        let Params { content, format } = params;
        super::decode(&content, &format.unwrap_or_else(|| "json".to_string())).await
    }
}
