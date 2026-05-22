use ast_grep_core::{matcher::NodeMatch, tree_sitter::StrDoc};

use super::{
    facts::{CodeFacts, record_definition, record_imported_symbol, record_use},
    language::CodeLanguage,
    util::{
        clean_string_literal, contains_identifier, first_identifier_owned, function_name,
        identifier_target, is_ignored_identifier, javascript_package_name, package_name,
        rust_imported_symbol, rust_package_name,
    },
};

/// Dispatch one ast-grep match to its language-specific normalizer.
///
/// Rule ids are the stable bridge between YAML files and Rust code. Matching on
/// ids here keeps each normalizer small and makes unsupported or future rules a
/// no-op until graph semantics are added for them.
pub(super) fn normalize_match(
    language: CodeLanguage,
    rule_id: &str,
    matched: &NodeMatch<StrDoc<CodeLanguage>>,
    facts: &mut CodeFacts,
) {
    match (language, rule_id) {
        (language, "ecmascript-import") if language.is_ecmascript() => {
            normalize_javascript_import(matched, facts)
        }
        (language, "ecmascript-assignment") if language.is_ecmascript() => {
            normalize_assignment(matched, facts)
        }
        (language, "ecmascript-function") if language.is_ecmascript() => {
            normalize_declaration(matched, "NAME", facts)
        }
        (language, "ecmascript-read") if language.is_ecmascript() => {
            normalize_io_match(matched, facts, true)
        }
        (language, "ecmascript-write") if language.is_ecmascript() => {
            normalize_io_match(matched, facts, false)
        }
        (language, "ecmascript-call") if language.is_ecmascript() => {
            normalize_call(matched, "FUNC", facts)
        }
        (CodeLanguage::Rust, "rust-import") => normalize_rust_import(matched, facts),
        (CodeLanguage::Rust, "rust-assignment") => normalize_assignment(matched, facts),
        (CodeLanguage::Rust, "rust-function") => normalize_declaration(matched, "NAME", facts),
        (CodeLanguage::Rust, "rust-read") => normalize_io_match(matched, facts, true),
        (CodeLanguage::Rust, "rust-write") => normalize_io_match(matched, facts, false),
        (CodeLanguage::Rust, "rust-call") => normalize_call(matched, "FUNC", facts),
        (CodeLanguage::Python, "python-import") => normalize_python_import(matched, facts),
        (CodeLanguage::Python, "python-assignment") => normalize_assignment(matched, facts),
        (CodeLanguage::Python, "python-function") => normalize_declaration(matched, "NAME", facts),
        (CodeLanguage::Python, "python-read") => normalize_io_match(matched, facts, true),
        (CodeLanguage::Python, "python-write") => normalize_io_match(matched, facts, false),
        (CodeLanguage::Python, "python-call") => normalize_call(matched, "FUNC", facts),
        (CodeLanguage::R, "r-import") => normalize_r_import(matched, facts),
        (CodeLanguage::R, "r-assignment") => normalize_assignment(matched, facts),
        (CodeLanguage::R, "r-read") => normalize_io_match(matched, facts, true),
        (CodeLanguage::R, "r-write") => normalize_io_match(matched, facts, false),
        (CodeLanguage::R, "r-call") => normalize_call(matched, "FUNC", facts),
        (CodeLanguage::Snakemake, "snakemake-rule") => {
            if let Some(node) = matched.field("name") {
                let name = node.text().into_owned();
                facts.record_workflow_rule(name, Some(node.range().start));
            }
        }
        _ => {}
    }
}

/// Normalize a Python import match.
///
/// Imports produce package facts, but their local aliases must also be tracked
/// so identifiers such as `pd` or `plt` do not become false cross-chunk symbol
/// dependencies.
fn normalize_python_import(matched: &NodeMatch<StrDoc<CodeLanguage>>, facts: &mut CodeFacts) {
    if let Some(module) = env_text(matched, "MODULE").and_then(|module| package_name(&module)) {
        facts.imports.insert(module);
    }

    if let Some(alias) = env_text(matched, "ALIAS").and_then(|alias| first_identifier_owned(&alias))
    {
        record_imported_symbol(facts, alias, env_range_start(matched, "ALIAS"));
    } else if let Some(module) =
        env_text(matched, "MODULE").and_then(|module| first_identifier_owned(&module))
    {
        record_imported_symbol(facts, module, env_range_start(matched, "MODULE"));
    }
}

