use crate::error::{SdkError, SdkResult};
use crate::types::message::Message;
use crate::types::response_format::{ResponseFormat, ResponseFormatType};

use super::cancel::AbortSignal;
use super::options::impl_common_builders;
use super::stream::{CollectedStreamResult, StreamOptions, stream_generate};
use super::tools::validate_against_schema;

/// Options for [`stream_object()`].
pub struct StreamObjectOptions<'a> {
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

impl<'a> StreamObjectOptions<'a> {
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

    /// Convert into `StreamOptions` plus the extracted schema.
    fn into_stream_opts(self) -> (StreamOptions<'a>, serde_json::Value) {
        let schema = self.schema.clone();

        let response_format = ResponseFormat {
            format_type: ResponseFormatType::JsonSchema,
            json_schema: Some(self.schema),
            strict: true,
        };

        let stream_opts = StreamOptions {
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

        (stream_opts, schema)
    }
}

impl_common_builders!(StreamObjectOptions<'a>);

/// Result of streaming structured output generation.
#[derive(Debug)]
pub struct StreamObjectResult {
    /// The collected streaming result including events and response.
    pub stream_result: CollectedStreamResult,
    /// The parsed, validated output object.
    pub output: serde_json::Value,
}

/// Stream structured output conforming to a JSON schema.
///
/// Uses `response_format: { type: "json_schema", ... }` and streams
/// events. After the stream completes, the accumulated text is parsed
/// as JSON and validated against the schema.
///
/// # Limitations
///
/// Spec §4.6 requires incremental JSON parsing to yield partial
/// objects as tokens arrive, enabling progressive UI rendering. The
/// current implementation collects the entire stream then parses once.
/// This is a correctness-preserving simplification; partial object
/// streaming can be added as a future enhancement.
///
/// # Errors
///
/// - `SdkError::NoObjectGenerated` if the final response cannot be
///   parsed as valid JSON or fails schema validation.
/// - All errors from [`stream_generate()`].
///
/// Schema validation failures are NOT retried (spec Section 6.6).
pub async fn stream_object(opts: StreamObjectOptions<'_>) -> SdkResult<StreamObjectResult> {
    let (stream_opts, schema) = opts.into_stream_opts();

    let stream_result = stream_generate(stream_opts).await?.collect().await?;

    // Parse the accumulated response text as JSON
    let text = stream_result.response.text();
    let output: serde_json::Value =
        serde_json::from_str(&text).map_err(|e| SdkError::NoObjectGenerated {
            message: format!("failed to parse streamed response as JSON: {e}"),
        })?;

    // Validate against schema
    validate_against_schema(&output, &schema)?;

    Ok(StreamObjectResult {
        stream_result,
        output,
    })
}
