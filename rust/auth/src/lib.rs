//! Authentication types and OAuth login flows for LLM providers.
//!
//! The [`credentials`] module provides the core credential abstraction
//! ([`AuthCredential`], [`StaticKey`], [`OAuthToken`]) used by LLM
//! client crates. These types are always available, even without the
//! `login` feature.
//!
//! When the `login` feature is enabled, this crate also provides
//! browser-based OAuth login flows for specific providers:
//!
//! - **Anthropic** — Authorization Code + PKCE with manual code paste.
//! - **Gemini** — Authorization Code + PKCE with local callback server.
//! - **`OpenAI` (Codex)** — Authorization Code + PKCE with local callback server.
//! - **GitHub Copilot** — Device Code Grant with poll-based verification.
//!
//! Credentials are persisted to the system keyring via [`stencila_secrets`]
//! and can be loaded into [`OAuthToken`] for automatic refresh during
//! API calls.

#![allow(clippy::result_large_err)]
#![warn(clippy::pedantic)]

// ---------------------------------------------------------------------------
// Core credential types (always available)
// ---------------------------------------------------------------------------

/// Authentication credentials (static keys and OAuth tokens).
mod credentials;
pub use credentials::*;

// ---------------------------------------------------------------------------
// OAuth login flows (feature-gated per provider)
// ---------------------------------------------------------------------------

/// Anthropic OAuth login flow (Authorization Code + PKCE).
#[cfg(feature = "anthropic")]
pub mod anthropic;

/// GitHub Copilot OAuth login flow (Device Code Grant).
#[cfg(feature = "copilot")]
pub mod copilot;

/// Google Gemini OAuth login flow (Authorization Code + PKCE).
#[cfg(feature = "gemini")]
pub mod gemini;

/// OpenAI OAuth login flow (Authorization Code + PKCE).
#[cfg(feature = "openai")]
pub mod openai;

// ---------------------------------------------------------------------------
// Shared OAuth utilities
// ---------------------------------------------------------------------------

/// Local HTTP callback server for OAuth redirects.
#[cfg(feature = "login")]
pub mod callback;

/// Credential persistence, loading, and orchestration.
#[cfg(feature = "login")]
pub mod persist;

/// PKCE (Proof Key for Code Exchange) challenge generation.
#[cfg(feature = "login")]
pub mod pkce;

// Re-export commonly used orchestration functions.
#[cfg(feature = "login")]
pub use persist::{build_oauth_token, load_auth_overrides};

// ---------------------------------------------------------------------------
// External credential detection
// ---------------------------------------------------------------------------

/// Auto-detect Claude Code OAuth credentials.
#[cfg(feature = "login")]
pub mod claude_code;

/// Auto-detect Codex CLI OAuth credentials.
#[cfg(feature = "login")]
pub mod codex_cli;

// ---------------------------------------------------------------------------
// CLI commands
// ---------------------------------------------------------------------------

/// Command-line interface for OAuth login commands.
#[cfg(feature = "cli")]
pub mod cli;
