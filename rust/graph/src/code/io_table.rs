//! A declarative table of I/O calls, and the collector that applies it.
//!
//! Detection used to be an exhaustive list of ast-grep patterns: every arity of
//! every namespacing of every function, a dozen lines apiece, and no way to
//! express a keyword argument at all. That last gap is why a separate
//! hand-written text scanner grew alongside the rules as a parallel
//! implementation of the same job.
//!
//! Here one [`IoSignature`] line covers every arity, every namespace
//! qualification, and the keyword form, because the collector walks call nodes
//! and consults the table rather than pattern-matching source shapes. Adding a
//! library is a one-line change.
//!
//! Matching on the parse tree also gives each fact the byte offset of the path
//! *argument* rather than of the enclosing call, which is what lets the
//! resolution pass find the expression it needs to evaluate.

use ast_grep_core::{AstGrep, tree_sitter::StrDoc};

use super::{
    ast::{SourceNode, call_argument_bindings, call_callee, callee_name, is_call_node},
    facts::{CodeFacts, IoDirection, IoFact, IoMode, record_definition},
    language::CodeLanguage,
    util::{
        clean_string_literal, function_name, identifier_target, is_path_wrapper_name,
        path_expression,
    },
};

/// One path-bearing argument of one recognized I/O function.
///
/// A signature names the argument twice — by position and by keyword — so a
/// single entry matches `read_csv(p)`, `pd.read_csv(p, sep=",")`, and
/// `read_csv(filepath_or_buffer=p)` alike.
#[derive(Debug, Clone, Copy)]
pub(super) struct IoSignature {
    /// Terminal callee name, ignoring any module qualification.
    callee: &'static str,

    /// Where the path expression sits in the call.
    source: PathSource,

    /// Direction of the operation.
    direction: IoDirection,

    /// Position of a mode argument, when the API takes one.
    ///
    /// `open(path, "w")` is a write despite living in the read table, so the
    /// mode argument reclassifies it.
    mode: Option<(usize, &'static str)>,

    /// Whether an assignment target receives the data that was read.
    captures_target: bool,

    /// Where the value being written comes from.
    value: ValueSource,

    /// Whether the mode argument must be present for this to count as I/O.
    ///
    /// An R connection is only I/O once it is opened: `file(path, "w")` writes,
    /// but a bare `file(path)` merely describes a connection that may never be
    /// used.
    requires_mode: bool,
}

/// Where the data a write operation stores comes from.
///
/// Knowing this turns a coarse "this script wrote that file" edge into the
/// precise "this variable reached that file", which is what makes lineage
/// through a script useful.
#[derive(Debug, Clone, Copy)]
enum ValueSource {
    /// The call has no identifiable value, or is a read.
    None,

    /// The receiver of a method call, as in `frame.to_csv(path)`.
    Receiver,

    /// A positional argument, as in `writeFileSync(path, data)`.
    Argument(usize),
}

/// Which positional argument carries the path.
#[derive(Debug, Clone, Copy)]
enum Position {
    /// A fixed zero-based index.
    Index(usize),

    /// No positional form; the argument must be named.
    ///
    /// R's `cat(x, y, file = "out")` takes variadic content first, so only the
    /// keyword identifies the path.
    KeywordOnly,

    /// The final positional argument.
    ///
    /// Some APIs are overloaded on arity — `credentials.sign(path)` and
    /// `credentials.sign(figure, path)` both write their last argument — which
    /// one rule expresses and a fixed index cannot.
    Last,
}

/// Where in a call the path expression is found.
#[derive(Debug, Clone, Copy)]
enum PathSource {
    /// An argument, named both by position and by keyword.
    Argument {
        /// Where the path argument sits among the positional arguments.
        position: Position,

        /// Keyword spelling of the path argument, when the API has one.
        keyword: &'static str,
    },

