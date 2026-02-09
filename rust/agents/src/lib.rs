//! Coding agent loop — a programmable agentic loop that pairs LLMs with
//! developer tools.
//!
//! This crate implements the [Coding Agent Loop Specification][spec],
//! providing a library-level interface for building coding agents. It
//! builds on [`stencila_models3`] for LLM communication and adds:
//!
//! - **Provider-aligned tool profiles** — each model family gets its native
//!   tools and system prompts.
//! - **Execution environment abstraction** — tools run locally by default
//!   but the same logic works in Docker, Kubernetes, or WASM.
//! - **Event-driven observation** — every agent action emits a typed event
//!   for UI rendering, logging, and integration.
//! - **Programmable control** — steering, follow-up queues, subagents, and
//!   mid-turn configuration changes.
//!
//! [spec]: https://github.com/anthropics/coding-agent-loop-spec

// AgentError is large (~176 bytes) because it wraps SdkError which embeds
// Option<serde_json::Value> for rich error context. This is a deliberate
// trade-off matching models3 — errors are not on the hot path.
#![allow(clippy::result_large_err)]

/// Agent-level error hierarchy.
pub mod error;

/// Core domain types: configuration, session state, turns, events.
pub mod types;
