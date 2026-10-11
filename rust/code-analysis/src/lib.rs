//! Deterministic, single-source static analysis for native and WebAssembly callers.
//!
//! See the crate README for the versioned JSON contract and wasm build environment.

mod analyze;
mod ast;
mod diagnostics;
mod facts;
mod io_table;
mod language;
mod normalize;
mod package;
mod reference;
mod resolve;
mod scan;
mod util;

// Rust's bare wasm allocator does not call libc. WASI uses a different libc and
// must not override its allocator with these symbols.
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
#[allow(unsafe_code)]
mod wasm_allocator;

use serde::{Deserialize, Serialize};

pub use analyze::analyze_source;
pub use diagnostics::{
    AnalysisDiagnostic, SourcePosition, SourceSpan, UnresolvedIoDiagnostic, UnresolvedIoReason,
};
pub use facts::{
    CodeFacts, ColumnFact, FunctionFact, IoDirection, IoFact, IoMode, IoPath, TemplatePath,
    VariableFlowFact, WorkflowScriptExecutionFact, WorkflowScriptExecutionKind, WorkflowUnitFacts,
};
pub use language::CodeLanguage;
pub use package::PackageFact;

/// Pure helpers shared with graph projection.
#[doc(hidden)]
pub mod integration {
    pub use crate::diagnostics::{source_position, unresolved_io_diagnostics};
    pub use crate::reference::has_non_local_uri_scheme;
    pub use crate::util::{direct_source_script, is_static_literal, path_name};
}

/// Version of the public JSON representation. Field/enum changes are breaking.
pub const FORMAT_VERSION: u32 = 1;

/// Static identity of the compiled analyzer, independent of host paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AnalyzerIdentity {
    /// Cargo package version.
    pub crate_version: &'static str,
    /// SHA-256 of rules, analysis sources, compatibility code, and build inputs.
    pub source_digest: &'static str,
    /// Exact grammar dependencies used by this build.
    pub grammars: &'static [GrammarVersion],
}

/// Pinned grammar version, including a Git revision where applicable.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GrammarVersion {
    /// Cargo package name.
    pub package: &'static str,
    /// Exact registry version, or the Git package's declared version.
    pub version: &'static str,
    /// Full Git revision; null for registry packages.
    pub git_commit: Option<&'static str>,
}

include!(concat!(env!("OUT_DIR"), "/identity.rs"));

/// Identity recorded by consumers alongside analysis results.
pub const ANALYZER_IDENTITY: AnalyzerIdentity = AnalyzerIdentity {
    crate_version: env!("CARGO_PKG_VERSION"),
    source_digest: env!("STENCILA_CODE_ANALYSIS_DIGEST"),
    grammars: GRAMMARS,
};

/// Versioned analysis of a source unit. No source is executed or read from disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Analysis {
    /// Public JSON format version.
    pub format_version: u32,
    /// Selected language.
    pub language: CodeLanguage,
    /// Caller-provided logical path, preserved verbatim.
    pub path: String,
    /// Public facts, excluding analyzer bookkeeping.
    pub facts: CodeFacts,
    /// Syntax and unresolved I/O diagnostics, in deterministic order.
    pub diagnostics: Vec<AnalysisDiagnostic>,
}

/// Analyze one source unit without filesystem, network, or asynchronous work.
///
/// Resolution is local to this unit; imported sibling files are not analyzed.
pub fn analyze_file(language: CodeLanguage, logical_path: &str, source: &str) -> Analysis {
    let facts = analyze_source(language, source);
    let diagnostics = diagnostics::collect_diagnostics(&facts, Some(source));
    Analysis {
        format_version: FORMAT_VERSION,
        language,
        path: logical_path.to_string(),
        facts,
        diagnostics,
    }
}
