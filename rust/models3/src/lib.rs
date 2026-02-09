// SdkError is large (~168 bytes) because ProviderDetails embeds
// Option<serde_json::Value>. This is a deliberate trade-off for rich error
// context; errors are not on the hot path. Boxing ProviderDetails would
// reduce size but adds indirection at every construction site.
#![allow(clippy::result_large_err)]
#![warn(clippy::pedantic)]

pub mod catalog;
pub mod error;
pub mod http;
pub mod provider;
pub mod retry;
pub mod types;