/// Normalize a JavaScript or TypeScript import match.
///
/// Imports can name runtime packages, local aliases, or CommonJS require
/// targets. Package facts are limited to bare module specifiers so relative
/// module paths do not become misleading software package nodes.
fn normalize_javascript_import(matched: &NodeMatch<StrDoc<CodeLanguage>>, facts: &mut CodeFacts) {
    if let Some(path) = env_text(matched, "PATH")
        && let Some(package) = javascript_package_name(&path)
    {
        facts.imports.insert(package);
    }

    for var in ["ALIAS", "TARGET"] {
        if let Some(alias) = env_text(matched, var).and_then(|alias| first_identifier_owned(&alias))
        {
            record_imported_symbol(facts, alias, env_range_start(matched, var));
        }
    }
}

/// Normalize an R import or namespace-qualified call match.
///
/// Both `library(pkg)` and `pkg::fn` identify software packages statically. The
/// package root is enough for graph-level package relationships.
fn normalize_r_import(matched: &NodeMatch<StrDoc<CodeLanguage>>, facts: &mut CodeFacts) {
    for name in ["PKG", "MODULE"] {
        if let Some(package) = env_text(matched, name).and_then(|package| package_name(&package)) {
            facts.imports.insert(package);
        }
    }
}

/// Normalize a Rust use item or extern crate match.
///
/// Rust imports identify Cargo-style package roots when the path starts outside
/// `std`, `core`, local crate modules, and other prelude-like namespaces. The
/// imported terminal name is also recorded so use items do not create symbol-use
/// noise in document reactivity.
fn normalize_rust_import(matched: &NodeMatch<StrDoc<CodeLanguage>>, facts: &mut CodeFacts) {
    let Some(module_node) = matched.get_env().get_match("MODULE") else {
        return;
    };
    let module = module_node.text();
    let module = module.trim();

    if let Some(package) = rust_package_name(module) {
        facts.imports.insert(package);
    }

    if let Some(symbol) = rust_imported_symbol(module) {
        record_imported_symbol(facts, symbol, Some(module_node.range().start));
    }
}

/// Normalize an assignment match.
///
/// Assignment targets define symbols. When the assignment came from a read rule
/// with a captured path, the target is also recorded as a dataframe-like source
/// so later column accesses can derive from the same static file.
fn normalize_assignment(matched: &NodeMatch<StrDoc<CodeLanguage>>, facts: &mut CodeFacts) {
    let Some(target_node) = matched.get_env().get_match("TARGET") else {
        return;
    };
    let target_text = target_node.text();
    let Some(target) = identifier_target(target_text.trim()) else {
        return;
    };
    if env_text(matched, "VALUE").is_some_and(|value| contains_identifier(&value, &target))
        && facts
            .definition_offsets
            .get(&target)
            .is_none_or(|definition_offset| *definition_offset >= target_node.range().start)
    {
        facts.read_before_write_symbols.insert(target.clone());
    }
    facts.assignments.insert(target.clone());
    record_definition(facts, &target, target_node.range().start);

    if let Some(path) = env_text(matched, "PATH").and_then(|path| clean_string_literal(&path)) {
        facts.variable_sources.insert(target, path);
    }
}

/// Normalize a function or workflow declaration match.
///
/// Declarations are kept distinct from assignments because the graph should
/// represent declared callables as functions or workflow rules, not variables.
fn normalize_declaration(
    matched: &NodeMatch<StrDoc<CodeLanguage>>,
    var: &str,
    facts: &mut CodeFacts,
) {
    if let Some(node) = matched.get_env().get_match(var) {
        let name = node.text();
        if let Some(name) = identifier_target(name.trim()) {
            facts.declarations.insert(name.to_string());
            record_definition(facts, &name, node.range().start);
        }
    }
}

