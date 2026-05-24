use std::collections::{BTreeMap, BTreeSet};

use crate::package::PackageFact;

/// Static facts extracted from one code unit.
///
/// This is the normalization boundary between parsing and graph construction.
/// It keeps language-specific AST shapes out of workspace and document
/// collectors, and it gives tests a compact way to validate extraction before
/// graph ids, node types, and edge directions are involved.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CodeFacts {
    /// Packages imported by this unit.
    ///
    /// Imports become software package nodes connected to the code unit. Only
    /// the statically identifiable package root is kept, so `import pandas as
    /// pd` and `from pandas import DataFrame` both record `pandas` with
    /// ecosystem `pypi`.
    pub imports: BTreeSet<PackageFact>,

    /// Variables assigned by this unit.
    ///
    /// Assignments are treated as symbol definitions. They feed both graph
    /// projection and document reactivity, where later chunks can depend on an
    /// earlier chunk that declared a used symbol.
    pub assignments: BTreeSet<String>,

    /// Functions declared by this unit.
    ///
    /// Declarations are modeled separately from assignments because they become
    /// function-like graph nodes while still acting as definitions for document
    /// dependency resolution.
    pub declarations: BTreeSet<String>,

    /// Function declarations with source ranges.
    ///
    /// These ranges let lean workspace graphs retain a function node only when
    /// it owns visible data-flow variables.
    pub function_declarations: BTreeSet<FunctionFact>,

    /// Symbols used by this unit.
    ///
    /// Uses are filtered after extraction to remove local definitions, imports,
    /// and common builtins. The remaining names are useful dependency signals
    /// rather than a complete lexical symbol table.
    pub uses: BTreeSet<String>,

    /// Function-like callees used by this unit.
    ///
    /// Calls provide a lightweight view of invoked functions or runtime actions
    /// without trying to resolve them to packages or local declarations.
    pub calls: BTreeSet<String>,

    /// File or URL paths read or written by this unit.
    ///
    /// I/O facts keep the resource expression and operation metadata together so
    /// extraction can improve independently from later graph projection and
    /// provenance inference.
    pub io: BTreeSet<IoFact>,

    /// Static dataframe column accesses discovered in this unit.
    ///
    /// Column facts are intentionally narrow: they represent literal column
    /// names on dataframes that can be tied back to a static read path. This is
    /// enough for useful dataflow edges without implementing dataframe semantics.
    pub columns: BTreeSet<ColumnFact>,

    /// Static data flow between variables in this unit.
    ///
    /// These facts are intentionally shallow: they record that one assigned
    /// variable was computed from another identifier in the same source unit.
    /// Graph projection later filters sources to known local definitions so
    /// package aliases, method names, and builtins do not become lineage hops.
    pub variable_flows: BTreeSet<VariableFlowFact>,

    /// Workflow units declared by this unit.
    ///
    /// Snakemake calls these rules, while Nextflow calls them processes. They
    /// are kept explicitly because workflow graphs often need to query
    /// unit-level inputs and outputs separately from the source file as a whole.
    /// They are not mirrored into `declarations`, because graph projection emits
    /// them through the dedicated workflow namespace instead of as generic
    /// functions.
    pub workflow_units: BTreeSet<String>,

    /// Static script links from workflow units.
    ///
    /// Script links connect a workflow file to external source files used by
    /// units. They are stored as literals first so workspace analysis can later
    /// resolve them to concrete file nodes when possible.
    pub script_links: BTreeSet<String>,

    /// Static facts grouped by workflow unit name.
    ///
    /// Whole-file facts are still retained for the source file itself, but
    /// unit-level facts prevent inputs and outputs from one rule or process
    /// being attached to every other unit in the same workflow file.
    pub workflow_unit_facts: BTreeMap<String, WorkflowUnitFacts>,

    /// Whether tree-sitter reported parse errors for this source.
    ///
    /// Syntax errors stop fact projection for the unit. A malformed script can
    /// still appear as a code node in the graph, but it should not emit partial
    /// dependencies from an unreliable tree.
    pub syntax_error: bool,

    /// Local aliases or names introduced by import statements.
    ///
    /// These names are implementation details used to filter `uses`: an import
    /// alias such as `pd` should not make a later chunk depend on a symbol named
    /// `pd`.
    pub(super) imported_symbols: BTreeSet<String>,

    /// Assignment targets tied to static source paths.
    ///
    /// Read calls such as `df = read_csv("data.csv")` seed this map so later
    /// column accesses on `df` can be connected back to the file that produced
    /// the dataframe.
    pub(super) variable_sources: BTreeMap<String, String>,

    /// Earliest byte offset where each symbol is defined.
    ///
    /// This is used to retain read-before-write uses such as `x = x + 1` for
    /// document reactivity while still filtering ordinary local uses.
    pub(super) definition_offsets: BTreeMap<String, usize>,

    /// Earliest byte offset where each symbol is used.
    pub(super) use_offsets: BTreeMap<String, usize>,

    /// Symbols that are read while computing their own first assignment.
    ///
    /// In code such as `x = x + 1`, the RHS use should still depend on a prior
    /// document chunk even though the unit also defines `x`.
    pub(super) read_before_write_symbols: BTreeSet<String>,
}

