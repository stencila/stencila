//! Resolve I/O path expressions against values proven from the same source file.
//!
//! The analyzer used to resolve a path only when it appeared as a string
//! literal directly in the argument of a recognized I/O call. Every other
//! shape — a module constant, a loop over a list of URLs, an f-string, a value
//! passed through a helper — degraded to an unknown path. That inverted the
//! incentive: the code shape the analyzer rewarded was the one a competent
//! author was least likely to write.
//!
//! This pass widens what resolves without giving up the conservative bias. It
//! builds a lexical value environment from the parse tree, then substitutes
//! only values that are *provably* visible at the operation:
//!
//! - a binding must be the innermost one visible at the use site, so shadowing
//!   is respected rather than guessed through;
//! - it must be assigned exactly once in that scope, so no branch can have
//!   carried a different value;
//! - it must be assigned unconditionally, and before the use;
//! - collection elements are enumerated only from literal collections, using
//!   language-correct iteration semantics;
//! - a function parameter resolves only when the function is declared once, is
//!   never rebound, and *every* call site passes a value this pass can prove.
//!
//! Interprocedural propagation is deliberately one level deep. Beyond that,
//! runtime observation is the better investment than deeper static dataflow.
//!
//! Wherever binding identity, control flow, or collection semantics are
//! ambiguous, the pass declines and records why. Declining leaves behaviour
//! exactly as it was, and the recorded reason becomes an author-facing
//! diagnostic instead of silence.

use std::{collections::BTreeSet, ops::Range};

use ast_grep_core::{AstGrep, tree_sitter::StrDoc};

use super::{
    ast::{SourceNode, call_argument_bindings, call_arguments, call_callee},
    diagnostics::UnresolvedIoReason,
    facts::{CodeFacts, IoFact, IoPath, TemplatePath},
    language::CodeLanguage,
    util::{is_identifier_like, is_path_wrapper_name, is_static_literal, unwrap_path_expression},
};

/// Maximum recursion depth when following bindings and nested expressions.
///
/// Single-file resolution is not a fixed-point analysis, so a small bound
/// terminates on mutually referring bindings without truncating any realistic
/// expression.
const MAX_DEPTH: usize = 8;

/// Maximum number of concrete values one path expression may produce.
///
/// A template interpolating two iterated collections multiplies out. The cap
/// keeps a pathological source file from generating an unbounded fact set.
const MAX_VALUES: usize = 32;

/// Resolve unresolved I/O paths in place.
///
/// Facts whose path resolves to several values — a helper called in a loop over
/// a list of URLs, for instance — expand into one fact per value, because such
/// an operation genuinely has that many upstream dependencies.
pub(super) fn resolve_io_paths(
    language: CodeLanguage,
    grep: &AstGrep<StrDoc<CodeLanguage>>,
    facts: &mut CodeFacts,
) {
    if !supports_resolution(language) {
        return;
    }

    let root = grep.root();
    let environment = Environment::build(language, &root);

    let mut resolved = BTreeSet::new();
    for fact in std::mem::take(&mut facts.io) {
        environment.resolve_fact(fact, &mut resolved);
    }

    for fact in &resolved {
        if fact.direction.reads()
            && let (Some(target), IoPath::Static(path)) = (&fact.target, &fact.path)
        {
            facts
                .variable_sources
                .entry(target.clone())
                .or_insert_with(|| path.clone());
        }
    }

    facts.io = resolved;
}

/// Whether value resolution is implemented for a language.
///
/// Resolution needs language-specific grammar knowledge for scopes, bindings,
/// and collection literals. Languages without that knowledge keep their
/// existing literal-only behaviour rather than resolving approximately.
fn supports_resolution(language: CodeLanguage) -> bool {
    matches!(
        language,
        CodeLanguage::Python
            | CodeLanguage::R
            | CodeLanguage::JavaScript
            | CodeLanguage::TypeScript
    )
}

/// How much resolution budget remains on one branch of the search.
#[derive(Debug, Clone, Copy)]
struct Budget {
    /// Recursion depth so far.
    depth: usize,

    /// Whether the single permitted interprocedural hop is still available.
    interprocedural: bool,
}

impl Budget {
    /// The budget available to a top-level path expression.
    fn new() -> Self {
        Self {
            depth: 0,
            interprocedural: true,
        }
    }

    /// Descend one expression level.
    fn deeper(self) -> Self {
        Self {
            depth: self.depth + 1,
            ..self
        }
    }

    /// Descend into a call site, spending the interprocedural hop.
    fn into_call_site(self) -> Self {
        Self {
            depth: self.depth + 1,
            interprocedural: false,
        }
    }

    /// Whether the depth bound has been exceeded.
    fn exhausted(self) -> bool {
        self.depth > MAX_DEPTH
    }
}

/// A lexical scope introduced by the module or a function body.
#[derive(Debug)]
struct Scope {
    /// Enclosing scope, or `None` for the module scope.
    parent: Option<usize>,

    /// Source range covered by the scope.
    range: Range<usize>,

    /// Name of the function owning the scope, when it has one.
    function: Option<String>,
}

/// One declared parameter of a function.
#[derive(Clone)]
struct Parameter<'r> {
    /// Parameter name.
    name: String,

    /// Default value expression, when the parameter has one.
    default: Option<SourceNode<'r>>,
}

/// A named function declared in the source file.
#[derive(Clone)]
struct FunctionInfo<'r> {
    /// Declared name.
    name: String,

    /// Index of the scope the function body introduces.
    scope: usize,

    /// Declared parameters, in order.
    parameters: Vec<Parameter<'r>>,

    /// Whether the function takes variadic or keyword-splat parameters.
    ///
    /// Variadic mapping cannot be resolved to a specific parameter, so such
    /// functions decline rather than contributing a partial set.
    variadic: bool,
}

