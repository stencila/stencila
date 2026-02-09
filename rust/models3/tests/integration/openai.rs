//! OpenAI-specific integration tests.
//!
//! All tests are env-gated behind `OPENAI_API_KEY`.

use stencila_models3::api::generate::{GenerateOptions, generate};
use stencila_models3::error::SdkResult;

use super::helpers;

/// provider_options: OpenAI custom headers passthrough.
#[tokio::test]
async fn provider_options_custom_headers() -> SdkResult<()> {
    if !helpers::has_provider("openai") {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    let opts = GenerateOptions::new(helpers::test_model("openai"))
        .prompt("Say hello.")
        .max_tokens(20)
        .provider("openai")
        .provider_options({
            let mut map = std::collections::HashMap::new();
            map.insert(
                "openai".into(),
                serde_json::json!({
                    "custom_headers": {
                        "X-Custom-Test-Header": "test-value"
                    }
                }),
            );
            map
        })
        .client(&client);

    let result = generate(opts).await?;
    assert!(
        !result.text.is_empty(),
        "openai: expected non-empty text with custom headers"
    );

    Ok(())
}