/// Static facts for one workflow unit.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowUnitFacts {
    /// File or URL paths read or written by the unit.
    pub io: BTreeSet<IoFact>,

    /// Static script file literals used by the unit.
    pub script_links: BTreeSet<String>,

    /// Function-like actions used by the unit, such as `shell`.
    pub calls: BTreeSet<String>,
}

/// A function declaration discovered in source code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct FunctionFact {
    /// Function name.
    pub name: String,

    /// Byte offset where the function declaration starts.
    pub start: usize,

    /// Byte offset where the function declaration ends.
    pub end: usize,
}

/// Workflow resource direction for whole-file and unit-level facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkflowResourceKind {
    /// Resource read by a workflow unit.
    Read,

    /// Resource generated by a workflow unit.
    Write,

    /// Source file used by a workflow unit.
    Script,
}

/// Static or dynamic path expression used by an I/O operation.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum IoPath {
    /// A concrete path or URL string literal.
    Static(String),

    /// A template expression with a partially static shape.
    Template(String),

    /// A dynamic expression whose concrete path is unknown.
    Unknown(String),
}

impl IoPath {
    /// Return the path or expression text carried by this fact.
    pub(crate) fn value(&self) -> &str {
        match self {
            Self::Static(value) | Self::Template(value) | Self::Unknown(value) => value,
        }
    }

    /// Return the stable kind label used in evidence details and synthetic ids.
    pub(crate) fn kind(&self) -> &'static str {
        match self {
            Self::Static(..) => "static",
            Self::Template(..) => "template",
            Self::Unknown(..) => "unknown",
        }
    }

    /// Whether this path can be resolved to a concrete workspace resource.
    pub(crate) fn is_static(&self) -> bool {
        matches!(self, Self::Static(..))
    }
}

/// Direction of a filesystem or URL I/O operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IoDirection {
    /// The operation reads from a resource.
    Read,

    /// The operation writes to a resource.
    Write,

    /// The operation may both read and write the resource.
    ReadWrite,
}

impl IoDirection {
    /// Whether this direction consumes an input resource.
    pub(crate) fn reads(self) -> bool {
        matches!(self, Self::Read | Self::ReadWrite)
    }

    /// Whether this direction generates or mutates an output resource.
    pub(crate) fn writes(self) -> bool {
        matches!(self, Self::Write | Self::ReadWrite)
    }
}

/// File mode detected from an I/O call when statically visible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum IoMode {
    /// Read-only mode.
    Read,

    /// Write/truncate or create mode.
    Write,

    /// Append mode.
    Append,

    /// Read-write update mode.
    ReadWrite,
}

/// A normalized I/O operation discovered in source code.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct IoFact {
    /// Operation direction.
    pub direction: IoDirection,

    /// Resource path or path expression.
    pub path: IoPath,

    /// Byte offset where the I/O operation was detected, when known.
    pub operation_offset: Option<usize>,

    /// Assignment target receiving a read result, when present.
    pub target: Option<String>,

    /// Byte offset of the read target, when present.
    pub target_offset: Option<usize>,

    /// Value or receiver being written, when present.
    pub value: Option<String>,

    /// Byte offset of the written value or receiver, when present.
    pub value_offset: Option<usize>,

    /// Function or method used for the I/O operation, when known.
    pub function: Option<String>,

    /// File mode, when present.
    pub mode: Option<IoMode>,
}

impl IoFact {
    /// Create an I/O fact with only direction and path.
    pub(crate) fn new(direction: IoDirection, path: IoPath) -> Self {
        Self {
            direction,
            path,
            operation_offset: None,
            target: None,
            target_offset: None,
            value: None,
            value_offset: None,
            function: None,
            mode: None,
        }
    }
}