    /// The receiver of a method call, as in `Path(p).read_text()`.
    ///
    /// Only a path-preserving wrapper receiver qualifies, so an unrelated
    /// object that happens to have a `read_text` method is not mistaken for a
    /// filesystem path.
    Receiver,
}

impl IoSignature {
    /// A read whose result is typically assigned.
    const fn read(callee: &'static str, position: usize, keyword: &'static str) -> Self {
        Self {
            callee,
            source: PathSource::Argument {
                position: Position::Index(position),
                keyword,
            },
            direction: IoDirection::Read,
            mode: None,
            captures_target: true,
            value: ValueSource::None,
            requires_mode: false,
        }
    }

    /// A read performed for its effect rather than its value.
    const fn peek(callee: &'static str, position: usize, keyword: &'static str) -> Self {
        Self {
            captures_target: false,
            ..Self::read(callee, position, keyword)
        }
    }

    /// A write.
    const fn write(callee: &'static str, position: usize, keyword: &'static str) -> Self {
        Self {
            callee,
            source: PathSource::Argument {
                position: Position::Index(position),
                keyword,
            },
            direction: IoDirection::Write,
            mode: None,
            captures_target: false,
            value: ValueSource::Receiver,
            requires_mode: false,
        }
    }

    /// A write whose destination is identified only by a keyword argument.
    const fn write_keyword(callee: &'static str, keyword: &'static str) -> Self {
        Self {
            callee,
            source: PathSource::Argument {
                position: Position::KeywordOnly,
                keyword,
            },
            direction: IoDirection::Write,
            mode: None,
            captures_target: false,
            value: ValueSource::None,
            requires_mode: false,
        }
    }

    /// A write whose path is the last positional argument.
    const fn write_last(callee: &'static str, keyword: &'static str) -> Self {
        Self {
            callee,
            source: PathSource::Argument {
                position: Position::Last,
                keyword,
            },
            direction: IoDirection::Write,
            mode: None,
            captures_target: false,
            value: ValueSource::Receiver,
            requires_mode: false,
        }
    }

    /// A read whose path is the wrapped receiver of a method call.
    const fn read_from(callee: &'static str) -> Self {
        Self {
            callee,
            source: PathSource::Receiver,
            direction: IoDirection::Read,
            mode: None,
            captures_target: true,
            value: ValueSource::None,
            requires_mode: false,
        }
    }

    /// A write whose path is the wrapped receiver of a method call.
    const fn write_to(callee: &'static str) -> Self {
        Self {
            callee,
            source: PathSource::Receiver,
            direction: IoDirection::Write,
            mode: None,
            captures_target: false,
            value: ValueSource::None,
            requires_mode: false,
        }
    }

    /// Name the positional argument holding the data being written.
    const fn with_value(self, position: usize) -> Self {
        Self {
            value: ValueSource::Argument(position),
            ..self
        }
    }

    /// Attach a mode argument that can reclassify the direction.
    const fn with_mode(self, position: usize, keyword: &'static str) -> Self {
        Self {
            mode: Some((position, keyword)),
            ..self
        }
    }

