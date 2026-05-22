use super::super::{
    facts::{CodeFacts, IoDirection, IoFact},
    language::CodeLanguage,
    util::{function_name, path_expression},
};

/// Collect named-argument I/O facts from source text.
///
/// The main I/O rules are ast-grep patterns. This text pass covers API shapes
/// where named arguments are common but grammar captures have proven brittle,
/// such as Python `read_csv(filepath_or_buffer=...)` and R `read.csv(file=...)`.
/// It is intentionally narrow: each marker names one known function and one
/// path-bearing argument, and the captured value is normalized as an `IoPath`
/// without trying to infer surrounding dataflow.
pub(in crate::code) fn collect_named_io_text_facts(
    language: CodeLanguage,
    source: &str,
    facts: &mut CodeFacts,
) {
    let markers: &[NamedIoMarker] = match language {
        CodeLanguage::Python => &[
            // Tabular dataframe readers and writers.
            NamedIoMarker::read("read_csv", "filepath_or_buffer"),
            NamedIoMarker::read("read_table", "filepath_or_buffer"),
            NamedIoMarker::read("read_excel", "io"),
            NamedIoMarker::read("read_json", "path_or_buf"),
            NamedIoMarker::read("read_html", "io"),
            NamedIoMarker::read("read_parquet", "path"),
            NamedIoMarker::read("read_feather", "path"),
            NamedIoMarker::read("read_pickle", "filepath_or_buffer"),
            NamedIoMarker::read("read_hdf", "path_or_buf"),
            NamedIoMarker::read("read_orc", "path"),
            NamedIoMarker::read("read_sas", "filepath_or_buffer"),
            NamedIoMarker::read("read_stata", "filepath_or_buffer"),
            NamedIoMarker::read("read_fwf", "filepath_or_buffer"),
            NamedIoMarker::read("read_xml", "path_or_buffer"),
            NamedIoMarker::read("read_spss", "path"),
            NamedIoMarker::read("read_file", "filename"),
            NamedIoMarker::write("to_csv", "path_or_buf"),
            NamedIoMarker::write("to_excel", "excel_writer"),
            NamedIoMarker::write("to_json", "path_or_buf"),
            NamedIoMarker::write("to_html", "buf"),
            NamedIoMarker::write("to_parquet", "path"),
            NamedIoMarker::write("to_feather", "path"),
            NamedIoMarker::write("to_pickle", "path"),
            NamedIoMarker::write("to_hdf", "path_or_buf"),
            NamedIoMarker::write("to_orc", "path"),
            NamedIoMarker::write("to_stata", "path"),
            NamedIoMarker::write("to_xml", "path_or_buffer"),
            NamedIoMarker::write("to_latex", "buf"),
            NamedIoMarker::write("to_markdown", "buf"),
            NamedIoMarker::write("to_file", "filename"),
            // Array, matrix, labeled-array, and chunked-store APIs.
            NamedIoMarker::read("load", "file"),
            NamedIoMarker::read("loadtxt", "fname"),
            NamedIoMarker::read("genfromtxt", "fname"),
            NamedIoMarker::read("fromfile", "file"),
            NamedIoMarker::read("loadmat", "file_name"),
            NamedIoMarker::read("open_dataset", "filename_or_obj"),
            NamedIoMarker::read("open_dataarray", "filename_or_obj"),
            NamedIoMarker::read("open_mfdataset", "paths"),
            NamedIoMarker::read("open_zarr", "store"),
            NamedIoMarker::write("save", "file"),
            NamedIoMarker::write("savez", "file"),
            NamedIoMarker::write("savez_compressed", "file"),
            NamedIoMarker::write("savetxt", "fname"),
            NamedIoMarker::write("savemat", "file_name"),
            NamedIoMarker::write("torch.save", "f"),
            NamedIoMarker::write("save_file", "filename"),
            NamedIoMarker::write("to_netcdf", "path"),
            NamedIoMarker::write("to_zarr", "store"),
            // Images, plots, media, and URL helpers.
            NamedIoMarker::write("savefig", "fname"),
            NamedIoMarker::write("imwrite", "uri"),
            NamedIoMarker::write("imsave", "fname"),
            NamedIoMarker::read("requests.get", "url"),
            NamedIoMarker::read("urlopen", "url"),
            NamedIoMarker::read("urlretrieve", "url"),
            NamedIoMarker::write("urlretrieve", "filename"),
        ],
        CodeLanguage::R => &[
            NamedIoMarker::read("read.csv", "file"),
            NamedIoMarker::read("read_csv", "file"),
            NamedIoMarker::read("readRDS", "file"),
            NamedIoMarker::write("write.csv", "file"),
            NamedIoMarker::write("write_csv", "file"),
            NamedIoMarker::write("saveRDS", "file"),
            NamedIoMarker::write("ggsave", "filename"),
        ],
        _ => return,
    };

    for marker in markers {
        collect_named_marker(source, marker, facts);
    }

    let positional_markers: &[PositionalIoMarker] = match language {
        CodeLanguage::Python => &[
            PositionalIoMarker::write("torch.save", 1),
            PositionalIoMarker::write("save_file", 1),
        ],
        _ => return,
    };

    for marker in positional_markers {
        collect_positional_marker(source, marker, facts);
    }
}

