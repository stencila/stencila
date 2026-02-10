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

## Phase 3: ExecutionEnvironment and Local Implementation

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 4.1 | read_file text with line numbers | spec_4_execution.rs | read_file_text_with_line_numbers | Pass |
| 4.1 | read_file offset and limit | spec_4_execution.rs | read_file_with_offset_and_limit | Pass |
| 4.1 | read_file not found | spec_4_execution.rs | read_file_not_found | Pass |
| 4.1 | read_file image detection | spec_4_execution.rs | read_file_image_returns_image_content, read_file_jpeg_detected | Pass |
| 4.1 | write_file | spec_4_execution.rs | write_file_creates_file, write_file_creates_parent_directories, write_file_overwrites_existing | Pass |
| 4.1 | file_exists | spec_4_execution.rs | file_exists_true_and_false | Pass |
| 4.1 | list_directory | spec_4_execution.rs | list_directory_basic, list_directory_with_depth | Pass |
| 4.2 | exec_command success | spec_4_execution.rs | exec_command_success, exec_command_exit_code, exec_command_stderr_captured | Pass |
| 4.2, 5.4 | exec_command timeout (flag, partial output, error message) | spec_4_execution.rs | exec_command_timeout, exec_command_partial_output_on_timeout, exec_command_timeout_message_in_stderr | Pass |
| 4.2 | exec_command working_dir | spec_4_execution.rs | exec_command_with_working_dir | Pass |
| 4.2 | exec_command custom env vars | spec_4_execution.rs | exec_command_with_custom_env_vars | Pass |
| 4.2 | Env var denylist | spec_4_execution.rs | filter_env_vars_filtered_excludes_api_keys, filter_env_vars_filtered_case_insensitive_deny | Pass |
| 4.2 | Env var allowlist | spec_4_execution.rs | filter_env_vars_filtered_allowlist_always_present | Pass |
| 4.2 | Env var policy variants | spec_4_execution.rs | filter_env_vars_inherit_all_includes_everything, filter_env_vars_inherit_none_only_allowlist, filter_env_vars_default_policy_is_filtered | Pass |
| 4.1 | grep search | spec_4_execution.rs | grep_basic_match, grep_case_insensitive, grep_max_results, grep_single_file, grep_path_not_found | Pass |
| 4.1 | glob search | spec_4_execution.rs | glob_basic, glob_path_not_found, glob_sorted_by_mtime_newest_first | Pass |
| 4.1 | Metadata | spec_4_execution.rs | working_directory_returns_configured_path, platform_returns_spec_value, os_version_returns_nonempty | Pass |
| App B | Io error variant | spec_1_types.rs | error_is_tool_error_for_all_tool_variants, error_code_values | Pass |

## Phase 4: Event System

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 2.9 | Channel construction (UUID) | spec_2_events.rs | channel_returns_emitter_and_receiver | Pass |
| 2.9 | Channel with deterministic ID | spec_2_events.rs | channel_with_id_uses_provided_id | Pass |
| 2.9 | SESSION_START event | spec_2_events.rs | emit_session_start | Pass |
| 2.9 | SESSION_END event (includes final_state) | spec_2_events.rs | emit_session_end | Pass |
| 2.9 | USER_INPUT event | spec_2_events.rs | emit_user_input | Pass |
| 2.9 | ASSISTANT_TEXT lifecycle | spec_2_events.rs | emit_assistant_text_lifecycle | Pass |
| 2.9 | TOOL_CALL lifecycle | spec_2_events.rs | emit_tool_call_lifecycle | Pass |
| 9.10 | TOOL_CALL_END full output | spec_2_events.rs | emit_tool_call_end_carries_full_untruncated_output | Pass |
| 2.9 | TOOL_CALL_END error | spec_2_events.rs | emit_tool_call_end_error | Pass |
| 2.9 | STEERING_INJECTED event | spec_2_events.rs | emit_steering_injected | Pass |
| 2.9 | TURN_LIMIT event | spec_2_events.rs | emit_turn_limit | Pass |
| 2.9 | LOOP_DETECTION event | spec_2_events.rs | emit_loop_detection | Pass |
| 2.9 | ERROR event | spec_2_events.rs | emit_error | Pass |
| 2.9 | Strict emission ordering | spec_2_events.rs | events_received_in_emission_order | Pass |
| 2.9 | All kinds carry session_id + timestamp | spec_2_events.rs | all_events_carry_session_id_and_timestamp | Pass |
| 2.9 | Receiver None after emitter dropped | spec_2_events.rs | receiver_returns_none_when_emitter_dropped | Pass |
| 2.9 | Silent discard after receiver dropped | spec_2_events.rs | emit_after_receiver_dropped_does_not_panic | Pass |