    /// Require the mode argument before treating the call as I/O.
    const fn opened(self) -> Self {
        Self {
            requires_mode: true,
            ..self
        }
    }
}

/// I/O signatures recognized in Python source.
const PYTHON: &[IoSignature] = &[
    // Core file handles. The mode argument reclassifies writes and updates.
    IoSignature::read("open", 0, "file").with_mode(1, "mode"),
    IoSignature::read("File", 0, "name").with_mode(1, "mode"),
    IoSignature::read_from("read_text"),
    IoSignature::read_from("read_bytes"),
    IoSignature::write_to("write_text"),
    IoSignature::write_to("write_bytes"),
    IoSignature::read_from("open").with_mode(0, "mode"),
    // Tabular readers.
    IoSignature::read("read_csv", 0, "filepath_or_buffer"),
    IoSignature::read("read_table", 0, "filepath_or_buffer"),
    IoSignature::read("read_excel", 0, "io"),
    IoSignature::read("read_json", 0, "path_or_buf"),
    IoSignature::read("read_html", 0, "io"),
    IoSignature::read("read_parquet", 0, "path"),
    IoSignature::read("read_feather", 0, "path"),
    IoSignature::read("read_pickle", 0, "filepath_or_buffer"),
    IoSignature::read("read_hdf", 0, "path_or_buf"),
    IoSignature::read("read_orc", 0, "path"),
    IoSignature::read("read_sas", 0, "filepath_or_buffer"),
    IoSignature::read("read_stata", 0, "filepath_or_buffer"),
    IoSignature::read("read_fwf", 0, "filepath_or_buffer"),
    IoSignature::read("read_xml", 0, "path_or_buffer"),
    IoSignature::read("read_spss", 0, "path"),
    IoSignature::read("read_file", 0, "filename"),
    IoSignature::read("read_database", 0, "source"),
    IoSignature::read("scan_csv", 0, "source"),
    IoSignature::read("scan_parquet", 0, "source"),
    IoSignature::read("scan_ndjson", 0, "source"),
    IoSignature::read("scan_ipc", 0, "source"),
    // Tabular writers.
    IoSignature::write("to_csv", 0, "path_or_buf"),
    IoSignature::write("to_excel", 0, "excel_writer"),
    IoSignature::write("to_json", 0, "path_or_buf"),
    IoSignature::write("to_html", 0, "buf"),
    IoSignature::write("to_parquet", 0, "path"),
    IoSignature::write("to_feather", 0, "path"),
    IoSignature::write("to_pickle", 0, "path"),
    IoSignature::write("to_hdf", 0, "path_or_buf"),
    IoSignature::write("to_orc", 0, "path"),
    IoSignature::write("to_stata", 0, "path"),
    IoSignature::write("to_xml", 0, "path_or_buffer"),
    IoSignature::write("to_latex", 0, "buf"),
    IoSignature::write("to_markdown", 0, "buf"),
    IoSignature::write("to_file", 0, "filename"),
    IoSignature::write("write_csv", 0, "file"),
    IoSignature::write("write_parquet", 0, "file"),
    IoSignature::write("write_ndjson", 0, "file"),
    IoSignature::write("write_ipc", 0, "file"),
    IoSignature::write("sink_csv", 0, "path"),
    IoSignature::write("sink_parquet", 0, "path"),
    // Arrays, matrices, labeled arrays, and chunked stores.
    IoSignature::read("load", 0, "file"),
    IoSignature::read("loadtxt", 0, "fname"),
    IoSignature::read("genfromtxt", 0, "fname"),
    IoSignature::read("fromfile", 0, "file"),
    IoSignature::read("loadmat", 0, "file_name"),
    IoSignature::read("open_dataset", 0, "filename_or_obj"),
    IoSignature::read("open_dataarray", 0, "filename_or_obj"),
    IoSignature::read("open_mfdataset", 0, "paths"),
    IoSignature::read("open_zarr", 0, "store"),
    IoSignature::write("save", 0, "file"),
    IoSignature::write("savez", 0, "file"),
    IoSignature::write("savez_compressed", 0, "file"),
    IoSignature::write("savetxt", 0, "fname"),
    IoSignature::write("savemat", 0, "file_name"),
    IoSignature::write("save_file", 1, "filename").with_value(0),
    // A qualified entry wins over the bare name: `torch.save(obj, f)` puts the
    // path second, where `numpy.save(file, arr)` puts it first.
    IoSignature::write("torch.save", 1, "f").with_value(0),
    IoSignature::write("tofile", 0, "fid"),
    IoSignature::write("dump", 1, "fp").with_value(0),
    // One line replaces the four hand-written `credentials.sign` permutations.
    IoSignature::write_last("sign", "output").with_value(0),
    IoSignature::write("to_netcdf", 0, "path"),
    IoSignature::write("to_zarr", 0, "store"),
    // Images, plots, and media.
    IoSignature::read("imread", 0, "fname"),
    IoSignature::read("imread_collection", 0, "load_pattern"),
    IoSignature::write("savefig", 0, "fname"),
    IoSignature::write("imwrite", 0, "uri"),
    IoSignature::write("imsave", 0, "fname"),
    // Object stores and remote filesystems address their objects by URI, which
    // the resolution pass treats like any other path.
    IoSignature::read("connect", 0, "database"),
    IoSignature::write("write_table", 1, "where"),
    // URL and download helpers, including httpx and requests.
    IoSignature::peek("get", 0, "url"),
    IoSignature::peek("post", 0, "url"),
    IoSignature::peek("head", 0, "url"),
    IoSignature::peek("stream", 1, "url"),
    IoSignature::peek("urlopen", 0, "url"),
    IoSignature::peek("urlretrieve", 0, "url"),
    IoSignature::write("urlretrieve", 1, "filename"),
    // Copy and move helpers read their source and write their destination.
    IoSignature::peek("copyfile", 0, "src"),
    IoSignature::write("copyfile", 1, "dst"),
    IoSignature::peek("copy", 0, "src"),
    IoSignature::write("copy", 1, "dst"),
    IoSignature::peek("copy2", 0, "src"),
    IoSignature::write("copy2", 1, "dst"),
    IoSignature::peek("copytree", 0, "src"),
    IoSignature::write("copytree", 1, "dst"),
    IoSignature::peek("move", 0, "src"),
    IoSignature::write("move", 1, "dst"),
];

/// I/O signatures recognized in JavaScript and TypeScript source.
const ECMASCRIPT: &[IoSignature] = &[
    IoSignature::read("readFileSync", 0, "path"),
    IoSignature::read("readFile", 0, "path"),
    IoSignature::read("createReadStream", 0, "path"),
    IoSignature::peek("fetch", 0, "url"),
    IoSignature::write("writeFileSync", 0, "file").with_value(1),
    IoSignature::write("writeFile", 0, "file").with_value(1),
    IoSignature::write("appendFileSync", 0, "file").with_value(1),
    IoSignature::write("appendFile", 0, "file").with_value(1),
    IoSignature::write("createWriteStream", 0, "path"),
];

/// I/O signatures recognized in R source.
const R: &[IoSignature] = &[
    // Connection constructors name a resource even when opened as a value; the
    // mode argument reclassifies a write connection.
    IoSignature::read("file", 0, "description")
        .with_mode(1, "open")
        .opened(),
    IoSignature::read("gzfile", 0, "description")
        .with_mode(1, "open")
        .opened(),
    IoSignature::read("bzfile", 0, "description")
        .with_mode(1, "open")
        .opened(),
    IoSignature::read("xzfile", 0, "description")
        .with_mode(1, "open")
        .opened(),
    IoSignature::read("unz", 0, "description")
        .with_mode(1, "open")
        .opened(),
    IoSignature::read("url", 0, "description")
        .with_mode(1, "open")
        .opened(),
    // Base R, tidyverse, data.table, and columnar readers.
    IoSignature::read("read.table", 0, "file"),
    IoSignature::read("read.csv", 0, "file"),
    IoSignature::read("read.csv2", 0, "file"),
    IoSignature::read("read.delim", 0, "file"),
    IoSignature::read("read.delim2", 0, "file"),
    IoSignature::read("read.fwf", 0, "file"),
    IoSignature::read("read_csv", 0, "file"),
    IoSignature::read("read_tsv", 0, "file"),
    IoSignature::read("read_delim", 0, "file"),
    IoSignature::read("read_fwf", 0, "file"),
    IoSignature::read("read_table", 0, "file"),
    IoSignature::read("vroom", 0, "file"),
    IoSignature::read("fread", 0, "input"),
    IoSignature::read("fread", 0, "file"),
    IoSignature::read("readRDS", 0, "file"),
    IoSignature::read("read_rds", 0, "file"),
    IoSignature::read("load", 0, "file"),
    IoSignature::read("readBin", 0, "con"),
    IoSignature::read("readChar", 0, "con"),
    IoSignature::read("readLines", 0, "con"),
    IoSignature::read("scan", 0, "file"),
    IoSignature::read("source", 0, "file"),
    IoSignature::read("dget", 0, "file"),
    IoSignature::read("read_excel", 0, "path"),
    IoSignature::read("read_sav", 0, "file"),
    IoSignature::read("read_dta", 0, "file"),
    IoSignature::read("read_parquet", 0, "file"),
    IoSignature::read("read_feather", 0, "file"),
    IoSignature::read("open_dataset", 0, "sources"),
    // Structured, HTML, spatial, and image readers.
    IoSignature::read("read_json", 0, "path"),
    IoSignature::read("read_xml", 0, "x"),
    IoSignature::read("read_html", 0, "x"),
    IoSignature::read("st_read", 0, "dsn"),
    IoSignature::read("read_sf", 0, "dsn"),
    IoSignature::read("rast", 0, "x"),
    IoSignature::read("vect", 0, "x"),
    IoSignature::read("image_read", 0, "path"),
    IoSignature::read("readPNG", 0, "source"),
    IoSignature::read("readJPEG", 0, "source"),
    IoSignature::read("readTIFF", 0, "source"),
    // Download and copy helpers read their source and write their destination.
    IoSignature::peek("download.file", 0, "url"),
    IoSignature::write("download.file", 1, "destfile"),
    IoSignature::peek("curl_download", 0, "url"),
    IoSignature::write("curl_download", 1, "destfile"),
    IoSignature::peek("file.copy", 0, "from"),
    IoSignature::write("file.copy", 1, "to"),
    IoSignature::peek("file.rename", 0, "from"),
    IoSignature::write("file.rename", 1, "to"),
    // Tabular and serialization writers take their data first.
    IoSignature::write("write.table", 1, "file").with_value(0),
    IoSignature::write("write.csv", 1, "file").with_value(0),
    IoSignature::write("write.csv2", 1, "file").with_value(0),
    IoSignature::write("write", 1, "file").with_value(0),
    IoSignature::write("writeLines", 1, "con").with_value(0),
    IoSignature::write("write_csv", 1, "file").with_value(0),
    IoSignature::write("write_tsv", 1, "file").with_value(0),
    IoSignature::write("write_delim", 1, "file").with_value(0),
    IoSignature::write("saveRDS", 1, "file").with_value(0),
    IoSignature::write("write_rds", 1, "file").with_value(0),
    IoSignature::write("vroom_write", 1, "file").with_value(0),
    IoSignature::write("fwrite", 1, "file").with_value(0),
    IoSignature::write("dump", 1, "file").with_value(0),
    IoSignature::write("dput", 1, "file").with_value(0),
    IoSignature::write("writeBin", 1, "con").with_value(0),
    IoSignature::write("writeChar", 1, "con").with_value(0),
    IoSignature::write("write_xlsx", 1, "path").with_value(0),
    IoSignature::write("write_sav", 1, "path").with_value(0),
    IoSignature::write("write_dta", 1, "path").with_value(0),
    IoSignature::write("write_parquet", 1, "sink").with_value(0),
    IoSignature::write("write_feather", 1, "sink").with_value(0),
    IoSignature::write("write_dataset", 1, "path").with_value(0),
    IoSignature::write("write_json", 1, "path").with_value(0),
    IoSignature::write("write_xml", 1, "file").with_value(0),
    IoSignature::write("st_write", 1, "dsn").with_value(0),
    IoSignature::write("write_sf", 1, "dsn").with_value(0),
    IoSignature::write("writeRaster", 1, "filename").with_value(0),
    IoSignature::write("writeVector", 1, "filename").with_value(0),
    IoSignature::write("image_write", 1, "path").with_value(0),
    IoSignature::write("writePNG", 1, "target").with_value(0),
    IoSignature::write("writeJPEG", 1, "target").with_value(0),
    IoSignature::write("writeTIFF", 1, "target").with_value(0),
    // Variadic writers identify their destination only by name.
    IoSignature::write_keyword("cat", "file"),
    IoSignature::write_keyword("save", "file"),
    // Graphics devices and whole-session writers take the path first.
    IoSignature::write("save.image", 0, "file"),
    IoSignature::write("sink", 0, "file"),
    IoSignature::write("ggsave", 0, "filename").with_value(1),
    IoSignature::write("png", 0, "filename"),
    IoSignature::write("pdf", 0, "file"),
    IoSignature::write("svg", 0, "filename"),
    IoSignature::write("jpeg", 0, "filename"),
    IoSignature::write("tiff", 0, "filename"),
    IoSignature::write("bmp", 0, "filename"),
    IoSignature::write("postscript", 0, "file"),
    IoSignature::write("cairo_pdf", 0, "filename"),
    IoSignature::write("svglite", 0, "file"),
    IoSignature::write("agg_png", 0, "filename"),
];

/// Return the signature table for a language.
fn signatures(language: CodeLanguage) -> &'static [IoSignature] {
    match language {
        CodeLanguage::Python => PYTHON,
        CodeLanguage::R => R,
        CodeLanguage::JavaScript | CodeLanguage::TypeScript => ECMASCRIPT,
        _ => &[],
    }
}

