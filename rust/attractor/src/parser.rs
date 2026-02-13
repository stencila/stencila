use indexmap::IndexMap;
use winnow::{
    ModalResult, Parser,
    ascii::{multispace0, multispace1},
    combinator::{alt, delimited, opt, peek, repeat, separated},
    token::{any, take_while},
};

use crate::error::{AttractorError, AttractorResult};
use crate::graph::{AttrValue, Edge, Graph, Node};
use crate::types::Duration;

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Parse a DOT digraph string into a [`Graph`].
///
/// Supports the subset of DOT syntax defined in the Attractor specification
/// (§2.1–2.13): directed graphs only, with typed attribute values, chained
/// edges, node/edge defaults, and subgraph scoping.
///
/// # Errors
///
/// Returns [`AttractorError::InvalidPipeline`] if the input is not valid DOT
/// or uses unsupported features (undirected graphs, `strict` modifier, etc.).
pub fn parse_dot(input: &str) -> AttractorResult<Graph> {
    // Pre-rejection checks before stripping comments
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err(AttractorError::InvalidPipeline {
            reason: "empty input".to_string(),
        });
    }
    if trimmed.starts_with("strict") {
        return Err(AttractorError::InvalidPipeline {
            reason: "strict modifier not supported".to_string(),
        });
    }
    if trimmed.starts_with("graph ") || trimmed.starts_with("graph\t") || trimmed == "graph" {
        return Err(AttractorError::InvalidPipeline {
            reason: "only directed graphs (digraph) are supported".to_string(),
        });
    }

    let cleaned = strip_comments(input)?;
    let mut remaining = cleaned.as_str();

    let (name, stmts) =
        parse_graph
            .parse_next(&mut remaining)
            .map_err(|e| AttractorError::InvalidPipeline {
                reason: format!("DOT parse error: {e}"),
            })?;

    // Check for trailing content (multiple graphs)
    let trailing = remaining.trim();
    if !trailing.is_empty() {
        return Err(AttractorError::InvalidPipeline {
            reason: "only one graph per file".to_string(),
        });
    }

    build_graph(name, &stmts)
}

// ---------------------------------------------------------------------------
// Comment stripping
// ---------------------------------------------------------------------------

/// Strip `//` line comments and `/* */` block comments, preserving content
/// inside quoted strings.
///
/// # Errors
///
/// Returns [`AttractorError::InvalidPipeline`] if a block comment is not terminated.
fn strip_comments(input: &str) -> AttractorResult<String> {
    let mut result = String::with_capacity(input.len());
    let chars: Vec<char> = input.chars().collect();
    let len = chars.len();
    let mut i = 0;

    while i < len {
        // Quoted string — pass through including any comment-like content
        if chars[i] == '"' {
            result.push('"');
            i += 1;
            while i < len {
                if chars[i] == '\\' && i + 1 < len {
                    result.push(chars[i]);
                    result.push(chars[i + 1]);
                    i += 2;
                } else if chars[i] == '"' {
                    result.push('"');
                    i += 1;
                    break;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
            }
        }
        // Line comment
        else if i + 1 < len && chars[i] == '/' && chars[i + 1] == '/' {
            // Skip to end of line
            i += 2;
            while i < len && chars[i] != '\n' {
                i += 1;
            }
        }
        // Block comment
        else if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
            i += 2;
            let mut depth = 1;
            while i < len && depth > 0 {
                if i + 1 < len && chars[i] == '/' && chars[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                } else if i + 1 < len && chars[i] == '*' && chars[i + 1] == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if depth > 0 {
                return Err(AttractorError::InvalidPipeline {
                    reason: "unterminated block comment".to_string(),
                });
            }
        }
        // Normal character
        else {
            result.push(chars[i]);
            i += 1;
        }
    }
    Ok(result)
}

// ---------------------------------------------------------------------------
// AST types (Pass 1 output)
// ---------------------------------------------------------------------------

/// A parsed DOT statement before graph construction.
#[derive(Debug, Clone)]
enum Statement {
    /// `graph [key=val, ...]`
    GraphAttr(IndexMap<String, AttrValue>),
    /// Top-level `key=val`
    GraphAttrDecl(String, AttrValue),
    /// `node [key=val, ...]`
    NodeDefaults(IndexMap<String, AttrValue>),
    /// `edge [key=val, ...]`
    EdgeDefaults(IndexMap<String, AttrValue>),
    /// A node declaration with optional attributes
    Node {
        id: String,
        attrs: IndexMap<String, AttrValue>,
    },
    /// An edge chain (A -> B -> C) with optional attributes
    Edge {
        chain: Vec<String>,
        attrs: IndexMap<String, AttrValue>,
    },
    /// A subgraph block
    Subgraph {
        name: Option<String>,
        stmts: Vec<Statement>,
    },
}

// ---------------------------------------------------------------------------
// Pass 1: Winnow parsers — DOT text → Vec<Statement>
// ---------------------------------------------------------------------------

/// Parse whitespace (spaces, tabs, newlines).
fn ws(input: &mut &str) -> ModalResult<()> {
    multispace0.void().parse_next(input)
}

/// Parse an optional semicolon (with surrounding whitespace).
fn opt_semi(input: &mut &str) -> ModalResult<()> {
    (ws, opt(';'), ws).void().parse_next(input)
}

/// DOT keywords that cannot be used as bare node identifiers.
const DOT_KEYWORDS: &[&str] = &["graph", "node", "edge", "subgraph", "digraph", "strict"];

/// Parse a bare identifier: `[A-Za-z_][A-Za-z0-9_]*`.
fn bare_identifier<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    (
        take_while(1, |c: char| c.is_ascii_alphabetic() || c == '_'),
        take_while(0.., |c: char| c.is_ascii_alphanumeric() || c == '_'),
    )
        .take()
        .parse_next(input)
}

