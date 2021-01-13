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
        input: String,

        /// TODO docs
        #[structopt(short, long)]
        from: Option<String>,
    }

    pub fn decode(args: Args) -> Result<Node> {
        let Args { input, from } = args;
        super::decode(input, from.unwrap_or_default())
    }
}

#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        pub input: String,

        pub from: Option<String>,
    }

    pub fn decode(params: Params) -> Result<Node> {
        let Params { input, from } = params;
        super::decode(input, from.unwrap_or_default())
    }
}

// Allow these for when no features are enabled
#[allow(unused_variables, unreachable_code)]
pub fn decode(input: String, from: String) -> Result<Node> {
    let node = match from.as_str() {
        #[cfg(feature = "json")]
        "json" => serde_json::from_str::<Node>(input.as_str())?,
        #[cfg(feature = "yaml")]
        "yaml" => serde_yaml::from_str::<Node>(input.as_str())?,
        _ => {
            #[cfg(feature = "request")]
            return crate::delegate::delegate(
                crate::methods::Method::Decode,
                rpc::Params {
                    input,
                    from: Some(from),
                },
            );

            #[cfg(not(feature = "request"))]
            anyhow::bail!("Unable to decode a node from format \"{}\"", from)
        }
    };
    Ok(node)
}
