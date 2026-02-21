//! Ollama-specific integration tests.
//!
//! All tests are env-gated behind `OLLAMA_BASE_URL` or `OLLAMA_HOST`.
#![allow(clippy::result_large_err)]

use stencila_models3::api::generate::{GenerateOptions, generate};
use stencila_models3::error::SdkResult;

use super::helpers;

/// Basic generation via Ollama.
#[tokio::test]
async fn basic_generation() -> SdkResult<()> {
    if !helpers::has_provider("ollama") {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    let opts = GenerateOptions::new(helpers::test_model("ollama"))
        .prompt("Say hello in one sentence.")
        .max_tokens(50)
        .provider("ollama")
        .client(&client);

    let result = generate(opts).await?;
    assert!(!result.text.is_empty(), "ollama: expected non-empty text");

    Ok(())
}