## Phase 5: Tool Registry

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 3.8 | Empty registry | spec_3_registry.rs | new_registry_is_empty, empty_registry_definitions_and_names | Pass |
| 3.8 | Register and lookup | spec_3_registry.rs | register_and_get, get_unknown_returns_none | Pass |
| 3.8 | Register validates definition | spec_3_registry.rs | register_rejects_invalid_definition, register_rejects_non_object_schema | Pass |
| 3.8 | Insertion-order definitions | spec_3_registry.rs | register_returns_definitions_in_order | Pass |
| 3.8 | Insertion-order names | spec_3_registry.rs | register_returns_names_in_order | Pass |
| 3.8 | Unregister | spec_3_registry.rs | unregister_existing, unregister_nonexistent | Pass |
| 3.8 | Override latest-wins | spec_3_registry.rs | register_override_latest_wins | Pass |
| 3.8 | Override preserves position | spec_3_registry.rs | register_override_preserves_position | Pass |
| 3.8 | RegisteredTool execution (direct) | spec_3_registry.rs | execute_tool_success, execute_tool_error_propagates | Pass |
| 3.8 | Integrated lookup → validate → execute | spec_3_registry.rs | lookup_validate_execute_integrated, lookup_validate_execute_rejects_invalid_args | Pass (unit) |
| 3.8 | Runtime schema validation before tool execution | spec_2_loop.rs | invalid_tool_args_returns_validation_error | Pass |
| 3.8 | Argument validation (valid) | spec_3_registry.rs | validate_arguments_valid | Pass |
| 3.8 | Argument validation (invalid) | spec_3_registry.rs | validate_arguments_invalid | Pass |
| 3.8 | Unknown tool validation | spec_3_registry.rs | validate_arguments_unknown_tool | Pass |
| 3.8 | Uncompilable schema graceful skip | spec_3_registry.rs | validate_arguments_uncompilable_schema_skips | Pass |
| 3.8 | Definitions are cloned | spec_3_registry.rs | definitions_clones_not_references | Pass |
| 3.8 | Multiple tools independent | spec_3_registry.rs | multiple_tools_independent | Pass |

## Phase 5b+6a: Core Tool Implementations

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 3.3 | read_file schema parity | spec_3_tools.rs | read_file_schema_matches_fixture | Pass |
| 3.3 | write_file schema parity | spec_3_tools.rs | write_file_schema_matches_fixture | Pass |
| 3.3 | edit_file schema parity | spec_3_tools.rs | edit_file_schema_matches_fixture | Pass |
| 3.3 | shell schema parity | spec_3_tools.rs | shell_schema_matches_fixture | Pass |
| 3.3 | grep schema parity | spec_3_tools.rs | grep_schema_matches_fixture | Pass |
| 3.3 | glob schema parity | spec_3_tools.rs | glob_schema_matches_fixture | Pass |
| 3.6 | read_many_files schema parity | spec_3_tools.rs | read_many_files_schema_matches_fixture | Pass |
| 3.6 | list_dir schema parity | spec_3_tools.rs | list_dir_schema_matches_fixture | Pass |
| 3.3 | read_file executor | spec_3_tools.rs | read_file_text_content, read_file_with_offset_and_limit, read_file_image_returns_image_with_text, read_file_large_image_falls_back_to_text, read_file_not_found | Pass |
| 3.3 | write_file executor | spec_3_tools.rs | write_file_success_with_byte_count, write_file_records_write, write_file_missing_content | Pass |
| 3.3 | edit_file executor | spec_3_tools.rs | edit_file_single_replace, edit_file_replace_all, edit_file_not_found, edit_file_old_string_missing, edit_file_not_unique, edit_file_correct_writeback | Pass |
| 3.3 | shell executor | spec_3_tools.rs | shell_success_format, shell_exit_code, shell_custom_timeout, shell_per_call_timeout_overrides_default | Pass |
| 2.2 | max_command_timeout_ms clamping | spec_2_loop.rs | shell_timeout_clamped_to_max | Pass |
| 3.3 | grep executor | spec_3_tools.rs | grep_basic, grep_with_options, grep_default_path, grep_missing_pattern | Pass |
| 3.3 | glob executor | spec_3_tools.rs | glob_basic, glob_empty_results, glob_default_path, glob_missing_pattern | Pass |
| 3.6 | read_many_files executor | spec_3_tools.rs | read_many_files_batch, read_many_files_partial_failure, read_many_files_empty_paths | Pass |
| 3.6 | list_dir executor | spec_3_tools.rs | list_dir_basic, list_dir_with_depth, list_dir_not_found | Pass |
| 3.3, 3.6 | Core registration (6 tools) | spec_3_tools.rs | register_core_tools_adds_six | Pass |
| 3.6 | Gemini registration (+2) | spec_3_tools.rs | register_gemini_tools_adds_two_more | Pass |
| 3.3 | strip_line_numbers helper | spec_3_tools.rs | strip_line_numbers_basic, strip_line_numbers_preserves_trailing_newline, strip_line_numbers_no_trailing_newline, strip_line_numbers_passthrough | Pass |
| 3.3 | required_str helper | spec_3_tools.rs | required_str_extracts_value, required_str_missing_returns_error | Pass |

