//! Unified LLM client for OpenAI, Anthropic, Gemini, Mistral, DeepSeek, and Ollama.
//!
//! This crate provides a single interface across multiple LLM providers,
//! using each provider's **native API** (not compatibility layers), plus an
//! OpenAI-compatible adapter for third-party Chat Completions endpoints.
//!
//! # Quick start
//!
//! ```no_run
//! use stencila_models3::api::generate::{GenerateOptions, generate};
//! use stencila_models3::client::Client;
//!
//! # async fn example() -> stencila_models3::error::SdkResult<()> {
//! let client = Client::from_env()?;
//! let result = generate(
//!     GenerateOptions::new("claude-sonnet-4-5-20250929")
//!         .prompt("Say hello")
//!         .client(&client),
//! ).await?;
//! println!("{}", result.text);
//! # Ok(())
//! # }
//! ```
//!
//! # Architecture
//!
//! - [`types`] — Unified request/response types shared across all providers.
//! - [`provider`] — The `ProviderAdapter` trait that each provider implements.
//! - [`providers`] — Concrete adapter implementations (OpenAI, Anthropic, Gemini, Mistral, DeepSeek, Ollama, Chat Completions).
//! - [`client`] — The `Client` orchestration layer with routing and middleware.
//! - [`middleware`] — The `Middleware` trait for intercepting requests and responses.
//! - [`api`] — High-level functions: `generate()`, `stream_generate()`, `generate_object()`.
//! - [`catalog`] — Model metadata catalog with lookup, listing, and runtime refresh.
//! - [`error`] — Unified error hierarchy with retryability classification.

// SdkError is large (~168 bytes) because ProviderDetails embeds
// Option<serde_json::Value>. This is a deliberate trade-off for rich error
// context; errors are not on the hot path. Boxing ProviderDetails would
// reduce size but adds indirection at every construction site.
#![allow(clippy::result_large_err)]
#![warn(clippy::pedantic)]

/// High-level generation, streaming, structured output, and tool execution.
pub mod api;
/// Model metadata catalog with lookup, listing, and runtime refresh.
pub mod catalog;
/// Client construction, provider routing, and middleware application.
pub mod client;
/// Unified error hierarchy and retryability classification.
pub mod error;
/// HTTP client helpers and SSE parser (mostly internal).
pub mod http;
/// Middleware trait for intercepting `complete()` and `stream()` calls.
pub mod middleware;
/// The `ProviderAdapter` trait and associated type aliases.
pub mod provider;
/// Concrete provider adapter implementations.
pub mod providers;
/// Retry policy and exponential backoff (mostly internal).
pub mod retry;
/// Secret retrieval with optional keyring support (via `secrets` feature).
pub mod secret;
/// Unified request, response, message, and content types.
pub mod types;
