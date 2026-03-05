# Stencila Interviews

Human-in-the-loop interview infrastructure for Stencila. An implementation human-in-the-loop sections of the [Attractor Specification](https://github.com/strongdm/attractor/blob/main/attractor-spec.md). Section numbers refer to that spec.

## What this crate contains

- **`Interviewer` trait** — the async trait for presenting questions to humans and receiving answers (§6.1)
- **`Question`, `QuestionType`, `QuestionOption`** — the question model (§6.2)
- **`Answer`, `AnswerValue`** — the answer model (§6.3)
- **Built-in implementations** — `AutoApproveInterviewer`, `CallbackInterviewer`, `QueueInterviewer`, `RecordingInterviewer` (§6.4)
- **`interviews` SQLite table** — context-agnostic persistence schema for recording human-in-the-loop interactions (requires `sqlite` feature)
- **`PersistentInterviewer`** — a decorator that wraps any `Interviewer` and persists interview records to the database (requires `sqlite` feature)

## Why it exists as a separate crate

The `Interviewer` trait and its associated types are needed by both the `attractor` crate (pipeline engine, for `WaitForHumanHandler` gates) and the `agents` crate (for the `ask_user` tool). Since `attractor` depends on `agents`, having `agents` depend back on `attractor` for these types would create a circular dependency. Extracting them into this small, standalone crate lets both depend on `stencila-interviews` directly — no duplicate traits, no lossy adapters, no information loss.

## The `interviews` table

The `interviews` table is context-agnostic: it records human-in-the-loop interactions from both workflow pipeline gates (`context_type = 'workflow'`, `context_id` = run ID) and standalone agent sessions (`context_type = 'agent_session'`, `context_id` = session ID). The `PersistentInterviewer` decorator writes to this table, so every interviewer implementation benefits from DB persistence when wrapped.

## Spec deviations

### `AnswerValue` wire format (§6.3)

The spec defines `AnswerValue` as a flat set of unit constants (`YES`, `NO`, `SKIPPED`, `TIMEOUT`) with selected values and free text carried in separate `Answer` fields (`value: String or AnswerValue`, `text: String`). This implementation unifies all answer semantics into `AnswerValue` enum variants, including `Selected(String)`, `MultiSelected(Vec<String>)`, and `Text(String)`. The enum uses adjacently tagged serde representation (`#[serde(tag = "type", content = "value")]`) for consistency with the codebase's enum tagging convention. Unit variants serialize as `{"type":"YES"}`, data variants as `{"type":"SELECTED","value":"A"}`.

### `MultiSelect` question type and `MultiSelected` answer value

The spec's §6.2 defines four question types (`YES_NO`, `MULTIPLE_CHOICE`, `FREEFORM`, `CONFIRMATION`). This implementation adds `MULTI_SELECT` (select multiple options from a list) and a corresponding `MultiSelected(Vec<String>)` answer value. This is motivated by §11.8's mention of `MULTI_SELECT` and by the multi-question interview pattern (e.g. Claude Code `AskUserQuestions` tool).

### Additional fields on `Question` and `QuestionOption`

The spec's §6.2 `Question` has `text`, `type`, `options`, `default`, `timeout_seconds`, `stage`, and `metadata`. This implementation adds `id: Option<String>` (for correlating questions with DB rows) and `header: Option<String>` (for rendering grouped/headed question forms). `QuestionOption` gains `description: Option<String>` for explanatory text alongside the label.

## Features

- **`sqlite`** — Enables the `interviews` table schema, `PersistentInterviewer`, and DB helper functions. Adds dependencies on `stencila-db`, `chrono`, and `uuid`.
