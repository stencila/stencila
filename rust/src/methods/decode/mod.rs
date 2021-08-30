use crate::formats::{format_type, FormatType};
use eyre::Result;
use maplit::hashmap;
use stencila_schema::Node;

#[cfg(feature = "decode-date")]
pub mod date;
#[cfg(feature = "decode-docx")]
pub mod docx;

#[cfg(feature = "decode-json")]
pub mod json;

#[cfg(feature = "decode-html")]
pub mod html;

#[cfg(feature = "decode-md")]
pub mod md;

#[cfg(feature = "decode-rmd")]
pub mod rmd;

#[cfg(feature = "decode-latex")]
pub mod latex;

#[cfg(feature = "decode-pandoc")]
pub mod pandoc;

#[cfg(feature = "decode-person")]
pub mod person;

#[cfg(feature = "decode-rpng")]
pub mod rpng;

#[cfg(feature = "decode-toml")]
pub mod toml;

#[cfg(feature = "decode-yaml")]
pub mod yaml;

pub mod code;
pub mod media;

/// Decode content in a particular format to a `Node`.
///
/// # Arguments
///
/// - `input`: the content to decode, either the content itself, or
///            for binary formats, the path to the file
/// - `format`: the format of the content e.g. `json`, `md`
pub async fn decode(input: &str, format: &str) -> Result<Node> {
    tracing::debug!(
        "Decoding string of length {} and format '{}'",
        input.len(),
        format
    );

    // Allow these for when no features are enabled
    #[allow(unused_variables, unreachable_code)]
    Ok(match format {
        #[cfg(feature = "decode-date")]
        "date" => date::decode(input)?,

        #[cfg(feature = "decode-docx")]
        "docx" => docx::decode(input).await?,

        #[cfg(feature = "decode-html")]
        "html" => html::decode(input, false)?,

        #[cfg(feature = "decode-json")]
        "json" => json::decode(input)?,

        #[cfg(feature = "decode-latex")]
        "latex" => latex::decode(input).await?,

        #[cfg(feature = "decode-pandoc")]
        "pandoc" => pandoc::decode(input, "pandoc", &[]).await?,

        #[cfg(feature = "decode-person")]
        "person" => person::decode(input)?,

        #[cfg(feature = "decode-md")]
        "md" => md::decode(input)?,

        #[cfg(feature = "decode-rmd")]
        "rmd" => rmd::decode(input)?,

        #[cfg(feature = "decode-rpng")]
        "rpng" => rpng::decode(input)?,

        #[cfg(feature = "decode-toml")]
        "toml" => toml::decode(input)?,

        #[cfg(feature = "decode-yaml")]
        "yaml" => yaml::decode(input)?,

        _ => {
            let format_type = format_type(format);
            match format_type {
                FormatType::AudioObject | FormatType::ImageObject | FormatType::VideoObject => {
                    media::decode(input, format_type)?
                }
                FormatType::SoftwareSourceCode => code::decode(input, format)?,
                _ => {
                    #[cfg(feature = "request")]
                    return crate::plugins::delegate(
                        super::Method::Decode,
                        hashmap! {
                            "innput".to_string() => serde_json::to_value(input)?,
                            "format".to_string() => serde_json::to_value(format)?
                        },
                    )
                    .await;

                    #[cfg(not(feature = "request"))]
                    eyre::bail!("Unable to decode format \"{}\"", format)
                }
            }
        }
    })
}

#[cfg(any(feature = "request", feature = "serve"))]
pub mod rpc {
    use super::*;
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    pub struct Params {
        pub input: String,
        pub format: String,
    }

    pub async fn decode(params: Params) -> Result<Node> {
        let Params { input, format } = params;
        super::decode(&input, &format).await
    }
}
