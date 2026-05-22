use std::{collections::BTreeSet, path::Path};

use crate::package::PackageFact;
use crate::reference::has_non_local_uri_scheme;

use super::language::CodeLanguage;

/// Return the first static string literal in a source fragment.
pub(super) fn first_static_string_literal(source: &str) -> Option<String> {
    let bytes = source.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if matches!(bytes[index], b'\'' | b'"') {
            let quote = bytes[index];
            let start = index + 1;
            let relative_end = bytes[start..].iter().position(|byte| *byte == quote)?;
            let end = start + relative_end;
            if let Ok(value) = std::str::from_utf8(&bytes[start..end])
                && is_static_literal(value)
            {
                return Some(value.to_string());
            }
            index = end + 1;
        } else {
            index += 1;
        }
    }
    None
}

/// Collect static string literals from a small source segment.
///
/// Several fallback scanners need only literal values, not a full AST walk. This
/// helper is intentionally simple and only accepts quoted strings that also pass
/// the static-literal guard used for resource paths.
pub(super) fn collect_string_literals(source: &str, values: &mut BTreeSet<String>) {
    let bytes = source.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if matches!(bytes[index], b'\'' | b'"') {
            let quote = bytes[index];
            let start = index + 1;
            let Some(relative_end) = bytes[start..].iter().position(|byte| *byte == quote) else {
                break;
            };
            let end = start + relative_end;
            if let Ok(value) = std::str::from_utf8(&bytes[start..end])
                && is_static_literal(value)
            {
                values.insert(value.to_string());
            }
            index = end + 1;
        } else {
            index += 1;
        }
    }
}

/// Convert a captured source snippet into a static string literal.
///
/// Captures often include prefixes, quotes, or surrounding syntax. This helper
/// accepts ordinary quoted strings and rejects formatted strings because their
/// runtime value cannot be known statically.
pub(super) fn clean_string_literal(raw: &str) -> Option<String> {
    let mut value = raw.trim();
    let quote_index = value.find(['\'', '"'])?;
    let prefix = &value[..quote_index];
    if prefix.chars().any(|char| matches!(char, 'f' | 'F')) {
        return None;
    }
    value = &value[quote_index..];
    let quote = value.chars().next()?;
    if !matches!(quote, '\'' | '"') {
        return None;
    }

    let quote_len = quote.len_utf8();
    let triple = value
        .as_bytes()
        .get(..3)
        .is_some_and(|bytes| bytes.iter().all(|byte| *byte == quote as u8));
    let delimiter_len = if triple { quote_len * 3 } else { quote_len };
    if value.len() < delimiter_len * 2 {
        return None;
    }
    let end_delimiter = &value[value.len() - delimiter_len..];
    if !end_delimiter.chars().all(|char| char == quote) {
        return None;
    }

    let literal = &value[delimiter_len..value.len() - delimiter_len];
    is_static_literal(literal).then(|| literal.to_string())
}

/// Check whether a literal is safe to represent as a concrete graph resource.
///
/// The guard rejects empty strings and common interpolation markers. It is a
/// conservative filter: missing a dynamic dependency is preferable to asserting
/// a path that might not exist at runtime.
pub(super) fn is_static_literal(value: &str) -> bool {
    !value.trim().is_empty()
        && !value.contains(['{', '}'])
        && !value.contains("$(")
        && !value.contains('`')
}

/// Extract a package root from an import-like capture.
///
/// The graph records packages at package-root granularity, so members, aliases,
/// namespace qualifiers, and submodules are stripped before validation.
pub(super) fn package_name(raw: &str) -> Option<String> {
    let clean = clean_string_literal(raw).unwrap_or_else(|| raw.trim().to_string());
    let before_alias = clean.split(" as ").next().unwrap_or(&clean).trim();
    let before_member = before_alias
        .split("::")
        .next()
        .unwrap_or(before_alias)
        .trim();
    let root = before_member
        .split('.')
        .next()
        .unwrap_or(before_member)
        .trim();
    is_identifier_like(root).then(|| root.to_string())
}

