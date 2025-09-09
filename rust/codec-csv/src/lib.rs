use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::{Datatable, DatatableColumn, Node, NodeType, Primitive},
};

/// A codec for tabular data formats (CSV, TSV)
pub struct CsvCodec;

#[async_trait]
impl Codec for CsvCodec {
    fn name(&self) -> &str {
        "csv"
    }

    fn supports_from_format(&self, format: &Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Csv | Format::Tsv => NoLoss,
            _ => None,
        }
    }

    fn supports_to_format(&self, format: &Format) -> CodecSupport {
        use CodecSupport::*;
        match format {
            Format::Csv | Format::Tsv => LowLoss,
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

    async fn from_str(
        &self,
        str: &str,
        options: Option<DecodeOptions>,
    ) -> Result<(Node, DecodeInfo)> {
        let options = options.unwrap_or_default();
        let format = options.format.unwrap_or(Format::Csv);

        let delimiter = match format {
            Format::Csv => b',',
            Format::Tsv => b'\t',
            _ => bail!("Format {} not supported for string decoding", format),
        };

        let cursor = std::io::Cursor::new(str.as_bytes());
        let mut reader = csv::ReaderBuilder::new()
            .delimiter(delimiter)
            .has_headers(true)
            .from_reader(cursor);

        // Get headers
        let headers = reader.headers()?.clone();
        let column_names: Vec<String> = headers.iter().map(|h| h.to_string()).collect();

        // Read all records into memory for type inference
        let mut raw_data: Vec<Vec<String>> = Vec::new();
        for result in reader.records() {
            let record = result?;
            raw_data.push(record.iter().map(|s| s.to_string()).collect());
        }

        // Create columns with type inference
        let columns: Vec<DatatableColumn> = column_names
            .iter()
            .enumerate()
            .map(|(col_index, column_name)| {
                let column_values = raw_data
                    .iter()
                    .map(|row| row.get(col_index).map(|s| s.as_str()).unwrap_or(""))
                    .collect();

                DatatableColumn::from_strings(column_name.clone(), column_values)
            })
            .collect();

        let datatable = Datatable::new(columns);
        Ok((Node::Datatable(datatable), DecodeInfo::none()))
    }

    async fn to_string(
        &self,
        node: &Node,
        options: Option<EncodeOptions>,
    ) -> Result<(String, EncodeInfo)> {
        let datatable = match node {
            Node::Datatable(dt) => dt,
            _ => bail!("Only Datatable nodes can be encoded to tabular formats"),
        };

        let options = options.unwrap_or_default();
        let format = options.format.unwrap_or(Format::Csv);

        let delimiter = match format {
            Format::Csv => b',',
            Format::Tsv => b'\t',
            _ => bail!("Format {} not supported for string encoding", format),
        };

        let mut bytes = Vec::new();
        {
            let mut writer = csv::WriterBuilder::new()
                .delimiter(delimiter)
                .from_writer(&mut bytes);

            // Write headers
            let headers: Vec<&str> = datatable
                .columns
                .iter()
                .map(|col| col.name.as_str())
                .collect();
            writer.write_record(&headers)?;

            // Determine the number of rows (assuming all columns have the same length)
            let num_rows = datatable
                .columns
                .first()
                .map(|col| col.values.len())
                .unwrap_or(0);

            // Write data rows
            for row_index in 0..num_rows {
                let row: Vec<String> = datatable
                    .columns
                    .iter()
                    .map(|col| primitive_to_string(&col.values[row_index]))
                    .collect();
                writer.write_record(&row)?;
            }

            writer.flush()?;
        } // writer is dropped here

        let string = String::from_utf8(bytes)?;
        Ok((string, EncodeInfo::none()))
    }
}

/// Convert a Primitive value to its string representation for CSV output.
///
/// Handles all Primitive types and converts them to appropriate string formats
/// that can be round-tripped through CSV parsing.
fn primitive_to_string(primitive: &Primitive) -> String {
    match primitive {
        Primitive::Null(_) => String::new(),
        Primitive::Boolean(b) => b.to_string(),
        Primitive::Integer(i) => i.to_string(),
        Primitive::UnsignedInteger(u) => u.to_string(),
        Primitive::Number(n) => n.to_string(),
        Primitive::String(s) => s.clone(),
        Primitive::Array(_) => format!("{primitive:?}"), // Fallback for complex types
        Primitive::Object(_) => format!("{primitive:?}"), // Fallback for complex types
    }
}
