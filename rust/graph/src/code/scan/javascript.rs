use super::super::{
    facts::{CodeFacts, record_imported_symbol},
    util::{first_identifier_owned, first_static_string_literal, javascript_package_name},
};

/// Collect JavaScript and TypeScript module imports from source text.
///
/// tree-sitter represents ES import clauses with several node shapes, and
/// ast-grep pattern snippets for those clauses are brittle across JavaScript and
/// TypeScript. This scanner handles the static module specifier and common local
/// aliases directly while leaving assignments and IO to ast-grep rules.
pub(in crate::code) fn collect_javascript_text_imports(source: &str, facts: &mut CodeFacts) {
    let mut offset = 0usize;
    for raw_line in source.lines() {
        let line = raw_line.trim();
        let line_offset = offset + raw_line.find(line).unwrap_or_default();

        if let Some(import_body) = line.strip_prefix("import ") {
            if let Some(specifier) = javascript_import_specifier(import_body)
                && let Some(package) = javascript_package_name(&specifier)
            {
                facts.imports.insert(package);
            }
            record_javascript_import_aliases(import_body, line_offset, facts);
        }

        if let Some(require_specifier) = javascript_require_specifier(line)
            && let Some(package) = javascript_package_name(&require_specifier)
        {
            facts.imports.insert(package);
        }
        if let Some(target) = javascript_require_target(line) {
            let target_offset = line.find(&target).map(|index| line_offset + index);
            record_imported_symbol(facts, target, target_offset);
        }

        offset += raw_line.len() + 1;
    }
}

/// Extract the module specifier from an ES import clause.
fn javascript_import_specifier(import_body: &str) -> Option<String> {
    let specifier_source = import_body
        .split_once(" from ")
        .map(|(_, specifier)| specifier)
        .unwrap_or(import_body);
    first_static_string_literal(specifier_source)
}

/// Extract the module specifier from a CommonJS require call.
fn javascript_require_specifier(line: &str) -> Option<String> {
    let start = line.find("require(")? + "require(".len();
    first_static_string_literal(&line[start..])
}

/// Extract the local binding target from a CommonJS require assignment.
fn javascript_require_target(line: &str) -> Option<String> {
    let (left, right) = line.split_once('=')?;
    if !right.contains("require(") {
        return None;
    }
    let left = left
        .trim()
        .strip_prefix("const ")
        .or_else(|| left.trim().strip_prefix("let "))
        .or_else(|| left.trim().strip_prefix("var "))
        .unwrap_or_else(|| left.trim());
    first_identifier_owned(left)
}

/// Record local aliases introduced by an ES import clause.
fn record_javascript_import_aliases(import_body: &str, line_offset: usize, facts: &mut CodeFacts) {
    let binding_source = import_body
        .split_once(" from ")
        .map(|(bindings, _)| bindings.trim())
        .unwrap_or_default();
    if binding_source.is_empty() || binding_source.starts_with(['\'', '"']) {
        return;
    }

    if let Some(alias_source) = binding_source.strip_prefix("* as ") {
        if let Some(alias) = first_identifier_owned(alias_source) {
            let offset = import_body
                .find(&alias)
                .map(|index| line_offset + "import ".len() + index);
            record_imported_symbol(facts, alias, offset);
        }
        return;
    }

    if binding_source.starts_with('{') {
        let Some(end) = binding_source.find('}') else {
            return;
        };
        for part in binding_source[1..end].split(',') {
            let alias_source = part.split(" as ").last().unwrap_or(part).trim();
            if let Some(alias) = first_identifier_owned(alias_source) {
                let offset = import_body
                    .find(&alias)
                    .map(|index| line_offset + "import ".len() + index);
                record_imported_symbol(facts, alias, offset);
            }
        }
        return;
    }

    if let Some(alias) = first_identifier_owned(binding_source) {
        let offset = import_body
            .find(&alias)
            .map(|index| line_offset + "import ".len() + index);
        record_imported_symbol(facts, alias, offset);
    }
}
