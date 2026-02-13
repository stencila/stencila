use std::{env, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use eyre::Result;
use stencila_auth::{AuthOptions, AuthOverrides};
use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    stencila_format::Format,
    tabulated::{Attribute, Cell, CellAlignment, Color, Tabulated},
};

use crate::secret::get_secret;
use stencila_auth::{claude_code, codex_cli};

use crate::catalog::{self, ModelInfo};

/// Manage and interact with generative AI models
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all available models</dim>
  <b>stencila models</b>

  <dim># List models as JSON</dim>
  <b>stencila models list</b> <c>--as</c> <g>json</g>

  <dim># Filter models by provider or ID prefix</dim>
  <b>stencila models list</b> <g>anthropic</g>

  <dim># Run a prompt with automatic model selection</dim>
  <b>stencila models run</b> <y>\"Explain photosynthesis\"</y>

  <dim># Run with a specific model</dim>
  <b>stencila models run</b> <y>\"Write a poem\"</y> <c>--model</c> <g>gpt-4o</g>

  <dim># Mix text and file arguments</dim>
  <b>stencila models run</b> <y>\"Summarize this file:\"</y> <g>document.txt</g>

  <dim># Dry run to see prompt construction</dim>
  <b>stencila models run</b> <y>\"Hello\"</y> <c>--dry-run</c>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Run(Run),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        self.run_with_auth(&AuthOptions::default()).await
    }

    /// Run the CLI command with the given auth options.
    ///
    /// Providers present in the overrides are treated as available even
    /// when no API key secret is set.
    pub async fn run_with_auth(self, auth: &AuthOptions) -> Result<()> {
        let Some(command) = self.command else {
            List::default().run(auth).await?;
            return Ok(());
        };

        match command {
            Command::List(list) => list.run(auth).await?,
            Command::Run(run) => run.run(auth).await?,
        }

        Ok(())
    }
}

/// List available models with their capabilities and pricing
#[derive(Default, Debug, Args)]
#[command(after_long_help = LIST_AFTER_LONG_HELP)]
struct List {
    /// Filter models by provider or ID prefix (e.g. "anthropic", "gpt-4")
    prefix: Option<String>,

    /// Output the list as JSON or YAML
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List all models in table format</dim>
  <b>stencila models list</b>

  <dim># Filter models by provider</dim>
  <b>stencila models list</b> <g>anthropic</g>

  <dim># Filter models by ID prefix</dim>
  <b>stencila models list</b> <g>gpt-4</g>

  <dim># Output models as JSON</dim>
  <b>stencila models list</b> <c>--as</c> <g>json</g>

  <dim># Output models as YAML</dim>
  <b>stencila models list</b> <c>--as</c> <g>yaml</g>
"
);

impl List {
    async fn run(self, auth: &AuthOptions) -> Result<()> {
        let mut models = catalog::list_models(None).map_err(|e| eyre::eyre!("{e}"))?;

        if let Some(prefix) = &self.prefix {
            let prefix_lower = prefix.to_lowercase();
            models.retain(|m| {
                m.id.to_lowercase().starts_with(&prefix_lower)
                    || m.provider.to_lowercase().starts_with(&prefix_lower)
            });
        }

        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &models)?.to_stdout();
            return Ok(());
        }

        let mut table = Tabulated::new();
        table.set_header([
            "Id",
            "Provider",
            "Capabilities",
            "Context",
            "Input $/M",
            "Output $/M",
        ]);

        for model in &models {
            let available = is_provider_available(&model.provider, &auth.overrides);
            let id_cell = if available {
                Cell::new(&model.id).add_attribute(Attribute::Bold)
            } else {
                Cell::new(&model.id).add_attribute(Attribute::Dim)
            };

            table.add_row([
                id_cell,
                provider_cell(&model.provider),
                Cell::new(format_capabilities(model)),
                context_cell(model.context_window),
                cost_cell(model.input_cost_per_million),
                cost_cell(model.output_cost_per_million),
            ]);
        }

        table.to_stdout();

        // Summary and legend
        let enabled = models
            .iter()
            .filter(|m| is_provider_available(&m.provider, &auth.overrides))
            .count();
        let total = models.len();
        if enabled < total {
            message!(
                "{} of {} models enabled. Use <b>stencila signin</>, <b>stencila auth login <<provider>> </>, or <b>stencila secrets set <<key>></> to enable more.\n\
                 Capabilities: <bold>T</>ools <bold>V</>ision <bold>R</>easoning",
                enabled,
                total
            );
        } else {
            message!(
                "{} of {} models enabled. Capabilities: <bold>T</>ools <bold>V</>ision <bold>R</>easoning",
                enabled,
                total
            );
        }

        Ok(())
    }
}

