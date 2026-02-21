use crate::error::{SdkError, SdkResult};
use crate::types::message::Message;
use crate::types::response_format::{ResponseFormat, ResponseFormatType};

use super::cancel::AbortSignal;
use super::generate::{GenerateOptions, generate};
use super::options::impl_common_builders;
use super::tools::validate_against_schema;
use super::types::GenerateResult;

/// Options for [`generate_object()`].
pub struct GenerateObjectOptions<'a> {
    pub model: String,
    pub prompt: Option<String>,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub schema: serde_json::Value,
    pub temperature: Option<f64>,
    pub max_tokens: Option<u64>,
    pub reasoning_effort: Option<String>,
    pub provider: Option<String>,
    pub provider_options: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub max_retries: u32,
    pub timeout: Option<crate::types::timeout::Timeout>,
    pub abort_signal: Option<AbortSignal>,
    pub client: Option<&'a crate::client::Client>,
}

impl<'a> GenerateObjectOptions<'a> {
    #[must_use]
    pub fn new(model: impl Into<String>, schema: serde_json::Value) -> Self {
        Self {
            model: model.into(),
            prompt: None,
            messages: Vec::new(),
            system: None,
            schema,
            temperature: None,
            max_tokens: None,
            reasoning_effort: None,
            provider: None,
            provider_options: None,
            max_retries: 2,
            timeout: None,
            abort_signal: None,
            client: None,
        }
    }

    /// Convert into `GenerateOptions` plus the extracted schema.
    fn into_generate_opts(self) -> (GenerateOptions<'a>, serde_json::Value) {
        let schema = self.schema.clone();

        let response_format = ResponseFormat {
            format_type: ResponseFormatType::JsonSchema,
            json_schema: Some(self.schema),
            strict: true,
        };

        let gen_opts = GenerateOptions {
            model: self.model,
            prompt: self.prompt,
            messages: self.messages,
            system: self.system,
            tools: Vec::new(),
            tool_choice: None,
            max_tool_rounds: 1,
            stop_when: None,
            response_format: Some(response_format),
            temperature: self.temperature,
            top_p: None,
            max_tokens: self.max_tokens,
            stop_sequences: None,
            reasoning_effort: self.reasoning_effort,
            provider: self.provider,
            provider_options: self.provider_options,
            max_retries: self.max_retries,
            timeout: self.timeout,
            abort_signal: self.abort_signal,
            client: self.client,
        };

        (gen_opts, schema)
    }
}

impl_common_builders!(GenerateObjectOptions<'a>);

/// Generate structured output conforming to a JSON schema.
///
/// Uses `response_format: { type: "json_schema", ... }` to instruct the
/// provider to produce structured JSON. The response text is parsed and
/// validated against the schema.
///
/// # Limitations
///
/// TODO: Spec §4.5 defines per-provider strategies for structured output.
/// Anthropic does not natively support `json_schema` response format and
/// requires a fallback: either inject schema instructions into the system
/// prompt, or use tool-based extraction (define a tool whose input schema
/// matches the desired output and force the model to call it). The current
/// implementation uses `json_schema` uniformly across all providers.
///
/// # Errors
///
/// - `SdkError::NoObjectGenerated` if the response cannot be parsed as
///   valid JSON or doesn't match the expected structure.
/// - All errors from [`generate()`](super::generate::generate).
///
/// **Retry constraint:** Schema validation failures are NOT retried
/// (they indicate model behavior, not transient errors).
pub async fn generate_object(opts: GenerateObjectOptions<'_>) -> SdkResult<GenerateResult> {
    let (gen_opts, schema) = opts.into_generate_opts();

    let mut result = generate(gen_opts).await?;

    // Parse the response text as JSON
    let text = &result.text;
    let parsed: serde_json::Value =
        serde_json::from_str(text).map_err(|e| SdkError::NoObjectGenerated {
            message: format!("failed to parse response as JSON: {e}"),
        })?;

    // Validate against schema
    validate_against_schema(&parsed, &schema)?;

    result.output = Some(parsed);
    Ok(result)
}
