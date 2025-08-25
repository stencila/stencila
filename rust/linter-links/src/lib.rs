use std::{collections::HashMap, path::Path, time::Duration};

use stencila_linter::{
    Format, Linter, LinterAvailability, LintingOptions, LintingOutput, NodeType,
    common::{
        async_trait::async_trait, eyre::Result, once_cell::sync::Lazy, reqwest::Client,
        tokio::sync::Mutex, tracing,
    },
    schema::{AuthorRoleName, CompilationMessage, MessageLevel, SoftwareApplication, Timestamp},
};
use version::STENCILA_USER_AGENT;

/// HTTP client for making link check requests
static HTTP_CLIENT: Lazy<Client> = Lazy::new(|| {
    Client::builder()
        .timeout(Duration::from_secs(15))
        .user_agent(STENCILA_USER_AGENT)
        .build()
        .expect("Failed to create HTTP client")
});

#[derive(Default)]
pub struct LinksLinter;

#[async_trait]
impl Linter for LinksLinter {
    fn name(&self) -> &str {
        "links"
    }

    fn node_types(&self) -> Vec<NodeType> {
        vec![NodeType::Link]
    }

    fn formats(&self) -> Vec<Format> {
        vec![Format::Text]
    }

    fn supports_formatting(&self) -> bool {
        false
    }

    fn supports_fixing(&self) -> bool {
        false
    }

    fn availability(&self) -> LinterAvailability {
        LinterAvailability::Available
    }

    #[tracing::instrument(skip(self))]
    async fn lint(
        &self,
        target: &str,
        _path: &Path,
        _options: &LintingOptions,
    ) -> Result<LintingOutput> {
        tracing::trace!("Linting with Links checker");

        let mut messages = None;
        if target.starts_with("http://") || target.starts_with("https://") {
            match is_url_accessible(target).await {
                Some(false) => {
                    // URL is inaccessible but network is available
                    messages = Some(vec![CompilationMessage {
                        level: MessageLevel::Warning,
                        error_type: Some("Inaccessible".to_string()),
                        message: format!("Link `{target}` is not accessible"),
                        ..Default::default()
                    }]);
                }
                Some(true) => {
                    // URL is accessible, check if we should warn about HTTP
                    if target.starts_with("http://") {
                        messages = Some(vec![CompilationMessage {
                            level: MessageLevel::Info,
                            error_type: Some("Insecure".to_string()),
                            message: format!("Link uses HTTP; it is probably better to use HTTPS"),
                            ..Default::default()
                        }]);
                    }
                }
                None => {
                    // Network is unavailable, skip checking
                    tracing::debug!("Network unavailable, skipping link check for: {target}");
                }
            }
        }

        let authors = Some(vec![
            SoftwareApplication::new("Stencila Links Linter".to_string()).into_author_role(
                AuthorRoleName::Linter,
                None,
                Some(Timestamp::now()),
            ),
        ]);

        Ok(LintingOutput {
            messages,
            authors,
            ..Default::default()
        })
    }
}

/// Check if we have internet connectivity by testing a reliable endpoint
/// Uses caching to avoid repeated connectivity checks
async fn has_internet_connectivity() -> bool {
    // Cache for storing connectivity status
    static CONNECTIVITY_CACHE: Lazy<Mutex<Option<(bool, std::time::Instant)>>> =
        Lazy::new(|| Mutex::new(None));

    // Cache connectivity result for 30 seconds
    const CACHE_DURATION_SECS: u64 = 30;

    {
        let cache = CONNECTIVITY_CACHE.lock().await;
        if let Some((result, timestamp)) = *cache {
            if timestamp.elapsed().as_secs() < CACHE_DURATION_SECS {
                tracing::debug!("Connectivity cache hit: {}", result);
                return result;
            }
        }
    }

    // Test connectivity with a reliable endpoint (DNS over HTTPS)
    let connectivity = match HTTP_CLIENT.head("https://1.1.1.1").send().await {
        Ok(_) => true,
        Err(error) => {
            tracing::debug!("Connectivity check failed: {}", error);
            false
        }
    };

    // Update cache
    {
        let mut cache = CONNECTIVITY_CACHE.lock().await;
        *cache = Some((connectivity, std::time::Instant::now()));
    }

    tracing::debug!("Internet connectivity: {}", connectivity);
    connectivity
}

/// Check if a URL is accessible by making a HEAD request
/// Returns None if no internet connectivity, Some(bool) otherwise  
/// Uses caching to avoid redundant requests
async fn is_url_accessible(url: &str) -> Option<bool> {
    // Cache for storing HTTP link check results to avoid redundant requests
    static LINK_CHECK_CACHE: Lazy<Mutex<HashMap<String, bool>>> =
        Lazy::new(|| Mutex::new(HashMap::new()));

    // Check internet connectivity first
    if !has_internet_connectivity().await {
        tracing::debug!("No internet connectivity, skipping URL check for: {}", url);
        return None;
    }

    // Check cache first
    {
        let cache = LINK_CHECK_CACHE.lock().await;
        if let Some(&result) = cache.get(url) {
            tracing::debug!("Cache hit for URL: {} -> {}", url, result);
            return Some(result);
        }
    }

    // Make HEAD request to check accessibility
    let result = match HTTP_CLIENT.head(url).send().await {
        Ok(response) => {
            let accessible = response.status().is_success();
            tracing::debug!(
                "HEAD request to {} returned status: {} (accessible: {})",
                url,
                response.status(),
                accessible
            );
            accessible
        }
        Err(error) => {
            tracing::debug!("HEAD request to {} failed: {}", url, error);
            false // URL inaccessible (DNS failure, 404, etc.)
        }
    };

    // Cache the result
    {
        let mut cache = LINK_CHECK_CACHE.lock().await;
        cache.insert(url.to_string(), result);
    }

    Some(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_url_accessibility_return_types() {
        // Test that the function returns the correct types
        // We can't test actual network calls reliably in CI, but we can test the interface
        let linter = LinksLinter::default();

        // Test with HTTP URL (should generate insecure warning if network is available)
        let result = linter
            .lint(
                "http://example.com",
                Path::new("test.md"),
                &LintingOptions::default(),
            )
            .await;
        assert!(result.is_ok());

        // Test with HTTPS URL
        let result = linter
            .lint(
                "https://example.com",
                Path::new("test.md"),
                &LintingOptions::default(),
            )
            .await;
        assert!(result.is_ok());

        // Test with non-HTTP URL (should not check)
        let result = linter
            .lint(
                "mailto:test@example.com",
                Path::new("test.md"),
                &LintingOptions::default(),
            )
            .await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.messages.is_none()); // No messages for non-HTTP links
    }

    #[tokio::test]
    async fn test_connectivity_check() {
        // Test the connectivity check function
        // This will either return true or false depending on network availability
        let has_connectivity = has_internet_connectivity().await;
        println!("Internet connectivity: {}", has_connectivity);

        // Test caching by calling again immediately
        let has_connectivity_cached = has_internet_connectivity().await;
        assert_eq!(has_connectivity, has_connectivity_cached);
    }
}