/// Parse an identifier that is not a DOT keyword.
fn identifier<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    let checkpoint = *input;
    let id = bare_identifier.parse_next(input)?;
    if DOT_KEYWORDS.contains(&id) {
        *input = checkpoint;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    Ok(id)
}

/// Parse a node ID — bare identifier only per §2.3.
fn node_id(input: &mut &str) -> ModalResult<String> {
    identifier.map(String::from).parse_next(input)
}

/// Parse a qualified identifier: `identifier ('.' identifier)+` for dotted keys.
fn qualified_id<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    (
        bare_identifier,
        repeat(1.., ('.', bare_identifier)).fold(|| (), |(), _| ()),
    )
        .take()
        .parse_next(input)
}

/// Parse an attribute key — qualified or simple identifier.
fn attr_key<'s>(input: &mut &'s str) -> ModalResult<&'s str> {
    alt((qualified_id, bare_identifier)).parse_next(input)
}

// ---------------------------------------------------------------------------
// Value parsers
// ---------------------------------------------------------------------------

/// Parse a quoted string value, handling escape sequences.
fn quoted_string_value(input: &mut &str) -> ModalResult<String> {
    let _ = '"'.parse_next(input)?;
    let mut result = String::new();
    loop {
        // Try to take non-special characters in bulk
        let chunk: &str = take_while(0.., |c: char| c != '"' && c != '\\').parse_next(input)?;
        result.push_str(chunk);

        // Check what's next
        let next = peek(any).parse_next(input)?;
        if next == '"' {
            let _ = any.parse_next(input)?;
            return Ok(result);
        }
        // Must be backslash — only spec-defined escapes are allowed (§2.2)
        let _ = any.parse_next(input)?; // consume '\'
        let escaped = any.parse_next(input)?;
        match escaped {
            'n' => result.push('\n'),
            't' => result.push('\t'),
            '\\' => result.push('\\'),
            '"' => result.push('"'),
            _ => {
                return Err(winnow::error::ErrMode::Cut(
                    winnow::error::ContextError::new(),
                ));
            }
        }
    }
}

/// Check that the next character is not an identifier continuation (alphanumeric or `_`).
///
/// Returns `true` if at a word boundary (EOF or non-identifier char follows).
fn at_word_boundary(input: &mut &str) -> bool {
    let peeked: ModalResult<char> = peek(any).parse_next(input);
    !matches!(peeked, Ok(c) if c.is_ascii_alphanumeric() || c == '_')
}