## Phase 6b: apply_patch Tool (v4a Format)

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| App A | apply_patch schema parity | spec_3_patch.rs | apply_patch_schema_matches_fixture | Pass |
| App A | v4a parse Add File | spec_3_patch.rs | parse_add_file | Pass |
| App A | v4a parse Delete File | spec_3_patch.rs | parse_delete_file | Pass |
| App A | v4a parse Update (single hunk) | spec_3_patch.rs | parse_update_single_hunk | Pass |
| App A | v4a parse Update with Move | spec_3_patch.rs | parse_update_with_move | Pass |
| App A | v4a parse multi-hunk | spec_3_patch.rs | parse_multi_hunk_update | Pass |
| App A | Parse error: missing begin | spec_3_patch.rs | parse_error_missing_begin | Pass |
| App A | Parse error: missing end | spec_3_patch.rs | parse_error_missing_end | Pass |
| App A | Parse error: update without hunks | spec_3_patch.rs | parse_error_update_without_hunks | Pass |
| App A | Parse error: trailing content after end marker | spec_3_patch.rs | parse_error_trailing_content_after_end_patch | Pass |
| App A | Parse error: empty hunk (EOF marker) | spec_3_patch.rs | parse_error_empty_hunk_lines | Pass |
| App A | Parse error: empty hunk (next hunk header) | spec_3_patch.rs | parse_error_empty_hunk_before_next_hunk | Pass |
| App A | Parse error: empty hunk (operation boundary) | spec_3_patch.rs | parse_error_empty_hunk_before_operation | Pass |
| App A | Applicator: add file | spec_3_patch.rs | apply_add_file_creates_file | Pass |
| App A | Applicator: delete file | spec_3_patch.rs | apply_delete_file_removes_file | Pass |
| App A | Applicator: update single hunk | spec_3_patch.rs | apply_update_file_single_hunk | Pass |
| App A | Applicator: update multi hunk | spec_3_patch.rs | apply_update_file_multi_hunk | Pass |
| App A | Applicator: move (rename) | spec_3_patch.rs | apply_update_with_move_renames | Pass |
| App A | Applicator: file not found | spec_3_patch.rs | apply_update_file_not_found | Pass |
| App A | Applicator: hunk mismatch | spec_3_patch.rs | apply_hunk_mismatch_returns_edit_conflict | Pass |
| App A | Fuzzy whitespace match | spec_3_patch.rs | apply_update_fuzzy_whitespace_match | Pass |
| App A | EOF marker before End Patch | spec_3_patch.rs | parse_eof_marker_before_end_patch | Pass |
| App A | EOF marker between operations | spec_3_patch.rs | parse_eof_marker_between_operations | Pass |
| App A | Update file >2000 lines | spec_3_patch.rs | apply_update_file_beyond_2000_lines | Pass |
| App A | Move to same path (no delete) | spec_3_patch.rs | apply_update_move_to_same_path | Pass |
| App A | Executor end-to-end | spec_3_patch.rs | apply_patch_executor_end_to_end | Pass |
| App A | context_hint disambiguates repeated pattern | spec_3_patch.rs | context_hint_disambiguates_repeated_pattern | Pass |
| App A | Empty context_hint falls back to first match | spec_3_patch.rs | context_hint_empty_falls_back_to_first_match | Pass |
| App A | Unfound context_hint falls back to first match | spec_3_patch.rs | context_hint_not_found_falls_back_to_first_match | Pass |
| App A | Fuzzy + context_hint disambiguation combined | spec_3_patch.rs | context_hint_fuzzy_whitespace_with_disambiguation | Pass |
| App A | context_hint prefers exact line over earlier substring | spec_3_patch.rs | context_hint_ignores_earlier_comment_substring | Pass |
| App A | OpenAI registration | spec_3_patch.rs | register_openai_tools_adds_one | Pass |
| Ext | delete_file removes file (impl extension for App A) | spec_4_execution.rs | delete_file_removes_file | Pass |
| Ext | delete_file not found (impl extension for App A) | spec_4_execution.rs | delete_file_not_found | Pass |