/// Normalize a function-like call match.
///
/// Calls are intentionally shallow: the callee name is useful for graph queries
/// even when static analysis cannot determine whether it is local, imported, or
/// dynamically rebound.
fn normalize_call(matched: &NodeMatch<StrDoc<CodeLanguage>>, var: &str, facts: &mut CodeFacts) {
    let Some(name) = env_text(matched, var).and_then(|name| function_name(&name)) else {
        return;
    };
    facts.calls.insert(name);
}

/// Normalize a static read or write match.
///
/// IO rules capture string literals and sometimes a target assignment. This
/// helper classifies `open`-style modes, records the resource direction, and
/// links assigned variables back to read paths when a dataframe-like source is
/// statically visible.
fn normalize_io_match(
    matched: &NodeMatch<StrDoc<CodeLanguage>>,
    facts: &mut CodeFacts,
    default_read: bool,
) {
    let Some(path) = env_text(matched, "PATH").and_then(|path| clean_string_literal(&path)) else {
        return;
    };

    let is_write = env_text(matched, "MODE")
        .and_then(|mode| clean_string_literal(&mode))
        .is_some_and(|mode| mode.contains(['w', 'a', 'x', '+']));

    if default_read && !is_write {
        facts.reads.insert(path.clone());
    } else {
        facts.writes.insert(path.clone());
    }

    if let Some(target_node) = matched.get_env().get_match("TARGET") {
        let target_text = target_node.text();
        if let Some(target) = identifier_target(target_text.trim()) {
            facts.assignments.insert(target.to_string());
            record_definition(facts, &target, target_node.range().start);
            facts.variable_sources.insert(target.to_string(), path);
        }
    }
}

/// Collect identifier uses from the parsed syntax tree.
///
/// Rule matches find declarations and known operations, but document reactivity
/// needs a broader symbol-use signal. Walking identifiers gives that signal,
/// and later filtering removes local definitions, imports, and common builtins.
pub(super) fn collect_identifier_uses(
    language: CodeLanguage,
    grep: &ast_grep_core::AstGrep<StrDoc<CodeLanguage>>,
    facts: &mut CodeFacts,
) {
    for node in grep.root().dfs() {
        if node.kind().as_ref() != "identifier" {
            continue;
        }
        let name = node.text();
        let name = name.trim();
        if is_ignored_identifier(language, name) || is_definition_identifier(&node) {
            continue;
        }
        record_use(facts, name, node.range().start);
    }
}

/// Return the start offset of a captured metavariable.
fn env_range_start(matched: &NodeMatch<StrDoc<CodeLanguage>>, var: &str) -> Option<usize> {
    matched
        .get_env()
        .get_match(var)
        .map(|node| node.range().start)
}

/// Check whether an identifier node is part of a definition site.
///
/// Definition identifiers should not be counted as uses of themselves. The
/// checks are deliberately limited to grammar shapes this module already
/// handles, which keeps the fallback conservative for unsupported constructs.
fn is_definition_identifier(node: &ast_grep_core::Node<StrDoc<CodeLanguage>>) -> bool {
    let Some(parent) = node.parent() else {
        return false;
    };

    match parent.kind().as_ref() {
        "assignment" | "augmented_assignment" => parent
            .field("left")
            .is_some_and(|left| left.range().contains(&node.range().start)),
        "variable_declarator" => parent
            .field("name")
            .is_some_and(|name| name.range().contains(&node.range().start)),
        "let_declaration" => parent
            .field("pattern")
            .is_some_and(|pattern| pattern.range().contains(&node.range().start)),
        "function_declaration"
        | "function_definition"
        | "function_item"
        | "rule_definition"
        | "checkpoint_definition" => parent
            .field("name")
            .is_some_and(|name| name.range() == node.range()),
        "aliased_import" | "import_clause" | "namespace_import" | "import_specifier" => parent
            .field("alias")
            .or_else(|| parent.field("name"))
            .is_some_and(|alias| alias.range().contains(&node.range().start)),
        _ => false,
    }
}

/// Read a captured ast-grep metavariable as trimmed text.
///
/// Normalizers operate on source snippets instead of AST node types so YAML
/// rules can stay small. Centralizing trimming keeps capture handling
/// consistent across languages.
fn env_text(matched: &NodeMatch<StrDoc<CodeLanguage>>, var: &str) -> Option<String> {
    matched
        .get_env()
        .get_match(var)
        .map(|node| node.text().trim().to_string())
}