/// Parse a boolean value (`true` or `false`), ensuring it's not a prefix of a longer word.
fn boolean_value(input: &mut &str) -> ModalResult<AttrValue> {
    let checkpoint = *input;
    let val = alt(("true".map(|_| true), "false".map(|_| false))).parse_next(input)?;

    if !at_word_boundary(input) {
        *input = checkpoint;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }
    Ok(AttrValue::Boolean(val))
}

/// Parse a duration value (integer followed by unit suffix).
fn duration_value(input: &mut &str) -> ModalResult<AttrValue> {
    let checkpoint = *input;
    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;
    let unit: &str = alt(("ms", "s", "m", "h", "d")).parse_next(input)?;

    if !at_word_boundary(input) {
        *input = checkpoint;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let spec_str = format!("{digits}{unit}");
    let dur = Duration::from_spec_str(&spec_str).map_err(|_| {
        *input = checkpoint;
        winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new())
    })?;
    Ok(AttrValue::Duration(dur))
}

/// Parse a float value: `-? [0-9]* '.' [0-9]+`.
fn float_value(input: &mut &str) -> ModalResult<AttrValue> {
    let checkpoint = *input;
    let neg: Option<&str> = opt("-").parse_next(input)?;
    let int_part: &str = take_while(0.., |c: char| c.is_ascii_digit()).parse_next(input)?;
    let _ = '.'.parse_next(input)?;
    let frac_part: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    let mut s = String::new();
    if neg.is_some() {
        s.push('-');
    }
    s.push_str(int_part);
    s.push('.');
    s.push_str(frac_part);

    let n: f64 = s.parse().map_err(|_| {
        *input = checkpoint;
        winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new())
    })?;
    Ok(AttrValue::Float(n))
}

/// Parse an integer value: `-? [0-9]+`.
fn integer_value(input: &mut &str) -> ModalResult<AttrValue> {
    let checkpoint = *input;
    let neg: Option<&str> = opt("-").parse_next(input)?;
    let digits: &str = take_while(1.., |c: char| c.is_ascii_digit()).parse_next(input)?;

    // Reject if followed by a dot (that's a float, not integer)
    if peek(opt('.')).parse_next(input)?.is_some() {
        *input = checkpoint;
        return Err(winnow::error::ErrMode::Backtrack(
            winnow::error::ContextError::new(),
        ));
    }

    let mut s = String::new();
    if neg.is_some() {
        s.push('-');
    }
    s.push_str(digits);

    let n: i64 = s.parse().map_err(|_| {
        *input = checkpoint;
        winnow::error::ErrMode::Backtrack(winnow::error::ContextError::new())
    })?;
    Ok(AttrValue::Integer(n))
}

/// Parse a bare identifier as a string value (e.g., `Mdiamond`, `box`, `LR`).
fn bare_identifier_value(input: &mut &str) -> ModalResult<AttrValue> {
    bare_identifier
        .map(|s: &str| AttrValue::String(s.to_string()))
        .parse_next(input)
}

/// Parse an attribute value.
///
/// Order matters: boolean before identifier-like, duration before plain integer,
/// float before integer. Bare identifiers are tried last as a fallback.
fn attr_value(input: &mut &str) -> ModalResult<AttrValue> {
    alt((
        quoted_string_value.map(AttrValue::String),
        boolean_value,
        duration_value,
        float_value,
        integer_value,
        bare_identifier_value,
    ))
    .parse_next(input)
}

// ---------------------------------------------------------------------------
// Attribute blocks
// ---------------------------------------------------------------------------

/// Parse a single `key = value` pair.
fn attr_pair(input: &mut &str) -> ModalResult<(String, AttrValue)> {
    let key = attr_key.parse_next(input)?;
    ws.parse_next(input)?;
    '='.parse_next(input)?;
    ws.parse_next(input)?;
    let value = attr_value.parse_next(input)?;
    Ok((key.to_string(), value))
}

/// Separator between attribute pairs: comma with surrounding whitespace per §2.3.
fn attr_sep(input: &mut &str) -> ModalResult<()> {
    (ws, ',', ws).void().parse_next(input)
}

