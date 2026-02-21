//! Mistral-specific integration tests.
//!
//! All tests are env-gated behind `MISTRAL_API_KEY`.
#![allow(clippy::result_large_err)]

use stencila_models3::api::generate::{GenerateOptions, generate};
use stencila_models3::error::SdkResult;

use super::helpers;

/// Basic generation via Mistral.
#[tokio::test]
async fn basic_generation() -> SdkResult<()> {
    if !helpers::has_provider("mistral") {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    let opts = GenerateOptions::new(helpers::test_model("mistral"))
        .prompt("Say hello in one sentence.")
        .max_tokens(50)
        .provider("mistral")
        .client(&client);

    let result = generate(opts).await?;
    assert!(!result.text.is_empty(), "mistral: expected non-empty text");

    Ok(())
}
