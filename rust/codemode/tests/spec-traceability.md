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

## Phase 2: Sandbox Core — Runtime, Globals, Console, Polyfills

| Spec Section | Requirement | Test File | Test Name(s) |
|---|---|---|---|
| §3.3 | Execute JS and return result via __codemode_result__ | spec_3_outer_tool.rs | execute_set_result_integer, execute_set_result_object, execute_no_result_returns_null, execute_result_with_logs |
| §3.3.1 | Console capture (log, debug, warn, error) | spec_9_logging.rs | console_log_basic, console_all_four_levels |
| §3.3.1 | Console timestamps (monotonic timeMs) | spec_9_logging.rs | console_timestamps_increase |
| §3.3.1 | Console serialization (primitives, objects, multiple args) | spec_9_logging.rs | console_primitive_serialization, console_object_serialization, console_multiple_args_concatenated |
| §3.3.1 | Console circular object fallback | spec_9_logging.rs | console_circular_object_fallback |
| §3.3.1 | Log truncation at max_log_bytes | spec_9_logging.rs | log_truncation_at_byte_limit, log_truncation_no_further_logs |
| §3.3.4 | Syntax errors → diagnostic (not propagated) | spec_3_outer_tool.rs | syntax_error_produces_diagnostic |
| §3.3.3 | Import failure → IMPORT_FAILURE diagnostic | spec_3_outer_tool.rs | import_failure_produces_diagnostic |
| §3.3.4 | Uncaught exceptions → diagnostic | spec_3_outer_tool.rs | uncaught_exception_produces_diagnostic |
| §3.3.4 | Prior logs preserved on fatal error | spec_3_outer_tool.rs | uncaught_exception_preserves_prior_logs |
| §3.5 | eval is deleted | spec_3_outer_tool.rs | eval_is_not_available |
| §3.5 | Function constructor is neutered | spec_3_outer_tool.rs | function_constructor_is_blocked |
| §3.5 | setTimeout/clearTimeout provided | spec_3_outer_tool.rs | set_timeout_fires_callback |
| §3.5 | Polyfills: URL, URLSearchParams | spec_3_outer_tool.rs | url_hostname_parsing, url_search_params_get |
| §3.5 | Polyfills: TextEncoder, TextDecoder | spec_3_outer_tool.rs | text_encoder_basic, text_decoder_basic |
| §3.2.4 | Timeout limit → SandboxLimit diagnostic | spec_3_outer_tool.rs | timeout_produces_sandbox_limit_diagnostic |
| §3.2.4 | Memory limit enforcement | spec_3_outer_tool.rs | memory_limit_produces_diagnostic |
| §1.4 | Fresh sandbox per execution (no state leakage) | spec_3_outer_tool.rs | fresh_sandbox_no_state_leakage |

## Phase 3: Module System — Discovery & Errors

| Spec Section | Requirement | Test File | Test Name(s) |
|---|---|---|---|
| §12.1 | specVersion is a semver string | spec_4_discovery.rs | spec_version_is_semver_string |
| §4.1 | listServers() returns all servers | spec_4_discovery.rs | list_servers_returns_all_servers, list_servers_empty_when_no_servers |
| §4.1 | describeServer() returns full info | spec_4_discovery.rs | describe_server_returns_full_info, describe_server_unknown_throws |
| §4.1 | listTools() with detail levels | spec_4_discovery.rs | list_tools_description_detail, list_tools_name_detail, list_tools_full_detail, list_tools_unknown_server_throws |
| §4.1 | getTool() returns full definition | spec_4_discovery.rs | get_tool_returns_full_definition, get_tool_unknown_throws, get_tool_unknown_server_throws_server_not_found |
| §4.1 | searchTools() with substring matching | spec_4_discovery.rs | search_tools_substring_match, search_tools_no_match, search_tools_filtered_by_server |
| §4.3 | Detail levels (name, description, full) filter fields | spec_4_discovery.rs | list_tools_name_detail, list_tools_description_detail, list_tools_full_detail, search_tools_with_name_detail, search_tools_with_full_detail |
| §11 | Host bridge is frozen, non-writable, non-configurable | spec_4_discovery.rs | host_bridge_is_frozen, host_bridge_is_not_writable, host_bridge_is_not_configurable |
| §7.1 | CodemodeError base class extends Error | spec_7_errors.rs | codemode_error_extends_error |
| §7.1 | SchemaValidationError hierarchy & properties | spec_7_errors.rs | schema_validation_error_hierarchy |
| §7.1 | ToolNotFoundError hierarchy & properties | spec_7_errors.rs | tool_not_found_error_hierarchy |
| §7.1 | ServerNotFoundError hierarchy & properties | spec_7_errors.rs | server_not_found_error_hierarchy |
| §7.1 | ToolCallError hierarchy & properties | spec_7_errors.rs | tool_call_error_hierarchy |
| §7.1 | AuthenticationError hierarchy & properties | spec_7_errors.rs | authentication_error_hierarchy |
| §7.1 | SandboxLimitError hierarchy & properties | spec_7_errors.rs | sandbox_limit_error_hierarchy |
| §7.1 | All 6 subclasses extend CodemodeError | spec_7_errors.rs | all_error_classes_extend_codemode_error |
| §7.3 | Error hint is null when omitted | spec_7_errors.rs | error_hint_is_null_when_omitted |

