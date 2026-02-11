use std::collections::HashMap;

use crate::error::CodemodeError;

/// JavaScript reserved words per §6.1.
///
/// If a tool name maps to one of these after character replacement,
/// the identifier MUST have `_` appended.
const JS_RESERVED_WORDS: &[&str] = &[
    "await",
    "break",
    "case",
    "class",
    "const",
    "continue",
    "debugger",
    "default",
    "delete",
    "do",
    "else",
    "export",
    "extends",
    "false",
    "finally",
    "for",
    "function",
    "if",
    "import",
    "in",
    "instanceof",
    "let",
    "new",
    "null",
    "return",
    "static",
    "super",
    "switch",
    "this",
    "throw",
    "true",
    "try",
    "typeof",
    "var",
    "void",
    "while",
    "with",
    "yield",
];

/// Whether a character is a valid JavaScript identifier character.
///
/// Per ECMAScript, identifier characters include Unicode letters, digits,
/// `_`, `$`, and combining marks. We use Rust's `char::is_alphanumeric()`
/// which covers Unicode letters and digits (a superset of `ID_Continue`
/// minus exotic combining marks, which is sufficient for MCP tool names).
fn is_js_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_' || c == '$'
}

/// Map a single MCP tool name to a JavaScript export identifier per §6.1.
///
/// Rules applied in order:
/// 1. Replace illegal identifier characters with `_`
/// 2. If result starts with a digit, prepend `_`
/// 3. If result is a JS reserved word, append `_`
///
/// This does NOT handle collisions — use [`resolve_export_collisions`] for that.
pub fn tool_name_to_export(name: &str) -> String {
    // Step 1: Replace illegal identifier characters with `_`.
    let replaced: String = name
        .chars()
        .map(|c| if is_js_identifier_char(c) { c } else { '_' })
        .collect();

    // Handle empty string edge case
    let replaced = if replaced.is_empty() {
        "_".to_string()
    } else {
        replaced
    };

    // Step 2: If starts with digit, prepend `_`
    let replaced = if replaced.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{replaced}")
    } else {
        replaced
    };

    // Step 3: If reserved word, append `_`
    if JS_RESERVED_WORDS.contains(&replaced.as_str()) {
        format!("{replaced}_")
    } else {
        replaced
    }
}

/// Resolve export name collisions for a set of tools per §6.1.
///
/// Input: slice of canonical MCP tool names.
/// Output: vec of `(canonical_name, export_name)` pairs.
///
/// Tools are sorted alphabetically by canonical name before assigning identifiers.
/// First occurrence keeps the clean name; subsequent collisions get `__N` suffix
/// (N starts at 2).
pub fn resolve_export_collisions(names: &[&str]) -> Vec<(String, String)> {
    // Sort alphabetically by canonical name for deterministic ordering
    let mut sorted: Vec<&str> = names.to_vec();
    sorted.sort();

    let mapped: Vec<(String, String)> = sorted
        .iter()
        .map(|&name| (name.to_string(), tool_name_to_export(name)))
        .collect();

    disambiguate(mapped, "__")
}

/// Normalize a server ID to a valid module path segment per §5.0.1.
///
/// Rules applied in order:
/// 1. Lowercase the entire string
/// 2. Replace characters outside `[a-z0-9-]` with `-`
/// 3. Collapse consecutive `-` to a single `-`
/// 4. Strip leading and trailing `-`
///
/// Returns an error if the result is empty (all characters were invalid).
pub fn normalize_server_id(id: &str) -> Result<String, CodemodeError> {
    // Step 1: Lowercase
    let lowered = id.to_lowercase();

    // Step 2: Replace non-[a-z0-9-] with `-`
    let replaced: String = lowered
        .chars()
        .map(|c| {
            if c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' {
                c
            } else {
                '-'
            }
        })
        .collect();

    // Step 3: Collapse consecutive `-`
    let mut collapsed = String::with_capacity(replaced.len());
    let mut prev_dash = false;
    for c in replaced.chars() {
        if c == '-' {
            if !prev_dash {
                collapsed.push(c);
            }
            prev_dash = true;
        } else {
            collapsed.push(c);
            prev_dash = false;
        }
    }

    // Step 4: Strip leading and trailing `-`
    let normalized = collapsed.trim_matches('-').to_string();

    if normalized.is_empty() {
        return Err(CodemodeError::InvalidServerId {
            server_id: id.to_string(),
        });
    }

    Ok(normalized)
}

/// Resolve server ID collisions after normalization per §5.0.1.
///
/// Input: slice of original server IDs.
/// Output: vec of `(original_id, normalized_id)` pairs.
///
/// If two servers produce the same normalized path, the first occurrence
/// (alphabetically by original ID) keeps the clean path; subsequent
/// collisions get `--N` suffix (N starts at 2).
///
/// Returns an error if any server ID normalizes to an empty string.
pub fn resolve_server_collisions(ids: &[&str]) -> Result<Vec<(String, String)>, CodemodeError> {
    // Sort alphabetically by original ID for deterministic ordering
    let mut sorted: Vec<&str> = ids.to_vec();
    sorted.sort();

    let mut mapped = Vec::with_capacity(sorted.len());
    for &id in &sorted {
        mapped.push((id.to_string(), normalize_server_id(id)?));
    }

    Ok(disambiguate(mapped, "--"))
}

/// Shared collision disambiguation: given `(original, mapped)` pairs (already
/// in deterministic order), append `{separator}{N}` to duplicates (N starts at 2,
/// first occurrence keeps the clean name).
fn disambiguate(pairs: Vec<(String, String)>, separator: &str) -> Vec<(String, String)> {
    let mut seen: HashMap<String, u32> = HashMap::new();
    let mut result = Vec::with_capacity(pairs.len());

    for (original, mapped) in pairs {
        let count = seen.entry(mapped.clone()).or_insert(0);
        *count += 1;

        let final_mapped = if *count == 1 {
            mapped
        } else {
            format!("{mapped}{separator}{count}")
        };
        result.push((original, final_mapped));
    }

    result
}
