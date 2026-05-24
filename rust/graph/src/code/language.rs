use std::{borrow::Cow, fmt, path::Path};

use ast_grep_core::{
    Language,
    matcher::{Pattern, PatternBuilder, PatternError},
    tree_sitter::{LanguageExt, StrDoc, TSLanguage},
};
use serde::Deserialize;
use stencila_codecs::Format;

/// Embedded ast-grep rules for Python source.
///
/// Rules live as YAML files so extraction patterns are easy to review and grow
/// without changing the Rust control flow. They are embedded at compile time to
/// keep graph construction deterministic and independent of runtime config.
const PYTHON_RULES: &str = concat!(
    include_str!("../../rules/python/import.yml"),
    "\n---\n",
    include_str!("../../rules/python/assignment.yml"),
    "\n---\n",
    include_str!("../../rules/python/function.yml"),
    "\n---\n",
    include_str!("../../rules/python/read.yml"),
    "\n---\n",
    include_str!("../../rules/python/write.yml"),
    "\n---\n",
    include_str!("../../rules/python/call.yml"),
);

/// Embedded ast-grep rules for R source.
///
/// The R rules cover package imports, assignments, common IO calls, and calls
/// that can be represented as graph facts without evaluating R code.
const R_RULES: &str = concat!(
    include_str!("../../rules/r/import.yml"),
    "\n---\n",
    include_str!("../../rules/r/assignment.yml"),
    "\n---\n",
    include_str!("../../rules/r/read.yml"),
    "\n---\n",
    include_str!("../../rules/r/write.yml"),
    "\n---\n",
    include_str!("../../rules/r/call.yml"),
);

/// Embedded ast-grep rules for Julia source.
///
/// Julia rules cover package imports, assignments, function declarations,
/// common CSV/file reads and writes, and calls that can be represented as graph
/// facts without evaluating Julia code.
const JULIA_RULES: &str = concat!(
    include_str!("../../rules/julia/import.yml"),
    "\n---\n",
    include_str!("../../rules/julia/assignment.yml"),
    "\n---\n",
    include_str!("../../rules/julia/function.yml"),
    "\n---\n",
    include_str!("../../rules/julia/read.yml"),
    "\n---\n",
    include_str!("../../rules/julia/write.yml"),
    "\n---\n",
    include_str!("../../rules/julia/call.yml"),
);

/// Embedded ast-grep rules for MATLAB source.
///
/// MATLAB rules cover package imports, assignments, function declarations,
/// common table/file reads and writes, and calls that can be represented as
/// graph facts without evaluating MATLAB code.
const MATLAB_RULES: &str = concat!(
    include_str!("../../rules/matlab/import.yml"),
    "\n---\n",
    include_str!("../../rules/matlab/assignment.yml"),
    "\n---\n",
    include_str!("../../rules/matlab/function.yml"),
    "\n---\n",
    include_str!("../../rules/matlab/read.yml"),
    "\n---\n",
    include_str!("../../rules/matlab/write.yml"),
    "\n---\n",
    include_str!("../../rules/matlab/call.yml"),
);

/// Embedded ast-grep rules for Snakemake source.
///
/// Snakemake still receives a small text fallback later in this module because
/// workflow directives are easier to normalize from source text than from every
/// grammar node shape exposed by tree-sitter.
const SNAKEMAKE_RULES: &str = concat!(
    include_str!("../../rules/snakemake/rule.yml"),
    "\n---\n",
    include_str!("../../rules/snakemake/directive.yml"),
);

macro_rules! ecmascript_rules {
    ($language:literal) => {
        concat!(
            "id: ecmascript-import\nlanguage: ",
            $language,
            "\n",
            include_str!("../../rules/ecmascript/import.yml"),
            "\n---\n",
            "id: ecmascript-assignment\nlanguage: ",
            $language,
            "\n",
            include_str!("../../rules/ecmascript/assignment.yml"),
            "\n---\n",
            "id: ecmascript-function\nlanguage: ",
            $language,
            "\n",
            include_str!("../../rules/ecmascript/function.yml"),
            "\n---\n",
            "id: ecmascript-read\nlanguage: ",
            $language,
            "\n",
            include_str!("../../rules/ecmascript/read.yml"),
            "\n---\n",
            "id: ecmascript-write\nlanguage: ",
            $language,
            "\n",
            include_str!("../../rules/ecmascript/write.yml"),
            "\n---\n",
            "id: ecmascript-call\nlanguage: ",
            $language,
            "\n",
            include_str!("../../rules/ecmascript/call.yml"),
        )
    };
}

