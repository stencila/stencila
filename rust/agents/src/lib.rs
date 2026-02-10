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

/// Async event delivery: emitter/receiver pair for session events (spec 2.9).
pub mod events;

/// Execution environment abstraction and local implementation (spec 4.1-4.2).
pub mod execution;

/// Tool output truncation: character-based and line-based (spec 5.1-5.3).
pub mod truncation;

/// Tool registry: name → executor mapping with validation (spec 3.8).
pub mod registry;

/// Provider profile trait: tool set + capability metadata (spec 3.2).
pub mod profile;

/// Provider profile implementations: OpenAI, Anthropic, Gemini (spec 3.4-3.6).
pub mod profiles;

/// System prompt helpers: environment context, git context (spec 6.3-6.4).
pub mod prompts;

/// Project document discovery: AGENTS.md, CLAUDE.md, etc. (spec 6.5).
pub mod project_docs;

/// Core tool implementations: executors and schema definitions (spec 3.3, 3.6).
pub mod tools;

/// Core domain types: configuration, session state, turns, events.
pub mod types;

/// Tool-call loop detection (spec 2.10).
pub mod loop_detection;

/// Subagent spawning and lifecycle management (spec 7.1-7.4).
pub mod subagents;

/// Agent session and core agentic loop (spec 2.1, 2.5-2.8, 2.10, Appendix B).
pub mod session;