/// What a name is bound to.
#[derive(Clone)]
enum BindingValue<'r> {
    /// Bound to the value of an expression.
    Expression(SourceNode<'r>),

    /// Bound successively to the elements of an iterable expression.
    Iteration {
        /// Iterable expression supplying the values.
        value: SourceNode<'r>,

        /// Loop body in which every iterated value may reach an operation.
        body: Range<usize>,
    },

    /// Bound successively to the keys of a mapping expression.
    MappingKeys {
        /// Mapping expression supplying the keys.
        value: SourceNode<'r>,

        /// Loop body in which every iterated key may reach an operation.
        body: Range<usize>,
    },

    /// Bound successively to the values of a mapping expression.
    MappingValues {
        /// Mapping expression supplying the values.
        value: SourceNode<'r>,

        /// Loop body in which every iterated value may reach an operation.
        body: Range<usize>,
    },

    /// Bound by a function parameter.
    Parameter,

    /// Bound by a construct this pass cannot evaluate, such as unpacking.
    Opaque,
}

/// One assignment of a name within a lexical scope.
#[derive(Clone)]
struct Binding<'r> {
    /// Bound name.
    name: String,

    /// Index of the scope the binding belongs to.
    scope: usize,

    /// Byte offset of the binding site.
    offset: usize,

    /// Whether the binding happens under a branch.
    conditional: bool,

    /// What the name is bound to.
    value: BindingValue<'r>,
}

/// The lexical value environment for one source file.
struct Environment<'r> {
    language: CodeLanguage,
    root: SourceNode<'r>,
    scopes: Vec<Scope>,
    bindings: Vec<Binding<'r>>,
    functions: Vec<FunctionInfo<'r>>,
}

/// Outcome of resolving one expression.
#[derive(Debug, Clone)]
enum Resolution {
    /// One or more concrete values proven from source.
    Values(Vec<String>),

    /// A template whose known segments are proven but which has gaps.
    Partial {
        /// Literal runs proven from source, in source order.
        segments: Vec<String>,

        /// Placeholder expressions that could not be resolved.
        unresolved: Vec<String>,
    },

    /// Resolution declined, with the reason it declined.
    Declined(UnresolvedIoReason),
}

/// A literal collection reached by following bindings.
#[derive(Clone)]
enum CollectionLiteral<'r> {
    /// An ordered sequence such as a list, tuple, vector, or array.
    Elements(Vec<SourceNode<'r>>),

    /// A mapping of key expressions to value expressions.
    Pairs(Vec<(SourceNode<'r>, SourceNode<'r>)>),
}

/// One piece of a template expression during partial evaluation.
#[derive(Debug, Clone)]
enum TemplatePart {
    /// Literal text from the template.
    Literal(String),

    /// A placeholder that resolved to one or more values.
    Values(Vec<String>),

    /// A placeholder that could not be resolved, with its source text.
    Unresolved(String),
}

impl<'r> Environment<'r> {
    /// Build the scope tree, binding table, and function table for a file.
    fn build(language: CodeLanguage, root: &SourceNode<'r>) -> Self {
        let mut environment = Self {
            language,
            root: root.clone(),
            scopes: vec![Scope {
                parent: None,
                range: root.range(),
                function: None,
            }],
            bindings: Vec::new(),
            functions: Vec::new(),
        };
        environment.walk(root, 0);
        environment
    }

    /// Walk a subtree, recording scopes and bindings.
    fn walk(&mut self, node: &SourceNode<'r>, scope: usize) {
        let scope = self.function_scope(node, scope).unwrap_or(scope);
        self.record_bindings(node, scope);

        for child in node.children() {
            self.walk(&child, scope);
        }
    }

    /// Open a new scope when a node declares a function body.
    fn function_scope(&mut self, node: &SourceNode<'r>, parent: usize) -> Option<usize> {
        if !is_function_node(self.language, node.kind().as_ref()) {
            return None;
        }

        let name = function_name(self.language, node);
        self.scopes.push(Scope {
            parent: Some(parent),
            range: node.range(),
            function: name.clone(),
        });
        let scope = self.scopes.len() - 1;

        let declared = function_parameters(node);
        for parameter in &declared.parameters {
            self.bindings.push(Binding {
                name: parameter.name.clone(),
                scope,
                offset: parameter.offset,
                conditional: false,
                value: BindingValue::Parameter,
            });
        }

        // Only named functions can be bound to a call site by name.
        if let Some(name) = name {
            self.functions.push(FunctionInfo {
                name,
                scope,
                parameters: declared
                    .parameters
                    .iter()
                    .map(|parameter| Parameter {
                        name: parameter.name.clone(),
                        default: parameter.default.clone(),
                    })
                    .collect(),
                variadic: declared.variadic,
            });
        }

        Some(scope)
    }

    /// Record any bindings introduced by a single node.
    fn record_bindings(&mut self, node: &SourceNode<'r>, scope: usize) {
        let kind = node.kind();
        let kind = kind.as_ref();

        match self.language {
            CodeLanguage::Python => self.record_python_bindings(node, kind, scope),
            CodeLanguage::R => self.record_r_bindings(node, kind, scope),
            _ => self.record_ecmascript_bindings(node, kind, scope),
        }
    }

    /// Record Python assignment, augmented assignment, and loop bindings.
    fn record_python_bindings(&mut self, node: &SourceNode<'r>, kind: &str, scope: usize) {
        match kind {
            "assignment" => {
                let (Some(left), Some(right)) = (node.field("left"), node.field("right")) else {
                    return;
                };
                self.record_target(&left, &right, node, scope);
            }
            "augmented_assignment" => {
                if let Some(left) = node.field("left") {
                    self.record_opaque_targets(&left, node, scope);
                }
            }
            "for_statement" => {
                let (Some(left), Some(right)) = (loop_variable(node), loop_sequence(node)) else {
                    return;
                };
                self.record_loop_target(&left, &right, node, scope);
            }
            _ => {}
        }
    }

