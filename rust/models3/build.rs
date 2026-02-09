//! Build-time catalog refresh.
//!
//! When the `REFRESH_MODEL_CATALOG` environment variable is set to `1`,
//! this build script fetches model listings from provider APIs and merges
//! newly discovered models into `src/catalog/models.json`.
//!
//! Required environment variables (per provider):
//!   - `OPENAI_API_KEY` — for OpenAI model listing
//!   - `ANTHROPIC_API_KEY` — for Anthropic model listing
//!   - `GEMINI_API_KEY` — for Gemini model listing
//!
//! Providers whose keys are absent are silently skipped.
//!
//! # Why `ureq`?
//!
//! Build scripts run synchronously at compile time — they cannot use an
//! async runtime (tokio/reqwest). `ureq` is a lightweight, blocking HTTP
//! client purpose-built for this use case.

// Build scripts use eprintln! for cargo diagnostic output.
#![allow(clippy::print_stderr)]

use std::collections::{HashMap, HashSet};
use std::path::Path;

use serde::{Deserialize, Serialize};

/// Mirrors `catalog::ModelInfo` for the build script (which cannot import
/// the crate's own types).
#[derive(Debug, Clone, Serialize, Deserialize)]
struct ModelInfo {
    id: String,
    provider: String,
    display_name: String,
    context_window: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_output: Option<u64>,
    #[serde(default)]
    supports_tools: bool,
    #[serde(default)]
    supports_vision: bool,
    #[serde(default)]
    supports_reasoning: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    input_cost_per_million: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_cost_per_million: Option<f64>,
    #[serde(default)]
    aliases: Vec<String>,
}

fn main() {
    // Always tell cargo to rerun if the catalog source changes
    println!("cargo:rerun-if-changed=src/catalog/models.json");
    println!("cargo:rerun-if-env-changed=REFRESH_MODEL_CATALOG");
    // Rerun when API keys change so refresh uses updated credentials
    println!("cargo:rerun-if-env-changed=OPENAI_API_KEY");
    println!("cargo:rerun-if-env-changed=ANTHROPIC_API_KEY");
    println!("cargo:rerun-if-env-changed=GEMINI_API_KEY");
    println!("cargo:rerun-if-env-changed=GOOGLE_API_KEY");

    if std::env::var("REFRESH_MODEL_CATALOG").as_deref() != Ok("1") {
        return;
    }

    eprintln!("build.rs: refreshing model catalog...");

    let catalog_path = Path::new("src/catalog/models.json");
    let existing_json =
        std::fs::read_to_string(catalog_path).unwrap_or_else(|_| String::from("[]"));
    let mut catalog: Vec<ModelInfo> =
        serde_json::from_str(&existing_json).expect("existing catalog/models.json is invalid JSON");

    // Use (provider, id) as composite key to avoid cross-provider collisions
    let mut known_keys: HashSet<(String, String)> = catalog
        .iter()
        .map(|m| (m.provider.clone(), m.id.clone()))
        .collect();

    let mut added = 0usize;

    // Anthropic
    if let Ok(api_key) = std::env::var("ANTHROPIC_API_KEY") {
        match fetch_anthropic_models(&api_key) {
            Ok(models) => {
                for m in models {
                    let key = (m.provider.clone(), m.id.clone());
                    if !known_keys.contains(&key) {
                        let pos = catalog
                            .iter()
                            .rposition(|c| c.provider == m.provider)
                            .map_or(catalog.len(), |last| last + 1);
                        known_keys.insert(key);
                        catalog.insert(pos, m);
                        added += 1;
                    }
                }
            }
            Err(e) => eprintln!("build.rs: Anthropic model list failed: {e}"),
        }
    }

    // Gemini (with GOOGLE_API_KEY fallback, matching runtime from_env behavior)
    if let Ok(api_key) =
        std::env::var("GEMINI_API_KEY").or_else(|_| std::env::var("GOOGLE_API_KEY"))
    {
        match fetch_gemini_models(&api_key) {
            Ok(models) => {
                for m in models {
                    let key = (m.provider.clone(), m.id.clone());
                    if !known_keys.contains(&key) {
                        let pos = catalog
                            .iter()
                            .rposition(|c| c.provider == m.provider)
                            .map_or(catalog.len(), |last| last + 1);
                        known_keys.insert(key);
                        catalog.insert(pos, m);
                        added += 1;
                    }
                }
            }
            Err(e) => eprintln!("build.rs: Gemini model list failed: {e}"),
        }
    }

    // OpenAI
    if let Ok(api_key) = std::env::var("OPENAI_API_KEY") {
        match fetch_openai_models(&api_key) {
            Ok(models) => {
                for m in models {
                    let key = (m.provider.clone(), m.id.clone());
                    if !known_keys.contains(&key) {
                        // Append after curated entries (end of provider group)
                        let pos = catalog
                            .iter()
                            .rposition(|c| c.provider == m.provider)
                            .map_or(catalog.len(), |last| last + 1);
                        known_keys.insert(key);
                        catalog.insert(pos, m);
                        added += 1;
                    }
                }
            }
            Err(e) => eprintln!("build.rs: OpenAI model list failed: {e}"),
        }
    }

    if added > 0 {
        eprintln!("build.rs: added {added} new model(s) to catalog");
    } else {
        eprintln!("build.rs: catalog is up-to-date, no new models");
    }

    // Enrich catalog with LiteLLM metadata (context windows, capabilities, costs)
    let enriched = match fetch_litellm_metadata() {
        Ok(metadata) => {
            enrich_catalog(&mut catalog, &metadata);
            eprintln!("build.rs: enriched catalog with LiteLLM metadata");
            true
        }
        Err(e) => {
            eprintln!("build.rs: LiteLLM metadata fetch failed (non-fatal): {e}");
            false
        }
    };

    if added > 0 || enriched {
        let json = serde_json::to_string_pretty(&catalog).expect("failed to serialize catalog");
        std::fs::write(catalog_path, json).expect("failed to write catalog/models.json");
    }
}