/// Embedded ast-grep rules for JavaScript source.
///
/// JavaScript and TypeScript share the same runtime-oriented extraction rules.
const JAVASCRIPT_RULES: &str = ecmascript_rules!("JavaScript");

/// Embedded ast-grep rules for TypeScript source.
///
/// TypeScript uses the same runtime-oriented extraction rules as JavaScript.
const TYPESCRIPT_RULES: &str = ecmascript_rules!("TypeScript");

/// Embedded ast-grep rules for Rust source.
///
/// Rust rules cover `use` items, local assignments, function declarations, and
/// common `std::fs`, `File`, and CSV-style path calls.
const RUST_RULES: &str = concat!(
    include_str!("../../rules/rust/import.yml"),
    "\n---\n",
    include_str!("../../rules/rust/assignment.yml"),
    "\n---\n",
    include_str!("../../rules/rust/function.yml"),
    "\n---\n",
    include_str!("../../rules/rust/read.yml"),
    "\n---\n",
    include_str!("../../rules/rust/write.yml"),
    "\n---\n",
    include_str!("../../rules/rust/call.yml"),
);

/// Programming languages supported by static code graph extraction.
///
/// This enum is both the public language selector for callers and the ast-grep
/// language adapter. Keeping those roles together ensures file detection, rule
/// selection, tree-sitter parsing, and graph id labels cannot drift apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub enum CodeLanguage {
    /// JavaScript source.
    ///
    /// JavaScript is analyzed through tree-sitter-javascript and covers module
    /// imports, CommonJS requires, assignments, calls, and common file IO.
    #[serde(alias = "javascript", alias = "js", alias = "JavaScript", alias = "JS")]
    JavaScript,

    /// TypeScript source.
    ///
    /// TypeScript uses the TypeScript tree-sitter grammar and shares JavaScript
    /// graph semantics, with type-only structure ignored in this static pass.
    #[serde(alias = "typescript", alias = "ts", alias = "TypeScript", alias = "TS")]
    TypeScript,

    /// Rust source.
    ///
    /// Rust is analyzed through tree-sitter-rust and focuses on `use` items,
    /// definitions, calls, and static filesystem path literals.
    #[serde(alias = "rust", alias = "rs", alias = "Rust", alias = "RUST")]
    Rust,

    /// Python source.
    ///
    /// Python is analyzed through tree-sitter-python and covers imports,
    /// assignments, function calls, static file IO, and simple dataframe column
    /// access patterns common in notebooks and scripts.
    #[serde(alias = "python", alias = "py", alias = "Python", alias = "PYTHON")]
    Python,

    /// R source.
    ///
    /// R is analyzed through tree-sitter-r and focuses on package imports,
    /// assignment forms, common read/write calls, and simple dataframe column
    /// access.
    #[serde(alias = "r", alias = "R")]
    R,

    /// Julia source.
    ///
    /// Julia is analyzed through tree-sitter-julia and focuses on package
    /// imports, assignments, function declarations, calls, and static file IO.
    #[serde(alias = "julia", alias = "jl", alias = "Julia", alias = "JULIA")]
    Julia,

    /// MATLAB source.
    ///
    /// MATLAB is analyzed through tree-sitter-matlab and focuses on imports,
    /// assignments, function declarations, calls, and static file IO.
    #[serde(alias = "matlab", alias = "m", alias = "Matlab", alias = "MATLAB")]
    Matlab,

    /// Snakemake workflow source.
    ///
    /// Snakemake is treated as a workflow language rather than a general Python
    /// dialect. The graph projection emphasizes rules, declared inputs and
    /// outputs, script links, and shell blocks.
    #[serde(
        alias = "snakemake",
        alias = "Snakemake",
        alias = "smk",
        alias = "Snakefile"
    )]
    Snakemake,

    /// Nextflow workflow source.
    ///
    /// Nextflow is handled with a dedicated text extractor for process blocks
    /// because the available Nextflow tree-sitter Rust binding is not currently
    /// usable as a direct dependency in this crate.
    #[serde(alias = "nextflow", alias = "Nextflow", alias = "nf", alias = "NF")]
    Nextflow,
}