#[derive(Debug, Clone, Copy)]
struct NamedIoMarker {
    /// Direction implied by the API and named argument.
    direction: IoDirection,

    /// Function spelling to search for in source text.
    function: &'static str,

    /// Named argument that carries the input or output path expression.
    argument: &'static str,
}

impl NamedIoMarker {
    fn read(function: &'static str, argument: &'static str) -> Self {
        Self {
            direction: IoDirection::Read,
            function,
            argument,
        }
    }

    fn write(function: &'static str, argument: &'static str) -> Self {
        Self {
            direction: IoDirection::Write,
            function,
            argument,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PositionalIoMarker {
    /// Direction implied by the API and positional argument.
    direction: IoDirection,

    /// Function spelling to search for in source text.
    function: &'static str,

    /// Zero-based argument index that carries the path expression.
    argument_index: usize,
}

impl PositionalIoMarker {
    fn write(function: &'static str, argument_index: usize) -> Self {
        Self {
            direction: IoDirection::Write,
            function,
            argument_index,
        }
    }
}

fn collect_named_marker(source: &str, marker: &NamedIoMarker, facts: &mut CodeFacts) {
    let call_marker = format!("{}(", marker.function);
    for call_index in call_indices(source, &call_marker) {
        let Some(call_source) = source.get(call_index..) else {
            continue;
        };
        let Some(arguments) = call_arguments(call_source) else {
            continue;
        };
        let Some(value) = named_argument_value(arguments, marker.argument) else {
            continue;
        };
        let Some(path) = path_from_argument_value(value) else {
            continue;
        };

        facts.io.insert(IoFact {
            direction: marker.direction,
            path,
            operation_offset: Some(call_index),
            target: None,
            target_offset: None,
            value: None,
            value_offset: None,
            function: function_name(marker.function),
            mode: None,
        });
    }
}

fn collect_positional_marker(source: &str, marker: &PositionalIoMarker, facts: &mut CodeFacts) {
    let call_marker = format!("{}(", marker.function);
    for call_index in call_indices(source, &call_marker) {
        let Some(call_source) = source.get(call_index..) else {
            continue;
        };
        let Some(arguments) = call_arguments(call_source) else {
            continue;
        };
        let segments = top_level_segments(arguments);
        let Some(value) = segments.get(marker.argument_index) else {
            continue;
        };
        if top_level_assignment(value).is_some() {
            continue;
        }
        let Some(path) = path_from_argument_value(value) else {
            continue;
        };

        facts.io.insert(IoFact {
            direction: marker.direction,
            path,
            operation_offset: Some(call_index),
            target: None,
            target_offset: None,
            value: None,
            value_offset: None,
            function: function_name(marker.function),
            mode: None,
        });
    }
}

/// Return call marker offsets that occur in executable source text.
///
/// This fallback scanner deliberately stays lexical. It does enough to avoid
/// matching commented-out code, string contents, and longer helper names such as
/// `my_read_csv(...)`, while leaving full parsing to the primary ast-grep pass.
fn call_indices(source: &str, call_marker: &str) -> Vec<usize> {
    let mut indices = Vec::new();
    let mut index = 0usize;

    while index < source.len() {
        let Some(char) = source[index..].chars().next() else {
            break;
        };

        match char {
            '#' => {
                index = skip_line_comment(source, index);
                continue;
            }
            '\'' | '"' | '`' => {
                index = skip_quoted(source, index, char);
                continue;
            }
            _ => {}
        }

        if source[index..].starts_with(call_marker) && is_call_boundary(source, index) {
            indices.push(index);
            index += call_marker.len();
        } else {
            index += char.len_utf8();
        }
    }

    indices
}

fn skip_line_comment(source: &str, index: usize) -> usize {
    source[index..]
        .find('\n')
        .map(|offset| index + offset + 1)
        .unwrap_or(source.len())
}

fn skip_quoted(source: &str, index: usize, quote: char) -> usize {
    let triple = matches!(quote, '\'' | '"')
        && source
            .as_bytes()
            .get(index..index + 3)
            .is_some_and(|bytes| bytes.iter().all(|byte| *byte == quote as u8));
    let delimiter_len = if triple { 3 } else { quote.len_utf8() };
    let delimiter = quote.to_string().repeat(delimiter_len);
    let mut escaped = false;
    let mut cursor = index + delimiter_len;

    while cursor < source.len() {
        if source[cursor..].starts_with(&delimiter) {
            return cursor + delimiter_len;
        }

        let Some(char) = source[cursor..].chars().next() else {
            break;
        };

        if !triple && escaped {
            escaped = false;
        } else if !triple && char == '\\' {
            escaped = true;
        }

        cursor += char.len_utf8();
    }

    source.len()
}

fn is_call_boundary(source: &str, index: usize) -> bool {
    source[..index]
        .chars()
        .next_back()
        .is_none_or(|char| !is_identifier_continue(char))
}

fn is_identifier_continue(char: char) -> bool {
    char == '_' || char.is_ascii_alphanumeric()
}

/// Return the argument source for the first call in a source slice.
///
/// The scanner starts each slice at a known `function(` marker. This helper then
/// walks until the matching closing parenthesis while respecting nested calls
/// and quoted strings, so later literals elsewhere in the file are not attached
/// to the current call.
fn call_arguments(source: &str) -> Option<&str> {
    let open = source.find('(')?;
    let mut depth = 1usize;
    let mut quote = None;
    let mut escaped = false;

    for (offset, char) in source[open + 1..].char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if char == '\\' {
                escaped = true;
            } else if char == delimiter {
                quote = None;
            }
            continue;
        }

        match char {
            '\'' | '"' | '`' => quote = Some(char),
            '(' => depth += 1,
            ')' => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    let end = open + 1 + offset;
                    return source.get(open + 1..end);
                }
            }
            _ => {}
        }
    }

    None
}