/// Parse an attribute block: `[ key=val, key=val, ... ]`.
fn attr_block(input: &mut &str) -> ModalResult<IndexMap<String, AttrValue>> {
    delimited(
        ('[', ws),
        opt(separated(1.., attr_pair, attr_sep)).map(|pairs: Option<Vec<(String, AttrValue)>>| {
            pairs
                .unwrap_or_default()
                .into_iter()
                .collect::<IndexMap<String, AttrValue>>()
        }),
        (ws, ']'),
    )
    .parse_next(input)
}

// ---------------------------------------------------------------------------
// Statement parsers
// ---------------------------------------------------------------------------

/// Parse `keyword [attrs] ;?` — shared logic for graph/node/edge defaults.
fn keyword_attr_block(
    mut keyword: &'static str,
    input: &mut &str,
) -> ModalResult<IndexMap<String, AttrValue>> {
    keyword.parse_next(input)?;
    ws.parse_next(input)?;
    let attrs = attr_block.parse_next(input)?;
    opt_semi.parse_next(input)?;
    Ok(attrs)
}

/// Parse `graph [attrs]`.
fn graph_attr_stmt(input: &mut &str) -> ModalResult<Statement> {
    keyword_attr_block("graph", input).map(Statement::GraphAttr)
}

/// Parse `node [attrs]`.
fn node_defaults_stmt(input: &mut &str) -> ModalResult<Statement> {
    keyword_attr_block("node", input).map(Statement::NodeDefaults)
}

/// Parse `edge [attrs]`.
fn edge_defaults_stmt(input: &mut &str) -> ModalResult<Statement> {
    keyword_attr_block("edge", input).map(Statement::EdgeDefaults)
}

/// Parse a top-level attribute declaration: `key = value`.
fn graph_attr_decl(input: &mut &str) -> ModalResult<Statement> {
    let (key, value) = attr_pair.parse_next(input)?;
    opt_semi.parse_next(input)?;
    Ok(Statement::GraphAttrDecl(key, value))
}

/// Parse an optional attribute block, returning an empty map if absent.
fn parse_optional_attrs(input: &mut &str) -> ModalResult<IndexMap<String, AttrValue>> {
    opt(attr_block)
        .map(Option::unwrap_or_default)
        .parse_next(input)
}

/// Parse an edge statement: `id -> id (-> id)* [attrs]?`.
fn edge_stmt(input: &mut &str) -> ModalResult<Statement> {
    let first = node_id.parse_next(input)?;
    ws.parse_next(input)?;

    // Check for undirected edge
    let checkpoint = *input;
    let undirected: ModalResult<&str> = "--".parse_next(input);
    if undirected.is_ok() {
        return Err(winnow::error::ErrMode::Cut(
            winnow::error::ContextError::new(),
        ));
    }
    *input = checkpoint;

    "->".parse_next(input)?;
    ws.parse_next(input)?;

    let second = node_id.parse_next(input)?;
    ws.parse_next(input)?;

    let mut chain = vec![first, second];

    // Parse additional chained edges
    loop {
        let checkpoint = *input;
        let arrow: ModalResult<&str> = "->".parse_next(input);
        if arrow.is_ok() {
            ws.parse_next(input)?;
            let next = node_id.parse_next(input)?;
            ws.parse_next(input)?;
            chain.push(next);
        } else {
            *input = checkpoint;
            break;
        }
    }

    let attrs = parse_optional_attrs(input)?;
    opt_semi.parse_next(input)?;

    Ok(Statement::Edge { chain, attrs })
}

/// Parse a node statement: `id [attrs]?`.
fn node_stmt(input: &mut &str) -> ModalResult<Statement> {
    let id = node_id.parse_next(input)?;
    ws.parse_next(input)?;
    let attrs = parse_optional_attrs(input)?;
    opt_semi.parse_next(input)?;
    Ok(Statement::Node { id, attrs })
}

/// Parse a subgraph: `subgraph name? { stmts }`.
fn subgraph_stmt(input: &mut &str) -> ModalResult<Statement> {
    let _ = "subgraph".parse_next(input)?;
    ws.parse_next(input)?;
    let name = opt(bare_identifier.map(String::from)).parse_next(input)?;
    ws.parse_next(input)?;
    let stmts = delimited(
        ('{', ws),
        repeat(0.., statement).fold(Vec::new, |mut acc, s| {
            acc.push(s);
            acc
        }),
        (ws, '}'),
    )
    .parse_next(input)?;
    opt_semi.parse_next(input)?;
    Ok(Statement::Subgraph { name, stmts })
}