impl CodeLanguage {
    /// Detect a supported source language from a Stencila format.
    ///
    /// `Format::is_code` is intentionally broader than the static analyzers in
    /// this crate. This function identifies the code formats that can be parsed
    /// for graph facts.
    pub(crate) fn from_format(format: &Format) -> Option<Self> {
        match format {
            Format::JavaScript => Some(Self::JavaScript),
            Format::TypeScript => Some(Self::TypeScript),
            Format::Rust => Some(Self::Rust),
            Format::Python => Some(Self::Python),
            Format::R => Some(Self::R),
            Format::Julia => Some(Self::Julia),
            Format::Matlab => Some(Self::Matlab),
            Format::Snakemake => Some(Self::Snakemake),
            Format::Nextflow => Some(Self::Nextflow),
            _ => None,
        }
    }

    /// Detect a supported source language from a filesystem path.
    ///
    /// Path and extension detection is owned by `Format`; this only narrows the
    /// result to the code formats that graph static analysis supports.
    pub fn from_path(path: &Path) -> Option<Self> {
        Self::from_format(&Format::from_path(path))
    }

    /// Detect a supported source language from a Stencila programming language string.
    ///
    /// Document code chunks and expressions often carry human-authored language
    /// labels rather than file extensions. This accepts the common aliases that
    /// map cleanly to the static analyzers and ignores everything else.
    pub fn from_programming_language(language: &str) -> Option<Self> {
        match language.trim().to_ascii_lowercase().as_str() {
            "javascript" | "js" | "node" | "nodejs" => Some(Self::JavaScript),
            "typescript" | "ts" => Some(Self::TypeScript),
            "rust" | "rs" => Some(Self::Rust),
            "python" | "py" => Some(Self::Python),
            "r" => Some(Self::R),
            "julia" | "jl" => Some(Self::Julia),
            "matlab" | "m" => Some(Self::Matlab),
            "snakemake" | "smk" | "snakefile" => Some(Self::Snakemake),
            "nextflow" | "nf" => Some(Self::Nextflow),
            _ => None,
        }
    }

    /// Return the display name used on generated Schema code nodes.
    ///
    /// The label is intentionally human-readable because it appears in
    /// `programmingLanguage` fields. Graph ids use a separate compact spelling.
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::JavaScript => "JavaScript",
            Self::TypeScript => "TypeScript",
            Self::Rust => "Rust",
            Self::Python => "Python",
            Self::R => "R",
            Self::Julia => "Julia",
            Self::Matlab => "MATLAB",
            Self::Snakemake => "Snakemake",
            Self::Nextflow => "Nextflow",
        }
    }

    /// Return the stable language component used in graph-local ids.
    ///
    /// Id components are lowercase and compact so symbol and function ids remain
    /// stable even if display labels change.
    pub(super) fn id_component(self) -> &'static str {
        match self {
            Self::JavaScript => "javascript",
            Self::TypeScript => "typescript",
            Self::Rust => "rust",
            Self::Python => "python",
            Self::R => "r",
            Self::Julia => "julia",
            Self::Matlab => "matlab",
            Self::Snakemake => "snakemake",
            Self::Nextflow => "nextflow",
        }
    }

    /// Whether this language uses the shared ECMAScript extraction rules.
    pub(super) fn is_ecmascript(self) -> bool {
        matches!(self, Self::JavaScript | Self::TypeScript)
    }

    /// Return the embedded rule bundle for this language.
    ///
    /// The caller treats rules as opaque YAML. Routing through the language enum
    /// keeps rule loading close to parser selection.
    pub(super) fn rules(self) -> &'static str {
        match self {
            Self::JavaScript => JAVASCRIPT_RULES,
            Self::TypeScript => TYPESCRIPT_RULES,
            Self::Rust => RUST_RULES,
            Self::Python => PYTHON_RULES,
            Self::R => R_RULES,
            Self::Julia => JULIA_RULES,
            Self::Matlab => MATLAB_RULES,
            Self::Snakemake => SNAKEMAKE_RULES,
            Self::Nextflow => "",
        }
    }
}