fn fetch_anthropic_models(api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let resp: serde_json::Value = ureq::get("https://api.anthropic.com/v1/models")
        .set("x-api-key", api_key)
        .set("anthropic-version", "2023-06-01")
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;

    Ok(resp
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id")?.as_str()?.to_string();
                    let display_name = m
                        .get("display_name")
                        .and_then(|n| n.as_str())
                        .unwrap_or(&id)
                        .to_string();
                    Some(ModelInfo {
                        id,
                        provider: "anthropic".into(),
                        display_name,
                        context_window: 0,
                        max_output: None,
                        supports_tools: false,
                        supports_vision: false,
                        supports_reasoning: false,
                        input_cost_per_million: None,
                        output_cost_per_million: None,
                        aliases: vec![],
                    })
                })
                .collect()
        })
        .unwrap_or_default())
}

fn fetch_openai_models(api_key: &str) -> Result<Vec<ModelInfo>, String> {
    fn is_excluded_openai_model(id: &str) -> bool {
        /// ID prefixes for non-chat OpenAI models.
        /// Canonical list is in `src/providers/common/openai_shared.rs`.
        const EXCLUDED_OPENAI_PREFIXES: &[&str] = &[
            "text-embedding-",
            "dall-e-",
            "gpt-image-",
            "tts-",
            "whisper-",
            "sora-",
            "davinci-",
            "babbage-",
            "codex-",
            "omni-moderation-",
            "chatgpt-image-",
            "computer-use-",
        ];

        /// Substrings that indicate non-chat OpenAI model variants.
        /// Canonical list is in `src/providers/common/openai_shared.rs`.
        const EXCLUDED_OPENAI_SUBSTRINGS: &[&str] = &[
            "-tts",
            "-realtime",
            "-audio-",
            "-transcribe",
            "-search-",
            "-deep-research",
        ];

        EXCLUDED_OPENAI_PREFIXES
            .iter()
            .any(|prefix| id.starts_with(prefix))
            || EXCLUDED_OPENAI_SUBSTRINGS
                .iter()
                .any(|sub| id.contains(sub))
    }

    let resp: serde_json::Value = ureq::get("https://api.openai.com/v1/models")
        .set("Authorization", &format!("Bearer {api_key}"))
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;

    Ok(resp
        .get("data")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    let id = m.get("id")?.as_str()?.to_string();
                    if is_excluded_openai_model(&id) {
                        return None;
                    }
                    Some(ModelInfo {
                        id: id.clone(),
                        provider: "openai".into(),
                        display_name: id,
                        context_window: 0,
                        max_output: None,
                        supports_tools: false,
                        supports_vision: false,
                        supports_reasoning: false,
                        input_cost_per_million: None,
                        output_cost_per_million: None,
                        aliases: vec![],
                    })
                })
                .collect()
        })
        .unwrap_or_default())
}