## Phase 7a: Provider Profiles

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 3.2 | ProviderProfile trait | spec_3_profiles.rs | profiles_usable_as_trait_objects | Pass |
| 3.2 | Profile id/model | spec_3_profiles.rs | openai_profile_id, openai_profile_model, anthropic_profile_id, anthropic_profile_model, gemini_profile_id, gemini_profile_model | Pass |
| 3.2 | Capability flags | spec_3_profiles.rs | openai_capability_flags, anthropic_capability_flags, gemini_capability_flags | Pass |
| 3.4 | OpenAI provider_options (None — reasoning.effort via unified Request) | spec_3_profiles.rs | openai_provider_options_are_none | Pass |
| 3.5 | Anthropic provider_options (beta_headers, auto_cache) | spec_3_profiles.rs | anthropic_provider_options_has_beta_headers | Pass |
| 3.6 | Gemini provider_options (safetySettings only; grounding deferred — see Limitations) | spec_3_profiles.rs | gemini_provider_options_has_safety_settings | Partial |
| 3.2 | build_system_prompt placeholder | spec_3_profiles.rs | build_system_prompt_contains_base_instructions | Pass |
| 3.2 | tools() returns definitions | spec_3_profiles.rs | openai_profile_tools_returns_definitions, tools_method_matches_registry_definitions | Pass |
| 3.4 | OpenAI tool set (6 tools) | spec_3_profiles.rs | openai_profile_tool_count, openai_profile_tool_names | Pass |
| 3.4 | OpenAI uses apply_patch not edit_file | spec_3_profiles.rs | openai_profile_has_apply_patch_not_edit_file | Pass |
| 3.5 | Anthropic tool set (6 tools) | spec_3_profiles.rs | anthropic_profile_tool_count, anthropic_profile_tool_names | Pass |
| 3.5 | Anthropic uses edit_file not apply_patch | spec_3_profiles.rs | anthropic_profile_has_edit_file_not_apply_patch | Pass |
| 3.6 | Gemini tool set (8 tools) | spec_3_profiles.rs | gemini_profile_tool_count, gemini_profile_tool_names | Pass |
| 3.6 | Gemini has read_many_files + list_dir | spec_3_profiles.rs | gemini_profile_has_gemini_specific_tools | Pass |
| 3.4 | OpenAI shell default timeout 10s | spec_3_profiles.rs | openai_shell_default_timeout_is_10s | Pass |
| 3.5 | Anthropic shell default timeout 120s | spec_3_profiles.rs | anthropic_shell_default_timeout_is_120s | Pass |
| 3.6 | Gemini shell default timeout 10s | spec_3_profiles.rs | gemini_shell_default_timeout_is_10s | Pass |
| 3.4-3.6 | Shell tool present in all profiles | spec_3_profiles.rs | all_profiles_have_shell_tool | Pass |
| 3.4-3.6 | Schema parity for assembled profiles | spec_3_profiles.rs | openai_profile_schema_parity, anthropic_profile_schema_parity, gemini_profile_schema_parity | Pass |
| 3.7 | Custom tool registration | spec_3_profiles.rs | custom_tool_registration | Pass |
| 3.7 | Custom tool override (latest-wins) | spec_3_profiles.rs | custom_tool_override_replaces_existing | Pass |
| 3.7 | Override preserves position | spec_3_profiles.rs | custom_tool_override_preserves_position | Pass |
| 7.1-7.4 | No subagent tools yet (Phase 9) | spec_3_profiles.rs | profiles_have_no_subagent_tools | Pass |
| 3.2 | Debug output | spec_3_profiles.rs | profile_debug_output | Pass |

