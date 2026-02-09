//! Retry logic with exponential backoff and jitter.
//!
//! Implements the retry policy from spec Section 6.6:
//!
//! - Exponential backoff: `delay = min(base_delay * multiplier^n, max_delay)`
//! - Jitter: `delay *= random(0.5, 1.5)` (±50%)
//! - `Retry-After` override: if the error carries a `retry_after` value
//!   less than `max_delay`, use it instead of the calculated delay. If it
//!   exceeds `max_delay`, raise the error immediately.
//!
//! Retries apply to **individual LLM calls**, not to entire multi-step
//! operations. Callers are responsible for enforcing this constraint
//! (e.g., no retry after partial stream delivery).
//!
//! # Why custom instead of a crate
//!
//! We evaluated `backon` (v1.6, ~10M downloads/month) and `tokio-retry2`
//! (v0.9.1). `backon` in particular has a well-designed `.adjust()` API
//! for Retry-After overrides. However, our retry logic is only ~60 lines
//! and encodes spec-exact semantics that would need careful mapping to any
//! crate: the jitter range is specifically ±50% (spec §6.6 requires
//! `RANDOM(0.5, 1.5)`, while `backon` uses a different range), and
//! `Retry-After` exceeding `max_delay` must abort immediately rather than
//! retry with a capped delay. The implementation is simple enough that the
//! maintenance cost is negligible compared to the integration glue an
//! external crate would require.

use std::future::Future;
use std::time::Duration;

use rand::Rng;

use crate::error::{SdkError, SdkResult};

/// Configuration for retry behavior.
///
/// All fields have defaults per spec Section 6.6.
#[derive(Debug, Clone)]
pub struct RetryPolicy {
    /// Maximum number of retry attempts (not counting the initial attempt).
    pub max_retries: u32,

    /// Initial delay in seconds before the first retry.
    pub base_delay: f64,

    /// Maximum delay between retries in seconds.
    pub max_delay: f64,

    /// Multiplier applied to the delay after each attempt.
    pub backoff_multiplier: f64,

    /// Whether to add random jitter (±50%) to the delay.
    pub jitter: bool,
}

impl Default for RetryPolicy {
    fn default() -> Self {
        Self {
            max_retries: 2,
            base_delay: 1.0,
            max_delay: 60.0,
            backoff_multiplier: 2.0,
            jitter: true,
        }
    }
}

impl RetryPolicy {
    /// Calculate the delay for attempt `n` (0-indexed) **without** jitter.
    #[must_use]
    pub fn base_delay_for_attempt(&self, attempt: u32) -> f64 {
        let delay = self.base_delay
            * self
                .backoff_multiplier
                .powi(i32::try_from(attempt).unwrap_or(i32::MAX));
        delay.min(self.max_delay)
    }

    /// Calculate the delay for attempt `n` (0-indexed) **with** jitter if enabled.
    #[must_use]
    pub fn delay_for_attempt(&self, attempt: u32) -> f64 {
        let base = self.base_delay_for_attempt(attempt);
        if self.jitter {
            apply_jitter(base)
        } else {
            base
        }
    }

    /// Determine the actual delay to use for a retry, accounting for
    /// `Retry-After` overrides from the error.
    ///
    /// Returns `None` if the error should NOT be retried (either because
    /// it's non-retryable, we've exhausted attempts, or `Retry-After`
    /// exceeds `max_delay`).
    #[must_use]
    pub fn resolve_delay(&self, error: &SdkError, attempt: u32) -> Option<f64> {
        if attempt >= self.max_retries {
            return None;
        }

        if !error.is_retryable() {
            return None;
        }

        // Check Retry-After override
        if let Some(retry_after) = error.retry_after() {
            // Guard against non-positive or non-finite values that would
            // cause Duration::from_secs_f64 to panic.
            if !retry_after.is_finite() || retry_after <= 0.0 {
                return Some(self.delay_for_attempt(attempt));
            }
            if retry_after > self.max_delay {
                // Retry-After exceeds max_delay — don't retry
                return None;
            }
            // Use provider's suggested delay
            return Some(retry_after);
        }

        Some(self.delay_for_attempt(attempt))
    }
}

/// Apply ±50% jitter to a delay value.
fn apply_jitter(delay: f64) -> f64 {
    let mut rng = rand::rng();
    let factor = rng.random_range(0.5..=1.5);
    delay * factor
}