    /// Record R assignment and loop bindings.
    fn record_r_bindings(&mut self, node: &SourceNode<'r>, kind: &str, scope: usize) {
        match kind {
            "binary_operator" => {
                let children = node.children().collect::<Vec<_>>();
                let Some(operator) = children
                    .iter()
                    .position(|child| matches!(child.kind().as_ref(), "<-" | "<<-" | "="))
                else {
                    return;
                };
                let (Some(left), Some(right)) = (
                    operator
                        .checked_sub(1)
                        .and_then(|index| children.get(index)),
                    children.get(operator + 1),
                ) else {
                    return;
                };
                self.record_target(left, right, node, scope);
            }
            "for_statement" => {
                let (Some(left), Some(right)) = (loop_variable(node), loop_sequence(node)) else {
                    return;
                };
                self.record_loop_target(&left, &right, node, scope);
            }
            _ => {}
        }
    }

    /// Record JavaScript and TypeScript declaration, assignment, and loop bindings.
    fn record_ecmascript_bindings(&mut self, node: &SourceNode<'r>, kind: &str, scope: usize) {
        match kind {
            "variable_declarator" => {
                let Some(left) = node.field("name") else {
                    return;
                };
                match node.field("value") {
                    Some(right) => self.record_target(&left, &right, node, scope),
                    None => self.record_opaque_targets(&left, node, scope),
                }
            }
            "assignment_expression" => {
                let (Some(left), Some(right)) = (node.field("left"), node.field("right")) else {
                    return;
                };
                self.record_target(&left, &right, node, scope);
            }
            "augmented_assignment_expression" => {
                if let Some(left) = node.field("left") {
                    self.record_opaque_targets(&left, node, scope);
                }
            }
            "for_in_statement" => {
                let (Some(left), Some(right)) = (node.field("left"), node.field("right")) else {
                    return;
                };
                self.record_loop_target(&left, &right, node, scope);
            }
            _ => {}
        }
    }

    /// Record a binding for a simple assignment target.
    ///
    /// Targets that are not a single identifier are recorded as opaque rather
    /// than skipped, so a destructured name still counts as assigned and is
    /// never resolved from an unrelated outer binding.
    fn record_target(
        &mut self,
        left: &SourceNode<'r>,
        right: &SourceNode<'r>,
        statement: &SourceNode<'r>,
        scope: usize,
    ) {
        if left.kind().as_ref() != "identifier" {
            self.record_opaque_targets(left, statement, scope);
            return;
        }

        self.bindings.push(Binding {
            name: left.text().trim().to_string(),
            scope,
            offset: left.range().start,
            conditional: self.is_conditional(statement, scope),
            value: BindingValue::Expression(right.clone()),
        });
    }

    /// Record a loop variable bound to the elements of an iterable.
    ///
    /// Within the loop body, every element of a literal collection is a value
    /// the operation genuinely uses, so enumerating them asserts nothing the
    /// source does not already say. The body range is retained so uses after
    /// the loop decline instead of inheriting every iterated value.
    fn record_loop_target(
        &mut self,
        left: &SourceNode<'r>,
        right: &SourceNode<'r>,
        statement: &SourceNode<'r>,
        scope: usize,
    ) {
        let Some(body) = statement.field("body").map(|body| body.range()) else {
            self.record_opaque_targets(left, statement, scope);
            return;
        };

        if left.kind().as_ref() == "identifier" {
            let value = match mapping_view(self.language, right) {
                Some((MappingView::Keys, mapping)) => BindingValue::MappingKeys {
                    value: mapping,
                    body: body.clone(),
                },
                Some((MappingView::Values, mapping)) => BindingValue::MappingValues {
                    value: mapping,
                    body: body.clone(),
                },
                Some((MappingView::Items, _)) => BindingValue::Opaque,
                None => BindingValue::Iteration {
                    value: right.clone(),
                    body,
                },
            };
            self.bindings.push(Binding {
                name: left.text().trim().to_string(),
                scope,
                offset: left.range().start,
                conditional: false,
                value,
            });
            return;
        }

        // `for key, value in mapping.items()` is the one unpacking shape with
        // semantics precise enough to propagate.
        let targets = left
            .children()
            .filter(|child| child.kind().as_ref() == "identifier")
            .collect::<Vec<_>>();
        if let ([key, value], Some((MappingView::Items, mapping))) =
            (targets.as_slice(), mapping_view(self.language, right))
        {
            self.bindings.push(Binding {
                name: key.text().trim().to_string(),
                scope,
                offset: key.range().start,
                conditional: false,
                value: BindingValue::MappingKeys {
                    value: mapping.clone(),
                    body: body.clone(),
                },
            });
            self.bindings.push(Binding {
                name: value.text().trim().to_string(),
                scope,
                offset: value.range().start,
                conditional: false,
                value: BindingValue::MappingValues {
                    value: mapping,
                    body,
                },
            });
            return;
        }

        self.record_opaque_targets(left, statement, scope);
    }

    /// Record every identifier in a target pattern as an unprovable binding.
    fn record_opaque_targets(
        &mut self,
        left: &SourceNode<'r>,
        statement: &SourceNode<'r>,
        scope: usize,
    ) {
        let conditional = self.is_conditional(statement, scope);
        let targets = if left.kind().as_ref() == "identifier" {
            vec![left.clone()]
        } else {
            left.dfs()
                .filter(|child| child.kind().as_ref() == "identifier")
                .collect()
        };

        for target in targets {
            self.bindings.push(Binding {
                name: target.text().trim().to_string(),
                scope,
                offset: target.range().start,
                conditional,
                value: BindingValue::Opaque,
            });
        }
    }

