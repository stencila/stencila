use std::collections::BTreeSet;

use ast_grep_config::{GlobalRules, from_yaml_string};
use ast_grep_core::tree_sitter::{LanguageExt, StrDoc};

use super::{
    facts::{CodeFacts, should_retain_use},
    language::CodeLanguage,
    normalize::{collect_identifier_uses, normalize_match},
    scan::{
        collect_column_facts, collect_javascript_text_imports, collect_nextflow_text_facts,
        collect_snakemake_text_facts,
    },
    util::is_ignored_identifier,
};

/// Analyze a string of source code into static graph facts.
///
/// This is the parser-facing entry point used by both tests and graph
/// collectors. It keeps analysis side-effect free: source text goes in, a
/// deterministic set of normalized facts comes out, and callers decide how to
/// project those facts into a graph.
///
/// Sources with tree-sitter syntax errors return an otherwise empty fact set
/// with `syntax_error` marked. That avoids emitting misleading partial
/// dependencies from malformed code while still letting callers represent the
/// code unit itself.
pub fn analyze_source(language: CodeLanguage, source: &str) -> CodeFacts {
    if language == CodeLanguage::Nextflow {
        let mut facts = CodeFacts::default();
        collect_nextflow_text_facts(source, &mut facts);
        return finish_facts(language, facts);
    }

    let grep = language.ast_grep(source);
    let mut facts = CodeFacts::default();

    if grep.root().dfs().any(|node| node.is_error()) {
        facts.syntax_error = true;
        return facts;
    }

    collect_rule_facts(language, &grep, &mut facts);
    collect_identifier_uses(language, &grep, &mut facts);
    collect_column_facts(language, source, &mut facts);
    finish_facts(language, facts)
}

/// Apply cross-language filtering after extraction.
///
/// Normalizers deliberately over-collect simple facts. This final pass removes
/// local definitions, import aliases, and builtins from the use/call sets so the
/// projected graph focuses on useful dependencies rather than parser trivia.
fn finish_facts(language: CodeLanguage, mut facts: CodeFacts) -> CodeFacts {
    let local_definitions = facts
        .assignments
        .iter()
        .chain(&facts.declarations)
        .chain(&facts.imported_symbols)
        .cloned()
        .collect::<BTreeSet<_>>();

    facts.uses = facts
        .uses
        .iter()
        .filter(|name| {
            !is_ignored_identifier(language, name)
                && !facts.imports.iter().any(|package| package.name == **name)
                && should_retain_use(name, &local_definitions, &facts)
        })
        .cloned()
        .collect();
    facts
        .calls
        .retain(|name| !is_ignored_identifier(language, name));

    facts
}

/// Run the embedded ast-grep rules and normalize every match.
///
/// Rule YAMLs carry the syntactic search patterns, while Rust normalization
/// decides how captured metavariables become graph facts. That split keeps
/// pattern maintenance declarative without scattering graph semantics across
/// configuration files.
fn collect_rule_facts(
    language: CodeLanguage,
    grep: &ast_grep_core::AstGrep<StrDoc<CodeLanguage>>,
    facts: &mut CodeFacts,
) {
    let Ok(configs) = from_yaml_string::<CodeLanguage>(language.rules(), &GlobalRules::default())
    else {
        return;
    };

    for config in configs {
        for matched in grep.root().find_all(&config.matcher) {
            normalize_match(language, config.id.as_str(), &matched, facts);
        }
    }

    if language.is_ecmascript() {
        collect_javascript_text_imports(grep.root().text().as_ref(), facts);
    }

    if language == CodeLanguage::Snakemake {
        collect_snakemake_text_facts(grep.root().text().as_ref(), facts);
    }
}