/// Parse any statement.
///
/// Order matters: try keyword-prefixed statements before node/edge to
/// disambiguate `graph [...]` from a node named `graph`.
fn statement(input: &mut &str) -> ModalResult<Statement> {
    ws.parse_next(input)?;
    alt((
        graph_attr_stmt,
        node_defaults_stmt,
        edge_defaults_stmt,
        subgraph_stmt,
        edge_stmt,
        graph_attr_decl,
        node_stmt,
    ))
    .parse_next(input)
}

/// Parse the top-level graph: `digraph name { stmts }`.
fn parse_graph(input: &mut &str) -> ModalResult<(String, Vec<Statement>)> {
    ws.parse_next(input)?;
    "digraph".parse_next(input)?;
    multispace1.parse_next(input)?;

    // Graph name must be a bare identifier per §2.2
    let name = bare_identifier.map(String::from).parse_next(input)?;

    ws.parse_next(input)?;

    let stmts = delimited(
        ('{', ws),
        repeat(0.., statement).fold(Vec::new, |mut acc, s| {
            acc.push(s);
            acc
        }),
        (ws, '}'),
    )
    .parse_next(input)?;

    ws.parse_next(input)?;

    Ok((name, stmts))
}

// ---------------------------------------------------------------------------
// Pass 2: Build graph from AST
// ---------------------------------------------------------------------------

/// Scope for tracking node/edge defaults during graph construction.
#[derive(Debug, Clone, Default)]
struct Scope {
    node_defaults: IndexMap<String, AttrValue>,
    edge_defaults: IndexMap<String, AttrValue>,
}

/// Build a [`Graph`] from the parsed statement AST.
fn build_graph(name: String, stmts: &[Statement]) -> AttractorResult<Graph> {
    let mut graph = Graph::new(name);
    let mut scope = Scope::default();
    process_statements(&mut graph, stmts, &mut scope, None, 0)?;
    Ok(graph)
}

/// Recursively process statements, maintaining scoped defaults.
///
/// `depth` tracks subgraph nesting: 0 at the root graph, incremented for each
/// subgraph level. Graph-level attributes (`graph [...]` and bare `key=value`)
/// are only written to the root graph (depth 0) — subgraph-internal graph attrs
/// are consumed solely by [`derive_subgraph_class`].
fn process_statements(
    graph: &mut Graph,
    stmts: &[Statement],
    scope: &mut Scope,
    subgraph_class: Option<&str>,
    depth: usize,
) -> AttractorResult<()> {
    for stmt in stmts {
        match stmt {
            Statement::GraphAttr(attrs) => {
                if depth == 0 {
                    extend_attrs(&mut graph.graph_attrs, attrs);
                }
            }
            Statement::GraphAttrDecl(key, value) => {
                if depth == 0 {
                    graph.graph_attrs.insert(key.clone(), value.clone());
                }
            }
            Statement::NodeDefaults(attrs) => {
                extend_attrs(&mut scope.node_defaults, attrs);
            }
            Statement::EdgeDefaults(attrs) => {
                extend_attrs(&mut scope.edge_defaults, attrs);
            }
            Statement::Node { id, attrs } => {
                insert_or_merge_node(graph, id, attrs, &scope.node_defaults, subgraph_class);
            }
            Statement::Edge { chain, attrs } => {
                // Expand chained edges: A -> B -> C → (A→B, B→C)
                for pair in chain.windows(2) {
                    let from = &pair[0];
                    let to = &pair[1];

                    // Ensure both endpoint nodes exist
                    ensure_node_exists(graph, from, &scope.node_defaults, subgraph_class);
                    ensure_node_exists(graph, to, &scope.node_defaults, subgraph_class);

                    // Build edge attrs: defaults first, then explicit attrs override
                    let merged = merge_attrs(&scope.edge_defaults, attrs);

                    let edge = Edge {
                        from: from.clone(),
                        to: to.clone(),
                        attrs: merged,
                    };
                    graph.add_edge(edge);
                }
            }
            Statement::Subgraph { name, stmts } => {
                // Derive class from subgraph label; inherit parent class
                // if this subgraph has no label of its own (§2.10).
                let child_class = derive_subgraph_class(name.as_deref(), stmts);
                let effective_class = child_class.as_deref().or(subgraph_class);

                // Push scope (clone current defaults for subgraph)
                let mut child_scope = scope.clone();

                process_statements(graph, stmts, &mut child_scope, effective_class, depth + 1)?;
            }
        }
    }
    Ok(())
}

