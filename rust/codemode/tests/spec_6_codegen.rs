use stencila_codemode::{
    CodemodeError, normalize_server_id, resolve_export_collisions, resolve_server_collisions,
    tool_name_to_export,
};

// ============================================================
// §6.1 Identifier Mapping — tool_name_to_export
// ============================================================

#[test]
fn identity_passthrough() {
    assert_eq!(tool_name_to_export("readFile"), "readFile");
    assert_eq!(tool_name_to_export("search"), "search");
    assert_eq!(tool_name_to_export("a1b2"), "a1b2");
}

#[test]
fn illegal_chars_replaced_with_underscore() {
    assert_eq!(tool_name_to_export("read-file"), "read_file");
    assert_eq!(tool_name_to_export("read.file"), "read_file");
    assert_eq!(tool_name_to_export("ns::tool"), "ns__tool");
    assert_eq!(tool_name_to_export("a b c"), "a_b_c");
    assert_eq!(tool_name_to_export("tool@v2"), "tool_v2");
}

#[test]
fn unicode_letters_preserved() {
    // Unicode letters are valid JS identifier chars and should not be replaced
    assert_eq!(tool_name_to_export("überTool"), "überTool");
    assert_eq!(tool_name_to_export("名前"), "名前");
    assert_eq!(tool_name_to_export("café"), "café");
    assert_eq!(tool_name_to_export("données"), "données");
}

#[test]
fn digit_prefix_gets_underscore() {
    assert_eq!(tool_name_to_export("123tool"), "_123tool");
    assert_eq!(tool_name_to_export("0"), "_0");
    assert_eq!(tool_name_to_export("9lives"), "_9lives");
}

#[test]
fn reserved_words_get_trailing_underscore() {
    let reserved = [
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
    for word in reserved {
        let export = tool_name_to_export(word);
        assert_eq!(export, format!("{word}_"), "reserved word: {word}");
    }
}

#[test]
fn non_reserved_not_suffixed() {
    // Words that look similar but are NOT reserved
    assert_eq!(tool_name_to_export("async"), "async"); // not in the spec's list
    assert_eq!(tool_name_to_export("constructor"), "constructor");
    assert_eq!(tool_name_to_export("undefined"), "undefined");
}

#[test]
fn dollar_sign_preserved() {
    assert_eq!(tool_name_to_export("$helper"), "$helper");
    assert_eq!(tool_name_to_export("get$"), "get$");
}

#[test]
fn combined_rules_digit_after_replacement() {
    // "1-tool" → "1_tool" (illegal char) → "_1_tool" (digit prefix)
    assert_eq!(tool_name_to_export("1-tool"), "_1_tool");
}

// ============================================================
// §6.1 Collision Resolution — resolve_export_collisions
// ============================================================

#[test]
fn no_collisions() {
    let result = resolve_export_collisions(&["alpha", "beta", "gamma"]);
    assert_eq!(result.len(), 3);
    assert_eq!(result[0], ("alpha".into(), "alpha".into()));
    assert_eq!(result[1], ("beta".into(), "beta".into()));
    assert_eq!(result[2], ("gamma".into(), "gamma".into()));
}

#[test]
fn two_way_collision() {
    let result = resolve_export_collisions(&["read.file", "read-file"]);
    // Alphabetical: "read-file" < "read.file"
    assert_eq!(result[0], ("read-file".into(), "read_file".into()));
    assert_eq!(result[1], ("read.file".into(), "read_file__2".into()));
}

#[test]
fn three_way_collision() {
    let result = resolve_export_collisions(&["a.b", "a-b", "a b"]);
    // Alphabetical: "a b" < "a-b" < "a.b"
    assert_eq!(result[0], ("a b".into(), "a_b".into()));
    assert_eq!(result[1], ("a-b".into(), "a_b__2".into()));
    assert_eq!(result[2], ("a.b".into(), "a_b__3".into()));
}

#[test]
fn collision_ordering_is_deterministic() {
    let fwd = resolve_export_collisions(&["z-x", "a-x"]);
    let rev = resolve_export_collisions(&["a-x", "z-x"]);
    assert_eq!(fwd, rev);
}

#[test]
fn mixed_collision_and_non_collision() {
    let result = resolve_export_collisions(&["foo-bar", "foo.bar", "unique"]);
    assert_eq!(result[0], ("foo-bar".into(), "foo_bar".into()));
    assert_eq!(result[1], ("foo.bar".into(), "foo_bar__2".into()));
    assert_eq!(result[2], ("unique".into(), "unique".into()));
}

// ============================================================
// §5.0.1 Server ID Normalization — normalize_server_id
// ============================================================

#[test]
fn server_id_passthrough() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("google-drive")?, "google-drive");
    assert_eq!(normalize_server_id("server1")?, "server1");
    Ok(())
}

