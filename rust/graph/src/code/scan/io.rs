use super::super::{
    facts::{CodeFacts, IoDirection, IoFact, IoPath, record_definition},
    language::CodeLanguage,
    util::{function_name, identifier_target, path_expression},
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
        // Python I/O is collected from the parse tree by the signature table in
        // `io_table.rs`, which covers positional and keyword forms together.
        CodeLanguage::Python => &[],
        // R I/O is collected from the parse tree by the signature table in
        // `io_table.rs`, which covers positional and keyword forms together.
        CodeLanguage::R => &[],
        CodeLanguage::Julia => &[
            // Core Julia file readers and handles.
            NamedIoMarker::read("open", "filename"),
            NamedIoMarker::read("open", "path"),
            NamedIoMarker::read("read", "filename"),
            NamedIoMarker::read("read", "path"),
            NamedIoMarker::read("readlines", "filename"),
            NamedIoMarker::read("readlines", "path"),
            NamedIoMarker::read("readline", "filename"),
            NamedIoMarker::read("readline", "path"),
            NamedIoMarker::read("readchomp", "filename"),
            NamedIoMarker::read("readchomp", "path"),
            NamedIoMarker::read("eachline", "filename"),
            NamedIoMarker::read("eachline", "path"),
            NamedIoMarker::read("readdlm", "source"),
            NamedIoMarker::read("readdlm", "input"),
            // Delimited and dataframe readers.
            NamedIoMarker::read("CSV.read", "file"),
            NamedIoMarker::read("CSV.read", "source"),
            NamedIoMarker::read("CSV.File", "file"),
            NamedIoMarker::read("CSV.File", "source"),
            NamedIoMarker::read("CSV.Rows", "file"),
            NamedIoMarker::read("CSV.Rows", "source"),
            NamedIoMarker::read("read_csv", "file"),
            NamedIoMarker::read("read_parquet", "path"),
            NamedIoMarker::read("read_parquet", "file"),
            NamedIoMarker::read("Table", "source"),
            NamedIoMarker::read("Table", "file"),
            NamedIoMarker::read("Stream", "source"),
            NamedIoMarker::read("Stream", "file"),
            // Serialization, arrays, workbooks, and scientific data stores.
            NamedIoMarker::read("load", "filename"),
            NamedIoMarker::read("load", "file"),
            NamedIoMarker::read("deserialize", "filename"),
            NamedIoMarker::read("npzread", "filename"),
            NamedIoMarker::read("npyread", "filename"),
            NamedIoMarker::read("matread", "file"),
            NamedIoMarker::read("readxlsx", "filename"),
            NamedIoMarker::read("openxlsx", "filename"),
            NamedIoMarker::read("h5open", "filename"),
            NamedIoMarker::read("h5read", "filename"),
            NamedIoMarker::read("NCDataset", "path"),
            NamedIoMarker::read("NCDataset", "filename"),
            // Structured, spatial, image, and plot readers.
            NamedIoMarker::read("parsefile", "filename"),
            NamedIoMarker::read("parsefile", "path"),
            NamedIoMarker::read("load_file", "filename"),
            NamedIoMarker::read("Raster", "filename"),
            NamedIoMarker::read("Raster", "path"),
            // URL and copy helpers.
            NamedIoMarker::status_read("download", "url"),
            NamedIoMarker::status_read("Downloads.download", "url"),
            NamedIoMarker::status_read("cp", "src"),
            NamedIoMarker::status_read("mv", "src"),
            // Core Julia file, delimited text, and serialization writers.
            NamedIoMarker::write("write", "filename"),
            NamedIoMarker::write("write", "path"),
            NamedIoMarker::write("writedlm", "f"),
            NamedIoMarker::write("writedlm", "filename"),
            NamedIoMarker::write("serialize", "filename"),
            // Delimited and dataframe writers.
            NamedIoMarker::write("CSV.write", "file"),
            NamedIoMarker::write("CSV.write", "source"),
            NamedIoMarker::write("write_csv", "file"),
            NamedIoMarker::write("write_parquet", "path"),
            NamedIoMarker::write("write_parquet", "file"),
            NamedIoMarker::write("write_table", "filename"),
            // Serialization, arrays, workbooks, and scientific data stores.
            NamedIoMarker::write("save", "filename"),
            NamedIoMarker::write("save", "file"),
            NamedIoMarker::write("bson", "filename"),
            NamedIoMarker::write("npzwrite", "filename"),
            NamedIoMarker::write("npywrite", "filename"),
            NamedIoMarker::write("matwrite", "file"),
            NamedIoMarker::write("writetable", "filename"),
            NamedIoMarker::write("h5write", "filename"),
            // Structured, spatial, image, and plot writers.
            NamedIoMarker::write("write_file", "filename"),
            NamedIoMarker::write("write_file", "path"),
            NamedIoMarker::write("write_json", "filename"),
            NamedIoMarker::write("write_json", "path"),
            NamedIoMarker::write("savefig", "filename"),
            NamedIoMarker::write("savefig", "fname"),
            NamedIoMarker::write("savefig", "path"),
            // URL and copy helpers.
            NamedIoMarker::write("download", "output"),
            NamedIoMarker::write("download", "path"),
            NamedIoMarker::write("Downloads.download", "output"),
            NamedIoMarker::write("Downloads.download", "path"),
            NamedIoMarker::write("cp", "dst"),
            NamedIoMarker::write("mv", "dst"),
        ],
        _ => return,
    };

    for marker in markers {
        collect_named_marker(language, source, marker, facts);
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

    /// Whether an assignment target receives the resource data.
    captures_target: bool,
}

