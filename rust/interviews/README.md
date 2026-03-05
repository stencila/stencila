# Stencila Interviews

Human-in-the-loop interview infrastructure for Stencila.

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

## Features

- **`sqlite`** — Enables the `interviews` table schema, `PersistentInterviewer`, and DB helper functions. Adds dependencies on `stencila-db`, `chrono`, and `uuid`.
