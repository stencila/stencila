use stencila_codec::{
    Codec, CodecSupport, DecodeInfo, DecodeOptions, EncodeInfo, EncodeOptions, async_trait,
    eyre::{Result, bail},
    stencila_format::Format,
    stencila_schema::{
        ArrayValidator, BooleanValidator, Datatable, DatatableColumn, IntegerValidator, Node,
        NodeType, Null, NumberValidator, Primitive, StringValidator, Validator,
    },
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
        let mut columns = Vec::new();
        for (col_index, column_name) in column_names.iter().enumerate() {
            let column_values: Vec<&str> = raw_data
                .iter()
                .map(|row| row.get(col_index).map(|s| s.as_str()).unwrap_or(""))
                .collect();

            let (values, validator) = infer_column_type_and_parse(&column_values)?;

            let mut column = DatatableColumn::new(column_name.clone(), values);
            column.validator = Some(validator);
            columns.push(column);
        }

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

/// Infer the column type from sample values and parse them into Primitives.
///
/// Examines all non-empty values to determine the most specific type that can
/// accommodate all values. Returns both the parsed values and an appropriate validator.
fn infer_column_type_and_parse(values: &[&str]) -> Result<(Vec<Primitive>, ArrayValidator)> {
    let mut has_integers = false;
    let mut has_floats = false;
    let mut has_booleans = false;
    let mut non_null_count = 0;

    // First pass: analyze types
    for value in values {
        if value.trim().is_empty() {
            continue;
        }

        non_null_count += 1;

        if value.parse::<bool>().is_ok() && (*value == "true" || *value == "false") {
            has_booleans = true;
        } else if value.parse::<i64>().is_ok() {
            has_integers = true;
        } else if value.parse::<f64>().is_ok() {
            has_floats = true;
        }
    }

    // Determine the most appropriate type
    let (parsed_values, validator) = if non_null_count == 0 {
        // All null/empty values - default to string
        let vals: Vec<Primitive> = values
            .iter()
            .map(|v| {
                if v.trim().is_empty() {
                    Primitive::Null(Null)
                } else {
                    Primitive::String(v.to_string())
                }
            })
            .collect();
        let mut validator = ArrayValidator::new();
        validator.items_validator =
            Some(Box::new(Validator::StringValidator(StringValidator::new())));
        (vals, validator)
    } else if has_booleans
        && non_null_count == values.iter().filter(|v| v.parse::<bool>().is_ok()).count()
    {
        // All non-null values are booleans
        let vals: Vec<Primitive> = values
            .iter()
            .map(|v| {
                if v.trim().is_empty() {
                    Primitive::Null(Null)
                } else {
                    match v.parse::<bool>() {
                        Ok(b) => Primitive::Boolean(b),
                        Err(_) => Primitive::String(v.to_string()),
                    }
                }
            })
            .collect();
        let mut validator = ArrayValidator::new();
        validator.items_validator = Some(Box::new(Validator::BooleanValidator(
            BooleanValidator::new(),
        )));
        (vals, validator)
    } else if has_floats {
        // Has floating point numbers
        let vals: Vec<Primitive> = values
            .iter()
            .map(|v| {
                if v.trim().is_empty() {
                    Primitive::Null(Null)
                } else {
                    match v.parse::<f64>() {
                        Ok(n) => Primitive::Number(n),
                        Err(_) => Primitive::String(v.to_string()),
                    }
                }
            })
            .collect();
        let mut validator = ArrayValidator::new();
        validator.items_validator =
            Some(Box::new(Validator::NumberValidator(NumberValidator::new())));
        (vals, validator)
    } else if has_integers {
        // Only integers
        let vals: Vec<Primitive> = values
            .iter()
            .map(|v| {
                if v.trim().is_empty() {
                    Primitive::Null(Null)
                } else {
                    match v.parse::<i64>() {
                        Ok(i) => Primitive::Integer(i),
                        Err(_) => Primitive::String(v.to_string()),
                    }
                }
            })
            .collect();
        let mut validator = ArrayValidator::new();
        validator.items_validator = Some(Box::new(Validator::IntegerValidator(
            IntegerValidator::new(),
        )));
        (vals, validator)
    } else {
        // Default to string
        let vals: Vec<Primitive> = values
            .iter()
            .map(|v| {
                if v.trim().is_empty() {
                    Primitive::Null(Null)
                } else {
                    Primitive::String(v.to_string())
                }
            })
            .collect();
        let mut validator = ArrayValidator::new();
        validator.items_validator =
            Some(Box::new(Validator::StringValidator(StringValidator::new())));
        (vals, validator)
    };

    Ok((parsed_values, validator))
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
