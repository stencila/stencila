//! Anthropic-specific integration tests.
//!
//! All tests are env-gated behind `ANTHROPIC_API_KEY`.
#![allow(clippy::result_large_err)]

use stencila_models3::api::generate::{GenerateOptions, generate};
use stencila_models3::error::SdkResult;

use super::helpers;

/// provider_options: Anthropic beta_headers passthrough.
#[tokio::test]
async fn provider_options_beta_headers() -> SdkResult<()> {
    if !helpers::has_provider("anthropic") {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    let opts = GenerateOptions::new(helpers::test_model("anthropic"))
        .prompt("Say hello.")
        .max_tokens(20)
        .provider("anthropic")
        .provider_options({
            let mut map = std::collections::HashMap::new();
            map.insert(
                "anthropic".into(),
                serde_json::json!({
                    "beta_headers": ["prompt-caching-2024-07-31"]
                }),
            );
            map
        })
        .client(&client);

    let result = generate(opts).await?;
    assert!(
        !result.text.is_empty(),
        "anthropic: expected non-empty text with beta headers"
    );

    Ok(())
}

/// provider_options: Anthropic auto_cache disabled.
#[tokio::test]
async fn provider_options_auto_cache_disabled() -> SdkResult<()> {
    if !helpers::has_provider("anthropic") {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    let opts = GenerateOptions::new(helpers::test_model("anthropic"))
        .prompt("Say hello.")
        .max_tokens(20)
        .provider("anthropic")
        .provider_options({
            let mut map = std::collections::HashMap::new();
            map.insert(
                "anthropic".into(),
                serde_json::json!({
                    "auto_cache": false
                }),
            );
            map
        })
        .client(&client);

    let result = generate(opts).await?;
    assert!(
        !result.text.is_empty(),
        "anthropic: expected non-empty text with auto_cache disabled"
    );

    Ok(())
}