    /// Whether a statement sits under a branch or loop within its scope.
    fn is_conditional(&self, statement: &SourceNode<'r>, scope: usize) -> bool {
        let scope_range = &self.scopes[scope].range;
        let mut current = statement.parent();
        while let Some(node) = current {
            if node.range() == *scope_range {
                return false;
            }
            if is_conditional_node(self.language, node.kind().as_ref()) {
                return true;
            }
            current = node.parent();
        }
        false
    }

    /// Resolve one I/O fact, adding the resulting facts to the output set.
    fn resolve_fact(&self, fact: IoFact, resolved: &mut BTreeSet<IoFact>) {
        if fact.path.is_static() {
            resolved.insert(fact);
            return;
        }

        let resolution = match self.path_node(&fact) {
            Some(node) => self.resolve_node(&node, Budget::new()),
            None => self.resolve_detached_path(&fact),
        };

        match resolution {
            Resolution::Values(values) => {
                let values = values
                    .into_iter()
                    .filter(|value| is_static_literal(value))
                    .collect::<Vec<_>>();
                if values.is_empty() {
                    resolved.insert(declined(fact, UnresolvedIoReason::NotALiteral));
                    return;
                }
                for value in values {
                    let mut resolved_fact = fact.clone();
                    resolved_fact.path = IoPath::Static(value);
                    resolved_fact.unresolved_reason = None;
                    resolved.insert(resolved_fact);
                }
            }
            Resolution::Partial {
                segments,
                unresolved,
            } => {
                let mut partial = fact.clone();
                partial.path = IoPath::Template(TemplatePath {
                    expression: fact.path.value().to_string(),
                    segments,
                    unresolved,
                });
                partial.unresolved_reason = Some(UnresolvedIoReason::PartialTemplate);
                resolved.insert(partial);
            }
            Resolution::Declined(reason) => {
                resolved.insert(declined(fact, reason));
            }
        }
    }

    /// Locate the parse-tree node holding an I/O fact's path expression.
    ///
    /// Rule matches record the offset of the captured path argument, so the
    /// node is usually the outermost expression starting there. Text-scanned
    /// facts record the offset of the whole call instead, so the expression is
    /// then searched for within it by text.
    fn path_node(&self, fact: &IoFact) -> Option<SourceNode<'r>> {
        let offset = fact.operation_offset?;
        let expression = fact.path.value();

        let outermost = self
            .root
            .dfs()
            .filter(|node| node.range().start == offset)
            .max_by_key(|node| node.range().end)?;

        if node_matches_expression(&outermost, expression) {
            return Some(outermost);
        }

