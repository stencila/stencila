//! Exposes format conversion functionality provided by Rust codecs

use std::path::PathBuf;

use napi::Result;
use napi_derive::napi;

use codecs::{Format, LossesResponse};
use common::{eyre, serde_json};

use crate::utilities::{generic_failure, invalid_arg};

/// Decoding options
#[napi(object)]
pub struct DecodeOptions {
    /// The format to be decode from
    pub format: Option<String>,

    /// What to do if there are losses when decoding from the input
    ///
    /// Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or
    /// a file path to write the losses to (`json` or `yaml` file extensions are supported).
    pub losses: Option<String>,
}

impl TryInto<codecs::DecodeOptions> for DecodeOptions {
    type Error = napi::Error;

    fn try_into(self) -> Result<codecs::DecodeOptions> {
        Ok(codecs::DecodeOptions {
            format: match self.format {
                Some(format) => Some(Format::from_name(&format).map_err(invalid_arg)?),
                None => None,
            },
            losses: self
                .losses
                .map(LossesResponse::from)
                .unwrap_or(LossesResponse::Warn),
            ..Default::default()
        })
    }
}

/// Encoding options
#[napi(object)]
pub struct EncodeOptions {
    /// The format to encode to
    pub format: Option<String>,

    /// Whether to encode as a standalone document
    ///
    /// Unless specified otherwise, this is the default when encoding to a file
    /// (as opposed to a string).
    pub standalone: Option<bool>,

    /// Whether to encode in compact form
    ///
    /// Some formats (e.g HTML and JSON) can be encoded in either compact
    /// or "pretty-printed" (e.g. indented) forms.
    pub compact: Option<bool>,

    /// What to do if there are losses when encoding to the output
    ///
    /// Possible values are "ignore", "trace", "debug", "info", "warn", "error", or "abort", or
    /// a file path to write the losses to (`json` or `yaml` file extensions are supported).
    pub losses: Option<String>,
}

impl TryInto<codecs::EncodeOptions> for EncodeOptions {
    type Error = napi::Error;

    fn try_into(self) -> Result<codecs::EncodeOptions> {
        Ok(codecs::EncodeOptions {
            format: match self.format {
                Some(format) => Some(Format::from_name(&format).map_err(invalid_arg)?),
                None => None,
            },
            standalone: self.standalone,
            compact: self.compact,
            losses: self
                .losses
                .map(LossesResponse::from)
                .unwrap_or(LossesResponse::Warn),
            ..Default::default()
        })
    }
}

/// Decode a Stencila Schema node from a string
#[napi]
pub async fn from_string(input: String, options: Option<DecodeOptions>) -> Result<String> {
    let options = match options {
        Some(options) => Some(options.try_into()?),
        None => None,
    };

    let node = codecs::from_str(&input, options)
        .await
        .map_err(generic_failure)?;

    serde_json::to_string(&node)
        .map_err(eyre::Report::new)
        .map_err(generic_failure)
}

/// Decode a Stencila Schema node from a file system path
#[napi]
pub async fn from_path(path: String, options: Option<DecodeOptions>) -> Result<String> {
    let path = PathBuf::from(path);
    let options = match options {
        Some(options) => Some(options.try_into()?),
        None => None,
    };

    let node = codecs::from_path(&path, options)
        .await
        .map_err(generic_failure)?;

    serde_json::to_string(&node)
        .map_err(eyre::Report::new)
        .map_err(generic_failure)
}

/// Encode a Stencila Schema node to a string
#[napi]
pub async fn to_string(json: String, options: Option<EncodeOptions>) -> Result<String> {
    let options = match options {
        Some(options) => Some(options.try_into()?),
        None => None,
    };

    let node = serde_json::from_str(&json)
        .map_err(eyre::Report::new)
        .map_err(generic_failure)?;

    codecs::to_string(&node, options)
        .await
        .map_err(generic_failure)
}

/// Encode a Stencila Schema node to a filesystem path
#[napi]
pub async fn to_path(json: String, path: String, options: Option<EncodeOptions>) -> Result<()> {
    let path = PathBuf::from(path);
    let options = match options {
        Some(options) => Some(options.try_into()?),
        None => None,
    };

    let node = serde_json::from_str(&json)
        .map_err(eyre::Report::new)
        .map_err(generic_failure)?;

    codecs::to_path(&node, &path, options)
        .await
        .map_err(generic_failure)
}

/// Convert a document from one format to another
#[napi]
pub async fn from_to(
    input: Option<String>,
    output: Option<String>,
    decode_options: Option<DecodeOptions>,
    encode_options: Option<EncodeOptions>,
) -> Result<String> {
    let input = input.map(PathBuf::from);
    let output = output.map(PathBuf::from);
    let decode_options = match decode_options {
        Some(options) => Some(options.try_into()?),
        None => None,
    };
    let encode_options = match encode_options {
        Some(options) => Some(options.try_into()?),
        None => None,
    };

    codecs::convert(
        input.as_deref(),
        output.as_deref(),
        decode_options,
        encode_options,
    )
    .await
    .map_err(generic_failure)
}
