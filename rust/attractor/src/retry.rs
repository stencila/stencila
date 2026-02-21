//! Retry policies and execution (§3.5–3.6).
//!
//! Provides configurable retry behavior with exponential backoff and
//! jitter for transient failures during node execution.

use std::panic::AssertUnwindSafe;
use std::path::Path;
use std::sync::Arc;

use futures::FutureExt;
use rand::RngExt;
use serde::{Deserialize, Serialize};

use crate::context::Context;
use crate::events::{EventEmitter, PipelineEvent};
use crate::graph::{Graph, Node};
use crate::handler::Handler;
use crate::types::{Outcome, StageStatus};

/// Backoff configuration for retry delays.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BackoffConfig {
    /// Initial delay in milliseconds before the first retry.
    pub initial_delay_ms: u64,
    /// Multiplier applied to the delay after each retry.
    pub backoff_factor: f64,
    /// Maximum delay in milliseconds (cap).
    pub max_delay_ms: u64,
    /// Whether to apply random jitter to the delay.
    pub jitter: bool,
}

impl Default for BackoffConfig {
    fn default() -> Self {
        Self {
            initial_delay_ms: 200,
            backoff_factor: 2.0,
            max_delay_ms: 60_000,
            jitter: true,
        }
    }
}

/// A retry policy combining attempt limits with backoff configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the initial attempt).
    pub max_attempts: u32,
    /// Backoff configuration for delays between retries.
    pub backoff: BackoffConfig,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_attempts: 1,
            backoff: BackoffConfig::default(),
        }
    }
}

/// Preset retry policies per §3.6.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RetryPreset {
    /// No retries (1 attempt).
    None,
    /// Standard retry: 5 attempts, 200ms initial, 2x backoff, jitter.
    Standard,
    /// Aggressive retry: 5 attempts, 500ms initial, 2x backoff, jitter.
    Aggressive,
    /// Linear retry: 3 attempts, 500ms initial, 1x backoff (fixed delay), no jitter.
    Linear,
    /// Patient retry: 3 attempts, 2s initial, 3x backoff, jitter.
    Patient,
}

impl RetryPreset {
    /// Convert this preset into a concrete [`RetryPolicy`].
    #[must_use]
    pub fn to_policy(self) -> RetryPolicy {
        match self {
            Self::None => RetryPolicy {
                max_attempts: 1,
                backoff: BackoffConfig::default(),
            },
            Self::Standard => RetryPolicy {
                max_attempts: 5,
                backoff: BackoffConfig {
                    initial_delay_ms: 200,
                    backoff_factor: 2.0,
                    max_delay_ms: 60_000,
                    jitter: true,
                },
            },
            Self::Aggressive => RetryPolicy {
                max_attempts: 5,
                backoff: BackoffConfig {
                    initial_delay_ms: 500,
                    backoff_factor: 2.0,
                    max_delay_ms: 60_000,
                    jitter: true,
                },
            },
            Self::Linear => RetryPolicy {
                max_attempts: 3,
                backoff: BackoffConfig {
                    initial_delay_ms: 500,
                    backoff_factor: 1.0,
                    max_delay_ms: 60_000,
                    jitter: false,
                },
            },
            Self::Patient => RetryPolicy {
                max_attempts: 3,
                backoff: BackoffConfig {
                    initial_delay_ms: 2000,
                    backoff_factor: 3.0,
                    max_delay_ms: 60_000,
                    jitter: true,
                },
            },
        }
    }
}

/// Calculate the delay for a given retry attempt.
///
/// Formula: `initial_delay_ms * factor^(attempt - 1)`, capped at `max_delay_ms`.
/// When jitter is enabled, the delay is multiplied by a random factor
/// in the range `[0.5, 1.5)`.
///
/// `attempt` is 1-based (first retry = attempt 1).
#[must_use]
pub fn delay_for_attempt(attempt: u32, config: &BackoffConfig) -> std::time::Duration {
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_wrap)]
    let base = (config.initial_delay_ms as f64)
        * config
            .backoff_factor
            .powi(i32::saturating_sub(attempt as i32, 1));
    #[allow(clippy::cast_precision_loss)]
    let capped = base.min(config.max_delay_ms as f64);

    let final_ms = if config.jitter {
        let jitter_factor = rand::rng().random_range(0.5..1.5);
        capped * jitter_factor
    } else {
        capped
    };

    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    std::time::Duration::from_millis(final_ms.max(0.0) as u64)
}

