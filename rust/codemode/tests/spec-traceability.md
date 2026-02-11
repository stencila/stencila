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