        outermost
            .dfs()
            .find(|node| node_matches_expression(node, expression))
    }

    /// Resolve a path expression that has no locatable parse-tree node.
    ///
    /// Keyword-argument forms are found by the text scanner, which only knows
    /// the offset of the whole call. A bare identifier can still be looked up
    /// in the value environment from that offset.
    fn resolve_detached_path(&self, fact: &IoFact) -> Resolution {
        let expression = fact.path.value();
        let (Some(offset), true) = (fact.operation_offset, is_identifier_like(expression)) else {
            return Resolution::Declined(UnresolvedIoReason::NotALiteral);
        };
        self.resolve_name(expression, offset, Budget::new())
    }

    /// Resolve an expression node to concrete values where possible.
    fn resolve_node(&self, node: &SourceNode<'r>, budget: Budget) -> Resolution {
        if budget.exhausted() {
            return Resolution::Declined(UnresolvedIoReason::NotALiteral);
        }

        let kind = node.kind();
        match kind.as_ref() {
            "identifier" => self.resolve_name(node.text().trim(), node.range().start, budget),
            "string" | "template_string" | "concatenated_string" => {
                self.resolve_string(node, budget)
            }
            "call" | "call_expression" => self.resolve_call(node, budget),
            "subscript" | "subscript_expression" => self.resolve_subscript(node, budget),
            "binary_operator" | "binary_expression" => self.resolve_concatenation(node, budget),
            "parenthesized_expression" | "argument" => {
                match node.children().find(|child| child.is_named()) {
                    Some(inner) => self.resolve_node(&inner, budget.deeper()),
                    None => Resolution::Declined(UnresolvedIoReason::NotALiteral),
                }
            }
            _ => Resolution::Declined(UnresolvedIoReason::NotALiteral),
        }
    }

    /// Resolve a name against the innermost binding visible at an offset.
    fn resolve_name(&self, name: &str, offset: usize, budget: Budget) -> Resolution {
        match self.lookup(name, offset) {
            Ok(binding) => self.resolve_binding(name, binding, offset, budget),
            Err(reason) => Resolution::Declined(reason),
        }
    }

    /// Find the binding that governs a name at a source offset.
    fn lookup(&self, name: &str, offset: usize) -> Result<&Binding<'r>, UnresolvedIoReason> {
        let mut scope = Some(self.innermost_scope(offset));
        while let Some(index) = scope {
            let visible = self
                .bindings
                .iter()
                .filter(|binding| binding.scope == index && binding.name == name)
                .collect::<Vec<_>>();

            if let Some(binding) = visible.first() {
                if visible.len() > 1 {
                    return Err(UnresolvedIoReason::MultipleAssignments {
                        name: name.to_string(),
                    });
                }
                if binding.conditional {
                    return Err(UnresolvedIoReason::ConditionalAssignment {
                        name: name.to_string(),
                    });
                }
                if binding.offset > offset {
                    return Err(UnresolvedIoReason::UseBeforeDefinition {
                        name: name.to_string(),
                    });
                }
                return Ok(binding);
            }

            scope = self.scopes[index].parent;
        }

        Err(UnresolvedIoReason::UnboundIdentifier {
            name: name.to_string(),
        })
    }

    /// Resolve the value a binding carries.
    fn resolve_binding(
        &self,
        name: &str,
        binding: &Binding<'r>,
        offset: usize,
        budget: Budget,
    ) -> Resolution {
        match &binding.value {
            BindingValue::Expression(node) => self.resolve_node(node, budget.deeper()),
            BindingValue::Iteration { value, body } if body.contains(&offset) => {
                self.resolve_iteration(value, budget.deeper())
            }
            BindingValue::MappingKeys { value, body } if body.contains(&offset) => {
                self.resolve_mapping(value, true, budget.deeper())
            }
            BindingValue::MappingValues { value, body } if body.contains(&offset) => {
                self.resolve_mapping(value, false, budget.deeper())
            }
            BindingValue::Iteration { .. }
            | BindingValue::MappingKeys { .. }
            | BindingValue::MappingValues { .. } => {
                Resolution::Declined(UnresolvedIoReason::ConditionalAssignment {
                    name: name.to_string(),
                })
            }
            BindingValue::Parameter => self.resolve_parameter(name, binding, budget),
            BindingValue::Opaque => Resolution::Declined(UnresolvedIoReason::NonLiteralValue {
                name: name.to_string(),
            }),
        }
    }

    /// Resolve a parameter from the values passed at every call site.
    ///
    /// A helper called with three different URLs genuinely has three upstream
    /// dependencies, so disagreeing call sites contribute all their values. But
    /// a single call site this pass cannot prove makes the whole set unsound,
    /// so it declines rather than attributing a partial set.
    fn resolve_parameter(&self, name: &str, binding: &Binding<'r>, budget: Budget) -> Resolution {
        let unresolved = || {
            Resolution::Declined(UnresolvedIoReason::UnresolvedParameter {
                parameter: name.to_string(),
                function: self.scopes[binding.scope]
                    .function
                    .clone()
                    .unwrap_or_else(|| "an anonymous function".to_string()),
            })
        };

        if !budget.interprocedural {
            return unresolved();
        }

        let Some(function) = self
            .functions
            .iter()
            .find(|function| function.scope == binding.scope)
        else {
            return unresolved();
        };
        if function.variadic {
            return unresolved();
        }
        let Some(index) = function
            .parameters
            .iter()
            .position(|parameter| parameter.name == name)
        else {
            return unresolved();
        };

        if !self.is_unambiguous_callee(function) {
            return Resolution::Declined(UnresolvedIoReason::AmbiguousCallee {
                callee: function.name.clone(),
            });
        }

        let mut values = Vec::new();
        let mut called = false;
        for call in self.root.dfs() {
            if !calls_function(&call, &function.name) {
                continue;
            }
            let Some(arguments) = call_argument_bindings(self.language, &call) else {
                // A splat or spread hides which parameter receives what.
                return unresolved();
            };
            let Some(argument) = arguments
                .for_parameter(index, name)
                .or_else(|| function.parameters[index].default.clone())
            else {
                return unresolved();
            };

            called = true;
            match self.resolve_node(&argument, budget.into_call_site()) {
                Resolution::Values(resolved) => values.extend(resolved),
                _ => return unresolved(),
            }
        }

        if !called || values.is_empty() || values.len() > MAX_VALUES {
            return unresolved();
        }
        Resolution::Values(values)
    }

    /// Whether a function name binds unambiguously to one local declaration.
    ///
    /// Two declarations of the same name, or a later rebinding of it, mean a
    /// call site cannot be attributed to this body with certainty.
    fn is_unambiguous_callee(&self, function: &FunctionInfo<'r>) -> bool {
        if self
            .functions
            .iter()
            .filter(|candidate| candidate.name == function.name)
            .count()
            > 1
        {
            return false;
        }

        let body = &self.scopes[function.scope].range;
        self.bindings
            .iter()
            .filter(|binding| binding.name == function.name)
            .all(|binding| match &binding.value {
                // Languages where functions are values bind the declaration
                // itself; that binding is the declaration, not a rebinding.
                BindingValue::Expression(node) => node.range() == *body,
                _ => false,
            })
    }

    /// Resolve every element produced by iterating an expression.
    fn resolve_iteration(&self, node: &SourceNode<'r>, budget: Budget) -> Resolution {
        match self.collection_literal(node, budget) {
            // Iterating a Python mapping yields its keys.
            Some(CollectionLiteral::Pairs(pairs)) if self.language == CodeLanguage::Python => {
                self.resolve_all(pairs.into_iter().map(|(key, _)| key), budget)
            }
            Some(CollectionLiteral::Elements(elements)) => self.resolve_all(elements, budget),
            _ => Resolution::Declined(UnresolvedIoReason::NotALiteral),
        }
    }

    /// Resolve the keys or values of a mapping expression.
    fn resolve_mapping(&self, node: &SourceNode<'r>, keys: bool, budget: Budget) -> Resolution {
        match self.collection_literal(node, budget) {
            Some(CollectionLiteral::Pairs(pairs)) => self.resolve_all(
                pairs
                    .into_iter()
                    .map(|(key, value)| if keys { key } else { value }),
                budget,
            ),
            _ => Resolution::Declined(UnresolvedIoReason::NotALiteral),
        }
    }

    /// Resolve a sequence of nodes, declining unless every one resolves.
    fn resolve_all(
        &self,
        nodes: impl IntoIterator<Item = SourceNode<'r>>,
        budget: Budget,
    ) -> Resolution {
        let mut values = Vec::new();
        for node in nodes {
            match self.resolve_node(&node, budget.deeper()) {
                Resolution::Values(resolved) => values.extend(resolved),
                other => return other,
            }
        }

        if values.is_empty() || values.len() > MAX_VALUES {
            return Resolution::Declined(UnresolvedIoReason::NotALiteral);
        }
        Resolution::Values(values)
    }

    /// Follow bindings until a literal collection is reached.
    fn collection_literal(
        &self,
        node: &SourceNode<'r>,
        budget: Budget,
    ) -> Option<CollectionLiteral<'r>> {
        if budget.exhausted() {
            return None;
        }

        if let Some(literal) = collection_literal_node(self.language, node) {
            return Some(literal);
        }

        if node.kind().as_ref() == "identifier" {
            let binding = self.lookup(node.text().trim(), node.range().start).ok()?;
            if let BindingValue::Expression(value) = &binding.value {
                return self.collection_literal(value, budget.deeper());
            }
        }

        None
    }

    /// Resolve a string or template literal, evaluating placeholders where possible.
    fn resolve_string(&self, node: &SourceNode<'r>, budget: Budget) -> Resolution {
        let mut parts = Vec::new();

        for child in node.children() {
            let kind = child.kind();
            match kind.as_ref() {
                "string_content" | "string_fragment" | "escape_sequence" => {
                    parts.push(TemplatePart::Literal(child.text().into_owned()));
                }
                "interpolation" | "template_substitution" => {
                    let inner = child
                        .children()
                        .find(|inner| inner.is_named() && inner.kind() != "format_specifier");
                    match inner {
                        Some(inner) => match self.resolve_node(&inner, budget.deeper()) {
                            Resolution::Values(values) => parts.push(TemplatePart::Values(values)),
                            _ => parts
                                .push(TemplatePart::Unresolved(inner.text().trim().to_string())),
                        },
                        None => {
                            parts.push(TemplatePart::Unresolved(child.text().trim().to_string()))
                        }
                    }
                }
                "string" | "template_string" => {
                    match self.resolve_string(&child, budget.deeper()) {
                        Resolution::Values(values) => parts.push(TemplatePart::Values(values)),
                        other => return other,
                    }
                }
                _ => {}
            }
        }

        combine_parts(parts)
    }

    /// Resolve a call to a path-preserving wrapper or a path-joining helper.
    fn resolve_call(&self, node: &SourceNode<'r>, budget: Budget) -> Resolution {
        let Some(callee) = call_callee(node) else {
            return Resolution::Declined(UnresolvedIoReason::NotALiteral);
        };
        let arguments = call_arguments(self.language, node);
        let name = callee
            .rsplit(['.', ':'])
            .find(|part| !part.is_empty())
            .unwrap_or(callee.as_str());

        if is_path_wrapper_name(name) && arguments.len() == 1 {
            return self.resolve_node(&arguments[0], budget.deeper());
        }

        if let Some(separator) = path_join_separator(&callee, name) {
            let mut operands = Vec::new();
            if name == "joinpath"
                && let Some(receiver) = node
                    .field("function")
                    .and_then(|function| function.field("object"))
            {
                operands.push(receiver);
            }
            operands.extend(arguments);

            let mut parts = Vec::new();
            for (index, argument) in operands.iter().enumerate() {
                if index > 0 && !separator.is_empty() {
                    parts.push(TemplatePart::Literal(separator.to_string()));
                }
                match self.resolve_node(argument, budget.deeper()) {
                    Resolution::Values(values) => parts.push(TemplatePart::Values(values)),
                    _ => parts.push(TemplatePart::Unresolved(argument.text().trim().to_string())),
                }
            }
            return combine_parts(parts);
        }

        // R vectors are calls rather than a dedicated literal node.
        if self.language == CodeLanguage::R && name == "c" && !arguments.is_empty() {
            return self.resolve_all(arguments, budget);
        }

        Resolution::Declined(UnresolvedIoReason::NotALiteral)
    }

    /// Resolve indexing into a literal collection with a literal index.
    fn resolve_subscript(&self, node: &SourceNode<'r>, budget: Budget) -> Resolution {
        let base = node.field("value").or_else(|| node.field("object"));
        let index = node.field("subscript").or_else(|| node.field("index"));
        let (Some(base), Some(index)) = (base, index) else {
            return Resolution::Declined(UnresolvedIoReason::NotALiteral);
        };

        let Some(collection) = self.collection_literal(&base, budget) else {
            return Resolution::Declined(UnresolvedIoReason::NotALiteral);
        };

        match collection {
            CollectionLiteral::Elements(elements) => {
                let Ok(position) = index.text().trim().parse::<usize>() else {
                    return Resolution::Declined(UnresolvedIoReason::NotALiteral);
                };
                match elements.get(position) {
                    Some(element) => self.resolve_node(element, budget.deeper()),
                    None => Resolution::Declined(UnresolvedIoReason::NotALiteral),
                }
            }
            CollectionLiteral::Pairs(pairs) => {
                let Resolution::Values(keys) = self.resolve_node(&index, budget.deeper()) else {
                    return Resolution::Declined(UnresolvedIoReason::NotALiteral);
                };
                let [key] = keys.as_slice() else {
                    return Resolution::Declined(UnresolvedIoReason::NotALiteral);
                };
                for (candidate, value) in &pairs {
                    if let Resolution::Values(resolved) =
                        self.resolve_node(candidate, budget.deeper())
                        && resolved.len() == 1
                        && resolved[0] == *key
                    {
                        return self.resolve_node(value, budget.deeper());
                    }
                }
                Resolution::Declined(UnresolvedIoReason::NotALiteral)
            }
        }
    }

    /// Resolve string concatenation with `+`.
    fn resolve_concatenation(&self, node: &SourceNode<'r>, budget: Budget) -> Resolution {
        let operator = node
            .field("operator")
            .or_else(|| node.children().find(|child| !child.is_named()));
        if operator.is_none_or(|operator| operator.text().trim() != "+") {
            return Resolution::Declined(UnresolvedIoReason::NotALiteral);
        }

        let (Some(left), Some(right)) = (node.field("left"), node.field("right")) else {
            return Resolution::Declined(UnresolvedIoReason::NotALiteral);
        };

        let mut parts = Vec::new();
        for operand in [left, right] {
            match self.resolve_node(&operand, budget.deeper()) {
                Resolution::Values(values) => parts.push(TemplatePart::Values(values)),
                _ => parts.push(TemplatePart::Unresolved(operand.text().trim().to_string())),
            }
        }
        combine_parts(parts)
    }

    /// Return the index of the innermost scope covering an offset.
    fn innermost_scope(&self, offset: usize) -> usize {
        self.scopes
            .iter()
            .enumerate()
            .filter(|(_, scope)| scope.range.contains(&offset))
            .min_by_key(|(_, scope)| scope.range.end - scope.range.start)
            .map_or(0, |(index, _)| index)
    }
}

