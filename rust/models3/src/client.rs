use std::collections::HashMap;

use stencila_auth::AuthOptions;

use crate::error::{SdkError, SdkResult};
use crate::middleware::{Middleware, NextComplete, NextStream};
use crate::provider::{BoxStream, ProviderAdapter};
use stencila_auth::{claude_code, codex_cli};

use crate::providers::{
    AnthropicAdapter, DeepSeekAdapter, GeminiAdapter, MistralAdapter, OllamaAdapter, OpenAIAdapter,
};
use crate::secret::get_secret;
use crate::types::request::Request;
use crate::types::response::Response;
use crate::types::stream_event::StreamEvent;

/// Default OpenAI base URL.
///
/// There are two endpoints for the OpenAI Responses API:
///
/// - **Standard API** (`api.openai.com/v1/responses`) — requires an
///   `OPENAI_API_KEY` or an OAuth token with the `api.responses.write` scope.
///
/// - **`ChatGPT` backend** (`chatgpt.com/backend-api/codex/responses`) — accepts
///   Codex CLI OAuth tokens that only carry basic scopes (`openid`, `profile`,
///   `email`, `offline_access`). This is the same endpoint used by the Codex
///   CLI itself and the pi-mono reference implementation:
///   <https://github.com/badlogic/pi-mono/blob/main/packages/ai/src/providers/openai-codex-responses.ts>
///
/// When `is_oauth` is true (Codex CLI credentials without a static API key),
/// we route to the `ChatGPT` backend. `OPENAI_BASE_URL` overrides either default.
fn openai_base_url(is_oauth: bool) -> String {
    std::env::var("OPENAI_BASE_URL").unwrap_or_else(|_| {
        if is_oauth {
            "https://chatgpt.com/backend-api/codex".to_string()
        } else {
            "https://api.openai.com/v1".to_string()
        }
    })
}

fn configured_providers() -> Option<Vec<String>> {
    let cwd = std::env::current_dir().ok()?;
    let config = stencila_config::load_and_validate(&cwd).ok()?;
    config.models.and_then(|models| models.providers)
}

fn provider_enabled(configured: Option<&Vec<String>>, provider: &str) -> bool {
    match configured {
        Some(providers) => providers.iter().any(|name| name == provider),
        None => true,
    }
}

/// How a provider's credentials were obtained.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CredentialSource {
    /// A standard API key environment variable (e.g. `ANTHROPIC_API_KEY`).
    /// The `&'static str` is always a string literal naming the env var.
    EnvApiKey(&'static str),
    /// Codex CLI OAuth credentials from `~/.codex/auth.json`.
    CodexCliOAuth,
    /// Claude Code OAuth credentials.
    ClaudeCodeOAuth,
    /// A `GOOGLE_API_KEY` fallback for Gemini.
    GoogleApiKeyFallback,
    /// OAuth override supplied via `AuthOptions`.
    AuthOverride,
    /// Auto-detected (e.g. Ollama via host env var).
    AutoDetected,
}

impl std::fmt::Display for CredentialSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EnvApiKey(var) => write!(f, "{var}"),
            Self::CodexCliOAuth => write!(f, "codex-cli-oauth"),
            Self::ClaudeCodeOAuth => write!(f, "claude-code-oauth"),
            Self::GoogleApiKeyFallback => write!(f, "GOOGLE_API_KEY"),
            Self::AuthOverride => write!(f, "auth-override"),
            Self::AutoDetected => write!(f, "auto-detected"),
        }
    }
}

/// The main orchestration layer.
///
/// Holds registered provider adapters, routes requests by provider
/// identifier, applies middleware, and manages configuration.
pub struct Client {
    providers: HashMap<String, Box<dyn ProviderAdapter>>,
    default_provider: Option<String>,
    middleware: Vec<Box<dyn Middleware>>,
    credential_sources: HashMap<String, CredentialSource>,
}

impl std::fmt::Debug for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Client")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .field("default_provider", &self.default_provider)
            .field("middleware_count", &self.middleware.len())
            .field("credential_sources", &self.credential_sources)
            .finish()
    }
}

