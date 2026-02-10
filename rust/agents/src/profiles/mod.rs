//! Provider profile implementations (spec 3.4-3.6).
//!
//! Each submodule provides a profile struct that implements
//! [`ProviderProfile`](crate::profile::ProviderProfile) with the
//! provider's native tool set and capability flags.

pub mod anthropic;
pub mod gemini;
pub mod openai;

pub use anthropic::AnthropicProfile;
pub use gemini::GeminiProfile;
pub use openai::OpenAiProfile;