/// Whether table-driven collection has replaced the rules for a language.
pub(super) fn supports_table(language: CodeLanguage) -> bool {
    !signatures(language).is_empty()
}

/// Collect I/O facts by walking call nodes and consulting the table.
pub(super) fn collect_table_io_facts(
    language: CodeLanguage,
    grep: &AstGrep<StrDoc<CodeLanguage>>,
    facts: &mut CodeFacts,
) {
    let table = signatures(language);
    if table.is_empty() {
        return;
    }

    for node in grep.root().dfs() {
        if !is_call_node(&node) {
            continue;
        }
        let Some(callee) = call_callee(&node) else {
            continue;
        };
        let name = callee_name(language, &callee);

        // A qualified spelling is more specific than a bare name, so when one
        // matches it displaces the bare entries rather than adding to them.
        let qualified = table
            .iter()
            .filter(|entry| is_qualified_signature(language, entry.callee))
            .filter(|entry| entry.callee == callee)
            .collect::<Vec<_>>();
        let matching = if qualified.is_empty() {
            table
                .iter()
                .filter(|entry| !is_qualified_signature(language, entry.callee))
                .filter(|entry| entry.callee == name)
                .collect::<Vec<_>>()
        } else {
            qualified
        };

        for signature in matching {
            collect_signature(language, &node, &callee, signature, facts);
        }
    }
}