## Phase 7b: System Prompts and Project Doc Discovery

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 6.3 | Environment context XML format (incl. knowledge cutoff, git status, recent commits) | spec_6_prompts.rs | environment_context_format | Pass |
| 6.3 | Environment context without git (no branch/status/commits/cutoff) | spec_6_prompts.rs | environment_context_no_git | Pass |
| 6.3 | Environment context includes date | spec_6_prompts.rs | environment_context_date_is_present | Pass |
| 6.4 | Gather git context in repo | spec_6_prompts.rs | gather_git_context_in_repo | Pass |
| 6.4 | Gather git context not a repo | spec_6_prompts.rs | gather_git_context_not_a_repo | Pass |
| 6.4 | Format git summary with data | spec_6_prompts.rs | format_git_summary_with_data | Pass |
| 6.4 | Format git summary not a repo | spec_6_prompts.rs | format_git_summary_not_a_repo | Pass |
| 6.5 | AGENTS.md always loaded | spec_6_prompts.rs | discover_agents_md_always_loaded | Pass |
| 6.5 | Anthropic loads CLAUDE.md not GEMINI.md | spec_6_prompts.rs | discover_anthropic_loads_claude_md_not_gemini | Pass |
| 6.5 | OpenAI loads .codex/instructions.md not CLAUDE.md | spec_6_prompts.rs | discover_openai_loads_codex_not_claude | Pass |
| 6.5 | Gemini loads GEMINI.md not CLAUDE.md | spec_6_prompts.rs | discover_gemini_loads_gemini_md_not_claude | Pass |
| 6.5 | Root-first, subdirectory appended | spec_6_prompts.rs | discover_root_first_subdirectory_appended | Pass |
| 6.5 | 32KB budget enforcement | spec_6_prompts.rs | discover_32kb_budget_enforcement | Pass |
| 6.5 | Budget stops at second file | spec_6_prompts.rs | discover_budget_stops_at_second_file | Pass |
| 6.5 | No docs returns empty | spec_6_prompts.rs | discover_no_docs_returns_empty | Pass |
| 6.5 | Nested path ordering | spec_6_prompts.rs | discover_nested_path | Pass |
| 6.1 | System prompt contains base instructions | spec_6_prompts.rs | system_prompt_contains_base_instructions | Pass |
| 6.1 | System prompt layer ordering | spec_6_prompts.rs | system_prompt_layer_ordering | Pass |
| 6.1 | Empty layers omitted | spec_6_prompts.rs | system_prompt_empty_layers_omitted | Pass |
| 6.1 | All layers assembled together | spec_6_prompts.rs | system_prompt_with_all_layers | Pass |
| 6.2 | OpenAI base instructions topics | spec_6_prompts.rs | openai_base_instructions_topics | Pass |
| 6.2 | Anthropic base instructions topics | spec_6_prompts.rs | anthropic_base_instructions_topics | Pass |
| 6.2 | Gemini base instructions topics | spec_6_prompts.rs | gemini_base_instructions_topics | Pass |
| 6.5 | instruction_files per provider | project_docs.rs (unit) | instruction_files_openai, instruction_files_anthropic, instruction_files_gemini, instruction_files_unknown_only_agents | Pass |
| 6.5 | directories_from_root_to_working_dir | project_docs.rs (unit) | directories_same_root_and_wd, directories_nested_wd, directories_wd_not_under_root, directories_sibling_path_not_misclassified | Pass |
| 6.5 | safe_truncation_point UTF-8 | project_docs.rs (unit) | safe_truncation_ascii, safe_truncation_multibyte | Pass |

