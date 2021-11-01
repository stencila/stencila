use crate::formats::{format_type, FormatType};
use codec_trait::Codec;
use eyre::Result;
use stencila_schema::Node;

#[cfg(feature = "decode-docx")]
pub mod docx;

#[cfg(feature = "decode-ipynb")]
pub mod ipynb;

#[cfg(feature = "decode-md")]
pub mod md;

#[cfg(feature = "decode-rmd")]
pub mod rmd;

#[cfg(feature = "decode-latex")]
pub mod latex;

#[cfg(feature = "decode-pandoc")]
pub mod pandoc;

#[cfg(feature = "decode-rpng")]
pub mod rpng;

// Modules for types of content, rather than specific formats

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
        "date" => codec_date::DateCodec::from_str(input)?,

        #[cfg(feature = "decode-docx")]
        "docx" => docx::decode(input).await?,

        #[cfg(feature = "decode-html")]
        "html" => codec_html::HtmlCodec::from_str(input)?,

        #[cfg(feature = "decode-ipynb")]
        "ipynb" => ipynb::decode(input)?,

        #[cfg(feature = "decode-json")]
        "json" => codec_json::JsonCodec::from_str(input)?,

        #[cfg(feature = "decode-json5")]
        "json5" => codec_json5::Json5Codec::from_str(input)?,

        #[cfg(feature = "decode-latex")]
        "latex" => latex::decode(input).await?,

        #[cfg(feature = "decode-pandoc")]
        "pandoc" => pandoc::decode(input, "pandoc", &[]).await?,

        #[cfg(feature = "decode-person")]
        "person" => codec_person::PersonCodec::from_str(input)?,

        #[cfg(feature = "decode-md")]
        "md" => md::decode(input)?,

        #[cfg(feature = "decode-rmd")]
        "rmd" => rmd::decode(input)?,

        #[cfg(feature = "decode-rpng")]
        "rpng" => rpng::decode(input)?,

        #[cfg(feature = "decode-toml")]
        "toml" => codec_toml::TomlCodec::from_str(input)?,

        #[cfg(feature = "decode-txt")]
        "txt" => codec_txt::TxtCodec::from_str(input)?,

        #[cfg(feature = "decode-yaml")]
        "yaml" => codec_yaml::YamlCodec::from_str(input)?,

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
                        maplit::hashmap! {
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
