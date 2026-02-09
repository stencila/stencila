//! HTTP response header parsing utilities.
//!
//! Extracts rate-limit information and `Retry-After` values from
//! provider response headers. Each provider uses slightly different
//! header names, so we check all common variants.

use reqwest::header::HeaderMap;

use crate::types::rate_limit::RateLimitInfo;

/// Parse rate-limit headers from a response into [`RateLimitInfo`].
///
/// Checks the following header families (used by OpenAI, Anthropic,
/// and other providers):
///
/// - `x-ratelimit-remaining-requests` / `x-ratelimit-limit-requests`
/// - `x-ratelimit-remaining-tokens` / `x-ratelimit-limit-tokens`
/// - `x-ratelimit-reset` (seconds since epoch or ISO 8601)
/// - Anthropic variants: `anthropic-ratelimit-*`
///
/// Returns `None` if no rate-limit headers are found.
#[must_use]
pub fn parse_rate_limit_headers(headers: &HeaderMap) -> Option<RateLimitInfo> {
    let requests_remaining = header_u64(headers, "x-ratelimit-remaining-requests")
        .or_else(|| header_u64(headers, "anthropic-ratelimit-requests-remaining"));

    let requests_limit = header_u64(headers, "x-ratelimit-limit-requests")
        .or_else(|| header_u64(headers, "anthropic-ratelimit-requests-limit"));

    let tokens_remaining = header_u64(headers, "x-ratelimit-remaining-tokens")
        .or_else(|| header_u64(headers, "anthropic-ratelimit-tokens-remaining"));

    let tokens_limit = header_u64(headers, "x-ratelimit-limit-tokens")
        .or_else(|| header_u64(headers, "anthropic-ratelimit-tokens-limit"));

    let reset_at = header_f64(headers, "x-ratelimit-reset")
        .or_else(|| header_f64(headers, "anthropic-ratelimit-requests-reset"));

    if requests_remaining.is_none()
        && requests_limit.is_none()
        && tokens_remaining.is_none()
        && tokens_limit.is_none()
        && reset_at.is_none()
    {
        return None;
    }

    Some(RateLimitInfo {
        requests_remaining,
        requests_limit,
        tokens_remaining,
        tokens_limit,
        reset_at,
    })
}

/// Parse the `Retry-After` header value as seconds.
///
/// Handles two formats per HTTP spec:
/// - Integer seconds (e.g. `30`)
/// - Float seconds (e.g. `1.5`)
///
/// Returns `None` for non-positive, NaN, infinite, or HTTP-date values.
/// HTTP-date format is not currently supported, which is acceptable
/// because LLM providers consistently use numeric seconds.
#[must_use]
pub fn parse_retry_after(headers: &HeaderMap) -> Option<f64> {
    let value = headers.get("retry-after")?.to_str().ok()?;
    let secs = value.trim().parse::<f64>().ok()?;
    if secs.is_finite() && secs > 0.0 {
        Some(secs)
    } else {
        None
    }
}

/// Extract a `u64` from a header value.
fn header_u64(headers: &HeaderMap, name: &str) -> Option<u64> {
    headers.get(name)?.to_str().ok()?.trim().parse().ok()
}

/// Extract an `f64` from a header value.
fn header_f64(headers: &HeaderMap, name: &str) -> Option<f64> {
    headers.get(name)?.to_str().ok()?.trim().parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_headers(pairs: &[(&str, &str)]) -> HeaderMap {
        let mut map = HeaderMap::new();
        for &(name, value) in pairs {
            map.insert(
                reqwest::header::HeaderName::from_bytes(name.as_bytes())
                    .expect("valid header name"),
                reqwest::header::HeaderValue::from_str(value).expect("valid header value"),
            );
        }
        map
    }

    #[test]
    fn parse_openai_rate_limit_headers() {
        let headers = make_headers(&[
            ("x-ratelimit-remaining-requests", "99"),
            ("x-ratelimit-limit-requests", "100"),
            ("x-ratelimit-remaining-tokens", "9500"),
            ("x-ratelimit-limit-tokens", "10000"),
        ]);
        let info = parse_rate_limit_headers(&headers);
        assert!(info.is_some());
        let info = info.expect("should have rate limit info");
        assert_eq!(info.requests_remaining, Some(99));
        assert_eq!(info.requests_limit, Some(100));
        assert_eq!(info.tokens_remaining, Some(9500));
        assert_eq!(info.tokens_limit, Some(10_000));
    }

    #[test]
    fn parse_anthropic_rate_limit_headers() {
        let headers = make_headers(&[
            ("anthropic-ratelimit-requests-remaining", "45"),
            ("anthropic-ratelimit-requests-limit", "50"),
            ("anthropic-ratelimit-tokens-remaining", "90000"),
            ("anthropic-ratelimit-tokens-limit", "100000"),
        ]);
        let info = parse_rate_limit_headers(&headers);
        assert!(info.is_some());
        let info = info.expect("should have rate limit info");
        assert_eq!(info.requests_remaining, Some(45));
        assert_eq!(info.requests_limit, Some(50));
        assert_eq!(info.tokens_remaining, Some(90_000));
        assert_eq!(info.tokens_limit, Some(100_000));
    }

    #[test]
    fn no_rate_limit_headers_returns_none() {
        let headers = make_headers(&[("content-type", "application/json")]);
        assert!(parse_rate_limit_headers(&headers).is_none());
    }

    #[test]
    fn partial_headers_still_returned() {
        let headers = make_headers(&[("x-ratelimit-remaining-requests", "5")]);
        let info = parse_rate_limit_headers(&headers);
        assert!(info.is_some());
        let info = info.expect("should have rate limit info");
        assert_eq!(info.requests_remaining, Some(5));
        assert_eq!(info.requests_limit, None);
        assert_eq!(info.tokens_remaining, None);
    }

    #[test]
    fn parse_retry_after_integer() {
        let headers = make_headers(&[("retry-after", "30")]);
        assert_eq!(parse_retry_after(&headers), Some(30.0));
    }

    #[test]
    fn parse_retry_after_float() {
        let headers = make_headers(&[("retry-after", "1.5")]);
        assert_eq!(parse_retry_after(&headers), Some(1.5));
    }

    #[test]
    fn parse_retry_after_missing() {
        let headers = make_headers(&[]);
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn parse_retry_after_invalid() {
        let headers = make_headers(&[("retry-after", "Wed, 21 Oct 2026 07:28:00 GMT")]);
        // HTTP-date not supported — returns None
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn parse_retry_after_negative() {
        let headers = make_headers(&[("retry-after", "-5")]);
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn parse_retry_after_zero() {
        let headers = make_headers(&[("retry-after", "0")]);
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn parse_retry_after_nan() {
        let headers = make_headers(&[("retry-after", "NaN")]);
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn parse_retry_after_inf() {
        let headers = make_headers(&[("retry-after", "inf")]);
        assert_eq!(parse_retry_after(&headers), None);
    }

    #[test]
    fn reset_at_parsed() {
        let headers = make_headers(&[("x-ratelimit-reset", "1700000000.5")]);
        let info = parse_rate_limit_headers(&headers).expect("should parse");
        assert_eq!(info.reset_at, Some(1_700_000_000.5));
    }
}