## Phase 8: Session and Core Agentic Loop

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 2.1 | Natural completion (text-only) | spec_2_loop.rs | natural_completion_text_only | Pass |
| 2.1 | Session id present | spec_2_loop.rs | session_id_is_present | Pass |
| 2.1 | Config returns default values | spec_2_loop.rs | config_returns_default_values | Pass |
| 2.5 | Single tool round | spec_2_loop.rs | single_tool_round | Pass |
| 2.5 | Multi tool rounds | spec_2_loop.rs | multi_tool_rounds | Pass |
| 2.5 | Unknown tool returns error result | spec_2_loop.rs | unknown_tool_returns_error_result | Pass |
| 2.5 | Tool error sent as error result | spec_2_loop.rs | tool_error_sent_as_error_result | Pass |
| 2.5 | Tool output full in event truncated for LLM | spec_2_loop.rs | tool_output_full_in_event_truncated_for_llm | Pass |
| 2.5 | Parallel tool execution | spec_2_loop.rs | parallel_tool_execution | Pass |
| 2.5 | Sequential tool execution | spec_2_loop.rs | sequential_tool_execution | Pass |
| 2.6 | Round limit reached | spec_2_loop.rs | round_limit_reached | Pass |
| 2.6 | Session turn limit | spec_2_loop.rs | session_turn_limit | Pass |
| 2.6 | Turn limit is idle not closed | spec_2_loop.rs | turn_limit_is_idle_not_closed | Pass |
| 2.6 | Abort stops loop | spec_2_loop.rs | abort_stops_loop | Pass |
| 2.6 | Abort mid tool loop | spec_2_loop.rs | abort_mid_tool_loop | Pass |
| 2.6 | Abort cancels in-flight tool execution | spec_2_loop.rs | abort_cancels_in_flight_tool_execution | Pass |
| 2.6 | Abort cancels in-flight LLM call | spec_2_loop.rs | abort_during_llm_call | Pass |
| 2.6 | Close emits session end only once | spec_2_loop.rs | close_emits_session_end_only_once | Pass |
| 2.6 | Submit on closed session errors | spec_2_loop.rs | submit_on_closed_session_errors | Pass |
| 2.7 | Steering injection between tool rounds | spec_2_loop.rs | steer_between_tool_rounds | Pass |
| 2.7 | Reasoning effort passthrough | spec_2_loop.rs | reasoning_effort_passthrough | Pass |
| 2.7 | Reasoning effort mid-session change | spec_2_loop.rs | reasoning_effort_mid_session_change | Pass |
| 2.8 | Follow-up after completion | spec_2_loop.rs | follow_up_after_completion | Pass |
| 2.8 | Follow-up processed after turn limit | spec_2_loop.rs | follow_up_processed_after_turn_limit | Pass |
| 2.8 | Sequential inputs | spec_2_loop.rs | sequential_inputs | Pass |
| 2.9 | Events natural completion | spec_2_loop.rs | events_natural_completion | Pass |
| 2.9 | Events tool loop | spec_2_loop.rs | events_tool_loop | Pass |
| 2.9 | Events session close (verifies final_state in SESSION_END) | spec_2_loop.rs | events_session_close | Pass |
| 2.9 | Streaming emits ASSISTANT_TEXT_DELTA events | spec_2_loop.rs | streaming_emits_text_deltas | Pass |
| 2.9 | Streaming event ordering (START < DELTA < END) | spec_2_loop.rs | streaming_text_start_before_delta_before_end | Pass |
| 2.9 | No delta for empty text (tool-call-only response) | spec_2_loop.rs | streaming_no_delta_for_empty_text | Pass |
| 2.9 | Streaming deltas emitted per tool-loop round | spec_2_loop.rs | streaming_tool_loop_emits_deltas_per_round | Pass |
| 2.9 | Incremental streaming with multiple deltas | spec_2_loop.rs | streaming_real_incremental_deltas | Pass |
| 2.9 | Non-streaming profile falls back to complete() | spec_2_loop.rs | non_streaming_profile_falls_back_to_complete | Pass |
| 2.9 | Pre-abort closes before LLM call (no TEXT events) | spec_2_loop.rs | pre_abort_closes_before_llm_call | Pass |
| 2.9 | In-flight abort preserves partial text in TEXT_END | spec_2_loop.rs | inflight_abort_preserves_partial_text_in_text_end | Pass |
| 2.9 | Error before streaming emits empty TEXT_END (strict pairing) | spec_2_loop.rs | error_before_streaming_emits_empty_text_end | Pass |
| 2.9 | Mid-stream error preserves partial text in TEXT_END | spec_2_loop.rs | midstream_error_preserves_partial_text_in_text_end | Pass |
| 2.10 | Loop detection injects steering | spec_2_loop.rs | loop_detection_injects_steering | Pass |
| 2.10 | Loop detection disabled | spec_2_loop.rs | loop_detection_disabled | Pass |
| 5.5 | Context usage warning when exceeding 80% | spec_2_loop.rs | context_usage_warning_emitted_at_80_percent | Pass |
| 5.5 | No warning below threshold | spec_2_loop.rs | context_usage_no_warning_below_threshold | Pass |
| App B | Authentication error closes session | spec_2_loop.rs | authentication_error_closes_session | Pass |
| App B | Context length error closes session with warning severity | spec_2_loop.rs | context_length_error_closes_session_with_warning_severity | Pass |
| App B | Server error closes session | spec_2_loop.rs | server_error_closes_session | Pass |
| App B | Rate limit error closes session | spec_2_loop.rs | rate_limit_error_closes_session | Pass |
| App B | Network error closes session | spec_2_loop.rs | network_error_closes_session | Pass |
| 2.1 | System prompt in request | spec_2_loop.rs | system_prompt_in_request | Pass |
| 2.1 | End-to-end prompt has base instructions and env context | spec_2_loop.rs | end_to_end_prompt_has_base_instructions_and_env_context | Pass |
| 6.1, 6.5 | Prompt includes project docs layer when present | spec_2_loop.rs | end_to_end_prompt_includes_project_docs_layer | Pass |
| 6.1 | Prompt in request matches build_system_prompt output | spec_2_loop.rs | end_to_end_prompt_in_request_matches_build_system_prompt | Pass |
| 2.1 | History to messages (user turn) | spec_2_loop.rs | history_to_messages_user_turn | Pass |
| 2.1 | History to messages (assistant with tools) | spec_2_loop.rs | history_to_messages_assistant_with_tools | Pass |
| 2.1 | History to messages (steering as user) | spec_2_loop.rs | history_to_messages_steering_as_user | Pass |
| 2.1 | Request includes tools from profile | spec_2_loop.rs | request_includes_tools_from_profile | Pass |
| 2.1 | Request has tool_choice auto | spec_2_loop.rs | request_has_tool_choice_auto | Pass |
| 2.1 | Request has provider_id | spec_2_loop.rs | request_has_provider_id | Pass |
| 9.12 | OpenAI profile has apply_patch tool | spec_2_loop.rs | openai_profile_has_apply_patch_tool | Pass |
| 9.12 | Anthropic profile has edit_file tool | spec_2_loop.rs | anthropic_profile_has_edit_file_tool | Pass |
| 9.12 | Gemini profile has extended tools | spec_2_loop.rs | gemini_profile_has_extended_tools | Pass |
| 9.12 | Parity: OpenAI session wires correct tools + shell 10s timeout | spec_2_loop.rs | parity_openai_session_wires_correct_tools | Pass |
| 9.12 | Parity: Anthropic session wires correct tools + shell 120s timeout | spec_2_loop.rs | parity_anthropic_session_wires_correct_tools | Pass |
| 9.12 | Parity: Gemini session wires correct tools + shell 10s timeout | spec_2_loop.rs | parity_gemini_session_wires_correct_tools | Pass |
| 9.12 | Parity: file-creation → write tool available | spec_2_loop.rs | parity_{openai,anthropic,gemini}_session_wires_correct_tools (asserts write_file/edit_file present) | Pass |
| 9.12 | Parity: read+edit → both tools available | spec_3_profiles.rs + spec_2_loop.rs | {openai,anthropic,gemini}_profile_tool_names + parity tests | Pass |
| 9.12 | Parity: shell timeout per provider | spec_2_loop.rs | parity tests (assert timeout_ms via CapturingExecEnv) | Pass |
| 9.12 | Parity: truncation shape (full in event, truncated for LLM) | spec_2_loop.rs | tool_output_full_in_event_truncated_for_llm | Pass |