impl Client {
    /// Create a client from environment variables.
    ///
    /// Reads standard environment variables for each provider and registers
    /// adapters for those whose keys are present.
    ///
    /// If `models.providers` is configured in `stencila.toml`, only those
    /// providers are considered here.
    ///
    /// | Provider  | Required               | Optional                                           |
    /// |-----------|------------------------|----------------------------------------------------|
    /// | OpenAI    | `OPENAI_API_KEY`       | `OPENAI_BASE_URL`, `OPENAI_ORG_ID`, `OPENAI_PROJECT_ID` |
    /// | Anthropic | `ANTHROPIC_API_KEY`    | `ANTHROPIC_BASE_URL`                               |
    /// | Gemini    | `GEMINI_API_KEY`       | `GEMINI_BASE_URL`                                  |
    /// | Mistral   | `MISTRAL_API_KEY`      | `MISTRAL_BASE_URL`                                 |
    /// | DeepSeek  | `DEEPSEEK_API_KEY`     | `DEEPSEEK_BASE_URL`                                |
    /// | Ollama    | *(auto-detected)*      | `OLLAMA_BASE_URL`, `OLLAMA_HOST`, `OLLAMA_API_KEY` |
    ///
    /// `GOOGLE_API_KEY` is accepted as a fallback for `GEMINI_API_KEY`.
    /// When `OPENAI_API_KEY` is absent, Codex CLI OAuth credentials from
    /// `~/.codex/auth.json` are used when available.
    ///
    /// Ollama is registered when `OLLAMA_BASE_URL` or `OLLAMA_HOST` is set.
    /// Use [`OllamaAdapter::is_available`] to probe for a running instance
    /// before registering manually.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if a provider key is present but
    /// the adapter cannot be constructed (e.g. invalid header values).
    /// Note: base URL format is not validated at construction; an invalid
    /// URL will surface as a request-time error.
    pub fn from_env() -> SdkResult<Self> {
        let configured = configured_providers();
        let mut builder = ClientBuilder::new();

        // OpenAI (native Responses API)
        if provider_enabled(configured.as_ref(), "openai")
            && let Some(api_key) = get_secret("OPENAI_API_KEY")
        {
            if codex_cli::load_credentials().is_some() {
                tracing::info!("OPENAI_API_KEY is set; ignoring Codex CLI OAuth credentials");
            }
            let base_url = openai_base_url(false);
            let org = std::env::var("OPENAI_ORG_ID").ok();
            let project = std::env::var("OPENAI_PROJECT_ID").ok();
            builder = builder
                .add_provider(OpenAIAdapter::with_config(api_key, base_url, org, project)?)
                .credential_source("openai", CredentialSource::EnvApiKey("OPENAI_API_KEY"));
        } else if provider_enabled(configured.as_ref(), "openai")
            && let Some(creds) = codex_cli::load_credentials()
        {
            tracing::debug!("Using Codex CLI OAuth credentials for OpenAI");
            let is_oauth = !creds.has_api_key();
            let (auth, account_id) = codex_cli::build_auth_credential(creds);
            let base_url = openai_base_url(is_oauth);
            let org = std::env::var("OPENAI_ORG_ID").ok();
            let project = std::env::var("OPENAI_PROJECT_ID").ok();
            builder = builder
                .add_provider(OpenAIAdapter::with_auth_and_account(
                    auth, base_url, org, project, account_id,
                )?)
                .credential_source("openai", CredentialSource::CodexCliOAuth);
        }

        // Anthropic
        if provider_enabled(configured.as_ref(), "anthropic")
            && get_secret("ANTHROPIC_API_KEY").is_some()
        {
            if claude_code::load_credentials().is_some() {
                tracing::info!("ANTHROPIC_API_KEY is set; ignoring Claude Code OAuth credentials");
            }
            builder = builder
                .add_provider(AnthropicAdapter::from_env()?)
                .credential_source(
                    "anthropic",
                    CredentialSource::EnvApiKey("ANTHROPIC_API_KEY"),
                );
        } else if provider_enabled(configured.as_ref(), "anthropic")
            && let Some(creds) = claude_code::load_credentials()
        {
            tracing::debug!("Using Claude Code OAuth credentials for Anthropic");
            let auth = claude_code::build_auth_credential(creds);
            let base_url = std::env::var("ANTHROPIC_BASE_URL").ok();
            builder = builder
                .add_provider(AnthropicAdapter::with_auth(auth, base_url)?)
                .credential_source("anthropic", CredentialSource::ClaudeCodeOAuth);
        }

        // Gemini (with GOOGLE_API_KEY fallback)
        if provider_enabled(configured.as_ref(), "gemini") && get_secret("GEMINI_API_KEY").is_some()
        {
            builder = builder
                .add_provider(GeminiAdapter::from_env()?)
                .credential_source("gemini", CredentialSource::EnvApiKey("GEMINI_API_KEY"));
        } else if provider_enabled(configured.as_ref(), "gemini")
            && let Some(api_key) = get_secret("GOOGLE_API_KEY")
        {
            let base_url = std::env::var("GEMINI_BASE_URL").ok();
            builder = builder
                .add_provider(GeminiAdapter::new(api_key, base_url)?)
                .credential_source("gemini", CredentialSource::GoogleApiKeyFallback);
        }

        // Mistral
        if provider_enabled(configured.as_ref(), "mistral")
            && get_secret("MISTRAL_API_KEY").is_some()
        {
            builder = builder
                .add_provider(MistralAdapter::from_env()?)
                .credential_source("mistral", CredentialSource::EnvApiKey("MISTRAL_API_KEY"));
        }

        // DeepSeek
        if provider_enabled(configured.as_ref(), "deepseek")
            && get_secret("DEEPSEEK_API_KEY").is_some()
        {
            builder = builder
                .add_provider(DeepSeekAdapter::from_env()?)
                .credential_source("deepseek", CredentialSource::EnvApiKey("DEEPSEEK_API_KEY"));
        }

        // Ollama (no API key required — register when explicitly configured)
        if provider_enabled(configured.as_ref(), "ollama")
            && (std::env::var("OLLAMA_BASE_URL").is_ok() || std::env::var("OLLAMA_HOST").is_ok())
        {
            builder = builder
                .add_provider(OllamaAdapter::from_env()?)
                .credential_source("ollama", CredentialSource::AutoDetected);
        }

        builder.build()
    }