/// Attach a reason to an I/O fact that stayed unresolved.
fn declined(mut fact: IoFact, reason: UnresolvedIoReason) -> IoFact {
    fact.unresolved_reason = Some(reason);
    fact
}

/// Whether a node's text is the path expression being looked for.
fn node_matches_expression(node: &SourceNode<'_>, expression: &str) -> bool {
    let text = node.text();
    let text = text.trim();
    text == expression || unwrap_path_expression(text) == expression
}

/// Combine template parts into a resolution.
///
/// Fully resolved parts multiply out into concrete values. A single
/// unresolvable placeholder keeps the proven runs as evidence but never yields
/// a resource, because the assembled string was never the path that was used.
fn combine_parts(parts: Vec<TemplatePart>) -> Resolution {
    if parts.is_empty() {
        return Resolution::Declined(UnresolvedIoReason::NotALiteral);
    }

    if parts
        .iter()
        .any(|part| matches!(part, TemplatePart::Unresolved(..)))
    {
        let mut segments = Vec::new();
        let mut unresolved = Vec::new();
        let mut current = String::new();
        for part in parts {
            match part {
                TemplatePart::Literal(text) => current.push_str(&text),
                TemplatePart::Values(values) => match values.as_slice() {
                    [value] => current.push_str(value),
                    _ if !current.is_empty() => segments.push(std::mem::take(&mut current)),
                    _ => {}
                },
                TemplatePart::Unresolved(expression) => {
                    if !current.is_empty() {
                        segments.push(std::mem::take(&mut current));
                    }
                    unresolved.push(expression);
                }
            }
        }
        if !current.is_empty() {
            segments.push(current);
        }
        return Resolution::Partial {
            segments,
            unresolved,
        };
    }

    let mut values = vec![String::new()];
    for part in parts {
        let options = match part {
            TemplatePart::Literal(text) => vec![text],
            TemplatePart::Values(values) => values,
            TemplatePart::Unresolved(..) => continue,
        };
        if values.len().saturating_mul(options.len()) > MAX_VALUES {
            return Resolution::Declined(UnresolvedIoReason::NotALiteral);
        }
        values = values
            .iter()
            .flat_map(|prefix| {
                options
                    .iter()
                    .map(|option| format!("{prefix}{option}"))
                    .collect::<Vec<_>>()
            })
            .collect();
    }

    if values.iter().all(String::is_empty) {
        return Resolution::Declined(UnresolvedIoReason::NotALiteral);
    }
    Resolution::Values(values)
}