impl fmt::Display for CodeLanguage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name())
    }
}

impl Language for CodeLanguage {
    /// Rewrite ast-grep metavariables for the selected language.
    ///
    /// ast-grep patterns use `$` for metavariables, which can conflict with R
    /// syntax. The custom expando keeps YAML rules readable while allowing the
    /// shared parser adapter to handle language-specific escaping.
    fn pre_process_pattern<'query>(&self, query: &'query str) -> Cow<'query, str> {
        pre_process_pattern(self.expando_char(), query)
    }

    /// Return the private metavariable sigil used after preprocessing.
    ///
    /// A non-ASCII character avoids colliding with ordinary source text in the
    /// supported languages. MATLAB identifiers must start with an ASCII letter,
    /// so it uses a valid identifier prefix for ast-grep pattern parsing.
    fn expando_char(&self) -> char {
        if matches!(self, Self::Matlab) {
            'Q'
        } else {
            'µ'
        }
    }

    /// Map a tree-sitter node kind to its numeric id.
    ///
    /// ast-grep stores compiled matchers against tree-sitter ids, so this simply
    /// delegates to whichever language parser the enum variant represents.
    fn kind_to_id(&self, kind: &str) -> u16 {
        self.get_ts_language().id_for_node_kind(kind, true)
    }

    /// Map a tree-sitter field name to its numeric id.
    ///
    /// Returning `None` lets ast-grep reject rules that refer to fields the
    /// underlying grammar does not expose.
    fn field_to_id(&self, field: &str) -> Option<u16> {
        self.get_ts_language()
            .field_id_for_name(field)
            .map(|field| field.get())
    }

    /// Detect the language for ast-grep path-based APIs.
    ///
    /// The trait method forwards to the inherent detector so callers and
    /// ast-grep use the same source discovery rules.
    fn from_path<P: AsRef<Path>>(path: P) -> Option<Self> {
        Self::from_path(path.as_ref())
    }

    /// Compile an ast-grep pattern for this language.
    ///
    /// Patterns are parsed as source snippets for the target language, which
    /// catches invalid rule syntax when embedded YAML is loaded.
    fn build_pattern(&self, builder: &PatternBuilder) -> Result<Pattern, PatternError> {
        builder.build(|source| StrDoc::try_new(source, *self))
    }
}

impl LanguageExt for CodeLanguage {
    /// Return the static tree-sitter language adapter.
    ///
    /// These adapters are linked into the crate so graph extraction does not
    /// depend on dynamic parser loading or local tree-sitter installations.
    fn get_ts_language(&self) -> TSLanguage {
        match self {
            Self::JavaScript => tree_sitter_javascript::LANGUAGE.into(),
            Self::TypeScript => tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            Self::Rust => tree_sitter_rust::LANGUAGE.into(),
            Self::Python => tree_sitter_python::LANGUAGE.into(),
            Self::R => tree_sitter_r::LANGUAGE.into(),
            Self::Julia => tree_sitter_julia::LANGUAGE.into(),
            Self::Matlab => tree_sitter_matlab::LANGUAGE.into(),
            Self::Snakemake => tree_sitter_snakemake::LANGUAGE.into(),
            Self::Nextflow => unreachable!("Nextflow uses the text extractor, not ast-grep"),
        }
    }
}

/// Rewrite ast-grep metavariable sigils in a rule pattern.
///
/// ast-grep expects a language-specific expando character internally. This
/// function converts uppercase metavariables and variadic captures while leaving
/// ordinary `$` characters available for languages such as R.
fn pre_process_pattern(expando: char, query: &str) -> Cow<'_, str> {
    let mut ret = Vec::with_capacity(query.len());
    let mut dollar_count = 0;
    for char in query.chars() {
        if char == '$' {
            dollar_count += 1;
            continue;
        }
        let need_replace = matches!(char, 'A'..='Z' | '_') || dollar_count == 3;
        let sigil = if need_replace { expando } else { '$' };
        ret.extend(std::iter::repeat_n(sigil, dollar_count));
        dollar_count = 0;
        ret.push(char);
    }
    let sigil = if dollar_count == 3 { expando } else { '$' };
    ret.extend(std::iter::repeat_n(sigil, dollar_count));
    Cow::Owned(ret.into_iter().collect())
}