/// Extract an npm package root from a JavaScript module specifier.
///
/// Relative imports are intentionally ignored because they name source files,
/// not installable packages. Bare and scoped package names are reduced to their
/// package root so `@scope/pkg/subpath` is represented as `@scope/pkg`.
pub(super) fn javascript_package_name(raw: &str) -> Option<PackageFact> {
    let clean = clean_string_literal(raw).unwrap_or_else(|| raw.trim().to_string());
    if let Some(module) = clean.strip_prefix("node:") {
        return is_javascript_package_root(module).then(|| PackageFact::new("node", module));
    }

    let clean = clean.as_str();
    if clean.starts_with(['.', '/', '#'])
        || clean.starts_with("//")
        || has_non_local_uri_scheme(clean)
    {
        return None;
    }

    let mut parts = clean.split('/');
    let first = parts.next()?.trim();
    let root = if first.starts_with('@') {
        let second = parts.next()?.trim();
        format!("{first}/{second}")
    } else {
        first.to_string()
    };

    if !is_javascript_package_root(&root) {
        return None;
    }

    let ecosystem = if is_node_builtin(&root) {
        "node"
    } else {
        "npm"
    };
    Some(PackageFact::new(ecosystem, root))
}

/// Validate a JavaScript package root.
///
/// npm package names allow characters that are not Rust/Python identifiers, so
/// they need a separate conservative predicate from symbol names.
fn is_javascript_package_root(value: &str) -> bool {
    if value.is_empty() || value.ends_with('/') || value.contains("//") {
        return false;
    }
    value
        .chars()
        .all(|char| char.is_ascii_alphanumeric() || matches!(char, '@' | '/' | '-' | '_' | '.'))
}

fn is_node_builtin(value: &str) -> bool {
    matches!(
        value,
        "assert"
            | "async_hooks"
            | "buffer"
            | "child_process"
            | "cluster"
            | "console"
            | "crypto"
            | "dgram"
            | "diagnostics_channel"
            | "dns"
            | "domain"
            | "events"
            | "fs"
            | "http"
            | "http2"
            | "https"
            | "inspector"
            | "module"
            | "net"
            | "os"
            | "path"
            | "perf_hooks"
            | "process"
            | "punycode"
            | "querystring"
            | "readline"
            | "repl"
            | "sea"
            | "stream"
            | "string_decoder"
            | "test"
            | "timers"
            | "tls"
            | "trace_events"
            | "tty"
            | "url"
            | "util"
            | "v8"
            | "vm"
            | "wasi"
            | "worker_threads"
            | "zlib"
    )
}

/// Identify Python standard-library modules.
///
/// Python imports become package facts only when they name an external
/// distribution. Standard-library modules ship with the interpreter, so
/// recording them as PyPI packages would create misleading joins with
/// manifest-declared dependencies — most acutely for names like `pathlib`
/// that also exist as deprecated PyPI projects.
pub(super) fn is_python_stdlib(value: &str) -> bool {
    matches!(
        value,
        "__future__"
            | "_thread"
            | "abc"
            | "argparse"
            | "array"
            | "ast"
            | "asynchat"
            | "asyncio"
            | "asyncore"
            | "atexit"
            | "audioop"
            | "base64"
            | "bdb"
            | "binascii"
            | "bisect"
            | "builtins"
            | "bz2"
            | "calendar"
            | "cgi"
            | "cgitb"
            | "chunk"
            | "cmath"
            | "cmd"
            | "code"
            | "codecs"
            | "codeop"
            | "collections"
            | "colorsys"
            | "compileall"
            | "concurrent"
            | "configparser"
            | "contextlib"
            | "contextvars"
            | "copy"
            | "copyreg"
            | "crypt"
            | "csv"
            | "ctypes"
            | "curses"
            | "dataclasses"
            | "datetime"
            | "dbm"
            | "decimal"
            | "difflib"
            | "dis"
            | "distutils"
            | "doctest"
            | "email"
            | "encodings"
            | "ensurepip"
            | "enum"
            | "errno"
            | "faulthandler"
            | "fcntl"
            | "filecmp"
            | "fileinput"
            | "fnmatch"
            | "fractions"
            | "ftplib"
            | "functools"
            | "gc"
            | "getopt"
            | "getpass"
            | "gettext"
            | "glob"
            | "graphlib"
            | "grp"
            | "gzip"
            | "hashlib"
            | "heapq"
            | "hmac"
            | "html"
            | "http"
            | "imaplib"
            | "imghdr"
            | "imp"
            | "importlib"
            | "inspect"
            | "io"
            | "ipaddress"
            | "itertools"
            | "json"
            | "keyword"
            | "lib2to3"
            | "linecache"
            | "locale"
            | "logging"
            | "lzma"
            | "mailbox"
            | "mailcap"
            | "marshal"
            | "math"
            | "mimetypes"
            | "mmap"
            | "modulefinder"
            | "msilib"
            | "msvcrt"
            | "multiprocessing"
            | "netrc"
            | "nis"
            | "nntplib"
            | "ntpath"
            | "numbers"
            | "operator"
            | "optparse"
            | "os"
            | "ossaudiodev"
            | "pathlib"
            | "pdb"
            | "pickle"
            | "pickletools"
            | "pipes"
            | "pkgutil"
            | "platform"
            | "plistlib"
            | "poplib"
            | "posix"
            | "posixpath"
            | "pprint"
            | "profile"
            | "pstats"
            | "pty"
            | "pwd"
            | "py_compile"
            | "pyclbr"
            | "pydoc"
            | "queue"
            | "quopri"
            | "random"
            | "re"
            | "readline"
            | "reprlib"
            | "resource"
            | "rlcompleter"
            | "runpy"
            | "sched"
            | "secrets"
            | "select"
            | "selectors"
            | "shelve"
            | "shlex"
            | "shutil"
            | "signal"
            | "site"
            | "smtpd"
            | "smtplib"
            | "sndhdr"
            | "socket"
            | "socketserver"
            | "spwd"
            | "sqlite3"
            | "ssl"
            | "stat"
            | "statistics"
            | "string"
            | "stringprep"
            | "struct"
            | "subprocess"
            | "sunau"
            | "symtable"
            | "sys"
            | "sysconfig"
            | "syslog"
            | "tabnanny"
            | "tarfile"
            | "telnetlib"
            | "tempfile"
            | "termios"
            | "textwrap"
            | "threading"
            | "time"
            | "timeit"
            | "tkinter"
            | "token"
            | "tokenize"
            | "tomllib"
            | "trace"
            | "traceback"
            | "tracemalloc"
            | "tty"
            | "turtle"
            | "types"
            | "typing"
            | "unicodedata"
            | "unittest"
            | "urllib"
            | "uu"
            | "uuid"
            | "venv"
            | "warnings"
            | "wave"
            | "weakref"
            | "webbrowser"
            | "winreg"
            | "winsound"
            | "wsgiref"
            | "xdrlib"
            | "xml"
            | "xmlrpc"
            | "zipapp"
            | "zipfile"
            | "zipimport"
            | "zlib"
            | "zoneinfo"
    )
}