/// Whether a node kind introduces a function scope.
fn is_function_node(language: CodeLanguage, kind: &str) -> bool {
    match language {
        CodeLanguage::Python => matches!(kind, "function_definition" | "lambda"),
        CodeLanguage::R => kind == "function_definition",
        _ => matches!(
            kind,
            "function_declaration"
                | "function_expression"
                | "generator_function_declaration"
                | "generator_function"
                | "arrow_function"
                | "method_definition"
        ),
    }
}

/// Whether a node kind makes an enclosed assignment branch- or loop-carried.
fn is_conditional_node(language: CodeLanguage, kind: &str) -> bool {
    match language {
        CodeLanguage::Python => matches!(
            kind,
            "if_statement"
                | "elif_clause"
                | "else_clause"
                | "while_statement"
                | "for_statement"
                | "try_statement"
                | "except_clause"
                | "finally_clause"
                | "match_statement"
                | "case_clause"
                | "conditional_expression"
        ),
        CodeLanguage::R => matches!(
            kind,
            "if_statement" | "while_statement" | "for_statement" | "repeat_statement"
        ),
        _ => matches!(
            kind,
            "if_statement"
                | "else_clause"
                | "switch_statement"
                | "switch_case"
                | "switch_default"
                | "while_statement"
                | "do_statement"
                | "for_statement"
                | "for_in_statement"
                | "try_statement"
                | "catch_clause"
                | "finally_clause"
                | "ternary_expression"
        ),
    }
}

/// Return the declared name of a function node, when it has one.
fn function_name(language: CodeLanguage, node: &SourceNode<'_>) -> Option<String> {
    if language == CodeLanguage::R {
        // R functions are values, so any name comes from the assignment.
        let parent = node.parent()?;
        let children = parent.children().collect::<Vec<_>>();
        let operator = children
            .iter()
            .position(|child| matches!(child.kind().as_ref(), "<-" | "<<-" | "="))?;
        let name = children.get(operator.checked_sub(1)?)?;
        return (name.kind() == "identifier").then(|| name.text().trim().to_string());
    }

    if let Some(name) = node.field("name") {
        return Some(name.text().trim().to_string());
    }

    // A function expression assigned to a name takes that name.
    let parent = node.parent()?;
    if !matches!(
        parent.kind().as_ref(),
        "variable_declarator" | "assignment_expression"
    ) {
        return None;
    }
    parent
        .field("name")
        .or_else(|| parent.field("left"))
        .filter(|name| name.kind() == "identifier")
        .map(|name| name.text().trim().to_string())
}

