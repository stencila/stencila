use std::collections::HashMap;

use crate::error::{SdkError, SdkResult};
use crate::middleware::{Middleware, NextComplete, NextStream};
use crate::provider::{BoxStream, ProviderAdapter};
use crate::providers::{AnthropicAdapter, GeminiAdapter, OpenAIAdapter};
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::stream_event::StreamEvent;

/// The main orchestration layer.
///
/// Holds registered provider adapters, routes requests by provider
/// identifier, applies middleware, and manages configuration.
pub struct Client {
    providers: HashMap<String, Box<dyn ProviderAdapter>>,
    default_provider: Option<String>,
    middleware: Vec<Box<dyn Middleware>>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .field("default_provider", &self.default_provider)
            .field("middleware_count", &self.middleware.len())
            .finish()
    }
}

impl Client {
    /// Create a client from environment variables.
    ///
    /// Reads standard environment variables for each provider and registers
    /// adapters for those whose keys are present. The first registered
    /// provider becomes the default.
    ///
    /// | Provider  | Required               | Optional                                           |
    /// |-----------|------------------------|----------------------------------------------------|
    /// | `OpenAI`    | `OPENAI_API_KEY`       | `OPENAI_BASE_URL`, `OPENAI_ORG_ID`, `OPENAI_PROJECT_ID` |
    /// | Anthropic | `ANTHROPIC_API_KEY`    | `ANTHROPIC_BASE_URL`                               |
    /// | Gemini    | `GEMINI_API_KEY`       | `GEMINI_BASE_URL`                                  |
    ///
    /// `GOOGLE_API_KEY` is accepted as a fallback for `GEMINI_API_KEY`.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if a provider key is present but
    /// the adapter cannot be constructed (e.g. invalid header values).
    /// Note: base URL format is not validated at construction; an invalid
    /// URL will surface as a request-time error.
    pub fn from_env() -> SdkResult<Self> {
        let mut builder = ClientBuilder::new();

        // OpenAI (native Responses API)
        if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
            let base_url = std::env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
            let org = std::env::var("OPENAI_ORG_ID").ok();
            let project = std::env::var("OPENAI_PROJECT_ID").ok();
            builder =
                builder.add_provider(OpenAIAdapter::with_config(api_key, base_url, org, project)?);
        }

        // Anthropic
        if std::env::var("ANTHROPIC_API_KEY").is_ok() {
            builder = builder.add_provider(AnthropicAdapter::from_env()?);
        }

        // Gemini (with GOOGLE_API_KEY fallback)
        if std::env::var("GEMINI_API_KEY").is_ok() {
            builder = builder.add_provider(GeminiAdapter::from_env()?);
        } else if let Ok(api_key) = std::env::var("GOOGLE_API_KEY") {
            let base_url = std::env::var("GEMINI_BASE_URL").ok();
            builder = builder.add_provider(GeminiAdapter::new(api_key, base_url)?);
        }