/// Check whether a provider's API key, OAuth credential, or equivalent is available.
fn is_provider_available(provider: &str, overrides: &AuthOverrides) -> bool {
    if overrides.contains_key(provider) {
        return true;
    }
    match provider {
        "openai" => {
            get_secret("OPENAI_API_KEY").is_some() || codex_cli::load_credentials().is_some()
        }
        "anthropic" => {
            get_secret("ANTHROPIC_API_KEY").is_some() || claude_code::load_credentials().is_some()
        }
        "gemini" => {
            get_secret("GEMINI_API_KEY").is_some() || get_secret("GOOGLE_API_KEY").is_some()
        }
        "mistral" => get_secret("MISTRAL_API_KEY").is_some(),
        "deepseek" => get_secret("DEEPSEEK_API_KEY").is_some(),
        "ollama" => env::var("OLLAMA_BASE_URL").is_ok() || env::var("OLLAMA_HOST").is_ok(),
        _ => false,
    }
}

/// Color and case a provider name cell.
fn provider_cell(provider: &str) -> Cell {
    let (label, color) = match provider {
        "openai" => ("OpenAI", Color::Green),
        "anthropic" => ("Anthropic", Color::Magenta),
        "gemini" => ("Gemini", Color::Blue),
        "mistral" => ("Mistral", Color::Cyan),
        "deepseek" => ("DeepSeek", Color::DarkCyan),
        "ollama" => ("Ollama", Color::DarkYellow),
        other => (other, Color::White),
    };
    Cell::new(label).fg(color)
}

/// Format model capabilities as a compact string (e.g. "T V R" or "T . .").
fn format_capabilities(model: &ModelInfo) -> String {
    let t = if model.supports_tools { "T" } else { "." };
    let v = if model.supports_vision { "V" } else { "." };
    let r = if model.supports_reasoning { "R" } else { "." };
    format!("{t} {v} {r}")
}

/// Format a context window as a right-aligned, color-banded cell.
fn context_cell(tokens: u64) -> Cell {
    let label = if tokens >= 1_000_000 {
        let millions = tokens as f64 / 1_000_000.0;
        if (millions - millions.round()).abs() < f64::EPSILON {
            format!("{}M", millions as u64)
        } else {
            format!("{millions:.1}M")
        }
    } else if tokens >= 1_000 {
        let thousands = tokens as f64 / 1_000.0;
        if (thousands - thousands.round()).abs() < f64::EPSILON {
            format!("{}K", thousands as u64)
        } else {
            format!("{thousands:.1}K")
        }
    } else {
        tokens.to_string()
    };

    let color = match tokens {
        0..=8_000 => Color::DarkRed,
        8_001..=32_000 => Color::DarkYellow,
        32_001..=128_000 => Color::Yellow,
        128_001..=256_000 => Color::Green,
        _ => Color::Cyan,
    };

    Cell::new(label)
        .fg(color)
        .set_alignment(CellAlignment::Right)
}

/// Format a cost value as a right-aligned cell, showing "—" when absent.
fn cost_cell(cost: Option<f64>) -> Cell {
    let cell = match cost {
        Some(c) => Cell::new(format!("${c:.2}")),
        None => Cell::new("—"),
    };
    cell.set_alignment(CellAlignment::Right)
}

/// Execute a prompt using a generative AI model
///
/// Constructs a prompt from the provided text and file arguments, then streams
/// the model's response to stdout. Arguments that correspond to existing file
/// paths are read and included as file content.
#[derive(Debug, Args)]
#[command(after_long_help = RUN_AFTER_LONG_HELP)]
struct Run {
    /// Text prompts and/or file paths (automatically detected)
    args: Vec<String>,

    /// Model id to use (e.g. "gpt-4o", "claude-sonnet-4-5-20250929")
    #[arg(long, short)]
    model: Option<String>,

    /// Provider name (e.g. "openai", "anthropic")
    #[arg(long, short)]
    provider: Option<String>,

    /// System message to set context or behavior
    #[arg(long)]
    system: Option<String>,

    /// Sampling temperature (0.0–2.0)
    #[arg(long)]
    temperature: Option<f64>,

    /// Maximum tokens to generate
    #[arg(long)]
    max_tokens: Option<u64>,