/// One declared parameter, with the position of its name.
struct DeclaredParameter<'r> {
    name: String,
    offset: usize,
    default: Option<SourceNode<'r>>,
}

/// The parameter list declared by a function node.
struct DeclaredParameters<'r> {
    parameters: Vec<DeclaredParameter<'r>>,
    variadic: bool,
}

/// Return the parameters declared by a function node.
fn function_parameters<'r>(node: &SourceNode<'r>) -> DeclaredParameters<'r> {
    let mut declared = DeclaredParameters {
        parameters: Vec::new(),
        variadic: false,
    };

    let Some(parameters) = node.field("parameters").or_else(|| {
        node.children().find(|child| {
            matches!(
                child.kind().as_ref(),
                "parameters" | "formal_parameters" | "lambda_parameters"
            )
        })
    }) else {
        return declared;
    };

    for child in parameters.children() {
        if !child.is_named() || child.kind() == "comment" {
            continue;
        }

        let kind = child.kind();
        if matches!(
            kind.as_ref(),
            "list_splat_pattern" | "dictionary_splat_pattern" | "rest_pattern" | "dots"
        ) || child.text().trim() == "..."
        {
            declared.variadic = true;
            continue;
        }

        let (name, default) = if kind == "identifier" {
            (Some(child.clone()), None)
        } else {
            let name = child
                .field("name")
                .or_else(|| child.field("pattern"))
                .or_else(|| child.field("left"))
                .or_else(|| child.children().find(|inner| inner.kind() == "identifier"));
            let default = child
                .field("value")
                .or_else(|| child.field("default"))
                .or_else(|| child.field("right"));
            (name, default)
        };

        let Some(name) = name.filter(|name| name.kind() == "identifier") else {
            // An unrecognized parameter shape must not silently shift the
            // positional index of the parameters that follow it.
            declared.variadic = true;
            continue;
        };

        declared.parameters.push(DeclaredParameter {
            name: name.text().trim().to_string(),
            offset: name.range().start,
            default,
        });
    }

    declared
}

/// Return the loop variable node of a `for` statement.
fn loop_variable<'r>(node: &SourceNode<'r>) -> Option<SourceNode<'r>> {
    if let Some(left) = node.field("left").or_else(|| node.field("variable")) {
        return Some(left);
    }

    let children = node.children().collect::<Vec<_>>();
    let keyword = children.iter().position(|child| child.kind() == "in")?;
    children.get(keyword.checked_sub(1)?).cloned()
}

/// Return the iterated expression of a `for` statement.
fn loop_sequence<'r>(node: &SourceNode<'r>) -> Option<SourceNode<'r>> {
    if let Some(right) = node.field("right").or_else(|| node.field("sequence")) {
        return Some(right);
    }

    let children = node.children().collect::<Vec<_>>();
    let keyword = children.iter().position(|child| child.kind() == "in")?;
    children.get(keyword + 1).cloned()
}

/// Whether a node is a direct call to a named function.
fn calls_function(node: &SourceNode<'_>, name: &str) -> bool {
    if !matches!(node.kind().as_ref(), "call" | "call_expression") {
        return false;
    }

    let callee = node
        .field("function")
        .or_else(|| node.children().find(|child| child.is_named()));
    callee.is_some_and(|callee| callee.kind() == "identifier" && callee.text().trim() == name)
}

/// Which view of a mapping an iterated expression produces.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MappingView {
    /// The mapping's keys.
    Keys,

    /// The mapping's values.
    Values,

    /// Key and value pairs.
    Items,
}

/// Recognize `mapping.keys()`, `mapping.values()`, and `mapping.items()`.
fn mapping_view<'r>(
    language: CodeLanguage,
    node: &SourceNode<'r>,
) -> Option<(MappingView, SourceNode<'r>)> {
    if language != CodeLanguage::Python || node.kind() != "call" {
        return None;
    }

    let callee = node.field("function")?;
    if callee.kind() != "attribute" {
        return None;
    }

    let method = callee.field("attribute")?;
    let view = match method.text().trim() {
        "keys" => MappingView::Keys,
        "values" => MappingView::Values,
        "items" => MappingView::Items,
        _ => return None,
    };

    Some((view, callee.field("object")?))
}

/// Interpret a node as a literal collection, when it is one.
fn collection_literal_node<'r>(
    language: CodeLanguage,
    node: &SourceNode<'r>,
) -> Option<CollectionLiteral<'r>> {
    let kind = node.kind();
    match kind.as_ref() {
        "list" | "tuple" | "set" | "array" => Some(CollectionLiteral::Elements(
            node.children().filter(|child| child.is_named()).collect(),
        )),
        "dictionary" | "object" => Some(CollectionLiteral::Pairs(
            node.children()
                .filter(|child| child.kind() == "pair")
                .filter_map(|child| Some((child.field("key")?, child.field("value")?)))
                .collect(),
        )),
        "call" if language == CodeLanguage::R => {
            let callee = call_callee(node)?;
            (callee == "c").then(|| CollectionLiteral::Elements(call_arguments(language, node)))
        }
        _ => None,
    }
}

/// Return the separator a path-joining helper inserts between its arguments.
fn path_join_separator(callee: &str, name: &str) -> Option<&'static str> {
    match name {
        "join" if callee.contains("path") => Some("/"),
        "joinpath" => Some("/"),
        "paste0" => Some(""),
        _ if callee == "file.path" => Some("/"),
        _ => None,
    }
}