## Phase 9: Subagents

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 7.2 | spawn_agent tool definition valid | spec_7_subagents.rs | spawn_agent_definition_valid | Pass |
| 7.2 | send_input tool definition valid | spec_7_subagents.rs | send_input_definition_valid | Pass |
| 7.2 | wait tool definition valid | spec_7_subagents.rs | wait_definition_valid | Pass |
| 7.2 | close_agent tool definition valid | spec_7_subagents.rs | close_agent_definition_valid | Pass |
| 7.2 | All four definitions returned | spec_7_subagents.rs | subagent_definitions_returns_four | Pass |
| 7.2 | Registration adds four tools | spec_7_subagents.rs | register_subagent_tools_adds_four_tools | Pass |
| 7.1, 7.3, 9.9 | Spawn creates independent session | spec_7_subagents.rs | spawn_creates_independent_session | Pass |
| 7.1, 9.9 | Spawn shares execution environment | spec_7_subagents.rs | spawn_shares_execution_environment | Pass |
| 7.2 | Spawn with custom max_turns (asserts turns_used) | spec_7_subagents.rs | spawn_with_custom_max_turns | Pass |
| 7.2 | Spawn with working_dir scopes system prompt | spec_7_subagents.rs | spawn_with_working_dir_scopes_system_prompt | Pass |
| 7.2 | Spawn with model override uses custom model | spec_7_subagents.rs | spawn_with_model_override_uses_custom_model | Pass |
| 7.3, 9.9 | Depth limiting blocks sub-sub-agents | spec_7_subagents.rs | depth_limiting_blocks_sub_sub_agents | Pass |
| 7.3 | Depth zero allows no subagents | spec_7_subagents.rs | depth_zero_allows_no_subagents | Pass |
| 7.2 | send_input success after spawn | spec_7_subagents.rs | send_input_success_after_spawn | Pass |
| 7.2 | wait success after spawn | spec_7_subagents.rs | wait_success_after_spawn | Pass |
| 7.2 | close_agent success after spawn | spec_7_subagents.rs | close_agent_success_after_spawn | Pass |
| 7.2 | close then wait returns error (handle removed) | spec_7_subagents.rs | close_then_wait_returns_error | Pass |
| 7.2 | send_input to failed agent returns error | spec_7_subagents.rs | send_input_to_failed_agent_returns_error | Pass |
| 7.2, 9.9 | Unknown agent_id error (send_input) | spec_7_subagents.rs | unknown_agent_id_returns_error | Pass |
| 7.2, 9.9 | Unknown agent_id error (wait) | spec_7_subagents.rs | wait_unknown_agent_returns_error | Pass |
| 7.2, 9.9 | Unknown agent_id error (close_agent) | spec_7_subagents.rs | close_unknown_agent_returns_error | Pass |
| 7.1 | is_subagent_tool classification | spec_7_subagents.rs | is_subagent_tool_recognizes_all_four, is_subagent_tool_rejects_regular_tools | Pass |
| 7.3 | SubAgentResult serde roundtrip | spec_7_subagents.rs | subagent_result_serde_roundtrip | Pass |
| 7.3 | SubAgentStatus serde roundtrip | spec_7_subagents.rs | subagent_status_serde_roundtrip | Pass |
| 7.2 | spawn_agent missing task parameter | spec_7_subagents.rs | spawn_without_task_returns_error | Pass |
| 7.1 | Auto-register subagent tools when depth allows | spec_7_subagents.rs | session_auto_registers_subagent_tools_when_depth_allows | Pass |
| 7.1 | Skip auto-register at max depth | spec_7_subagents.rs | session_does_not_register_subagent_tools_at_max_depth | Pass |
| 9.9 | OpenAI profile includes subagent tools | spec_7_subagents.rs | openai_profile_includes_subagent_tools_after_registration | Pass |
| 9.9 | Anthropic profile includes subagent tools | spec_7_subagents.rs | anthropic_profile_includes_subagent_tools_after_registration | Pass |
| 9.9 | Gemini profile includes subagent tools | spec_7_subagents.rs | gemini_profile_includes_subagent_tools_after_registration | Pass |