    /// Write output to the specified file instead of stdout
    #[arg(long, short)]
    output: Option<PathBuf>,

    /// Show prompt construction without executing
    #[arg(long)]
    dry_run: bool,
}

pub static RUN_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Run with automatic model selection</dim>
  <b>stencila models run</b> <y>\"Explain quantum computing\"</y>

  <dim># Run with a specific model</dim>
  <b>stencila models run</b> <y>\"Write a haiku\"</y> <c>--model</c> <g>gpt-4o</g>

  <dim># Use a specific provider</dim>
  <b>stencila models run</b> <y>\"Hello\"</y> <c>--provider</c> <g>anthropic</g>

  <dim># Multiple text arguments</dim>
  <b>stencila models run</b> <y>\"Analyze this data:\"</y> <y>\"temperature: 23C, humidity: 65%\"</y>

  <dim># Mix text and file paths (files detected automatically)</dim>
  <b>stencila models run</b> <y>\"Summarize:\"</y> <g>report.txt</g>

  <dim># Write output to a file</dim>
  <b>stencila models run</b> <y>\"Generate a story\"</y> <c>--output</c> <g>story.md</g>

  <dim># Dry run to see prompt construction</dim>
  <b>stencila models run</b> <y>\"Hello world\"</y> <c>--dry-run</c>
"
);

impl Run {
    #[allow(clippy::print_stdout, clippy::print_stderr)]
    async fn run(self, auth: &AuthOptions) -> Result<()> {
        // Build prompt from args: detect file paths and read their content
        let mut parts = Vec::new();
        for arg in &self.args {
            let path = PathBuf::from(arg);
            if path.exists() && path.is_file() {
                let content = std::fs::read_to_string(&path)
                    .map_err(|e| eyre::eyre!("Failed to read {}: {e}", path.display()))?;
                parts.push(format!("--- {} ---\n{content}", path.display()));
            } else {
                parts.push(arg.clone());
            }
        }

        let prompt = parts.join("\n\n");

        if prompt.is_empty() {
            return Err(eyre::eyre!(
                "No prompt provided. Pass text and/or file paths as arguments."
            ));
        }

        // Resolve model: use --model if given, otherwise pick the latest for the provider
        let client = if auth.overrides.is_empty() {
            crate::client::Client::from_env()
        } else {
            crate::client::Client::from_env_with_auth(auth)
        }
        .map_err(|e| eyre::eyre!("{e}"))?;

        let (model_id, resolved_provider) =
            resolve_model_and_provider(&self.model, &self.provider, &client)?;
        let provider_label = resolved_provider.as_deref().unwrap_or("<default>");

        // Dry run: show the prompt and selected model, then exit
        if self.dry_run {
            Code::new(Format::Markdown, "# Prompt\n").to_stdout();
            Code::new(Format::Markdown, &prompt).to_stdout();

            Code::new(Format::Markdown, "\n# Model\n").to_stdout();
            let model_info = catalog::get_model_info(&model_id).map_err(|e| eyre::eyre!("{e}"))?;
            match model_info {
                Some(info) => {
                    Code::new_from(Format::Yaml, &info)?.to_stdout();
                }
                None => {
                    Code::new(Format::Yaml, &format!("id: {model_id}\n(not in catalog)"))
                        .to_stdout();
                }
            }
            Code::new(
                Format::Markdown,
                &format!("\n# Provider\n{provider_label}\n"),
            )
            .to_stdout();

            if let Some(ref system) = self.system {
                Code::new(Format::Markdown, "\n# System prompt\n").to_stdout();
                Code::new(Format::Markdown, system).to_stdout();
            }

            return Ok(());
        }

        // Stream the response
        let mut opts = crate::api::stream::StreamOptions::new(&model_id)
            .prompt(&prompt)
            .client(&client);

        if let Some(ref provider) = resolved_provider {
            opts = opts.provider(provider);
        }
        if let Some(ref system) = self.system {
            opts = opts.system(system);
        }
        if let Some(temp) = self.temperature {
            opts = opts.temperature(temp);
        }
        if let Some(max) = self.max_tokens {
            opts = opts.max_tokens(max);
        }

        let mut stream = crate::api::stream::stream_generate(opts)
            .await
            .map_err(|e| {
                eyre::eyre!("model run failed (model: {model_id}, provider: {provider_label}): {e}")
            })?;

        // Consume the stream event by event so we can both print deltas
        // incrementally and capture usage at the end.
        let mut collected_text = String::new();
        let writing_to_file = self.output.is_some();

        while let Some(event_result) = stream.next_event().await {
            let event = event_result.map_err(|e| {
                eyre::eyre!("model run failed (model: {model_id}, provider: {provider_label}): {e}")
            })?;
            if event.event_type == crate::types::stream_event::StreamEventType::TextDelta {
                if let Some(ref delta) = event.delta {
                    if writing_to_file {
                        collected_text.push_str(delta);
                    } else {
                        print!("{delta}");
                        collected_text.push_str(delta);
                    }
                }
            }
        }

        // Get usage from the accumulated response
        let usage = stream.response().map(|r| r.usage).unwrap_or_default();

        if let Some(ref path) = self.output {
            std::fs::write(path, &collected_text)
                .map_err(|e| eyre::eyre!("Failed to write {}: {e}", path.display()))?;
            eprintln!("Wrote {} bytes to {}", collected_text.len(), path.display());
        } else {
            // Ensure we end on a newline
            if !collected_text.ends_with('\n') {
                println!();
            }
        }

        print_usage_summary(&usage, &model_id);

        Ok(())
    }
}