/// Identify R base distribution packages.
///
/// These packages ship with R itself and are not on CRAN, so importing them
/// should not create graph edges that join to a `cran/<name>` declared
/// dependency.
pub(super) fn is_r_base_package(value: &str) -> bool {
    matches!(
        value,
        "base"
            | "compiler"
            | "datasets"
            | "graphics"
            | "grDevices"
            | "grid"
            | "methods"
            | "parallel"
            | "splines"
            | "stats"
            | "stats4"
            | "tcltk"
            | "tools"
            | "translations"
            | "utils"
    )
}

/// Extract a Cargo-style package root from a Rust import path.
///
/// Standard library and local module roots are skipped because they are not
/// external software packages in the workspace graph.
pub(super) fn rust_package_name(raw: &str) -> Option<String> {
    let root = raw.split("::").next()?.trim();
    if matches!(root, "std" | "core" | "alloc" | "crate" | "self" | "super") {
        return None;
    }
    is_identifier_like(root).then(|| root.to_string())
}

/// Extract a local name introduced by a Rust import path.
///
/// Recording this suppresses false symbol-use dependencies from imported names
/// without trying to model every nested brace import form.
pub(super) fn rust_imported_symbol(raw: &str) -> Option<String> {
    let before_alias = raw.split(" as ").next().unwrap_or(raw).trim();
    let symbol = before_alias
        .rsplit("::")
        .find(|part| !part.is_empty())
        .unwrap_or(before_alias)
        .trim();
    is_identifier_like(symbol).then(|| symbol.to_string())
}

/// Validate an assignment or declaration target as a simple identifier.
///
/// Complex patterns such as destructuring are skipped in this first pass
/// because graph symbol nodes should represent concrete names.
pub(super) fn identifier_target(raw: &str) -> Option<String> {
    let target = raw.trim();
    is_identifier_like(target).then(|| target.to_string())
}

/// Extract the terminal callable name from a function-like capture.
///
/// Calls such as `pkg.fn()` or `pkg::fn()` are reduced to `fn` for the callable
/// node while package relationships are handled by import-specific rules.
pub(super) fn function_name(raw: &str) -> Option<String> {
    let clean = raw.trim();
    let name = clean
        .rsplit(['.', ':'])
        .find(|part| !part.is_empty())
        .unwrap_or(clean)
        .trim();
    is_identifier_like(name).then(|| name.to_string())
}

