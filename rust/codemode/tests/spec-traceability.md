# Spec Traceability Matrix

Maps test cases to spec sections in `specs/codemode.md`.

## Phase 1: Types, Errors, Identifiers

| Spec Section | Requirement | Test File | Test Name(s) |
|---|---|---|---|
| §3.2 | RunRequest fields (code, limits, requestedCapabilities) | spec_3_outer_tool.rs | run_request_serialization, run_request_minimal |
| §3.2.4 | Limits (timeoutMs, maxMemoryBytes, maxLogBytes, maxToolCalls) | spec_3_outer_tool.rs | run_request_serialization |
| §3.3 | RunResponse fields (logs, result, diagnostics, toolTrace) | spec_3_outer_tool.rs | run_response_serialization, run_response_default |
| §3.3.1 | Log entry (level, message, timeMs) | spec_3_outer_tool.rs | log_level_serialization, run_response_serialization |
| §3.3.2 | ToolTrace entry (serverId, toolName, durationMs, ok, error) | spec_3_outer_tool.rs | tool_trace_error_field, run_response_serialization |
| §3.3.3 | Diagnostic (severity, code, message, hint, path, errorClass) | spec_3_outer_tool.rs | diagnostic_severity_serialization, diagnostic_code_serialization |
| §3.3.4 | Result defaults to null | spec_3_outer_tool.rs | run_response_default |
| §4.2.1 | ServerInfo serialization | spec_3_outer_tool.rs | server_info_serialization, server_info_optional_fields_omitted |
| §4.2.2 | ServerDescription serialization | spec_3_outer_tool.rs | server_description_serialization |
| §4.2.3 | ToolSummary serialization | spec_3_outer_tool.rs | tool_summary_serialization |
| §4.2.4 | ToolDefinition serialization | spec_3_outer_tool.rs | tool_definition_serialization |
| §4.2.5 | SearchResults serialization | spec_3_outer_tool.rs | search_results_serialization |
| §4.2.6 | ListToolsOptions serialization | spec_3_outer_tool.rs | list_tools_options_serialization |
| §4.2.7 | SearchToolsOptions serialization | spec_3_outer_tool.rs | search_tools_options_serialization, search_tools_options_defaults_omit_none |
| §4.3 | DetailLevel enum (name, description, full) | spec_3_outer_tool.rs | detail_level_serialization |
| §5.0.1 | Server ID normalization rules | spec_6_codegen.rs | server_id_passthrough through server_id_all_invalid_chars_returns_invalid_server_id_error |
| §5.0.1 | Server ID collision disambiguation (--N) | spec_6_codegen.rs | server_no_collisions through server_collision_with_invalid_id_returns_error |
| §6.1 | Illegal char replacement | spec_6_codegen.rs | illegal_chars_replaced_with_underscore, unicode_letters_preserved |
| §6.1 | Leading digit handling | spec_6_codegen.rs | digit_prefix_gets_underscore |
| §6.1 | Reserved word handling | spec_6_codegen.rs | reserved_words_get_trailing_underscore, non_reserved_not_suffixed |
| §6.1 | Export collision disambiguation (__N) | spec_6_codegen.rs | two_way_collision, three_way_collision, collision_ordering_is_deterministic |
| §7.1 | Error types (CodemodeError variants) | (type definitions, runtime tested in later phases) | — |
