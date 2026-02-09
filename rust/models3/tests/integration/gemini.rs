//! Gemini-specific integration tests.
//!
//! All tests are env-gated behind `GEMINI_API_KEY` / `GOOGLE_API_KEY`.

use stencila_models3::api::generate::{GenerateOptions, generate};
use stencila_models3::error::SdkResult;

use super::helpers;

/// provider_options: Gemini safety settings passthrough.
#[tokio::test]
async fn provider_options_safety_settings() -> SdkResult<()> {
    if !helpers::has_provider("gemini") {
        return Ok(());
    }

    let client = helpers::client_from_env()?;

    let opts = GenerateOptions::new(helpers::test_model("gemini"))
        .prompt("Say hello.")
        .max_tokens(20)
        .provider("gemini")
        .provider_options({
            let mut map = std::collections::HashMap::new();
            map.insert(
                "gemini".into(),
                serde_json::json!({
                    "safetySettings": [{
                        "category": "HARM_CATEGORY_HARASSMENT",
                        "threshold": "BLOCK_ONLY_HIGH"
                    }]
                }),
            );
            map
        })
        .client(&client);

    let result = match generate(opts).await {
        Ok(result) => result,
        Err(error) if helpers::should_skip_live_provider_error(&error) => return Ok(()),
        Err(error) => return Err(error),
    };
    assert!(
        !result.text.is_empty(),
        "gemini: expected non-empty text with safety settings"
    );

    Ok(())
}