/// Whether a table entry names a fully qualified callee.
///
/// Dots qualify Python and ECMAScript calls, but are ordinary characters in R
/// identifiers such as `read.csv`; R uses `::` for namespace qualification.
fn is_qualified_signature(language: CodeLanguage, callee: &str) -> bool {
    match language {
        CodeLanguage::R => callee.contains("::"),
        _ => callee.contains('.'),
    }
}

/// Record the I/O fact for one signature matching one call node.
fn collect_signature(
    language: CodeLanguage,
    node: &SourceNode<'_>,
    callee: &str,
    signature: &IoSignature,
    facts: &mut CodeFacts,
) {
    // `Path(path).open(mode)` shares its terminal name with built-in `open`,
    // but its first argument is the mode rather than the path. Once a
    // path-preserving receiver has been identified, only the receiver signature
    // is applicable.
    if language == CodeLanguage::Python
        && matches!(signature.source, PathSource::Argument { .. })
        && wrapper_receiver(node).is_some()
    {
        return;
    }

    let Some(arguments) = call_argument_bindings(language, node) else {
        return;
    };
    let Some(path_node) = (match signature.source {
        PathSource::Argument { position, keyword } => {
            let index = match position {
                Position::Index(index) => index,
                Position::Last => arguments.positional.len().saturating_sub(1),
                Position::KeywordOnly => usize::MAX,
            };
            arguments.for_parameter(index, keyword)
        }
        PathSource::Receiver => wrapper_receiver(node),
    }) else {
        return;
    };

    let path = path_expression(path_node.text().trim());
    if path.value().is_empty() {
        return;
    }

    let mode = signature
        .mode
        .and_then(|(position, keyword)| arguments.for_parameter(position, keyword))
        .and_then(|node| clean_string_literal(node.text().trim()))
        .map(|mode| io_mode(&mode));
    if signature.requires_mode && mode.is_none() {
        return;
    }

    let direction = match (signature.direction, mode) {
        (_, Some(IoMode::ReadWrite)) => IoDirection::ReadWrite,
        (IoDirection::Read, Some(IoMode::Write | IoMode::Append)) => IoDirection::Write,
        (direction, _) => direction,
    };

    let (target, target_offset) = if signature.captures_target {
        assignment_target(node)
            .map(|target| (Some(target.0), Some(target.1)))
            .unwrap_or((None, None))
    } else {
        (None, None)
    };

    let value = match signature.value {
        ValueSource::None => None,
        ValueSource::Receiver => receiver_name(node),
        ValueSource::Argument(position) => arguments
            .positional
            .get(position)
            .and_then(|node| identifier_target(node.text().trim())),
    };

    facts.io.insert(IoFact {
        direction,
        path,
        operation_offset: Some(path_node.range().start),
        target: target.clone(),
        target_offset,
        value,
        value_offset: None,
        function: function_name(callee_name(language, callee)),
        mode,
        unresolved_reason: None,
    });

    if let (Some(target), Some(offset)) = (target, target_offset) {
        facts.assignments.insert(target.clone());
        record_definition(facts, &target, offset);
    }
}