fn fetch_litellm_metadata() -> Result<HashMap<String, serde_json::Value>, String> {
    let url = "https://raw.githubusercontent.com/BerriAI/litellm/main/model_prices_and_context_window.json";
    let resp: serde_json::Value = ureq::get(url)
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;

    let map: HashMap<String, serde_json::Value> =
        serde_json::from_value(resp).map_err(|e| e.to_string())?;
    Ok(map)
}

fn enrich_catalog(catalog: &mut [ModelInfo], metadata: &HashMap<String, serde_json::Value>) {
    for model in catalog.iter_mut() {
        let litellm_key = match model.provider.as_str() {
            "anthropic" | "openai" => model.id.clone(),
            "gemini" => format!("gemini/{}", model.id),
            _ => continue,
        };

        let Some(entry) = metadata.get(&litellm_key) else {
            continue;
        };

        if let Some(v) = entry.get("max_input_tokens").and_then(|v| v.as_u64()) {
            model.context_window = v;
        }
        if let Some(v) = entry.get("max_output_tokens").and_then(|v| v.as_u64()) {
            model.max_output = Some(v);
        }
        if let Some(v) = entry
            .get("supports_function_calling")
            .and_then(|v| v.as_bool())
        {
            model.supports_tools = v;
        }
        if let Some(v) = entry.get("supports_vision").and_then(|v| v.as_bool()) {
            model.supports_vision = v;
        }
        if let Some(v) = entry.get("supports_reasoning").and_then(|v| v.as_bool()) {
            model.supports_reasoning = v;
        }
        if let Some(v) = entry.get("input_cost_per_token").and_then(|v| v.as_f64()) {
            model.input_cost_per_million = Some(v * 1_000_000.0);
        }
        if let Some(v) = entry.get("output_cost_per_token").and_then(|v| v.as_f64()) {
            model.output_cost_per_million = Some(v * 1_000_000.0);
        }
    }
}

fn fetch_gemini_models(api_key: &str) -> Result<Vec<ModelInfo>, String> {
    let url = format!("https://generativelanguage.googleapis.com/v1beta/models?key={api_key}");
    let resp: serde_json::Value = ureq::get(&url)
        .call()
        .map_err(|e| e.to_string())?
        .into_json()
        .map_err(|e| e.to_string())?;

    Ok(resp
        .get("models")
        .and_then(|d| d.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|m| {
                    // Only include models that support generateContent
                    let supports_generate = m
                        .get("supportedGenerationMethods")
                        .and_then(|v| v.as_array())
                        .is_some_and(|methods| {
                            methods
                                .iter()
                                .any(|v| v.as_str() == Some("generateContent"))
                        });
                    if !supports_generate {
                        return None;
                    }

                    let raw_name = m.get("name")?.as_str()?;
                    let id = raw_name
                        .strip_prefix("models/")
                        .unwrap_or(raw_name)
                        .to_string();
                    let display_name = m
                        .get("displayName")
                        .and_then(|n| n.as_str())
                        .unwrap_or(&id)
                        .to_string();
                    let context_window = m
                        .get("inputTokenLimit")
                        .and_then(|n| n.as_u64())
                        .unwrap_or(0);
                    let max_output = m.get("outputTokenLimit").and_then(|n| n.as_u64());
                    Some(ModelInfo {
                        id,
                        provider: "gemini".into(),
                        display_name,
                        context_window,
                        max_output,
                        supports_tools: false,
                        supports_vision: false,
                        supports_reasoning: false,
                        input_cost_per_million: None,
                        output_cost_per_million: None,
                        aliases: vec![],
                    })
                })
                .collect()
        })
        .unwrap_or_default())
}