/// Return the first identifier-like prefix as an owned string.
///
/// This small adapter keeps import alias handling ergonomic when facts need to
/// store the result beyond the source snippet's lifetime.
pub(super) fn first_identifier_owned(raw: &str) -> Option<String> {
    first_identifier(raw).map(ToString::to_string)
}

/// Return the first identifier-like prefix from a string.
///
/// Fallback scanners use this for simple cases such as R `$` column access,
/// where the next token is enough to identify a static name.
pub(super) fn first_identifier(raw: &str) -> Option<&str> {
    let raw = raw.trim_start();
    let end = raw
        .char_indices()
        .find_map(|(index, char)| (!is_identifier_continue(char)).then_some(index))
        .unwrap_or(raw.len());
    let candidate = &raw[..end];
    is_identifier_like(candidate).then_some(candidate)
}

/// Check whether source text contains a complete identifier token.
pub(super) fn contains_identifier(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(index, _)| {
        let before = source[..index].chars().next_back();
        let after = source[(index + name.len())..].chars().next();
        !before.is_some_and(is_identifier_continue) && !after.is_some_and(is_identifier_continue)
    })
}

/// Check whether text is a simple identifier-like name.
///
/// The predicate is shared by packages, symbols, and callables so every graph id
/// derived from source text passes the same conservative validation.
pub(super) fn is_identifier_like(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first == '_' || first.is_alphabetic()) && chars.all(is_identifier_continue)
}

/// Check whether a character can continue an identifier-like name.
///
/// Dots are allowed so R names such as `data.frame` can be recognized where the
/// language treats them as ordinary symbols.
fn is_identifier_continue(char: char) -> bool {
    char == '_' || char == '.' || char.is_alphanumeric()
}

/// Check whether an identifier should be ignored as a dependency signal.
///
/// The ignore list removes short throwaway names and common builtins that would
/// otherwise create noisy variable nodes or false document dependencies. It is
/// deliberately small so authored symbols are not hidden unexpectedly.
pub(super) fn is_ignored_identifier(language: CodeLanguage, name: &str) -> bool {
    if name.len() <= 1 && name != "x" && name != "y" {
        return true;
    }
    match language {
        CodeLanguage::JavaScript | CodeLanguage::TypeScript | CodeLanguage::Nextflow => matches!(
            name,
            "Array"
                | "Boolean"
                | "Date"
                | "Infinity"
                | "JSON"
                | "Math"
                | "NaN"
                | "Number"
                | "Object"
                | "Promise"
                | "String"
                | "console"
                | "exports"
                | "fetch"
                | "module"
                | "process"
                | "require"
                | "setInterval"
                | "setTimeout"
                | "undefined"
        ),
        CodeLanguage::Rust => matches!(
            name,
            "Box"
                | "Clone"
                | "Debug"
                | "Default"
                | "Err"
                | "None"
                | "Ok"
                | "Option"
                | "Result"
                | "Self"
                | "Some"
                | "String"
                | "Vec"
                | "alloc"
                | "core"
                | "crate"
                | "self"
                | "std"
                | "super"
        ),
        CodeLanguage::Python | CodeLanguage::Snakemake => matches!(
            name,
            "False"
                | "None"
                | "True"
                | "abs"
                | "all"
                | "any"
                | "dict"
                | "float"
                | "int"
                | "len"
                | "list"
                | "max"
                | "min"
                | "open"
                | "print"
                | "range"
                | "set"
                | "str"
                | "sum"
                | "tuple"
        ),
        CodeLanguage::R => matches!(
            name,
            "FALSE" | "NULL" | "TRUE" | "c" | "data.frame" | "library" | "require"
        ),
        CodeLanguage::Julia => matches!(
            name,
            "DataFrame"
                | "Dict"
                | "false"
                | "length"
                | "missing"
                | "nothing"
                | "open"
                | "print"
                | "println"
                | "read"
                | "true"
                | "using"
                | "write"
        ),
        CodeLanguage::Matlab => matches!(
            name,
            "NaN" | "Inf" | "false" | "true" | "function" | "import" | "end" | "pi"
        ),
    }
}

/// Return the display name for a path-like string.
///
/// Schema file and code nodes need a short name for display while ids and path
/// fields keep the full scoped path.
pub(super) fn path_name(path: &str) -> String {
    Path::new(path)
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or(path)
        .to_string()
}