/// Resolve a model id and provider from CLI flags and client defaults.
///
/// - If `--model` is omitted, chooses latest model for selected/default provider.
/// - If `--model` is provided, resolves exact id/alias first, then unambiguous prefix.
/// - If provider remains unspecified, it is inferred from the resolved catalog model.
fn resolve_model_and_provider(
    model_flag: &Option<String>,
    provider_flag: &Option<String>,
    client: &crate::client::Client,
) -> Result<(String, Option<String>)> {
    match model_flag {
        None => {
            let provider = provider_flag
                .as_deref()
                .or(client.default_provider())
                .ok_or_else(|| {
                    eyre::eyre!(
                        "No --model specified and no model provider available. \
                        Use `stencila signin` or `stencila secrets set` to enable."
                    )
                })?;
            let info = catalog::get_latest_model(provider, None)
                .map_err(|e| eyre::eyre!("{e}"))?
                .ok_or_else(|| eyre::eyre!("No models found for provider '{provider}'"))?;
            Ok((info.id, Some(provider.to_string())))
        }
        Some(raw_model) => {
            // 1) Exact ID/alias lookup
            if let Some(info) =
                catalog::get_model_info(raw_model).map_err(|e| eyre::eyre!("{e}"))?
            {
                if let Some(provider) = provider_flag
                    && provider != &info.provider
                {
                    return Err(eyre::eyre!(
                        "Model '{raw_model}' resolves to provider '{}', but --provider is '{provider}'",
                        info.provider
                    ));
                }
                return Ok((info.id, Some(info.provider)));
            }

            // 2) Prefix lookup (e.g. --model claude -> latest matching claude-* model)
            let candidates =
                catalog::list_models(provider_flag.as_deref()).map_err(|e| eyre::eyre!("{e}"))?;
            let matches: Vec<_> = candidates
                .into_iter()
                .filter(|m| {
                    m.id.starts_with(raw_model)
                        || m.aliases.iter().any(|alias| alias.starts_with(raw_model))
                })
                .collect();

            match matches.len() {
                0 => Ok((raw_model.clone(), provider_flag.clone())),
                1 => {
                    let info = &matches[0];
                    Ok((info.id.clone(), Some(info.provider.clone())))
                }
                _ => {
                    // Catalog order is newest/best first within provider groups.
                    // If all prefix matches belong to one provider, pick the first.
                    let first_provider = &matches[0].provider;
                    if matches.iter().all(|m| &m.provider == first_provider) {
                        let info = &matches[0];
                        Ok((info.id.clone(), Some(info.provider.clone())))
                    } else {
                        let preview = matches
                            .iter()
                            .take(5)
                            .map(|m| m.id.as_str())
                            .collect::<Vec<_>>()
                            .join(", ");
                        Err(eyre::eyre!(
                            "Model '{raw_model}' is ambiguous across providers. Matches: {preview}. \
                            Specify a more precise --model or add --provider."
                        ))
                    }
                }
            }
        }
    }
}

/// Print a usage summary to stderr.
#[allow(clippy::print_stderr)]
fn print_usage_summary(usage: &crate::types::usage::Usage, model_id: &str) {
    eprintln!(
        "\n[{model_id}] {} input + {} output = {} total tokens",
        usage.input_tokens, usage.output_tokens, usage.total_tokens
    );
}