/// Callback invoked before each retry attempt.
///
/// Receives the error, the attempt number (0-indexed), and the delay in seconds.
pub type OnRetryFn = dyn Fn(&SdkError, u32, f64) + Send + Sync;

/// Predicate that decides whether a specific error should be retried.
///
/// This is checked **in addition to** `SdkError::is_retryable()`. Both must
/// agree for a retry to occur. This allows callers to impose additional
/// constraints such as:
/// - No retry after partial stream delivery
/// - No retry for schema validation failures
pub type ShouldRetryFn = dyn Fn(&SdkError) -> bool + Send + Sync;

/// Retry an async operation according to the given policy.
///
/// The `operation` closure is called repeatedly until it succeeds, returns
/// a non-retryable error, or the retry budget is exhausted. Between
/// attempts, the function sleeps for the calculated backoff delay.
///
/// # Retry constraints (spec §6.6)
///
/// - Only retryable errors trigger a retry.
/// - The optional `should_retry` predicate can impose additional constraints
///   (e.g., no retry after partial stream delivery, no retry for validation
///   failures). When provided, both `is_retryable()` and the predicate must
///   return `true` for a retry to occur.
/// - `Retry-After` values override the calculated delay.
/// - `Retry-After` exceeding `max_delay` causes an immediate error return.
/// - `max_retries = 0` disables retries entirely.
///
/// The optional `on_retry` callback is invoked before each retry with
/// the error, attempt number, and delay.
///
/// # Errors
///
/// Returns the last `SdkError` if all retry attempts are exhausted,
/// the error is non-retryable, or the predicate rejects the retry.
pub async fn retry<F, Fut, T>(
    policy: &RetryPolicy,
    mut operation: F,
    should_retry: Option<&ShouldRetryFn>,
    on_retry: Option<&OnRetryFn>,
) -> SdkResult<T>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = SdkResult<T>>,
{
    let mut attempt: u32 = 0;

    loop {
        match operation().await {
            Ok(value) => return Ok(value),
            Err(error) => {
                // Check caller-supplied predicate before policy
                let predicate_allows = should_retry.is_none_or(|p| p(&error));
                if predicate_allows && let Some(delay) = policy.resolve_delay(&error, attempt) {
                    if let Some(callback) = on_retry {
                        callback(&error, attempt, delay);
                    }
                    tokio::time::sleep(Duration::from_secs_f64(delay)).await;
                    attempt += 1;
                    continue;
                }
                return Err(error);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::ProviderDetails;

    use super::*;

    // ── RetryPolicy delay calculations ──────────────────────────────

    #[test]
    fn default_policy_values() {
        let policy = RetryPolicy::default();
        assert_eq!(policy.max_retries, 2);
        assert!((policy.base_delay - 1.0).abs() < f64::EPSILON);
        assert!((policy.max_delay - 60.0).abs() < f64::EPSILON);
        assert!((policy.backoff_multiplier - 2.0).abs() < f64::EPSILON);
        assert!(policy.jitter);
    }

    #[test]
    fn base_delay_exponential_growth() {
        let policy = RetryPolicy {
            jitter: false,
            ..RetryPolicy::default()
        };
        assert!((policy.base_delay_for_attempt(0) - 1.0).abs() < f64::EPSILON);
        assert!((policy.base_delay_for_attempt(1) - 2.0).abs() < f64::EPSILON);
        assert!((policy.base_delay_for_attempt(2) - 4.0).abs() < f64::EPSILON);
        assert!((policy.base_delay_for_attempt(3) - 8.0).abs() < f64::EPSILON);
        assert!((policy.base_delay_for_attempt(4) - 16.0).abs() < f64::EPSILON);
    }

    #[test]
    fn max_delay_caps_backoff() {
        let policy = RetryPolicy {
            max_delay: 10.0,
            jitter: false,
            ..RetryPolicy::default()
        };
        // Attempt 4 would be 16.0 but capped at 10.0
        assert!((policy.base_delay_for_attempt(4) - 10.0).abs() < f64::EPSILON);
        // Attempt 10 also capped
        assert!((policy.base_delay_for_attempt(10) - 10.0).abs() < f64::EPSILON);
    }

    #[test]
    fn jitter_within_range() {
        let policy = RetryPolicy::default();
        // Run many iterations to check jitter stays in [0.5, 1.5] * base
        for attempt in 0..5 {
            let base = policy.base_delay_for_attempt(attempt);
            for _ in 0..100 {
                let delay = policy.delay_for_attempt(attempt);
                assert!(
                    delay >= base * 0.5 && delay <= base * 1.5,
                    "jitter out of range: delay={delay}, base={base}"
                );
            }
        }
    }

    #[test]
    fn no_jitter_returns_exact() {
        let policy = RetryPolicy {
            jitter: false,
            ..RetryPolicy::default()
        };
        assert!((policy.delay_for_attempt(0) - 1.0).abs() < f64::EPSILON);
        assert!((policy.delay_for_attempt(1) - 2.0).abs() < f64::EPSILON);
    }

    // ── resolve_delay ───────────────────────────────────────────────

    #[test]
    fn resolve_delay_non_retryable_returns_none() {
        let policy = RetryPolicy::default();
        let err = SdkError::Authentication {
            message: "bad key".into(),
            details: ProviderDetails::default(),
        };
        assert!(policy.resolve_delay(&err, 0).is_none());
    }

    #[test]
    fn resolve_delay_exhausted_returns_none() {
        let policy = RetryPolicy {
            max_retries: 2,
            ..RetryPolicy::default()
        };
        let err = SdkError::Server {
            message: "error".into(),
            details: ProviderDetails::default(),
        };
        // Attempts 0 and 1 should work, attempt 2 should be exhausted
        assert!(policy.resolve_delay(&err, 0).is_some());
        assert!(policy.resolve_delay(&err, 1).is_some());
        assert!(policy.resolve_delay(&err, 2).is_none());
    }

    #[test]
    fn resolve_delay_retry_after_override() {
        let policy = RetryPolicy::default();
        let err = SdkError::RateLimit {
            message: "too fast".into(),
            details: crate::error::ProviderDetails {
                retry_after: Some(5.0),
                retryable: true,
                ..ProviderDetails::default()
            },
        };
        let delay = policy.resolve_delay(&err, 0);
        assert_eq!(delay, Some(5.0));
    }

    #[test]
    fn resolve_delay_retry_after_exceeds_max_returns_none() {
        let policy = RetryPolicy {
            max_delay: 30.0,
            ..RetryPolicy::default()
        };
        let err = SdkError::RateLimit {
            message: "too fast".into(),
            details: crate::error::ProviderDetails {
                retry_after: Some(120.0),
                retryable: true,
                ..ProviderDetails::default()
            },
        };
        assert!(policy.resolve_delay(&err, 0).is_none());
    }

    #[test]
    fn resolve_delay_negative_retry_after_falls_back_to_backoff() {
        let policy = RetryPolicy {
            jitter: false,
            ..RetryPolicy::default()
        };
        let err = SdkError::RateLimit {
            message: "too fast".into(),
            details: crate::error::ProviderDetails {
                retry_after: Some(-1.0),
                retryable: true,
                ..ProviderDetails::default()
            },
        };
        // Should fall back to normal backoff instead of using negative value
        let delay = policy.resolve_delay(&err, 0);
        assert_eq!(delay, Some(1.0));
    }

    #[test]
    fn resolve_delay_nan_retry_after_falls_back_to_backoff() {
        let policy = RetryPolicy {
            jitter: false,
            ..RetryPolicy::default()
        };
        let err = SdkError::RateLimit {
            message: "too fast".into(),
            details: crate::error::ProviderDetails {
                retry_after: Some(f64::NAN),
                retryable: true,
                ..ProviderDetails::default()
            },
        };
        let delay = policy.resolve_delay(&err, 0);
        assert_eq!(delay, Some(1.0));
    }

    #[test]
    fn zero_max_retries_disables_retry() {
        let policy = RetryPolicy {
            max_retries: 0,
            ..RetryPolicy::default()
        };
        let err = SdkError::Server {
            message: "error".into(),
            details: ProviderDetails::default(),
        };
        assert!(policy.resolve_delay(&err, 0).is_none());
    }

    // ── retry() function ────────────────────────────────────────────

    #[tokio::test]
    async fn retry_succeeds_on_first_attempt() -> SdkResult<()> {
        let policy = RetryPolicy::default();
        let result = retry(&policy, || async { Ok::<_, SdkError>(42) }, None, None).await?;
        assert_eq!(result, 42);
        Ok(())
    }

    #[tokio::test]
    async fn retry_succeeds_after_transient_failure() -> SdkResult<()> {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: 0.001, // Very short for tests
            jitter: false,
            ..RetryPolicy::default()
        };

        let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        let result = retry(
            &policy,
            move || {
                let attempt = attempt_clone.clone();
                async move {
                    let n = attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if n < 2 {
                        Err(SdkError::Server {
                            message: format!("attempt {n}"),
                            details: ProviderDetails::default(),
                        })
                    } else {
                        Ok(42)
                    }
                }
            },
            None,
            None,
        )
        .await?;

        assert_eq!(result, 42);
        assert_eq!(attempt.load(std::sync::atomic::Ordering::SeqCst), 3);
        Ok(())
    }

    #[tokio::test]
    async fn retry_returns_error_when_exhausted() {
        let policy = RetryPolicy {
            max_retries: 1,
            base_delay: 0.001,
            jitter: false,
            ..RetryPolicy::default()
        };

        let result: SdkResult<()> = retry(
            &policy,
            || async {
                Err(SdkError::Server {
                    message: "always fails".into(),
                    details: ProviderDetails::default(),
                })
            },
            None,
            None,
        )
        .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn retry_does_not_retry_non_retryable() {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: 0.001,
            ..RetryPolicy::default()
        };

        let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        let result: SdkResult<()> = retry(
            &policy,
            move || {
                let attempt = attempt_clone.clone();
                async move {
                    attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Err(SdkError::Authentication {
                        message: "bad key".into(),
                        details: ProviderDetails::default(),
                    })
                }
            },
            None,
            None,
        )
        .await;

        assert!(result.is_err());
        // Should have been called exactly once — no retries
        assert_eq!(attempt.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_calls_on_retry_callback() -> SdkResult<()> {
        let policy = RetryPolicy {
            max_retries: 2,
            base_delay: 0.001,
            jitter: false,
            ..RetryPolicy::default()
        };

        let callback_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let callback_count_clone = callback_count.clone();

        let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        let on_retry = move |_err: &SdkError, _attempt: u32, _delay: f64| {
            callback_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        };

        let result = retry(
            &policy,
            move || {
                let attempt = attempt_clone.clone();
                async move {
                    let n = attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if n < 1 {
                        Err(SdkError::Server {
                            message: "fail".into(),
                            details: ProviderDetails::default(),
                        })
                    } else {
                        Ok(42)
                    }
                }
            },
            None,
            Some(&on_retry),
        )
        .await?;

        assert_eq!(result, 42);
        assert_eq!(callback_count.load(std::sync::atomic::Ordering::SeqCst), 1);
        Ok(())
    }

    #[tokio::test]
    async fn retry_predicate_can_reject_retryable_error() {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: 0.001,
            ..RetryPolicy::default()
        };

        let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        // Predicate rejects Stream errors (simulates "no retry after partial stream")
        let should_retry = |err: &SdkError| !matches!(err, SdkError::Stream { .. });

        let result: SdkResult<()> = retry(
            &policy,
            move || {
                let attempt = attempt_clone.clone();
                async move {
                    attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Err(SdkError::Stream {
                        message: "partial delivery".into(),
                    })
                }
            },
            Some(&should_retry),
            None,
        )
        .await;

        assert!(result.is_err());
        // Stream errors are is_retryable() but predicate rejects — only 1 attempt
        assert_eq!(attempt.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn retry_predicate_allows_retryable_error() -> SdkResult<()> {
        let policy = RetryPolicy {
            max_retries: 3,
            base_delay: 0.001,
            jitter: false,
            ..RetryPolicy::default()
        };

        let attempt = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        // Predicate allows Server errors but rejects Stream errors
        let should_retry = |err: &SdkError| matches!(err, SdkError::Server { .. });

        let result = retry(
            &policy,
            move || {
                let attempt = attempt_clone.clone();
                async move {
                    let n = attempt.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    if n < 1 {
                        Err(SdkError::Server {
                            message: "transient".into(),
                            details: ProviderDetails::default(),
                        })
                    } else {
                        Ok(42)
                    }
                }
            },
            Some(&should_retry),
            None,
        )
        .await?;

        assert_eq!(result, 42);
        assert_eq!(attempt.load(std::sync::atomic::Ordering::SeqCst), 2);
        Ok(())
    }
}