impl NamedIoMarker {
    fn read(function: &'static str, argument: &'static str) -> Self {
        Self {
            direction: IoDirection::Read,
            function,
            argument,
            captures_target: true,
        }
    }

    fn status_read(function: &'static str, argument: &'static str) -> Self {
        Self {
            direction: IoDirection::Read,
            function,
            argument,
            captures_target: false,
        }
    }

    fn write(function: &'static str, argument: &'static str) -> Self {
        Self {
            direction: IoDirection::Write,
            function,
            argument,
            captures_target: false,
        }
    }
}

fn collect_named_marker(
    language: CodeLanguage,
    source: &str,
    marker: &NamedIoMarker,
    facts: &mut CodeFacts,
) {
    let call_marker = format!("{}(", marker.function);
    for call_index in call_indices(language, source, &call_marker) {
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
        let (target, target_offset) = if marker.captures_target {
            assignment_target_before_call(language, source, call_index)
                .map(|(target, offset)| (Some(target), Some(offset)))
                .unwrap_or((None, None))
        } else {
            (None, None)
        };

        facts.io.insert(IoFact {
            direction: marker.direction,
            path: path.clone(),
            operation_offset: Some(call_index),
            target: target.clone(),
            target_offset,
            value: None,
            value_offset: None,
            function: function_name(marker.function),
            mode: None,
            unresolved_reason: None,
        });

        if let Some(target) = target {
            facts.assignments.insert(target.clone());
            if let Some(target_offset) = target_offset {
                record_definition(facts, &target, target_offset);
            }
            if marker.direction.reads()
                && let IoPath::Static(path) = path
            {
                facts.variable_sources.insert(target, path);
            }
        }
    }
}

fn call_indices(language: CodeLanguage, source: &str, call_marker: &str) -> Vec<usize> {
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

        if source[index..].starts_with(call_marker) && is_call_boundary(language, source, index) {
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

fn is_call_boundary(language: CodeLanguage, source: &str, index: usize) -> bool {
    source[..index]
        .chars()
        .next_back()
        .is_none_or(|char| !is_identifier_continue(language, char))
}

fn is_identifier_continue(language: CodeLanguage, char: char) -> bool {
    char == '_' || char.is_ascii_alphanumeric() || (language == CodeLanguage::R && char == '.')
}

fn assignment_target_before_call(
    language: CodeLanguage,
    source: &str,
    call_index: usize,
) -> Option<(String, usize)> {
    if !matches!(language, CodeLanguage::R | CodeLanguage::Julia) {
        return None;
    }

    let statement_start = statement_start_before_call(source, call_index)?;
    let prefix = source.get(statement_start..call_index)?;
    if !is_top_level_statement_prefix(prefix) {
        return None;
    }

    let trimmed_end = prefix.trim_end().len();
    let prefix = prefix.get(..trimmed_end)?;
    let (target_source, target_end) = if language == CodeLanguage::R
        && let Some(operator_index) = prefix.rfind("<-")
    {
        prefix
            .get(..operator_index)
            .map(|target| (target, operator_index))?
    } else if let Some(operator_index) = prefix.rfind('=') {
        prefix
            .get(..operator_index)
            .map(|target| (target, operator_index))?
    } else {
        return None;
    };

    let target_trimmed = target_source.trim();
    let target = identifier_target(target_trimmed)?;
    let leading = target_source
        .len()
        .saturating_sub(target_source.trim_start().len());
    let target_offset = statement_start + leading;

    (target_end == target_source.len()).then_some((target, target_offset))
}

fn statement_start_before_call(source: &str, call_index: usize) -> Option<usize> {
    let prefix = source.get(..call_index)?;
    prefix
        .char_indices()
        .rev()
        .find_map(|(index, char)| matches!(char, '\n' | ';' | '{' | '}').then_some(index + 1))
        .or(Some(0))
}

fn is_top_level_statement_prefix(source: &str) -> bool {
    let mut depth = 0usize;
    let mut quote = None;
    let mut escaped = false;

    for char in source.chars() {
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
            '(' | '[' => depth += 1,
            ')' | ']' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }

    depth == 0 && quote.is_none()
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
            ',' | ';' if depth == 0 => {
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
