use std::time::Duration;

use crate::error::{SdkError, SdkResult};
use crate::retry::{self, RetryPolicy};
use crate::types::message::Message;
use crate::types::response::Response;
use crate::types::response_format::ResponseFormat;
use crate::types::tool::ToolChoice;
use crate::types::usage::Usage;

use super::cancel::AbortSignal;
use super::options::{RequestTemplate, impl_common_builders, resolve_client};
use super::tools::{
    build_messages, execute_all_tools, has_active_tools, has_passive_tool_calls,
    should_execute_tools, tool_definitions, tool_result_messages, validate_prompt_messages,
};
use super::types::{GenerateResult, StepResult, StopCondition, Tool};

/// Options for [`generate()`].
///
/// Use [`GenerateOptions::new()`] and the builder methods to construct.
pub struct GenerateOptions<'a> {
    pub model: String,
    pub prompt: Option<String>,
    pub messages: Vec<Message>,
    pub system: Option<String>,
    pub tools: Vec<Tool>,
    pub tool_choice: Option<ToolChoice>,
    pub max_tool_rounds: u32,
    pub stop_when: Option<StopCondition>,
    pub response_format: Option<ResponseFormat>,
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u64>,
    pub stop_sequences: Option<Vec<String>>,
    pub reasoning_effort: Option<String>,
    pub provider: Option<String>,
    pub provider_options: Option<std::collections::HashMap<String, serde_json::Value>>,
    pub max_retries: u32,
    pub timeout: Option<crate::types::timeout::Timeout>,
    pub abort_signal: Option<AbortSignal>,
    pub client: Option<&'a crate::client::Client>,
}

impl GenerateOptions<'_> {
    /// Create options with the required model parameter.
    #[must_use]
    pub fn new(model: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: None,
            messages: Vec::new(),
            system: None,
            tools: Vec::new(),
            tool_choice: None,
            max_tool_rounds: 1,
            stop_when: None,
            response_format: None,
            temperature: None,
            top_p: None,
            max_tokens: None,
            stop_sequences: None,
            reasoning_effort: None,
            provider: None,
            provider_options: None,
            max_retries: 2,
            timeout: None,
            abort_signal: None,
            client: None,
        }
    }

    #[must_use]
    pub fn tools(mut self, tools: Vec<Tool>) -> Self {
        self.tools = tools;
        self
    }

    #[must_use]
    pub fn tool_choice(mut self, choice: ToolChoice) -> Self {
        self.tool_choice = Some(choice);
        self
    }

    #[must_use]
    pub fn max_tool_rounds(mut self, rounds: u32) -> Self {
        self.max_tool_rounds = rounds;
        self
    }

    #[must_use]
    pub fn stop_when(mut self, condition: StopCondition) -> Self {
        self.stop_when = Some(condition);
        self
    }

    #[must_use]
    pub fn response_format(mut self, format: ResponseFormat) -> Self {
        self.response_format = Some(format);
        self
    }

    #[must_use]
    pub fn top_p(mut self, top_p: f64) -> Self {
        self.top_p = Some(top_p);
        self
    }

    #[must_use]
    pub fn stop_sequences(mut self, sequences: Vec<String>) -> Self {
        self.stop_sequences = Some(sequences);
        self
    }
}

impl_common_builders!(GenerateOptions<'a>);

impl GenerateOptions<'_> {
    /// Build a [`RequestTemplate`] from these options.
    fn request_template(&self) -> RequestTemplate {
        RequestTemplate {
            model: self.model.clone(),
            provider: self.provider.clone(),
            tool_choice: self.tool_choice.clone(),
            response_format: self.response_format.clone(),
            temperature: self.temperature,
            top_p: self.top_p,
            max_tokens: self.max_tokens,
            stop_sequences: self.stop_sequences.clone(),
            reasoning_effort: self.reasoning_effort.clone(),
            provider_options: self.provider_options.clone(),
            timeout: self.timeout,
        }
    }
}

/// High-level blocking generation with automatic tool execution loop.
///
/// Wraps `Client.complete()` with:
/// - Prompt standardization (prompt string → user message)
/// - Multi-step tool execution loop for active tools
/// - Automatic retries per individual LLM call
/// - Usage aggregation across steps
///
/// # Errors
///
/// - `SdkError::Configuration` if both `prompt` and `messages` are provided,
///   or if no client is available.
/// - Provider errors are propagated from the underlying `Client.complete()`.
/// - `SdkError::Abort` if the abort signal is triggered.
#[allow(clippy::too_many_lines)]
pub async fn generate(opts: GenerateOptions<'_>) -> SdkResult<GenerateResult> {
    validate_prompt_messages(opts.prompt.as_deref(), &opts.messages)?;

    // Extract total timeout before moving opts into the inner loop.
    let total_timeout = opts.timeout.and_then(|t| t.total);

    let inner = generate_inner(opts);

    // Apply total timeout if configured (spec §4.7).
    match total_timeout {
        Some(secs) => tokio::time::timeout(Duration::from_secs_f64(secs), inner)
            .await
            .map_err(|_| SdkError::RequestTimeout {
                message: format!("total timeout of {secs}s exceeded"),
            })?,
        None => inner.await,
    }
}

