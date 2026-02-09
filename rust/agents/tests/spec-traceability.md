# Spec Traceability Matrix

Maps test cases to sections of the [Coding Agent Loop Specification](../specs/coding-agent-loop-spec.md).

## Phase 1: Core Types and Error Hierarchy

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 2.2 | SessionConfig defaults | spec_1_types.rs | session_config_defaults_match_spec | Pass |
| 2.2 | SessionConfig serde | spec_1_types.rs | session_config_serde_roundtrip, session_config_from_partial_json, session_config_custom_tool_limits | Pass |
| 2.2 | SessionConfig negative | spec_1_types.rs | session_config_invalid_json_rejected, session_config_wrong_type_rejected | Pass |
| 2.2 | ReasoningEffort enum | spec_1_types.rs | reasoning_effort_known_values, reasoning_effort_custom_value, reasoning_effort_as_str, reasoning_effort_display, reasoning_effort_roundtrip, session_config_with_reasoning_effort | Pass |
| 2.3 | SessionState enum | spec_1_types.rs | session_state_all_variants_exist, session_state_default_is_idle, session_state_equality | Pass |
| 2.3 | SessionState serde | spec_1_types.rs | session_state_serde_roundtrip, session_state_serializes_screaming_snake | Pass |
| 2.3 | SessionState negative | spec_1_types.rs | session_state_invalid_string_rejected | Pass |
| 2.4 | Turn::User | spec_1_types.rs | turn_user_construction, turn_user_has_valid_timestamp, turn_serde_roundtrip_user | Pass |
| 2.4 | Turn::Assistant | spec_1_types.rs | turn_assistant_construction, turn_serde_roundtrip_assistant_with_tool_calls | Pass |
| 2.4 | Turn::ToolResults | spec_1_types.rs | turn_tool_results_construction | Pass |
| 2.4 | Turn::System | spec_1_types.rs | turn_system_construction | Pass |
| 2.4 | Turn::Steering | spec_1_types.rs | turn_steering_construction | Pass |
| 2.4 | Turn tagged serde | spec_1_types.rs | turn_tagged_serialization | Pass |
| 2.4 | Turn timestamp required | spec_1_types.rs | turn_deser_missing_timestamp_rejected, turn_deser_with_explicit_timestamp, turn_timestamp_accessor | Pass |
| 2.9 | EventKind variants | spec_1_types.rs | event_kind_all_variants_exist (13 kinds) | Pass |
| 2.9 | EventKind serde | spec_1_types.rs | event_kind_serde_screaming_snake, event_kind_equality | Pass |
| 2.9 | EventKind negative | spec_1_types.rs | event_kind_invalid_string_rejected | Pass |
| 2.9 | SessionEvent | spec_1_types.rs | session_event_construction, session_event_serde_roundtrip, session_event_empty_data | Pass |
| 2.9 | SessionEvent negative | spec_1_types.rs | session_event_missing_required_fields_rejected | Pass |
| 4.1 | ExecResult record | spec_1_types.rs | exec_result_construction, exec_result_serde_roundtrip | Pass |
| 4.1 | ExecResult negative | spec_1_types.rs | exec_result_missing_field_rejected | Pass |
| 4.1 | DirEntry record | spec_1_types.rs | dir_entry_file, dir_entry_directory, dir_entry_serde_roundtrip | Pass |
| 3.3 | GrepOptions | spec_1_types.rs | grep_options_defaults, grep_options_serde_roundtrip | Pass |
| App B | Tool-level errors | spec_1_types.rs | error_display_*, error_is_tool_error_for_all_tool_variants, error_code_values | Pass |
| App B | Session-level errors (agent) | spec_1_types.rs | error_display_*, error_is_session_error_for_agent_native_variants | Pass |
| App B, 2.8 | Session-level SDK errors (non-retryable) | spec_1_types.rs | error_is_session_error_for_all_non_retryable_sdk_variants (11 variants via !is_retryable()) | Pass |
| App B | Non-session SDK errors (retryable) | spec_1_types.rs | error_is_not_session_error_for_retryable_sdk_variants (5 variants) | Pass |
| App B | SdkError wrapper | spec_1_types.rs | error_from_sdk_error, error_sdk_wrapper_display | Pass |
| App B | Error serialization | spec_1_types.rs | error_serialize_json | Pass |