/// Clone-insert all entries from `source` into `target`.
fn extend_attrs(target: &mut IndexMap<String, AttrValue>, source: &IndexMap<String, AttrValue>) {
    for (k, v) in source {
        target.insert(k.clone(), v.clone());
    }
}

/// Merge two attribute maps: `base` provides defaults, `overrides` wins on conflict.
fn merge_attrs(
    base: &IndexMap<String, AttrValue>,
    overrides: &IndexMap<String, AttrValue>,
) -> IndexMap<String, AttrValue> {
    let mut merged = base.clone();
    extend_attrs(&mut merged, overrides);
    merged
}

/// Insert or merge a node into the graph, applying defaults and subgraph class.
///
/// When the node already exists, only *explicit* attrs from this declaration are
/// applied (defaults do not overwrite prior explicit attrs per §2.10/§2.11).
/// Subgraph class is always appended regardless.
fn insert_or_merge_node(
    graph: &mut Graph,
    id: &str,
    explicit_attrs: &IndexMap<String, AttrValue>,
    node_defaults: &IndexMap<String, AttrValue>,
    subgraph_class: Option<&str>,
) {
    if let Some(existing) = graph.get_node_mut(id) {
        // Node already exists: only apply explicit attrs (not defaults)
        extend_attrs(&mut existing.attrs, explicit_attrs);
        // Always append subgraph class
        if let Some(class) = subgraph_class {
            append_class(&mut existing.attrs, class);
        }
    } else {
        // New node: defaults first, then explicit override
        let mut merged = merge_attrs(node_defaults, explicit_attrs);
        if let Some(class) = subgraph_class {
            append_class(&mut merged, class);
        }
        let mut node = Node::new(id);
        node.attrs = merged;
        graph.add_node(node);
    }
}

/// Append a class name to the `class` attribute, comma-separating if non-empty.
fn append_class(attrs: &mut IndexMap<String, AttrValue>, class: &str) {
    let existing = attrs.get("class").and_then(AttrValue::as_str).unwrap_or("");
    let new_class = if existing.is_empty() {
        class.to_string()
    } else {
        format!("{existing},{class}")
    };
    attrs.insert("class".to_string(), AttrValue::String(new_class));
}

/// Ensure a node exists in the graph, creating it with defaults if needed.
fn ensure_node_exists(
    graph: &mut Graph,
    id: &str,
    node_defaults: &IndexMap<String, AttrValue>,
    subgraph_class: Option<&str>,
) {
    if graph.get_node(id).is_none() {
        insert_or_merge_node(graph, id, &IndexMap::new(), node_defaults, subgraph_class);
    }
}

/// Derive a CSS-like class name from a subgraph label.
///
/// Lowercases, replaces spaces with hyphens, strips non-alphanumeric characters
/// (except hyphens).
fn derive_subgraph_class(_name: Option<&str>, stmts: &[Statement]) -> Option<String> {
    // Only labels derive classes per §2.10 — subgraph names do not.
    // Use last assignment (DOT semantics: last value wins).
    let label = stmts
        .iter()
        .filter_map(|s| match s {
            Statement::GraphAttr(attrs) => attrs
                .get("label")
                .and_then(AttrValue::as_str)
                .map(String::from),
            Statement::GraphAttrDecl(key, value) if key == "label" => {
                value.as_str().map(String::from)
            }
            _ => None,
        })
        .next_back();

    let source = label.as_deref()?;
    if source.is_empty() {
        return None;
    }

    let class: String = source
        .to_lowercase()
        .chars()
        .map(|c| if c == ' ' { '-' } else { c })
        .filter(|c| c.is_ascii_alphanumeric() || *c == '-')
        .collect();

    if class.is_empty() { None } else { Some(class) }
}
