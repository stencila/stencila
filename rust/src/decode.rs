use crate::nodes::Node;
use anyhow::Result;

#[cfg(feature = "cli")]
pub mod cli {
    use super::*;
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    #[structopt(
        about = "Decode", // TODO about
        setting = structopt::clap::AppSettings::DeriveDisplayOrder
    )]
    pub struct Args {
        /// TODO docs
        content: String,

        /// TODO docs
        #[structopt(short, long)]
        format: Option<String>,
    }

    pub fn decode(args: Args) -> Result<()> {
        let Args { content, format } = args;

        super::decode(content, format.unwrap_or_default())?;

        Ok(())
    }
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
        super::decode(content, format.unwrap_or_default())
    }
}

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub fn decode(content: String, format: String) -> Result<Node> {
    let node = match format.as_str() {
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
            anyhow::bail!("Unable to decode a node from format \"{}\"", from)
        }
    };
    Ok(node)
}
