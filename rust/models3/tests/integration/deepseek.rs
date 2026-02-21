//! DeepSeek-specific integration tests.
//!
//! All tests are env-gated behind `DEEPSEEK_API_KEY`.
#![allow(clippy::result_large_err)]

use stencila_models3::api::generate::{GenerateOptions, generate};
use stencila_models3::error::SdkResult;

use super::helpers;

/// Basic generation via DeepSeek.
#[tokio::test]
async fn basic_generation() -> SdkResult<()> {
    if !helpers::has_provider("deepseek") {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    let opts = GenerateOptions::new(helpers::test_model("deepseek"))
        .prompt("Say hello in one sentence.")
        .max_tokens(50)
        .provider("deepseek")
        .client(&client);

    let result = generate(opts).await?;
    assert!(!result.text.is_empty(), "deepseek: expected non-empty text");

    Ok(())
}
