use std::path::Path;

use polars::prelude::*;

use codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, async_trait,
    eyre::{Result, bail},
    format::Format,
    schema::{Node, NodeType},
    status::Status,
};

mod conversion;
mod formats;

use conversion::{dataframe_to_datatable, datatable_to_dataframe};
use formats::{
    read_arrow, read_csv, read_parquet, read_tsv, write_arrow, write_csv, write_parquet, write_tsv,
};

/// A codec for tabular data formats (CSV, TSV, Parquet, Arrow, Avro)
pub struct CsvCodec;

#[async_trait]
impl Codec for CsvCodec {
    fn name(&self) -> &str {
        "csv"
    }

    fn status(&self) -> Status {
        Status::Beta
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Csv | Format::Tsv | Format::Parquet | Format::Arrow => NoLoss,
            _ => None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Csv | Format::Tsv => LowLoss,
            Format::Parquet | Format::Arrow => NoLoss,
            _ => None,
        }
    }

    fn supports_from_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        match node_type {
            NodeType::Datatable => NoLoss,
            _ => None,
        }
    }

    fn supports_to_type(&self, node_type: NodeType) -> CodecSupport {
        use CodecSupport::*;
        match node_type {
            NodeType::Datatable => NoLoss,
            _ => None,
        }
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
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let node = decode_from_str(str, options)?;
        Ok((node, DecodeInfo::none()))
    }

    async fn to_path(
        &self,
        node: &Node,
        path: &Path,
        options: Option<EncodeOptions>,
    ) -> Result<EncodeInfo> {
        encode_to_path(node, path, options)?;
        Ok(EncodeInfo::none())
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let string = encode_to_string(node, options)?;
        Ok((string, EncodeInfo::none()))
    }
}

/// Decode a [`Datatable`] from a file path.
///
/// Automatically detects the format based on the file extension if not specified in options.
/// Supports CSV, TSV, Parquet, and Arrow formats. The file is read using Polars DataFrame
/// operations for efficient processing, then converted to Stencila's [`Datatable`] schema.
pub fn decode_from_path(path: &Path, options: Option<DecodeOptions>) -> Result<Node> {
    let options = options.unwrap_or_default();
    let format = options.format.unwrap_or_else(|| Format::from_path(path));

    let df = match format {
        Format::Csv => read_csv(path)?,
        Format::Tsv => read_tsv(path)?,
        Format::Parquet => read_parquet(path)?,
        Format::Arrow => read_arrow(path)?,
        _ => bail!("Unsupported format: {}", format),
    };

    Ok(Node::Datatable(dataframe_to_datatable(df)?))
}

/// Decode a [`Datatable`] from a string.
///
/// Defaults to CSV format if not specified in options. Currently supports CSV and TSV
/// formats for string decoding. Uses an in-memory cursor to parse the string data
/// through Polars, which is then converted to a [`Datatable`].
pub fn decode_from_str(str: &str, options: Option<DecodeOptions>) -> Result<Node> {
    let options = options.unwrap_or_default();
    let format = options.format.unwrap_or(Format::Csv);

    let df = match format {
        Format::Csv => {
            let cursor = std::io::Cursor::new(str.as_bytes());
            CsvReader::new(cursor).finish()?
        }
        Format::Tsv => {
            // CsvReader doesn't support separator on cursors in polars 0.43
            // So we'll use the default CSV reader for now
            // This is a limitation that should be addressed in future versions
            let cursor = std::io::Cursor::new(str.as_bytes());
            CsvReader::new(cursor).finish()?
        }
        _ => bail!("Format {} not supported for string decoding", format),
    };

    Ok(Node::Datatable(dataframe_to_datatable(df)?))
}

/// Encode a [`Node`] containing a [`Datatable`] to a file path.
///
/// Automatically determines the output format from the file extension if not specified.
/// The [`Datatable`] is first converted to a Polars DataFrame for efficient serialization,
/// then written to the specified format (CSV, TSV, Parquet, or Arrow).
pub fn encode_to_path(node: &Node, path: &Path, options: Option<EncodeOptions>) -> Result<()> {
    let datatable = match node {
        Node::Datatable(dt) => dt,
        _ => bail!("Only Datatable nodes can be encoded to tabular formats"),
    };

    let options = options.unwrap_or_default();
    let format = options.format.unwrap_or_else(|| Format::from_path(path));

    let df = datatable_to_dataframe(datatable)?;

    match format {
        Format::Csv => write_csv(&df, path)?,
        Format::Tsv => write_tsv(&df, path)?,
        Format::Parquet => write_parquet(&df, path)?,
        Format::Arrow => write_arrow(&df, path)?,
        _ => bail!("Unsupported format: {}", format),
    }

    Ok(())
}

/// Encode a [`Node`] containing a [`Datatable`] to a string.
///
/// Defaults to CSV format if not specified in options. Currently supports CSV and TSV
/// formats for string encoding. The [`Datatable`] is converted to a Polars DataFrame
/// and then serialized to an in-memory buffer before returning as a string.
pub fn encode_to_string(node: &Node, options: Option<EncodeOptions>) -> Result<String> {
    let datatable = match node {
        Node::Datatable(dt) => dt,
        _ => bail!("Only Datatable nodes can be encoded to tabular formats"),
    };

    let options = options.unwrap_or_default();
    let format = options.format.unwrap_or(Format::Csv);

    let df = datatable_to_dataframe(datatable)?;

    match format {
        Format::Csv => {
            let mut bytes = Vec::new();
            let mut writer = CsvWriter::new(&mut bytes);
            writer.finish(&mut df.clone())?;
            Ok(String::from_utf8(bytes)?)
        }
        Format::Tsv => {
            let mut bytes = Vec::new();
            let mut writer = CsvWriter::new(&mut bytes).with_separator(b'\t');
            writer.finish(&mut df.clone())?;
            Ok(String::from_utf8(bytes)?)
        }
        _ => bail!("Format {} not supported for string encoding", format),
    }
}