        builder.build()
    }

    /// Start building a client with explicit configuration.
    #[must_use]
    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    /// Send a request and return a complete response.
    ///
    /// Routes to the adapter identified by `request.provider` or the
    /// default provider. Applies middleware in registration order.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if no provider can be resolved.
    /// Provider errors are propagated as-is.
    pub async fn complete(&self, request: Request) -> SdkResult<Response> {
        let provider = self.resolve_provider(&request)?;

        if self.middleware.is_empty() {
            return provider.complete(request).await;
        }

        let chain = self.build_complete_chain(provider);
        chain(request).await
    }

    /// Send a request and return a stream of events.
    ///
    /// Routes to the adapter identified by `request.provider` or the
    /// default provider. Applies middleware in registration order.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if no provider can be resolved.
    /// Connection errors are returned from the outer future; per-event
    /// errors appear as `Err` items in the stream.
    pub async fn stream(
        &self,
        request: Request,
    ) -> SdkResult<BoxStream<'_, SdkResult<StreamEvent>>> {
        let provider = self.resolve_provider(&request)?;

        if self.middleware.is_empty() {
            return provider.stream(request).await;
        }

        let chain = self.build_stream_chain(provider);
        chain(request).await
    }

    /// Close all registered providers, releasing resources.
    ///
    /// # Errors
    ///
    /// Returns the first error encountered; remaining providers are
    /// still closed on a best-effort basis.
    pub async fn close(&self) -> SdkResult<()> {
        let mut first_error: Option<SdkError> = None;
        for provider in self.providers.values() {
            if let Err(e) = provider.close().await
                && first_error.is_none()
            {
                first_error = Some(e);
            }
        }
        match first_error {
            Some(e) => Err(e),
            None => Ok(()),
        }
    }

    /// The names of all registered providers.
    #[must_use]
    pub fn provider_names(&self) -> Vec<&str> {
        self.providers.keys().map(String::as_str).collect()
    }

    /// The default provider name, if set.
    #[must_use]
    pub fn default_provider(&self) -> Option<&str> {
        self.default_provider.as_deref()
    }

    /// The number of registered middleware.
    #[must_use]
    pub fn middleware_count(&self) -> usize {
        self.middleware.len()
    }

    /// Iterate over all registered provider adapters.
    ///
    /// Used by [`crate::catalog::refresh`] to query each provider for its
    /// available models.
    pub fn providers(&self) -> impl Iterator<Item = &dyn ProviderAdapter> {
        self.providers.values().map(AsRef::as_ref)
    }

    /// Resolve the provider adapter for a request.
    fn resolve_provider(&self, request: &Request) -> SdkResult<&dyn ProviderAdapter> {
        let name = request
            .provider
            .as_deref()
            .or(self.default_provider.as_deref())
            .ok_or_else(|| SdkError::Configuration {
                message: "no provider specified and no default provider set".into(),
            })?;

        self.providers
            .get(name)
            .map(AsRef::as_ref)
            .ok_or_else(|| SdkError::Configuration {
                message: format!("unknown provider: {name}"),
            })
    }

    /// Build the middleware chain for `complete()`.
    ///
    /// Folds right-to-left so the first-registered middleware executes
    /// first on the request path (onion model).
    fn build_complete_chain<'a>(&'a self, provider: &'a dyn ProviderAdapter) -> NextComplete<'a> {
        let mut next: NextComplete<'a> = Box::new(move |req| provider.complete(req));

        for mw in self.middleware.iter().rev() {
            let inner = next;
            next = Box::new(move |req| mw.handle_complete(req, inner));
        }

        next
    }

    /// Build the middleware chain for `stream()`.
    fn build_stream_chain<'a>(&'a self, provider: &'a dyn ProviderAdapter) -> NextStream<'a> {
        let mut next: NextStream<'a> = Box::new(move |req| provider.stream(req));

        for mw in self.middleware.iter().rev() {
            let inner = next;
            next = Box::new(move |req| mw.handle_stream(req, inner));
        }

        next
    }
}

/// Builder for constructing a [`Client`] with explicit configuration.
pub struct ClientBuilder {
    providers: HashMap<String, Box<dyn ProviderAdapter>>,
    default_provider: Option<String>,
    middleware: Vec<Box<dyn Middleware>>,
}

impl std::fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .field("default_provider", &self.default_provider)
            .field("middleware_count", &self.middleware.len())
            .finish()
    }
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ClientBuilder {
    /// Create a new empty builder.
    #[must_use]
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
            middleware: Vec::new(),
        }
    }

    /// Register a provider adapter.
    ///
    /// The adapter's `name()` is used as the registration key.
    /// The first added provider becomes the default unless overridden
    /// with [`default_provider`](Self::default_provider).
    #[must_use]
    pub fn add_provider(mut self, adapter: impl ProviderAdapter + 'static) -> Self {
        let name = adapter.name().to_string();
        if self.default_provider.is_none() {
            self.default_provider = Some(name.clone());
        }
        self.providers.insert(name, Box::new(adapter));
        self
    }

    /// Set the default provider explicitly.
    ///
    /// Overrides the automatic first-registered default.
    #[must_use]
    pub fn default_provider(mut self, name: impl Into<String>) -> Self {
        self.default_provider = Some(name.into());
        self
    }

    /// Append a middleware to the chain.
    ///
    /// Middleware executes in registration order for the request phase
    /// and in reverse order for the response phase.
    #[must_use]
    pub fn middleware(mut self, mw: impl Middleware + 'static) -> Self {
        self.middleware.push(Box::new(mw));
        self
    }

    /// Build the [`Client`].
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if an explicit `default_provider`
    /// was set but no adapter is registered under that name.
    pub fn build(self) -> SdkResult<Client> {
        // Validate: if default_provider is set, it must exist in providers.
        // When default_provider was set by add_provider (automatic), this is
        // always true. When set explicitly, it could be wrong.
        if let Some(ref default) = self.default_provider
            && !self.providers.contains_key(default)
        {
            return Err(SdkError::Configuration {
                message: format!("default provider '{default}' is not registered"),
            });
        }

        Ok(Client {
            providers: self.providers,
            default_provider: self.default_provider,
            middleware: self.middleware,
        })
    }
}