## Phase 10: Live Integration Tests

| Spec Section | Requirement | Test File | Test(s) | Status |
|---|---|---|---|---|
| 9.12 | Simple file creation (parity) | spec_9_acceptance.rs | parity_simple_file_creation | Pass (env-gated) |
| 9.12 | Read then edit (parity) | spec_9_acceptance.rs | parity_read_then_edit | Pass (env-gated) |
| 9.12 | Multi-file edit (parity) | spec_9_acceptance.rs | parity_multi_file_edit | Pass (env-gated) |
| 9.12 | Shell execution (parity) | spec_9_acceptance.rs | parity_shell_execution | Pass (env-gated) |
| 9.12 | Shell timeout (parity) | spec_9_acceptance.rs | parity_shell_timeout | Pass (env-gated) |
| 9.12 | Grep + glob (parity) | spec_9_acceptance.rs | parity_grep_glob | Pass (env-gated) |
| 9.12 | Multi-step task (parity) | spec_9_acceptance.rs | parity_multi_step_task | Pass (env-gated) |
| 9.12 | Truncation (parity) | spec_9_acceptance.rs | parity_truncation | Pass (env-gated) |
| 9.12 | Provider-specific editing (parity) | spec_9_acceptance.rs | parity_provider_specific_editing | Pass (env-gated) |
| 9.13 | File creation (smoke) | spec_9_acceptance.rs | smoke_file_creation | Pass (env-gated) |
| 9.13 | Read and edit (smoke) | spec_9_acceptance.rs | smoke_read_and_edit | Pass (env-gated) |
| 9.13 | Shell execution (smoke) | spec_9_acceptance.rs | smoke_shell_execution | Pass (env-gated) |
| 9.13 | Truncation (smoke) | spec_9_acceptance.rs | smoke_truncation | Pass (env-gated) |
| 9.13 | Steering (smoke) | spec_9_acceptance.rs | smoke_steering | Pass (env-gated) |
| 9.13 | Subagent (smoke) | spec_9_acceptance.rs | smoke_subagent | Pass (env-gated) |
| 9.13 | Timeout (smoke) | spec_9_acceptance.rs | smoke_timeout | Pass (env-gated) |

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

## Spec Gaps and Deviations

Intentional spec deviations and current deferred items are documented in the crate README:

- `README.md` → `## Deviations`
- `README.md` → `## Limitations`