/// Return the receiver of a method call when it wraps a path.
///
/// `Path("data.csv").read_text()` addresses a file; `buffer.read_text()` on an
/// arbitrary object does not, so only a path-preserving wrapper call qualifies.
fn wrapper_receiver<'r>(node: &SourceNode<'r>) -> Option<SourceNode<'r>> {
    let callee = node.field("function")?;
    if !matches!(callee.kind().as_ref(), "attribute" | "member_expression") {
        return None;
    }
    let object = callee.field("object")?;
    if !is_call_node(&object) {
        return None;
    }
    let name = call_callee(&object)?;
    if !is_path_wrapper_name(callee_name(CodeLanguage::Python, &name)) {
        return None;
    }
    call_argument_bindings(CodeLanguage::Python, &object)?
        .positional
        .first()
        .cloned()
}

/// Return the receiver variable of a method call, when it is a plain name.
fn receiver_name(node: &SourceNode<'_>) -> Option<String> {
    let callee = node.field("function")?;
    if !matches!(callee.kind().as_ref(), "attribute" | "member_expression") {
        return None;
    }
    let object = callee.field("object")?;
    (object.kind() == "identifier").then(|| object.text().trim().to_string())?;
    identifier_target(object.text().trim())
}

/// Return the variable an enclosing assignment binds the call's result to.
/// Only the call's own result counts. `text = open(path).read()` binds the file
/// *contents*, not the handle, and attributing the read to `text` would assert
/// a lineage hop this pass has not proven.
fn assignment_target(node: &SourceNode<'_>) -> Option<(String, usize)> {
    let mut current = node.clone();
    while let Some(parent) = current.parent() {
        match parent.kind().as_ref() {
            "assignment" | "variable_declarator" => {
                let value = parent.field("right").or_else(|| parent.field("value"))?;
                if value.range() != current.range() {
                    return None;
                }
                let left = parent.field("left").or_else(|| parent.field("name"))?;
                let target = identifier_target(left.text().trim())?;
                return Some((target, left.range().start));
            }
            // R assigns with `<-`, which the grammar models as an operator.
            "binary_operator" => {
                let children = parent.children().collect::<Vec<_>>();
                let operator = children
                    .iter()
                    .position(|child| matches!(child.kind().as_ref(), "<-" | "<<-" | "="))?;
                let value = children.get(operator + 1)?;
                if value.range() != current.range() {
                    return None;
                }
                let left = children.get(operator.checked_sub(1)?)?;
                let target = identifier_target(left.text().trim())?;
                return Some((target, left.range().start));
            }
            "parenthesized_expression" => current = parent,
            _ => return None,
        }
    }
    None
}

/// Classify a file mode string.
fn io_mode(mode: &str) -> IoMode {
    if mode.contains('+') {
        IoMode::ReadWrite
    } else if mode.contains('a') {
        IoMode::Append
    } else if mode.contains(['w', 'x']) {
        IoMode::Write
    } else {
        IoMode::Read
    }
}