#[test]
fn server_id_uppercase_lowered() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("Google-Drive")?, "google-drive");
    assert_eq!(normalize_server_id("ALLCAPS")?, "allcaps");
    assert_eq!(normalize_server_id("CamelCase")?, "camelcase");
    Ok(())
}

#[test]
fn server_id_special_chars_replaced() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("my_server")?, "my-server");
    assert_eq!(normalize_server_id("ns::server")?, "ns-server");
    assert_eq!(normalize_server_id("server.v2")?, "server-v2");
    assert_eq!(normalize_server_id("a b c")?, "a-b-c");
    Ok(())
}

#[test]
fn server_id_consecutive_dashes_collapsed() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("a---b")?, "a-b");
    assert_eq!(normalize_server_id("a__b")?, "a-b");
    assert_eq!(normalize_server_id("a-_-b")?, "a-b");
    Ok(())
}

#[test]
fn server_id_leading_trailing_stripped() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("-server-")?, "server");
    assert_eq!(normalize_server_id("--server--")?, "server");
    assert_eq!(normalize_server_id("___server___")?, "server");
    Ok(())
}

#[test]
fn server_id_complex_mixed() -> Result<(), CodemodeError> {
    assert_eq!(normalize_server_id("My Server (v2.1)")?, "my-server-v2-1");
    assert_eq!(normalize_server_id("@scope/package")?, "scope-package");
    Ok(())
}

#[test]
fn server_id_all_invalid_chars_returns_invalid_server_id_error() {
    // Verify the correct error variant is returned
    for input in &["___", "---", "", "@#$"] {
        match normalize_server_id(input) {
            Err(CodemodeError::InvalidServerId { server_id }) => {
                assert_eq!(server_id, *input);
            }
            other => panic!("expected InvalidServerId for {input:?}, got {other:?}"),
        }
    }
}

// ============================================================
// §5.0.1 Server Collision Resolution — resolve_server_collisions
// ============================================================

#[test]
fn server_no_collisions() -> Result<(), CodemodeError> {
    let result = resolve_server_collisions(&["server-a", "server-b"])?;
    assert_eq!(result[0], ("server-a".into(), "server-a".into()));
    assert_eq!(result[1], ("server-b".into(), "server-b".into()));
    Ok(())
}

#[test]
fn server_collision_disambiguation() -> Result<(), CodemodeError> {
    let result = resolve_server_collisions(&["Server_A", "server-a"])?;
    // Alphabetical: "Server_A" < "server-a"
    assert_eq!(result[0], ("Server_A".into(), "server-a".into()));
    assert_eq!(result[1], ("server-a".into(), "server-a--2".into()));
    Ok(())
}

#[test]
fn server_collision_deterministic() -> Result<(), CodemodeError> {
    let fwd = resolve_server_collisions(&["server-a", "Server_A"])?;
    let rev = resolve_server_collisions(&["Server_A", "server-a"])?;
    assert_eq!(fwd, rev);
    Ok(())
}

#[test]
fn server_three_way_collision() -> Result<(), CodemodeError> {
    let result = resolve_server_collisions(&["A_B", "a-b", "a.b"])?;
    // All normalize to "a-b". Alphabetical: "A_B" < "a-b" < "a.b"
    assert_eq!(result[0], ("A_B".into(), "a-b".into()));
    assert_eq!(result[1], ("a-b".into(), "a-b--2".into()));
    assert_eq!(result[2], ("a.b".into(), "a-b--3".into()));
    Ok(())
}

#[test]
fn server_collision_with_invalid_id_returns_error() {
    let result = resolve_server_collisions(&["server-a", "___"]);
    assert!(result.is_err());
}
