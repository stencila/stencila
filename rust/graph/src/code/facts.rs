use std::collections::{BTreeMap, BTreeSet};

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
    /// pd` and `from pandas import DataFrame` both record `pandas`.
    pub imports: BTreeSet<String>,

    /// Variables assigned by this unit.
    ///
    /// Assignments are treated as symbol definitions. They feed both graph
    /// projection and document reactivity, where later chunks can depend on an
    /// earlier chunk that declared a used symbol.
    pub assignments: BTreeSet<String>,

    /// Functions or workflow rules declared by this unit.
    ///
    /// Declarations are modeled separately from assignments because they become
    /// function-like graph nodes while still acting as definitions for document
    /// dependency resolution.
    pub declarations: BTreeSet<String>,

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

    /// Static file or URL literals read by this unit.
    ///
    /// Only concrete string literals are recorded. Dynamic expressions are
    /// skipped so the graph does not imply file dependencies that static
    /// analysis cannot prove.
    pub reads: BTreeSet<String>,

    /// Static file literals written by this unit.
    ///
    /// Writes become generated resources and can be paired with reads to create
    /// derived data edges. URLs are intentionally rare here because most write
    /// APIs in this pass target local files.
    pub writes: BTreeSet<String>,

    /// Static dataframe column accesses discovered in this unit.
    ///
    /// Column facts are intentionally narrow: they represent literal column
    /// names on dataframes that can be tied back to a static read path. This is
    /// enough for useful dataflow edges without implementing dataframe semantics.
    pub columns: BTreeSet<ColumnFact>,

    /// Snakemake workflow rules declared by this unit.
    ///
    /// Rules are kept explicitly because workflow graphs often need to query
    /// rule-level inputs and outputs separately from the source file as a whole.
    pub workflow_rules: BTreeSet<String>,

    /// Static script links from Snakemake rules.
    ///
    /// Script links connect a workflow file to external source files used by
    /// rules. They are stored as literals first so workspace analysis can later
    /// resolve them to concrete file nodes when possible.
    pub script_links: BTreeSet<String>,

    /// Static facts grouped by Snakemake rule name.
    ///
    /// Whole-file facts are still retained for the source file itself, but
    /// rule-level facts prevent inputs and outputs from one rule being attached to
    /// every other rule in the same workflow file.
    pub workflow_rule_facts: BTreeMap<String, WorkflowRuleFacts>,

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

/// Static facts for one Snakemake workflow rule.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WorkflowRuleFacts {
    /// Static input file or URL literals read by the rule.
    pub reads: BTreeSet<String>,

    /// Static output file literals generated by the rule.
    pub writes: BTreeSet<String>,

    /// Static script file literals used by the rule.
    pub script_links: BTreeSet<String>,

    /// Function-like actions used by the rule, such as `shell`.
    pub calls: BTreeSet<String>,
}

/// Workflow resource direction for whole-file and rule-level facts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum WorkflowResourceKind {
    /// Resource read by a workflow unit.
    Read,

    /// Resource generated by a workflow unit.
    Write,

    /// Source file used by a workflow unit.
    Script,
}

impl CodeFacts {
    /// Record a workflow rule or process declaration.
    pub(super) fn record_workflow_rule(&mut self, name: String, offset: Option<usize>) {
        self.workflow_rules.insert(name.clone());
        self.declarations.insert(name.clone());
        self.workflow_rule_facts.entry(name.clone()).or_default();
        if let Some(offset) = offset {
            record_definition(self, &name, offset);
        }
    }

    /// Add static workflow resources to whole-file and optional rule-level facts.
    pub(super) fn extend_workflow_resources(
        &mut self,
        rule: Option<&str>,
        kind: WorkflowResourceKind,
        literals: BTreeSet<String>,
    ) {
        match kind {
            WorkflowResourceKind::Read => self.reads.extend(literals.iter().cloned()),
            WorkflowResourceKind::Write => self.writes.extend(literals.iter().cloned()),
            WorkflowResourceKind::Script => self.script_links.extend(literals.iter().cloned()),
        }

        let Some(rule) = rule else {
            return;
        };
        let rule_facts = self
            .workflow_rule_facts
            .entry(rule.to_string())
            .or_default();
        match kind {
            WorkflowResourceKind::Read => rule_facts.reads.extend(literals),
            WorkflowResourceKind::Write => rule_facts.writes.extend(literals),
            WorkflowResourceKind::Script => rule_facts.script_links.extend(literals),
        }
    }

    /// Record a call-like workflow action for the whole file and optional rule.
    pub(super) fn record_workflow_call(&mut self, rule: Option<&str>, call: impl Into<String>) {
        let call = call.into();
        self.calls.insert(call.clone());
        if let Some(rule) = rule {
            self.workflow_rule_facts
                .entry(rule.to_string())
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
