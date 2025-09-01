use std::path::Path;

use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, async_trait,
    eyre::{Result, bail},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
};

mod conversion;
mod formats;

use formats::{read_ods, read_xls, read_xlsx};

/// A codec for spreadsheet formats (XLSX, XLS, ODS)
///
/// This codec provides read-only support for Microsoft Excel and OpenDocument
/// spreadsheet formats using the calamine crate. It focuses on converting
/// spreadsheet data to Stencila's [`Datatable`] format for further processing.
pub struct XlsxCodec;

#[async_trait]
impl Codec for XlsxCodec {
    fn name(&self) -> &str {
        "xlsx"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Xlsx | Format::Xls | Format::Ods => LowLoss,
            _ => None,
        }
    }

    fn supports_to_format(&self, _format: &Format) -> CodecSupport {
        CodecSupport::None
    }

    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        match node_type {
            NodeType::Datatable => LowLoss,
            _ => None,
        }
    }

    fn supports_to_type(&self, _node_type: NodeType) -> CodecSupport {
        CodecSupport::None
    }

    async fn from_path(
        &self,
        path: &Path,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, Option<Node>, DecodeInfo)> {
        let node = decode_from_path(path, options)?;
        Ok((node, None, DecodeInfo::none()))
    }

    async fn from_str(
        &self,
        _str: &str,
        _options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        bail!("Decoding spreadsheet formats from string is not supported. Use from_path instead.")
    }

    async fn to_path(
        &self,
        _node: &Node,
        _path: &Path,
        _options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        bail!(
            "Writing spreadsheet formats is not currently supported. Consider using CSV, or other tabular data format."
        )
    }

    async fn to_string(
        &self,
        _node: &Node,
        _options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        bail!(
            "Encoding spreadsheet formats to string is not supported. Consider using CSV, or other tabular data format."
        )
    }
}

/// Decode a [`Datatable`] from a spreadsheet file path.
///
/// Automatically detects the format based on the file extension if not specified in options.
/// Supports XLSX, XLS, and ODS formats. The file is read using calamine, taking the first
/// worksheet and treating the first row as column headers.
pub fn decode_from_path(path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
    let options = options.unwrap_or_default();
    let format = options.format.unwrap_or_else(|| Format::from_path(path));

    let datatable = match format {
        Format::Xlsx => read_xlsx(path)?,
        Format::Xls => read_xls(path)?,
        Format::Ods => read_ods(path)?,
        _ => bail!("Unsupported format: {format}. Supported formats are XLSX, XLS, and ODS.",),
    };

    Ok(Node::Datatable(datatable))
}