/// Inner implementation of `generate()` — separated so total timeout can wrap it.
async fn generate_inner(opts: GenerateOptions<'_>) -> SdkResult<GenerateResult> {
    let client = resolve_client(opts.client)?;

    let mut conversation = build_messages(
        opts.prompt.as_deref(),
        &opts.messages,
        opts.system.as_deref(),
    );

    let tool_defs = if opts.tools.is_empty() {
        None
    } else {
        Some(tool_definitions(&opts.tools))
    };

    let has_active = has_active_tools(&opts.tools);
    let retry_policy = RetryPolicy {
        max_retries: opts.max_retries,
        ..RetryPolicy::default()
    };

    let per_step_timeout = opts.timeout.and_then(|t| t.per_step);
    let template = opts.request_template();

    let mut steps: Vec<StepResult> = Vec::new();
    let mut total_usage = Usage::default();

    for round_num in 0..=opts.max_tool_rounds {
        if let Some(ref signal) = opts.abort_signal {
            signal.check()?;
        }

        let request = template.to_request(&conversation, tool_defs.as_ref());

        let complete_future = retry::retry(
            &retry_policy,
            || client.complete(request.clone()),
            None,
            None,
        );

        // Apply per-step timeout if configured (spec §4.7).
        let response = match per_step_timeout {
            Some(secs) => tokio::time::timeout(Duration::from_secs_f64(secs), complete_future)
                .await
                .map_err(|_| SdkError::RequestTimeout {
                    message: format!("per-step timeout of {secs}s exceeded"),
                })?,
            None => complete_future.await,
        }?;

        let (step, tool_results) =
            build_step(&response, &opts.tools, has_active, round_num, &opts).await;
        total_usage = total_usage + response.usage.clone();
        steps.push(step);

        if should_stop(
            &steps,
            &response,
            &tool_results,
            has_active,
            round_num,
            &opts,
        ) {
            break;
        }

        // Continue conversation with tool results
        let (assistant_msg, result_msgs) = tool_result_messages(&response, &tool_results);
        conversation.push(assistant_msg);
        conversation.extend(result_msgs);
    }

    build_generate_result(steps, total_usage)
}

async fn build_step(
    response: &Response,
    tools: &[Tool],
    has_active: bool,
    round_num: u32,
    opts: &GenerateOptions<'_>,
) -> (StepResult, Vec<crate::types::tool::ToolResult>) {
    let tool_calls = response.tool_calls();

    let tool_results = if has_active
        && should_execute_tools(&tool_calls, &response.finish_reason)
        && round_num < opts.max_tool_rounds
    {
        execute_all_tools(tools, &tool_calls, opts.abort_signal.as_ref()).await
    } else {
        Vec::new()
    };

    let step = StepResult::new(response, tool_calls, tool_results.clone());
    (step, tool_results)
}

fn should_stop(
    steps: &[StepResult],
    response: &Response,
    tool_results: &[crate::types::tool::ToolResult],
    has_active: bool,
    round_num: u32,
    opts: &GenerateOptions<'_>,
) -> bool {
    let tool_calls = response.tool_calls();

    if !should_execute_tools(&tool_calls, &response.finish_reason) {
        return true; // model is done
    }
    if round_num >= opts.max_tool_rounds {
        return true; // budget exhausted
    }
    if !has_active {
        return true; // passive tools only
    }
    if tool_results.is_empty() {
        return true; // no results to feed back
    }
    // Mixed active+passive: stop if any passive tool was called so
    // the caller can handle those calls manually.
    if has_passive_tool_calls(&opts.tools, &tool_calls) {
        return true;
    }
    if let Some(ref stop_fn) = opts.stop_when
        && stop_fn(steps)
    {
        return true; // custom stop condition
    }
    false
}

fn build_generate_result(steps: Vec<StepResult>, total_usage: Usage) -> SdkResult<GenerateResult> {
    let last = steps
        .last()
        .ok_or_else(|| crate::error::SdkError::Configuration {
            message: "generate() completed with no steps".into(),
        })?
        .clone();

    Ok(GenerateResult {
        text: last.text,
        reasoning: last.reasoning,
        tool_calls: last.tool_calls,
        tool_results: last.tool_results,
        finish_reason: last.finish_reason,
        usage: last.usage,
        total_usage,
        steps,
        response: last.response,
        output: None,
    })
}
