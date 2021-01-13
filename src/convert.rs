use crate::decode::decode;
use crate::delegate::delegate;
use crate::encode::encode;
use crate::err;
use crate::error::Error;
use crate::methods::Method;
use crate::nodes::Node;
use crate::validate::validate;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

#[derive(Debug, Serialize, Deserialize, StructOpt)]
#[structopt(
    about = "Convert content from one format to another",
    setting = structopt::clap::AppSettings::DeriveDisplayOrder
)]
pub struct Params {
    /// Content, path or URL to read
    input: String,

    /// Path or URL to write to
    output: String,

    #[structopt(short, long, default_value)]
    from: String,

    #[structopt(short, long, default_value)]
    to: String,
}

pub fn convert_params(params: Params) -> Result<Node> {
    let Params {
        input,
        output,
        from,
        to,
        ..
    } = params;

    convert(&input, &output, &from, &to)
}

pub fn convert(input: &str, output: &str, from: &str, to: &str) -> Result<Node> {
    // TODO: Attempt to delegate if can't handle both from and to formats
    if false {
        return delegate(
            Method::Convert,
            Params {
                input: String::from(input),
                output: String::from(output),
                from: String::from(from),
                to: String::from(to),
            },
        );
    }
    // TODO: If unable to delegate then attempt the following (which may involve transfer of decoded node to peer)
    let node = decode(input, from)?;
    // TODO: reshape prior to encoding
    encode(node, to)
}