## Phase 2: Tool Output Truncation

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 5.1 | truncate_output below limit | spec_5_truncation.rs | truncate_output_below_limit_passthrough, truncate_output_exactly_at_limit_passthrough, truncate_output_empty_string_passthrough | Pass |
| 5.1 | head_tail mode | spec_5_truncation.rs | truncate_output_head_tail_splits_evenly, truncate_output_head_tail_marker_contains_removed_count, truncate_output_head_tail_marker_mentions_event_stream, truncate_output_head_tail_odd_limit, truncate_output_large_input | Pass |
| 5.1 | tail mode | spec_5_truncation.rs | truncate_output_tail_mode_keeps_end, truncate_output_tail_mode_marker_contains_removed_count, truncate_output_tail_mode_no_head_content | Pass |
| 5.1 | UTF-8 / multibyte safety | spec_5_truncation.rs | truncate_output_multibyte_head_tail_no_panic, truncate_output_multibyte_tail_no_panic, truncate_output_emoji_no_panic, truncate_output_mixed_ascii_multibyte | Pass |
| 5.1 | Zero/small limit boundaries | spec_5_truncation.rs | truncate_output_zero_limit_head_tail, truncate_output_zero_limit_tail, truncate_output_limit_one_head_tail | Pass |
| 5.2 | Default policies (consolidated) | spec_5_truncation.rs | default_policies_char_limits_match_spec_table, default_policies_modes_match_spec_table, default_policies_line_limits_match_spec_table | Pass |
| 5.2 | TruncationConfig overrides | spec_5_truncation.rs | truncation_config_default_has_empty_overrides, truncation_config_custom_char_limit_overrides_default, truncation_config_custom_line_limit_overrides_default | Pass |
| 5.3 | truncate_lines below limit | spec_5_truncation.rs | truncate_lines_below_limit_passthrough, truncate_lines_exactly_at_limit_passthrough, truncate_lines_empty_string_passthrough, truncate_lines_single_line_over_limit_passthrough | Pass |
| 5.3 | truncate_lines head/tail | spec_5_truncation.rs | truncate_lines_head_tail_split, truncate_lines_marker_format | Pass |
| 5.3 | truncate_lines zero limit | spec_5_truncation.rs | truncate_lines_zero_limit | Pass |
| 5.3 | Full pipeline ordering | spec_5_truncation.rs | truncate_tool_output_char_truncation_runs_first, truncate_tool_output_line_truncation_runs_after_char | Pass |
| 5.3 | Pipeline per-tool behavior | spec_5_truncation.rs | truncate_tool_output_below_all_limits, truncate_tool_output_unknown_tool_uses_generous_default, truncate_tool_output_read_file_no_line_limit, truncate_tool_output_grep_uses_tail_mode, truncate_tool_output_shell_uses_head_tail_mode | Pass |
| 5.1 | TruncationMode traits/serde | spec_5_truncation.rs | truncation_mode_debug_and_clone, truncation_mode_serde_roundtrip | Pass |

## Spec 9 Conformance Coverage

| Spec 9 Section | Covered By | Test Type | Phase |
|---|---|---|---|
| 9.1 Core Loop | spec_2_loop.rs | Mock Client | 8 |
| 9.2 Provider Profiles | spec_3_profiles.rs | Deterministic | 7a |
| 9.3 Tool Execution | spec_3_registry.rs + spec_3_tools.rs | Mock ExecEnv | 5 + 6 |
| 9.4 Execution Environment | spec_4_execution.rs | Local filesystem (tempdir) | 3 |
| 9.5 Tool Output Truncation | spec_5_truncation.rs | Pure functions | 2 |
| 9.6 Steering | spec_2_loop.rs | Mock Client | 8 |
| 9.7 Reasoning Effort | spec_2_loop.rs | Mock Client | 8 |
| 9.8 System Prompts | spec_6_prompts.rs | Deterministic | 7b |
| 9.9 Subagents | spec_7_subagents.rs | Mock Client | 9 |
| 9.10 Event System | spec_2_events.rs + spec_2_loop.rs | Deterministic | 4 + 8 |
| 9.11 Error Handling | spec_2_loop.rs | Mock Client | 8 |
| 9.12 Parity Matrix | spec_2_loop.rs (shape) + spec_9_acceptance.rs (live) | Mock + env-gated | 8 + 10 |
| 9.13 Smoke Test | spec_9_acceptance.rs | Env-gated only | 10 |