    /// Create a client from environment variables with authentication overrides.
    ///
    /// Like [`from_env`](Self::from_env), but providers whose names appear in
    /// `options.overrides` use the supplied [`AuthCredential`] instead of reading
    /// keys from the environment. This is the primary entry point for OAuth-based
    /// authentication.
    ///
    /// A provider with an override is registered even if its corresponding
    /// environment variable is absent — the override *is* the credential.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if an override key does not match
    /// any known provider name (to prevent silent typos like `"opanai"`).
    #[allow(clippy::too_many_lines)]
    pub fn from_env_with_auth(options: &AuthOptions) -> SdkResult<Self> {
        // Validate override keys against known provider names.
        let overrides = &options.overrides;
        for key in overrides.keys() {
            if !stencila_config::KNOWN_MODEL_PROVIDERS.contains(&key.as_str()) {
                return Err(SdkError::Configuration {
                    message: format!(
                        "unknown provider in auth overrides: '{key}'. \
                         Known providers: {}",
                        stencila_config::KNOWN_MODEL_PROVIDERS.join(", ")
                    ),
                });
            }
        }

        let configured = configured_providers();
        let mut builder = ClientBuilder::new();

        // OpenAI (native Responses API)
        if provider_enabled(configured.as_ref(), "openai")
            && let Some(auth) = overrides.get("openai")
        {
            let base_url = openai_base_url(options.openai_account_id.is_some());
            let org = std::env::var("OPENAI_ORG_ID").ok();
            let project = std::env::var("OPENAI_PROJECT_ID").ok();
            builder = builder
                .add_provider(OpenAIAdapter::with_auth_and_account(
                    auth.clone(),
                    base_url,
                    org,
                    project,
                    options.openai_account_id.clone(),
                )?)
                .credential_source("openai", CredentialSource::AuthOverride);
        } else if provider_enabled(configured.as_ref(), "openai")
            && let Some(api_key) = get_secret("OPENAI_API_KEY")
        {
            let base_url = openai_base_url(false);
            let org = std::env::var("OPENAI_ORG_ID").ok();
            let project = std::env::var("OPENAI_PROJECT_ID").ok();
            builder = builder
                .add_provider(OpenAIAdapter::with_config(api_key, base_url, org, project)?)
                .credential_source("openai", CredentialSource::EnvApiKey("OPENAI_API_KEY"));
        } else if provider_enabled(configured.as_ref(), "openai")
            && let Some(creds) = codex_cli::load_credentials()
        {
            tracing::debug!("Using Codex CLI OAuth credentials for OpenAI");
            let is_oauth = !creds.has_api_key();
            let (auth, detected_account_id) = codex_cli::build_auth_credential(creds);
            let account_id = detected_account_id.or_else(|| options.openai_account_id.clone());
            let base_url = openai_base_url(is_oauth);
            let org = std::env::var("OPENAI_ORG_ID").ok();
            let project = std::env::var("OPENAI_PROJECT_ID").ok();
            builder = builder
                .add_provider(OpenAIAdapter::with_auth_and_account(
                    auth, base_url, org, project, account_id,
                )?)
                .credential_source("openai", CredentialSource::CodexCliOAuth);
        }

        // Anthropic
        if provider_enabled(configured.as_ref(), "anthropic")
            && let Some(auth) = overrides.get("anthropic")
        {
            let base_url = std::env::var("ANTHROPIC_BASE_URL").ok();
            builder = builder.add_provider(AnthropicAdapter::with_auth(auth.clone(), base_url)?);
            builder = builder.credential_source("anthropic", CredentialSource::AuthOverride);
        } else if provider_enabled(configured.as_ref(), "anthropic")
            && get_secret("ANTHROPIC_API_KEY").is_some()
        {
            builder = builder
                .add_provider(AnthropicAdapter::from_env()?)
                .credential_source(
                    "anthropic",
                    CredentialSource::EnvApiKey("ANTHROPIC_API_KEY"),
                );
        } else if provider_enabled(configured.as_ref(), "anthropic")
            && let Some(creds) = claude_code::load_credentials()
        {
            tracing::debug!("Using Claude Code OAuth credentials for Anthropic");
            let auth = claude_code::build_auth_credential(creds);
            let base_url = std::env::var("ANTHROPIC_BASE_URL").ok();
            builder = builder
                .add_provider(AnthropicAdapter::with_auth(auth, base_url)?)
                .credential_source("anthropic", CredentialSource::ClaudeCodeOAuth);
        }

        // Gemini (with GOOGLE_API_KEY fallback)
        if provider_enabled(configured.as_ref(), "gemini")
            && let Some(auth) = overrides.get("gemini")
        {
            let base_url = std::env::var("GEMINI_BASE_URL").ok();
            builder = builder
                .add_provider(GeminiAdapter::with_auth(auth.clone(), base_url)?)
                .credential_source("gemini", CredentialSource::AuthOverride);
        } else if provider_enabled(configured.as_ref(), "gemini")
            && get_secret("GEMINI_API_KEY").is_some()
        {
            builder = builder.add_provider(GeminiAdapter::from_env()?);
            builder =
                builder.credential_source("gemini", CredentialSource::EnvApiKey("GEMINI_API_KEY"));
        } else if provider_enabled(configured.as_ref(), "gemini")
            && let Some(api_key) = get_secret("GOOGLE_API_KEY")
        {
            let base_url = std::env::var("GEMINI_BASE_URL").ok();
            builder = builder
                .add_provider(GeminiAdapter::new(api_key, base_url)?)
                .credential_source("gemini", CredentialSource::GoogleApiKeyFallback);
        }

        // Mistral
        if provider_enabled(configured.as_ref(), "mistral")
            && let Some(auth) = overrides.get("mistral")
        {
            let base_url = std::env::var("MISTRAL_BASE_URL").ok();
            builder = builder
                .add_provider(MistralAdapter::with_auth(auth.clone(), base_url)?)
                .credential_source("mistral", CredentialSource::AuthOverride);
        } else if provider_enabled(configured.as_ref(), "mistral")
            && get_secret("MISTRAL_API_KEY").is_some()
        {
            builder = builder
                .add_provider(MistralAdapter::from_env()?)
                .credential_source("mistral", CredentialSource::EnvApiKey("MISTRAL_API_KEY"));
        }

        // DeepSeek
        if provider_enabled(configured.as_ref(), "deepseek")
            && let Some(auth) = overrides.get("deepseek")
        {
            let base_url = std::env::var("DEEPSEEK_BASE_URL").ok();
            builder = builder
                .add_provider(DeepSeekAdapter::with_auth(auth.clone(), base_url)?)
                .credential_source("deepseek", CredentialSource::AuthOverride);
        } else if provider_enabled(configured.as_ref(), "deepseek")
            && get_secret("DEEPSEEK_API_KEY").is_some()
        {
            builder = builder
                .add_provider(DeepSeekAdapter::from_env()?)
                .credential_source("deepseek", CredentialSource::EnvApiKey("DEEPSEEK_API_KEY"));
        }

        // Ollama
        if provider_enabled(configured.as_ref(), "ollama")
            && let Some(auth) = overrides.get("ollama")
        {
            let base_url = OllamaAdapter::base_url_from_env_or_default();
            builder = builder
                .add_provider(OllamaAdapter::with_auth(base_url, Some(auth.clone()))?)
                .credential_source("ollama", CredentialSource::AuthOverride);
        } else if provider_enabled(configured.as_ref(), "ollama")
            && (std::env::var("OLLAMA_BASE_URL").is_ok() || std::env::var("OLLAMA_HOST").is_ok())
        {
            builder = builder
                .add_provider(OllamaAdapter::from_env()?)
                .credential_source("ollama", CredentialSource::AutoDetected);
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
    /// Routes to the adapter identified by `request.provider`, model inference,
    /// or selected provider order. Applies middleware in registration order.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if no provider can be resolved.
    /// Provider errors are propagated as-is.
    pub async fn complete(&self, request: Request) -> SdkResult<Response> {
        let mut request = request;
        Self::resolve_model(&mut request)?;
        let provider = self.resolve_provider(&request)?;

        if self.middleware.is_empty() {
            return provider.complete(request).await;
        }

        let chain = self.build_complete_chain(provider);
        chain(request).await
    }

    /// Send a request and return a stream of events.
    ///
    /// Routes to the adapter identified by `request.provider`, model inference,
    /// or selected provider order. Applies middleware in registration order.
    ///
    /// # Errors
    ///
    /// Returns `SdkError::Configuration` if no provider can be resolved.
    /// Connection errors are returned from the outer future; per-event
    /// errors appear as `Err` items in the stream.
    pub async fn stream(
        &self,
        mut request: Request,
    ) -> SdkResult<BoxStream<'_, SdkResult<StreamEvent>>> {
        Self::resolve_model(&mut request)?;
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
        self.select_provider()
    }

    /// Select a provider when no explicit provider is specified.
    ///
    /// Uses `models.providers` order when configured, otherwise falls back
    /// to the first registered provider.
    #[must_use]
    pub fn select_provider(&self) -> Option<&str> {
        if let Some(configured) = configured_providers() {
            for provider in configured {
                if self.providers.contains_key(&provider) {
                    return Some(self.providers.get_key_value(&provider)?.0.as_str());
                }
            }
        }

        self.default_provider.as_deref()
    }

    /// Whether a provider with the given name is registered.
    #[must_use]
    pub fn has_provider(&self, name: &str) -> bool {
        self.providers.contains_key(name)
    }

    /// The credential source for a registered provider, if tracked.
    #[must_use]
    pub fn get_credential_source(&self, name: &str) -> Option<&CredentialSource> {
        self.credential_sources.get(name)
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

    /// If `request.model` is a catalog alias, replace it with the
    /// canonical model ID so provider adapters send the correct value
    /// to the upstream API.
    fn resolve_model(request: &mut Request) -> SdkResult<()> {
        if let Some(info) = crate::catalog::get_model_info(&request.model)?
            && info.id != request.model
        {
            request.model = info.id;
        }
        Ok(())
    }

    /// Resolve the provider adapter for a request.
    fn resolve_provider(&self, request: &Request) -> SdkResult<&dyn ProviderAdapter> {
        let inferred_provider = if request.provider.is_none() {
            self.infer_provider_from_model(&request.model)?
        } else {
            None
        };

        let name = if let Some(explicit) = request.provider.as_deref() {
            explicit
        } else if let Some(ref inferred) = inferred_provider {
            inferred.as_str()
        } else {
            self.select_provider()
                .ok_or_else(|| SdkError::Configuration {
                    message: "no provider specified and no available model provider found".into(),
                })?
        };

        self.providers
            .get(name)
            .map(AsRef::as_ref)
            .ok_or_else(|| SdkError::Configuration {
                message: format!("unknown provider: {name}"),
            })
    }

    /// Infer a provider from a model ID/alias using the catalog, falling
    /// back to name-based heuristics for models not yet in the catalog.
    ///
    /// Returns:
    /// - `Ok(Some(provider))` when inference is possible
    /// - `Ok(None)` when no heuristic matches
    /// - `Err(Configuration)` when the model maps to multiple providers
    ///
    /// # Errors
    ///
    /// Returns [`SdkError::Configuration`] when the model name matches
    /// multiple configured providers and the choice is ambiguous.
    pub fn infer_provider_from_model(&self, model: &str) -> SdkResult<Option<String>> {
        let catalog = crate::catalog::read_catalog()?;
        let mut matches: Vec<String> = Vec::new();

        // Check by direct model ID
        for info in &catalog.models {
            if info.id == model && !matches.contains(&info.provider) {
                matches.push(info.provider.clone());
            }
        }

        // If not found by ID, check alias index
        if matches.is_empty()
            && let Some((provider, _)) = catalog.aliases.get(model)
        {
            matches.push(provider.clone());
        }
        drop(catalog);

        if matches.is_empty() {
            // Catalog miss — fall back to name-based heuristics so that
            // new or unlisted models still route to the right provider.
            return Ok(Self::infer_provider_from_name(model));
        }

        // If only one provider matches, use it directly.
        if matches.len() == 1 {
            return Ok(matches.first().cloned());
        }

        // If multiple providers match, but only one is configured in this
        // client, use that provider.
        let configured: Vec<String> = matches
            .into_iter()
            .filter(|name| self.providers.contains_key(name))
            .collect();
        if configured.len() == 1 {
            return Ok(configured.first().cloned());
        }

        Err(SdkError::Configuration {
            message: format!(
                "model '{model}' is ambiguous across providers; specify request.provider"
            ),
        })
    }

    /// Infer a provider from a model name using prefix/substring heuristics.
    ///
    /// This handles models not yet in the catalog (e.g. newly released models,
    /// dated snapshots, or fine-tuned variants). This is a static method that
    /// does not require a `Client` instance or catalog access.
    #[must_use]
    pub fn infer_provider_from_name(model: &str) -> Option<String> {
        let m = model.to_ascii_lowercase();

        // Anthropic: claude-*, models always start with "claude"
        if m.starts_with("claude") {
            return Some("anthropic".into());
        }

        // OpenAI: gpt-*, o1-*, o3-*, o4-*, chatgpt-*, ft:gpt-*
        if m.starts_with("gpt")
            || m.starts_with("o1")
            || m.starts_with("o3")
            || m.starts_with("o4")
            || m.starts_with("chatgpt")
            || m.starts_with("ft:gpt")
        {
            return Some("openai".into());
        }

        // Google: gemini-*
        if m.starts_with("gemini") {
            return Some("gemini".into());
        }

        // Mistral: mistral-*, codestral-*, pixtral-*, ministral-*
        if m.starts_with("mistral")
            || m.starts_with("codestral")
            || m.starts_with("pixtral")
            || m.starts_with("ministral")
        {
            return Some("mistral".into());
        }

        // DeepSeek: deepseek-*
        if m.starts_with("deepseek") {
            return Some("deepseek".into());
        }

        None
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
    credential_sources: HashMap<String, CredentialSource>,
}

impl std::fmt::Debug for ClientBuilder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClientBuilder")
            .field("providers", &self.providers.keys().collect::<Vec<_>>())
            .field("default_provider", &self.default_provider)
            .field("middleware_count", &self.middleware.len())
            .field("credential_sources", &self.credential_sources)
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
            credential_sources: HashMap::new(),
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

    /// Register a provider adapter under a custom name.
    ///
    /// Use this when an adapter should serve requests for a provider
    /// name different from its own `name()` (e.g. using the Chat
    /// Completions adapter to serve `"openai"` catalog models).
    #[must_use]
    pub fn add_provider_as(
        mut self,
        name: impl Into<String>,
        adapter: impl ProviderAdapter + 'static,
    ) -> Self {
        let name = name.into();
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

    /// Record how a provider's credentials were obtained.
    #[must_use]
    pub fn credential_source(
        mut self,
        provider: impl Into<String>,
        source: CredentialSource,
    ) -> Self {
        self.credential_sources.insert(provider.into(), source);
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
            credential_sources: self.credential_sources,
        })
    }
}