## Phase 4: Server Modules — Tool Bindings & Validation

| Spec Section | Requirement | Test File | Test Name(s) |
|---|---|---|---|
| §5.1 | Import and call tool returns result | spec_5_server_modules.rs | import_and_call_tool_returns_result |
| §5.1 | Call tool with no args / empty / undefined / null | spec_5_server_modules.rs | call_tool_with_no_args_sends_empty_object, call_tool_with_empty_object_succeeds, call_tool_with_undefined_sends_empty_object, call_tool_with_null_sends_empty_object |
| §5.1 | Call tool with valid schema input | spec_5_server_modules.rs | call_tool_with_valid_schema_input |
| §5.1 | No schema allows any input | spec_5_server_modules.rs | no_schema_allows_any_input |
| §5.1 | Input JSON passed through to server | spec_5_server_modules.rs | input_json_passed_to_server |
| §5.2 | __meta__ export shape (serverId, serverName, serverVersion, tools) | spec_5_server_modules.rs | meta_has_correct_shape |
| §5.2 | __meta__ tool entries (toolName, exportName, description) | spec_5_server_modules.rs | meta_tools_have_correct_entries |
| §5.2 | __meta__ is deeply frozen | spec_5_server_modules.rs | meta_is_frozen |
| §5.2 | __meta__ serverVersion empty when absent | spec_5_server_modules.rs | meta_server_version_empty_when_absent |
| §5.3.2 | structuredContent takes priority | spec_5_server_modules.rs | structured_content_takes_priority |
| §5.3.2 | Single text → string | spec_5_server_modules.rs | single_text_unwraps_to_string |
| §5.3.2 | Image content → content array | spec_5_server_modules.rs | image_content_returns_full_array |
| §5.3.2 | Audio content → content array | spec_5_server_modules.rs | audio_content_returns_full_array |
| §5.3.2 | Multiple text → content array | spec_5_server_modules.rs | multiple_text_returns_full_array |
| §5.3.2 | Empty content → empty array | spec_5_server_modules.rs | empty_content_returns_empty_array |
| §7.2 | Missing required → SchemaValidationError | spec_5_server_modules.rs | schema_validation_missing_required_field |
| §7.2 | Wrong type with expected/received/path | spec_5_server_modules.rs | schema_validation_wrong_type |
| §7.2 | Hint present on schema errors | spec_5_server_modules.rs | schema_validation_hint_present |
| §7.2 | instanceof checks (SchemaValidationError, CodemodeError, Error) | spec_5_server_modules.rs | schema_validation_instanceof_checks |
| §7.2 | Invalid schema gracefully skipped | spec_5_server_modules.rs | invalid_schema_gracefully_skipped |
| §3.3.2 | Tool trace recorded on success | spec_5_server_modules.rs | tool_trace_recorded_on_success |
| §3.3.2 | Tool trace has duration_ms | spec_5_server_modules.rs | tool_trace_has_duration |
| §3.3.2 | Tool trace ok=false on error | spec_5_server_modules.rs | tool_trace_recorded_on_error |
| §3.3.2 | Multiple tool traces recorded in order | spec_5_server_modules.rs | tool_trace_multiple_calls |
| §3.3.2 | Tool trace absent when no calls | spec_5_server_modules.rs | tool_trace_absent_when_no_calls |
| §3.2.4 | maxToolCalls enforced (N succeed, N+1 fails) | spec_5_server_modules.rs | max_tool_calls_enforced |
| §3.2.4 | maxToolCalls exact boundary | spec_5_server_modules.rs | max_tool_calls_exact_boundary |
| §5.3 | isError=true → ToolCallError | spec_5_server_modules.rs | is_error_true_throws_tool_call_error |
| §5.3 | Rust error → ToolCallError | spec_5_server_modules.rs | rust_error_throws_tool_call_error |
| §5.3 | Rust error records trace | spec_5_server_modules.rs | rust_error_records_trace |
| §5.3 | Unknown tool → ToolNotFoundError envelope | spec_5_server_modules.rs | unknown_tool_via_bridge_throws_tool_not_found_error |
| §5.3 | Unknown tool → ToolNotFoundError class | spec_5_server_modules.rs | unknown_tool_via_generated_handler_throws_tool_not_found |
| §7.2 | Required-field path points to missing property | spec_5_server_modules.rs | schema_validation_missing_required_field |
| §5.1 | Cross-server orchestration | spec_5_server_modules.rs | cross_server_orchestration |
| §5.1 | Cross-server tool trace | spec_5_server_modules.rs | cross_server_tool_trace |
| §5.1 | Discovery and server modules coexist | spec_5_server_modules.rs | discovery_and_server_modules_coexist |