/// Find one named argument value within a call argument list.
///
/// Arguments are split only at top-level commas. This keeps values such as
/// `file.path("data", sample)` or `Path("data") / name` together before
/// comparing the left-hand side with the marker argument name.
fn named_argument_value<'a>(arguments: &'a str, argument: &str) -> Option<&'a str> {
    for segment in top_level_segments(arguments) {
        let Some((name, value)) = top_level_assignment(segment) else {
            continue;
        };
        if name.trim() == argument {
            return Some(value.trim());
        }
    }
    None
}

/// Split a comma-separated expression at top-level separators.
///
/// This is a lightweight source scanner rather than a language parser. It is
/// enough for function-call arguments because it tracks bracket depth and
/// strings, while deliberately leaving language-specific expression semantics to
/// `path_expression`.
fn top_level_segments(source: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut start = 0usize;
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;

    for (index, char) in source.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if char == '\\' {
                escaped = true;
            } else if char == delimiter {
                quote = None;
            }
            continue;
        }

        match char {
            '\'' | '"' | '`' => quote = Some(char),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                if let Some(segment) = source.get(start..index) {
                    segments.push(segment);
                }
                start = index + char.len_utf8();
            }
            _ => {}
        }
    }

    if let Some(segment) = source.get(start..) {
        segments.push(segment);
    }

    segments
}

/// Split a named argument at a top-level assignment operator.
///
/// This deliberately ignores `=` inside strings and nested expressions so
/// positional paths such as `"outputs/model=v1.pt"` are not mistaken for named
/// arguments.
fn top_level_assignment(source: &str) -> Option<(&str, &str)> {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;

    for (index, char) in source.char_indices() {
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if char == '\\' {
                escaped = true;
            } else if char == delimiter {
                quote = None;
            }
            continue;
        }

        match char {
            '\'' | '"' | '`' => quote = Some(char),
            '(' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth = depth.saturating_sub(1),
            '=' if depth == 0 => {
                let name = source.get(..index)?;
                let value = source.get(index + char.len_utf8()..)?;
                return Some((name, value));
            }
            _ => {}
        }
    }

    None
}

/// Normalize a named argument value into an I/O path expression.
///
/// Empty values are ignored. Static string literals become concrete resources,
/// template-like strings keep their shape, and other expressions are retained as
/// unknown paths so graph projection can still surface uncertain I/O evidence.
fn path_from_argument_value(source: &str) -> Option<super::super::facts::IoPath> {
    let trimmed = source.trim();
    (!trimmed.is_empty()).then(|| path_expression(trimmed))
}