/// Build a retry policy for a node from its attributes and graph defaults.
///
/// Resolution order:
/// 1. Node attribute `max_retries`
/// 2. Graph attribute `default_max_retry`
/// 3. Default: 0 retries (1 attempt)
#[must_use]
pub fn build_retry_policy(node: &Node, graph: &Graph) -> RetryPolicy {
    let max_retries = node
        .get_attr("max_retries")
        .and_then(super::graph::AttrValue::as_i64)
        .or_else(|| {
            graph
                .get_graph_attr("default_max_retry")
                .and_then(super::graph::AttrValue::as_i64)
        })
        .unwrap_or(0);

    // Clamp to [0, u32::MAX] to prevent both negative wrap and
    // large-positive truncation (§2.6).
    let max_retries = max_retries.clamp(0, i64::from(u32::MAX));
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    let max_attempts = (max_retries as u32).saturating_add(1);

    RetryPolicy {
        max_attempts,
        backoff: BackoffConfig::default(),
    }
}

/// Execute a handler with retry logic.
///
/// Catches panics via `AssertUnwindSafe` + `catch_unwind`. Sets
/// `internal.retry_count.<node_id>` in the context on each retry.
///
/// Returns the final outcome after retries are exhausted or a
/// terminal result is reached.
#[allow(clippy::too_many_arguments)]
pub async fn execute_with_retry(
    handler: &Arc<dyn Handler>,
    node: &Node,
    context: &Context,
    graph: &Graph,
    logs_root: &Path,
    policy: &RetryPolicy,
    emitter: &dyn EventEmitter,
    stage_index: usize,
) -> Outcome {
    let allows_partial = node
        .get_attr("allow_partial")
        .and_then(super::graph::AttrValue::as_bool)
        .unwrap_or(false);

    let mut attempt = 0u32;

    loop {
        attempt += 1;

        // Execute with panic catching
        let result = AssertUnwindSafe(handler.execute(node, context, graph, logs_root))
            .catch_unwind()
            .await;

        let outcome = match result {
            Ok(Ok(outcome)) => outcome,
            Ok(Err(err)) => {
                if err.is_retryable() && attempt < policy.max_attempts {
                    emit_retry_and_sleep(context, &node.id, attempt, policy, emitter, stage_index)
                        .await;
                    continue;
                }
                return Outcome::fail(format!("handler error: {err}"));
            }
            Err(_panic) => {
                if attempt < policy.max_attempts {
                    emit_retry_and_sleep(context, &node.id, attempt, policy, emitter, stage_index)
                        .await;
                    continue;
                }
                return Outcome::fail("handler panicked during execution".to_string());
            }
        };

        if outcome.status == StageStatus::Retry {
            if attempt < policy.max_attempts {
                emit_retry_and_sleep(context, &node.id, attempt, policy, emitter, stage_index)
                    .await;
                continue;
            }
            // Exhausted retries
            if allows_partial {
                reset_retry_count(context, &node.id);
                return Outcome {
                    status: StageStatus::PartialSuccess,
                    failure_reason: "retries exhausted, accepting partial".into(),
                    ..outcome
                };
            }
            return Outcome::fail(
                outcome
                    .failure_reason
                    .clone()
                    .unwrap_or_else(|| "retries exhausted".into()),
            );
        }

        // Reset retry counter on success/partial success (§3.5).
        if outcome.status.is_success() {
            reset_retry_count(context, &node.id);
        }

        return outcome;
    }
}

/// Record retry count, emit a retrying event, and sleep for the backoff delay.
async fn emit_retry_and_sleep(
    context: &Context,
    node_id: &str,
    attempt: u32,
    policy: &RetryPolicy,
    emitter: &dyn EventEmitter,
    stage_index: usize,
) {
    set_retry_count(context, node_id, attempt);
    emitter.emit(PipelineEvent::StageRetrying {
        node_id: node_id.to_string(),
        stage_index,
        attempt,
        max_attempts: policy.max_attempts,
    });
    let delay = delay_for_attempt(attempt, &policy.backoff);
    tokio::time::sleep(delay).await;
}

/// Set the retry count for a node in the context.
fn set_retry_count(context: &Context, node_id: &str, count: u32) {
    let key = format!("internal.retry_count.{node_id}");
    context.set(key, serde_json::Value::Number(count.into()));
}

/// Reset the retry counter for a node after successful completion (§3.5).
fn reset_retry_count(context: &Context, node_id: &str) {
    let key = format!("internal.retry_count.{node_id}");
    context.set(key, serde_json::Value::Number(0.into()));
}

// The Outcome::failure_reason is a String, not Option<String>.
// Provide a helper trait to handle the "unwrap_or_else" pattern above cleanly.
trait FailureReasonExt {
    fn unwrap_or_else(self, f: impl FnOnce() -> String) -> String;
}

impl FailureReasonExt for String {
    fn unwrap_or_else(self, f: impl FnOnce() -> String) -> String {
        if self.is_empty() { f() } else { self }
    }
}
