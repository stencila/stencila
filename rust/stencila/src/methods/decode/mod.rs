use crate::formats::{format_type, FormatType};
use codec_trait::Codec;
use eyre::Result;
use stencila_schema::Node;

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

    // TODO: Pass options to this function?
    let options = None;

    // Allow these for when no features are enabled
    #[allow(unused_variables, unreachable_code)]
    Ok(match format {
        #[cfg(feature = "decode-date")]
        "date" => codec_date::DateCodec::from_str(input, options)?,

        #[cfg(feature = "decode-docx")]
        "docx" => codec_docx::DocxCodec::from_str_async(input, options).await?,

        #[cfg(feature = "decode-html")]
        "html" => codec_html::HtmlCodec::from_str(input, options)?,

        #[cfg(feature = "decode-ipynb")]
        "ipynb" => codec_ipynb::IpynbCodec::from_str(input, options)?,

        #[cfg(feature = "decode-json")]
        "json" => codec_json::JsonCodec::from_str(input, options)?,

        #[cfg(feature = "decode-json5")]
        "json5" => codec_json5::Json5Codec::from_str(input, options)?,

        #[cfg(feature = "decode-latex")]
        "latex" => codec_latex::LatexCodec::from_str_async(input, options).await?,

        #[cfg(feature = "decode-pandoc")]
        "pandoc" => codec_pandoc::PandocCodec::from_str_async(input, options).await?,

        #[cfg(feature = "decode-person")]
        "person" => codec_person::PersonCodec::from_str(input, options)?,

        #[cfg(feature = "decode-md")]
        "md" => codec_md::MarkdownCodec::from_str(input, options)?,

        #[cfg(feature = "decode-rmd")]
        "rmd" => codec_rmd::RmdCodec::from_str(input, options)?,

        #[cfg(feature = "decode-rpng")]
        "rpng" => codec_rpng::RpngCodec::from_str(input, options)?,

        #[cfg(feature = "decode-toml")]
        "toml" => codec_toml::TomlCodec::from_str(input, options)?,

        #[cfg(feature = "decode-txt")]
        "txt" => codec_txt::TxtCodec::from_str(input, options)?,

        #[cfg(feature = "decode-yaml")]
        "yaml" => codec_yaml::YamlCodec::from_str(input, options)?,

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