impl CodeFacts {
    /// Record a workflow unit declaration.
    pub(super) fn record_workflow_unit(&mut self, name: String, offset: Option<usize>) {
        self.workflow_units.insert(name.clone());
        self.workflow_unit_facts.entry(name.clone()).or_default();
        if let Some(offset) = offset {
            record_definition(self, &name, offset);
        }
    }

    /// Add workflow resources to whole-file and optional unit-level facts.
    pub(super) fn extend_workflow_resources(
        &mut self,
        unit: Option<&str>,
        kind: WorkflowResourceKind,
        paths: BTreeSet<IoPath>,
    ) {
        let direction = match kind {
            WorkflowResourceKind::Read => Some(IoDirection::Read),
            WorkflowResourceKind::Write => Some(IoDirection::Write),
            WorkflowResourceKind::Script => None,
        };

        if let Some(direction) = direction {
            self.io.extend(
                paths
                    .iter()
                    .cloned()
                    .map(|path| IoFact::new(direction, path)),
            );
        } else {
            self.script_links.extend(paths.iter().filter_map(|path| {
                if let IoPath::Static(path) = path {
                    Some(path.clone())
                } else {
                    None
                }
            }));
        }

        let Some(unit) = unit else {
            return;
        };
        let unit_facts = self
            .workflow_unit_facts
            .entry(unit.to_string())
            .or_default();
        if let Some(direction) = direction {
            unit_facts
                .io
                .extend(paths.into_iter().map(|path| IoFact::new(direction, path)));
        } else {
            unit_facts
                .script_links
                .extend(paths.into_iter().filter_map(|path| {
                    if let IoPath::Static(path) = path {
                        Some(path)
                    } else {
                        None
                    }
                }));
        }
    }

    /// Record a call-like workflow action for the whole file and optional unit.
    pub(super) fn record_workflow_call(&mut self, unit: Option<&str>, call: impl Into<String>) {
        let call = call.into();
        self.calls.insert(call.clone());
        if let Some(unit) = unit {
            self.workflow_unit_facts
                .entry(unit.to_string())
                .or_default()
                .calls
                .insert(call);
        }
    }
}

/// A dataframe column access.
///
/// Column facts capture the smallest useful unit of tabular dataflow. They do
/// not model dataframe transformations; they only say that a named column was
/// accessed through a dataframe variable, optionally with the static file path
/// that introduced that variable.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ColumnFact {
    /// Dataframe variable used for the access.
    ///
    /// The variable keeps the fact tied to local source code and helps avoid
    /// merging unrelated columns that happen to share a name.
    pub dataframe: String,

    /// Column name.
    ///
    /// Only literal column names are captured because inferred or computed names
    /// would make the graph overstate what static analysis knows.
    pub column: String,

    /// Static file path that produced the dataframe when known.
    ///
    /// When present, graph projection can relate the column node to a concrete
    /// file resource. Without it the column is still scoped to the code unit.
    pub source: Option<String>,
}

/// A static dependency from one variable to another.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct VariableFlowFact {
    /// Variable used by an assignment expression.
    pub source: String,

    /// Variable assigned from the source.
    pub target: String,

    /// Byte offset of the assignment target.
    pub target_offset: usize,
}

/// Record an imported local symbol and its definition position when known.
pub(super) fn record_imported_symbol(facts: &mut CodeFacts, name: String, offset: Option<usize>) {
    facts.imported_symbols.insert(name.clone());
    if let Some(offset) = offset {
        record_definition(facts, &name, offset);
    }
}

/// Record the earliest known definition offset for a symbol.
pub(super) fn record_definition(facts: &mut CodeFacts, name: &str, offset: usize) {
    facts
        .definition_offsets
        .entry(name.to_string())
        .and_modify(|existing| *existing = (*existing).min(offset))
        .or_insert(offset);
}

/// Record the earliest known use offset for a symbol.
pub(super) fn record_use(facts: &mut CodeFacts, name: &str, offset: usize) {
    facts.uses.insert(name.to_string());
    facts
        .use_offsets
        .entry(name.to_string())
        .and_modify(|existing| *existing = (*existing).min(offset))
        .or_insert(offset);
}

/// Decide whether a symbol use remains external after local definition filtering.
pub(super) fn should_retain_use(
    name: &str,
    local_definitions: &BTreeSet<String>,
    facts: &CodeFacts,
) -> bool {
    if !local_definitions.contains(name) {
        return true;
    }

    let Some(use_offset) = facts.use_offsets.get(name) else {
        return false;
    };
    let Some(definition_offset) = facts.definition_offsets.get(name) else {
        return false;
    };

    use_offset < definition_offset || facts.read_before_write_symbols.contains(name)
}
