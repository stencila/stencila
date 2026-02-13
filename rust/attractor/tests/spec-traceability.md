# Spec Traceability Matrix

Maps each spec section to the test file(s) and specific test functions that validate it. Updated as implementation progresses.

## Status Key

- Pending: not yet implemented
- Partial: some tests exist
- Complete: all required behaviors tested

## Traceability

| Spec Section | Description | Test File | Status |
|---|---|---|---|
| §2.1 Supported Subset | DOT subset constraints | `spec_2_parser.rs` | Complete |
| §2.2 BNF Grammar | Parser grammar coverage | `spec_2_parser.rs` | Complete |
| §2.3 Key Constraints | One digraph, bare IDs, commas, directed only | `spec_2_parser.rs` | Partial |
| §2.4 Value Types | String, Integer, Float, Boolean, Duration | `spec_1_types.rs`, `spec_2_parser.rs` | Complete |
| §2.5 Graph Attributes | goal, label, model_stylesheet, defaults | `spec_2_parser.rs` | Complete |
| §2.6 Node Attributes | All node attribute types and defaults (auto_status deferred to Phase 5) | `spec_2_parser.rs`, `spec_3_engine.rs` | Partial |
| §2.7 Edge Attributes | label, condition, weight, fidelity, loop_restart (fresh run dir) | `spec_2_parser.rs`, `spec_3_engine.rs` | Partial |
| §2.8 Shape-to-Handler Mapping | Shape resolution to handler types | `spec_2_parser.rs` | Complete |
| §2.9 Chained Edges | A -> B -> C expansion | `spec_2_parser.rs` | Complete |
| §2.10 Subgraphs | Scoping defaults, class derivation | `spec_2_parser.rs` | Partial |
| §2.11 Default Blocks | node [...], edge [...] defaults | `spec_2_parser.rs` | Complete |
| §2.12 Class Attribute | Comma-separated classes | `spec_2_parser.rs` | Partial |
| §2.13 Minimal Examples | 3 example pipelines | `spec_2_parser.rs` | Complete |
| §3.1 Run Lifecycle | 5-phase lifecycle | `spec_3_engine.rs` | Complete |
| §3.2 Core Execution Loop | Traversal algorithm | `spec_3_engine.rs` | Complete |
| §3.3 Edge Selection | 5-step priority algorithm | `spec_3_engine.rs` | Complete |
| §3.4 Goal Gate Enforcement | Goal gate check at exit | `spec_3_engine.rs` | Complete |
| §3.5 Retry Logic | max_retries, backoff, jitter | `spec_3_engine.rs` | Complete |
| §3.6 Retry Policy | Preset policies, backoff config | `spec_3_engine.rs` | Complete |
| §3.7 Failure Routing | Fail edge, retry_target chain | `spec_3_engine.rs` | Complete |
| §3.8 Concurrency Model | Single-threaded traversal | `spec_3_engine.rs` | Complete |
| §4.1 Handler Interface | Handler trait contract | `spec_3_engine.rs` | Complete |
| §4.2 Handler Registry | Type-based resolution | `spec_3_engine.rs` | Complete |
| §4.3 Start Handler | No-op SUCCESS | `spec_3_engine.rs` | Complete |
| §4.4 Exit Handler | No-op SUCCESS | `spec_3_engine.rs` | Complete |
| §4.5 Codergen Handler | LLM task, backend, simulation | `spec_4_handlers.rs` | Partial |
| §4.6 Wait For Human | Edge-derived choices, accelerators | `spec_6_human.rs` | Pending |
| §4.7 Conditional Handler | No-op, routing via engine | `spec_3_engine.rs` | Complete |
| §4.8 Parallel Handler | Fan-out, join/error policies | `spec_4_parallel.rs` | Pending |
| §4.9 Fan-In Handler | Heuristic selection | `spec_4_parallel.rs` | Pending |
| §4.10 Tool Handler | Shell command execution | `spec_4_handlers.rs` | Partial |
| §4.12 Custom Handlers | Handler contract, panic catching | `spec_3_engine.rs` | Complete |
| §5.1 Context | Key-value store, built-in keys | `spec_1_types.rs` | Partial |
| §5.2 Outcome | Status, preferred_label, context_updates | `spec_1_types.rs` | Complete |
| §5.3 Checkpoint | Save/load, resume behavior | `spec_1_types.rs`, `spec_5_resume.rs` | Partial |
| §5.4 Context Fidelity | Fidelity modes, resolution, thread_id | `spec_5_state.rs` | Pending |
| §5.5 Artifact Store | File-backed/in-memory storage | `spec_5_state.rs` | Pending |
| §5.6 Run Directory | Directory structure, manifest.json | `spec_3_engine.rs` | Complete |
| §6.1 Interviewer Interface | ask, ask_multiple, inform | `spec_6_human.rs` | Pending |
| §6.2 Question Model | QuestionType variants | `spec_6_human.rs` | Pending |
| §6.3 Answer Model | AnswerValue variants | `spec_6_human.rs` | Pending |
| §6.4 Built-In Interviewers | AutoApprove, Queue, Callback, Recording | `spec_6_human.rs` | Pending |
| §6.5 Timeout Handling | Default answer, TIMEOUT | `spec_6_human.rs` | Pending |
| §7.1 Diagnostic Model | Diagnostic, Severity | `spec_7_validation.rs` | Pending |
| §7.2 Built-In Lint Rules | 13 rules | `spec_7_validation.rs` | Pending |
| §7.3 Validation API | validate(), validate_or_raise() | `spec_7_validation.rs` | Pending |
| §7.4 Custom Lint Rules | LintRule trait, registration | `spec_7_validation.rs` | Pending |
| §8.1 Stylesheet Overview | CSS-like model configuration | `spec_8_stylesheet.rs` | Pending |
| §8.2 Stylesheet Grammar | Selector/declaration parsing | `spec_8_stylesheet.rs` | Pending |
| §8.3 Selectors and Specificity | *, .class, #id precedence | `spec_8_stylesheet.rs` | Pending |
| §8.4 Recognized Properties | llm_model, llm_provider, reasoning_effort | `spec_8_stylesheet.rs` | Pending |
| §8.5 Application Order | Resolution precedence | `spec_8_stylesheet.rs` | Pending |
| §8.6 Example | Full stylesheet example | `spec_8_stylesheet.rs` | Pending |
| §9.1 AST Transforms | Transform trait and pipeline | `spec_9_transforms.rs` | Complete |
| §9.2 Built-In Transforms | Variable expansion, stylesheet | `spec_9_transforms.rs` | Partial |
| §9.3 Custom Transforms | Registration and ordering | `spec_9_transforms.rs` | Complete |
| §9.6 Events | Pipeline/Stage/Parallel/Interview/Checkpoint events | `spec_3_engine.rs` | Partial |
| §10.1–10.5 Conditions | Grammar, semantics, evaluation | `spec_10_conditions.rs` | Complete |
| §10.6 Condition Examples | 5 verbatim examples | `spec_10_conditions.rs` | Complete |
| §11.12 Parity Matrix | 21 cross-feature test cases | `spec_11_acceptance.rs` | Pending |
| §11.13 Integration Smoke | End-to-end with LLM callback | `spec_11_acceptance.rs` | Pending |
