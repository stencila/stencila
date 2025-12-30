//! CLI commands for managing workspace outputs
//!
//! This module provides commands to list configured outputs and push them
//! to Stencila Cloud workspace outputs.

use std::{
    collections::{HashMap, HashSet},
    env::current_dir,
    path::{Path, PathBuf},
};

use clap::{Args, Parser, Subcommand};
use eyre::{Result, bail, eyre};
use glob::{Pattern, glob};
use indexmap::IndexMap;
use tempfile::TempDir;

use stencila_cli_utils::{
    AsFormat, Code, ToStdout,
    color_print::cstr,
    message,
    tabulated::{Cell, CellAlignment, Color, Tabulated},
};
use stencila_cloud::{ensure_workspace, outputs::UploadResult};
use stencila_codec_utils::{GitRef, get_current_ref};
use stencila_codecs::{DecodeOptions, EncodeOptions};
use stencila_config::{
    OutputCommand, OutputConfig, OutputTarget, SpreadMode, config, config_add_output,
    config_remove_output,
};
use stencila_document::Document;
use stencila_format::Format;
use stencila_spread::{Run, SpreadConfig, apply_template};

/// Manage workspace outputs
#[derive(Debug, Parser)]
#[command(alias = "output", after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List configured outputs</dim>
  <b>stencila outputs</>
  <b>stencila outputs list</>
  <b>stencila outputs list --as toml</>

  <dim># Add an output</dim>
  <b>stencila outputs add report.pdf report.md</>
  <b>stencila outputs add report.pdf report.md --command render --refs main</>

  <dim># Remove an output</dim>
  <b>stencila outputs remove report.pdf</>

  <dim># Push all outputs to cloud</dim>
  <b>stencila outputs push</>

  <dim># Push specific outputs</dim>
  <b>stencila outputs push \"report.pdf\" \"data/*.csv\"</>

  <dim># Dry run (process but don't upload)</dim>
  <b>stencila outputs push --dry-run</>

  <dim># Force push (ignore refs filter)</dim>
  <b>stencila outputs push --force</>
"
);

#[derive(Debug, Subcommand)]
enum Command {
    List(List),
    Add(Add),
    Remove(Remove),
    Push(Push),
}

impl Cli {
    pub async fn run(self) -> Result<()> {
        let command = self.command.unwrap_or(Command::List(List::default()));

        match command {
            Command::List(list) => list.run().await,
            Command::Add(add) => add.run().await,
            Command::Remove(remove) => remove.run().await,
            Command::Push(push) => push.run().await,
        }
    }
}

/// List configured outputs
#[derive(Debug, Default, Args)]
#[command(alias = "ls", after_long_help = LIST_AFTER_LONG_HELP)]
pub struct List {
    /// Output format
    #[arg(long, short)]
    r#as: Option<AsFormat>,
}

pub static LIST_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># List configured outputs in table format</dim>
  <b>stencila outputs list</>

  <dim># List in JSON, YAML, or TOML format</dim>
  <b>stencila outputs list --as json</>
  <b>stencila outputs list --as yaml</>
  <b>stencila outputs list --as toml</>
"
);

impl List {
    pub async fn run(self) -> Result<()> {
        let workspace_dir = current_dir()?;
        let cfg = config(&workspace_dir)?;
        let outputs = cfg.outputs.unwrap_or_default();

        if outputs.is_empty() {
            message("ℹ️ No outputs configured in stencila.toml");
            return Ok(());
        }

        // Output as formatted code if --as specified
        if let Some(format) = self.r#as {
            Code::new_from(format.into(), &outputs)?.to_stdout();
            return Ok(());
        }

        // Build table
        let mut table = Tabulated::new();
        table.set_header(["Output", "Source", "Command", "Refs"]);

        for (key, target) in &outputs {
            let (source, command, refs) = match target {
                OutputTarget::Source(path) => {
                    let source_ext = Path::new(path.as_str())
                        .extension()
                        .and_then(|e| e.to_str());
                    let key_ext = Path::new(key).extension().and_then(|e| e.to_str());

                    // Auto-detect command
                    let command = match (source_ext, key_ext) {
                        (Some(s), Some(k)) if s == k => "none",
                        _ => "render",
                    };

                    (path.as_str().to_string(), command, "any".to_string())
                }
                OutputTarget::Config(cfg) => {
                    // Determine source
                    let source = if let Some(pattern) = &cfg.pattern {
                        pattern.clone()
                    } else if let Some(src) = &cfg.source {
                        src.clone()
                    } else {
                        key.clone()
                    };

                    // Determine command
                    let command = match cfg.command {
                        Some(OutputCommand::Render) => "render",
                        Some(OutputCommand::Convert) => "convert",
                        Some(OutputCommand::None) => "none",
                        None => {
                            // Auto-detect
                            let source_ext =
                                Path::new(&source).extension().and_then(|e| e.to_str());
                            let key_ext = Path::new(key).extension().and_then(|e| e.to_str());
                            match (source_ext, key_ext) {
                                (Some(s), Some(k)) if s == k => "none",
                                _ => "render",
                            }
                        }
                    };

                    // Format refs
                    let refs = cfg
                        .refs
                        .as_ref()
                        .map(|r| r.join(", "))
                        .unwrap_or_else(|| "any".to_string());

                    (source, command, refs)
                }
            };

            // Color the command
            let command_cell = match command {
                "render" => Cell::new(command).fg(Color::Green),
                "convert" => Cell::new(command).fg(Color::Cyan),
                "none" => Cell::new(command).fg(Color::Grey),
                _ => Cell::new(command),
            };

            table.add_row([
                Cell::new(key),
                Cell::new(source).fg(Color::Grey),
                command_cell.set_alignment(CellAlignment::Center),
                Cell::new(refs).fg(Color::Grey),
            ]);
        }

        table.to_stdout();

        Ok(())
    }
}

/// Add an output configuration
#[derive(Debug, Args)]
#[command(after_long_help = ADD_AFTER_LONG_HELP)]
pub struct Add {
    /// Output path
    ///
    /// This is the path where the output will be stored, e.g., "report.pdf"
    /// or "{region}/report.pdf" for spread outputs.
    output: String,

    /// Source file path
    ///
    /// The source file to render or convert. If not provided, the output
    /// is used as the source path (for static file copies).
    source: Option<String>,

    /// Processing command
    #[arg(long, short, value_enum)]
    command: Option<OutputCommand>,

    /// Git ref patterns for when to process this output
    ///
    /// Supports glob patterns and optional type prefixes:
    /// "main", "v*", "release/*" (matches any ref type),
    /// "branch:main" (matches only branches),
    /// "tag:v*" (matches only tags),
    /// "commit:*" (matches any commit SHA for CI builds on detached HEAD)
    #[arg(long, short, value_delimiter = ',')]
    refs: Option<Vec<String>>,

    /// Glob pattern for matching multiple source files
    ///
    /// Use this instead of source for multi-file outputs.
    /// When using --pattern, the output path must contain exactly one `*`
    /// which will be replaced with the matched file's stem (path without extension).
    /// The output must also include an extension to determine output format (e.g., "reports/*.pdf").
    /// Example: output "reports/*.pdf" with pattern "src/*.md" maps "src/intro.md" to "reports/intro.pdf"
    #[arg(long, short)]
    pattern: Option<String>,

    /// Glob patterns to exclude from pattern matches
    #[arg(long, short, value_delimiter = ',')]
    exclude: Option<Vec<String>>,

    /// Spread mode for multi-variant outputs (grid or zip)
    ///
    /// Use with outputs containing placeholders like "{region}/report.pdf".
    /// - grid: Cartesian product of all argument values (default)
    /// - zip: Positional pairing (all arguments must have same length)
    #[arg(long, value_enum)]
    spread: Option<SpreadMode>,

    /// Arguments for spread outputs (comma-delimited key=val1,val2 pairs)
    ///
    /// Example: stencila outputs add "{region}/report.pdf" report.md -- region=north,south
    #[arg(last = true, allow_hyphen_values = true)]
    arguments: Vec<String>,
}

pub static ADD_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Add a simple output (render report.md to report.pdf)</dim>
  <b>stencila outputs add report.pdf report.md</>

  <dim># Add with explicit render command</dim>
  <b>stencila outputs add report.pdf report.md --command render</>

  <dim># Add output that only runs on main branch</dim>
  <b>stencila outputs add report.pdf report.md --refs main</>

  <dim># Add static file (copy as-is)</dim>
  <b>stencila outputs add data.csv</>

  <dim># Add pattern-based outputs</dim>
  <b>stencila outputs add \"exports/*.pdf\" --pattern \"exports/*.md\"</>

  <dim># Add spread output (generates multiple variants)</dim>
  <b>stencila outputs add \"{region}/report.pdf\" report.md --command render -- region=north,south</>

  <dim># Add spread with multiple arguments (grid mode)</dim>
  <b>stencila outputs add \"{region}/{year}/data.pdf\" report.md --command render -- region=north,south year=2024,2025</>

  <dim># Add spread with zip mode</dim>
  <b>stencila outputs add \"{q}-report.pdf\" report.md --command render --spread zip -- q=q1,q2,q3,q4</>
"
);

impl Add {
    pub async fn run(self) -> Result<()> {
        // Parse arguments from CLI format into HashMap
        let arguments = if self.arguments.is_empty() {
            None
        } else {
            Some(Self::parse_arguments(&self.arguments)?)
        };

        let config_path = config_add_output(
            &self.output,
            self.source.as_deref(),
            self.command,
            self.refs.as_deref(),
            self.pattern.as_deref(),
            self.exclude.as_deref(),
            self.spread,
            arguments.as_ref(),
        )?;

        if !self.arguments.is_empty() {
            let mode = self.spread.unwrap_or_default();
            message!(
                "✅ Added spread output `{}` (mode: {:?}) to {}",
                self.output,
                mode,
                config_path.display()
            );
        } else {
            message!(
                "✅ Added output `{}` to {}",
                self.output,
                config_path.display()
            );
        }

        Ok(())
    }

    /// Parse arguments from CLI format "key=val1,val2" into HashMap
    fn parse_arguments(args: &[String]) -> Result<HashMap<String, Vec<String>>> {
        let mut result = HashMap::new();

        for arg in args {
            let parts: Vec<&str> = arg.splitn(2, '=').collect();
            if parts.len() != 2 {
                bail!(
                    "Invalid argument format '{}'. Expected 'key=val1,val2'",
                    arg
                );
            }

            let key = parts[0].trim().to_string();
            let values: Vec<String> = parts[1].split(',').map(|s| s.trim().to_string()).collect();

            if key.is_empty() {
                bail!("Argument key cannot be empty in '{}'", arg);
            }
            if values.is_empty() || values.iter().all(|v| v.is_empty()) {
                bail!("Argument '{}' must have at least one value", key);
            }

            result.insert(key, values);
        }

        Ok(result)
    }
}

/// Remove an output configuration
#[derive(Debug, Args)]
#[command(alias="rm", after_long_help = REMOVE_AFTER_LONG_HELP)]
pub struct Remove {
    /// Output to remove
    output: String,
}

pub static REMOVE_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Remove an output</dim>
  <b>stencila outputs remove report.pdf</>

  <dim># Remove a spread output</dim>
  <b>stencila outputs remove \"{region}/report.pdf\"</>
"
);

impl Remove {
    pub async fn run(self) -> Result<()> {
        let config_path = config_remove_output(&self.output)?;

        message!(
            "✅ Removed output `{}` from {}",
            self.output,
            config_path.display()
        );

        Ok(())
    }
}

/// Push outputs to Stencila Cloud
#[derive(Debug, Args)]
#[command(after_long_help = PUSH_AFTER_LONG_HELP)]
pub struct Push {
    /// Specific outputs to push (all if empty)
    ///
    /// Supports glob patterns for matching multiple outputs.
    pub outputs: Vec<String>,

    /// Force push (ignore refs filter and re-upload unchanged files)
    #[arg(long, short)]
    pub force: bool,

    /// Dry run - process but don't upload
    #[arg(long)]
    pub dry_run: bool,
}

pub static PUSH_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Push all outputs</dim>
  <b>stencila outputs push</>

  <dim># Push specific outputs</dim>
  <b>stencila outputs push \"report.pdf\"</>

  <dim># Push outputs matching a pattern</dim>
  <b>stencila outputs push \"*.pdf\"</>

  <dim># Dry run to preview what would be uploaded</dim>
  <b>stencila outputs push --dry-run</>

  <dim># Force push, ignoring refs filter</dim>
  <b>stencila outputs push --force</>
"
);

/// Resolved source with its mapped output path
struct ResolvedSource {
    /// Absolute path to source file
    source_path: PathBuf,
    /// Mapped output key (for upload)
    output_path: String,
}

impl Push {
    pub async fn run(self) -> Result<()> {
        let workspace_dir = current_dir()?;

        // Get current git ref (supports branch, tag, or commit SHA)
        let git_ref = get_current_ref(Some(&workspace_dir))
            .ok_or_else(|| eyre!("Unable to determine git ref: not in a git repository"))?;

        // Load outputs from config
        let cfg = config(&workspace_dir)?;
        let outputs = cfg.outputs.unwrap_or_default();

        if outputs.is_empty() {
            message("ℹ️ No outputs configured in stencila.toml");
            return Ok(());
        }

        // Parse and validate output patterns first
        let output_patterns: Vec<Pattern> = self
            .outputs
            .iter()
            .map(|output| {
                Pattern::new(output).map_err(|e| eyre!("Invalid glob pattern `{output}`: {e}"))
            })
            .collect::<Result<Vec<_>>>()?;

        // Filter to requested outputs (or all if none specified)
        let outputs: HashMap<_, _> = if output_patterns.is_empty() {
            outputs
        } else {
            outputs
                .into_iter()
                .filter(|(k, _)| output_patterns.iter().any(|p| p.matches(k)))
                .collect()
        };

        if outputs.is_empty() {
            let patterns = self
                .outputs
                .iter()
                .map(|o| format!("`{o}`"))
                .collect::<Vec<_>>()
                .join(", ");
            message!("ℹ️ No outputs match the specified patterns: {patterns}");
            return Ok(());
        }

        // Get workspace ID (ensures authenticated) - only needed for actual uploads
        let workspace_id = if self.dry_run {
            String::new() // Not used in dry-run mode
        } else {
            let (id, _) = ensure_workspace(&workspace_dir).await?;
            id
        };

        // Create temp directory for processed files
        let temp_dir = TempDir::new()?;

        // Process each output
        let mut uploaded = 0;
        let mut pending = 0;
        let mut skipped = 0;
        let mut processed = 0;

        for (key, target) in &outputs {
            // Extract config from target
            let (output_config, command) = match target {
                OutputTarget::Source(_) => (None, None),
                OutputTarget::Config(c) => (Some(c), c.command),
            };

            // Check refs filter (skip if doesn't match current ref)
            let refs = output_config.and_then(|c| c.refs.as_ref());
            if !self.force && !matches_refs(refs, &git_ref) {
                message!(
                    "⏭️  Skipping {} (refs filter doesn't match {})",
                    key,
                    git_ref.ref_name()
                );
                skipped += 1;
                continue;
            }

            // Resolve source files (handles patterns with glob expansion)
            let sources = resolve_sources(key, target, &workspace_dir)?;

            // Expand spreads (handles {arg} placeholders and grid/zip modes)
            let runs = expand_spreads(key, output_config, &git_ref)?;

            // Process and upload each combination of source × run
            for run in &runs {
                for resolved in &sources {
                    // Compose final output path
                    let final_output_key = if resolved.output_path != *key {
                        // Pattern output: start with pattern-mapped path, then interpolate spreads
                        apply_template(&resolved.output_path, run)?
                    } else {
                        // Non-pattern: apply template to original key
                        apply_template(key, run)?
                    };

                    // Process the output
                    let temp_file = process_output(
                        &resolved.source_path,
                        &final_output_key,
                        command,
                        &run.values,
                        &temp_dir,
                    )
                    .await?;
                    processed += 1;

                    if self.dry_run {
                        message!("📝 Would upload `{final_output_key}`");
                    } else {
                        // Determine content type from the output path
                        let content_type =
                            Format::from_path(Path::new(&final_output_key)).media_type();

                        // Upload to cloud
                        let result = stencila_cloud::outputs::upload_output(
                            &workspace_id,
                            git_ref.ref_type(),
                            git_ref.ref_name(),
                            &final_output_key,
                            &temp_file,
                            &content_type,
                        )
                        .await?;

                        match result {
                            UploadResult::Uploaded => {
                                message!("✅ Uploaded `{final_output_key}`");
                                uploaded += 1;
                            }
                            UploadResult::ApprovalRequired => {
                                message!("⏳ Pending approval `{final_output_key}`");
                                pending += 1;
                            }
                            UploadResult::Skipped => {
                                message!("⏭️  Unchanged `{final_output_key}`");
                                skipped += 1;
                            }
                        }
                    }
                }
            }
        }

        if self.dry_run {
            message!("📋 Dry run complete. {processed} outputs would be processed.",);
        } else {
            message!(
                "✅ Done. {uploaded} uploaded, {pending} uploaded and pending approval, {skipped} unchanged."
            );
            message!("🔗 Outputs available at: https://{workspace_id}.stencila.build");
        }

        Ok(())
    }
}

/// Match refs patterns against current git ref
///
/// Patterns can optionally include ref type prefix:
/// - "main" - matches branch or tag named "main"
/// - "branch:main" - matches only branch named "main"
/// - "tag:v*" - matches only tags starting with "v"
/// - "commit:*" - matches any commit (for CI builds)
fn matches_refs(refs: Option<&Vec<String>>, git_ref: &GitRef) -> bool {
    let Some(patterns) = refs else {
        return true;
    };

    let ref_type = git_ref.ref_type();
    let ref_name = git_ref.ref_name();

    patterns.iter().any(|pattern| {
        if let Some((prefix, name_pattern)) = pattern.split_once(':') {
            // Type-qualified pattern: "branch:main", "tag:v*"
            prefix == ref_type && Pattern::new(name_pattern).is_ok_and(|p| p.matches(ref_name))
        } else {
            // Name-only pattern: matches any ref type
            Pattern::new(pattern).is_ok_and(|p| p.matches(ref_name))
        }
    })
}

/// Maps a source file to its output path based on key template
///
/// For pattern outputs:
/// - Extract relative path from source (relative to pattern base)
/// - Replace `*` in key with path stem (no extension)
/// - Use extension from key template
fn map_source_to_output(
    source: &Path,
    pattern: &str,
    key_template: &str,
    workspace_dir: &Path,
) -> Result<String> {
    // Validate key template has exactly one *
    let star_count = key_template.matches('*').count();
    if star_count != 1 {
        bail!(
            "Pattern output key must contain exactly one '*', found {} in: {}",
            star_count,
            key_template
        );
    }

    // Convert absolute source path to workspace-relative
    let source_rel = source
        .strip_prefix(workspace_dir)
        .map_err(|_| eyre!("Source path is not under workspace: {}", source.display()))?;

    // Find the base directory (part before first wildcard in pattern)
    let pattern_base_str = pattern
        .split(['*', '?'])
        .next()
        .unwrap_or("")
        .trim_end_matches('/')
        .trim_end_matches('\\');
    let pattern_base = Path::new(pattern_base_str);

    // Strip pattern base from source to get the matched portion
    let relative = source_rel.strip_prefix(pattern_base).unwrap_or(source_rel);

    // Extract stem (path without extension)
    // Normalize to forward slashes for cross-platform consistency
    let stem = relative
        .with_extension("")
        .to_string_lossy()
        .trim_start_matches('/')
        .replace('\\', "/");

    // Replace * in key template with stem
    let output_path = key_template.replacen('*', &stem, 1);

    Ok(output_path)
}

/// Resolve sources for an output target
fn resolve_sources(
    key: &str,
    target: &OutputTarget,
    workspace_dir: &Path,
) -> Result<Vec<ResolvedSource>> {
    match target {
        OutputTarget::Source(path) => {
            // Single source file -> direct key mapping
            Ok(vec![ResolvedSource {
                source_path: workspace_dir.join(path.as_str()),
                output_path: key.to_string(),
            }])
        }
        OutputTarget::Config(cfg) => {
            if let Some(pattern) = &cfg.pattern {
                // Glob expand (returns absolute paths)
                let glob_pattern = workspace_dir.join(pattern);
                let matches: Vec<PathBuf> = glob(&glob_pattern.to_string_lossy())?
                    .filter_map(|r| r.ok())
                    .collect();

                // Apply exclude patterns
                let matches = if let Some(excludes) = &cfg.exclude {
                    matches
                        .into_iter()
                        .filter(|abs_path| {
                            // Convert to workspace-relative for exclude matching
                            let rel_path = abs_path
                                .strip_prefix(workspace_dir)
                                .map(|p| p.to_string_lossy().replace('\\', "/"))
                                .unwrap_or_else(|_| abs_path.to_string_lossy().to_string());

                            // Check if any exclude pattern matches
                            !excludes
                                .iter()
                                .any(|ex| Pattern::new(ex).is_ok_and(|p| p.matches(&rel_path)))
                        })
                        .collect()
                } else {
                    matches
                };

                // Map each source to its output path
                matches
                    .into_iter()
                    .map(|source| {
                        let output_path =
                            map_source_to_output(&source, pattern, key, workspace_dir)?;
                        Ok(ResolvedSource {
                            source_path: source,
                            output_path,
                        })
                    })
                    .collect()
            } else {
                // Single source (or source = key if None)
                let source = cfg.source.as_deref().unwrap_or(key);
                Ok(vec![ResolvedSource {
                    source_path: workspace_dir.join(source),
                    output_path: key.to_string(),
                }])
            }
        }
    }
}

/// Expand spreads using the stencila_spread crate
fn expand_spreads(
    key_template: &str,
    config: Option<&OutputConfig>,
    git_ref: &GitRef,
) -> Result<Vec<Run>> {
    // Validate reserved placeholders match git ref type
    validate_reserved_placeholders(key_template, git_ref)?;

    let Some(config) = config else {
        // No config - create a single run with reserved placeholders only
        let mut values = IndexMap::new();
        add_reserved_placeholders(&mut values, key_template, git_ref);
        return Ok(vec![Run::new(1, values)]);
    };

    // Check for conflicts between reserved and user placeholders
    if let Some(arguments) = &config.arguments {
        check_reserved_conflicts(arguments)?;
    }

    let Some(arguments) = &config.arguments else {
        // No spread - create a single run with reserved placeholders only
        let mut values = IndexMap::new();
        add_reserved_placeholders(&mut values, key_template, git_ref);
        return Ok(vec![Run::new(1, values)]);
    };

    // Convert OutputConfig arguments to spread format
    let arg_strings: Vec<(String, String)> = arguments
        .iter()
        .map(|(k, v)| (k.clone(), v.join(",")))
        .collect();
    let arg_refs: Vec<(&str, &str)> = arg_strings
        .iter()
        .map(|(k, v)| (k.as_str(), v.as_str()))
        .collect();

    let mode = match config.spread.unwrap_or_default() {
        SpreadMode::Grid => stencila_spread::SpreadMode::Grid,
        SpreadMode::Zip => stencila_spread::SpreadMode::Zip,
    };

    let spread_config = SpreadConfig::from_arguments(
        mode,
        &arg_refs,
        &[],  // No explicit cases
        1000, // max_runs limit
    )?;

    let mut runs = spread_config.generate_runs()?;

    // Add reserved placeholders to each run's values
    for run in &mut runs {
        add_reserved_placeholders(&mut run.values, key_template, git_ref);
    }

    // Collision detection using apply_template
    let mut seen_keys = HashSet::new();
    for run in &runs {
        let output_key = apply_template(key_template, run)?;
        if !seen_keys.insert(output_key.clone()) {
            bail!("Spread produces duplicate output path: {}", output_key);
        }
    }

    Ok(runs)
}

/// Check that reserved placeholders are valid for current git ref
fn validate_reserved_placeholders(template: &str, git_ref: &GitRef) -> Result<()> {
    if template.contains("{branch}") && !matches!(git_ref, GitRef::Branch(_)) {
        bail!("{{branch}} placeholder used but current ref is not a branch");
    }
    if template.contains("{tag}") && !matches!(git_ref, GitRef::Tag(_)) {
        bail!("{{tag}} placeholder used but current ref is not a tag");
    }
    Ok(())
}

/// Check that user arguments don't conflict with reserved placeholders
fn check_reserved_conflicts(arguments: &HashMap<String, Vec<String>>) -> Result<()> {
    const RESERVED: &[&str] = &["branch", "tag", "i"];
    for name in arguments.keys() {
        if RESERVED.contains(&name.as_str()) {
            bail!(
                "Argument name '{}' conflicts with reserved placeholder {{{}}}",
                name,
                name
            );
        }
    }
    Ok(())
}

/// Add reserved placeholders to run values
fn add_reserved_placeholders(
    values: &mut IndexMap<String, String>,
    template: &str,
    git_ref: &GitRef,
) {
    if template.contains("{branch}")
        && let GitRef::Branch(name) = git_ref
    {
        values.insert("branch".to_string(), name.clone());
    }
    if template.contains("{tag}")
        && let GitRef::Tag(name) = git_ref
    {
        values.insert("tag".to_string(), name.clone());
    }
}

/// Determine output format from file extension
fn format_from_path(path: &str) -> Result<Format> {
    let ext = Path::new(path)
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| eyre!("Output path has no extension: {}", path))?;

    let format = Format::from_name(ext);
    if format == Format::Unknown {
        bail!("Unknown output format extension: .{}", ext);
    }
    Ok(format)
}

/// Determine command based on source/output extensions
fn auto_detect_command(source: &Path, output_path: &str) -> OutputCommand {
    let source_ext = source.extension().and_then(|e| e.to_str());
    let output_ext = Path::new(output_path).extension().and_then(|e| e.to_str());

    match (source_ext, output_ext) {
        (Some(s), Some(o)) if s == o => OutputCommand::None,
        _ => OutputCommand::Render,
    }
}

/// Process a single output and return path to the processed file
async fn process_output(
    source: &Path,
    output_key: &str,
    command: Option<OutputCommand>,
    arguments: &IndexMap<String, String>,
    temp_dir: &TempDir,
) -> Result<PathBuf> {
    // Auto-detect command if not specified
    let command = command.unwrap_or_else(|| auto_detect_command(source, output_key));

    // Determine output format from key extension
    let output_format = format_from_path(output_key)?;

    // Create temp output path preserving full directory structure
    let normalized_key = output_key
        .replace('\\', "/")
        .trim_start_matches('/')
        .to_string();
    let temp_output = temp_dir.path().join(&normalized_key);

    // Ensure parent directories exist
    if let Some(parent) = temp_output.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    match command {
        OutputCommand::Render => {
            // Open document
            let doc = Document::open(source, None).await?;

            // Execute the document with arguments
            let args: Vec<(&str, &str)> = arguments
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();
            doc.call(&args, Default::default()).await?;

            // Export to target format
            let encode_options = EncodeOptions {
                format: Some(output_format),
                ..Default::default()
            };
            doc.export(&temp_output, Some(encode_options)).await?;
        }
        OutputCommand::Convert => {
            // Pure format transformation (no execution)
            let encode_options = EncodeOptions {
                format: Some(output_format),
                ..Default::default()
            };
            stencila_codecs::convert(
                Some(source),
                Some(temp_output.as_path()),
                Some(DecodeOptions::default()),
                Some(encode_options),
            )
            .await?;
        }
        OutputCommand::None => {
            // Copy file as-is
            tokio::fs::copy(source, &temp_output).await?;
        }
    }

    Ok(temp_output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_matches_refs() {
        // No patterns = always match
        assert!(matches_refs(None, &GitRef::Branch("main".to_string())));
        assert!(matches_refs(None, &GitRef::Tag("v1.0".to_string())));

        // Simple name patterns
        let patterns = vec!["main".to_string()];
        assert!(matches_refs(
            Some(&patterns),
            &GitRef::Branch("main".to_string())
        ));
        assert!(!matches_refs(
            Some(&patterns),
            &GitRef::Branch("develop".to_string())
        ));

        // Glob patterns
        let patterns = vec!["v*".to_string()];
        assert!(matches_refs(
            Some(&patterns),
            &GitRef::Tag("v1.0".to_string())
        ));
        assert!(matches_refs(
            Some(&patterns),
            &GitRef::Tag("v2.0.0".to_string())
        ));
        assert!(!matches_refs(
            Some(&patterns),
            &GitRef::Tag("release-1.0".to_string())
        ));

        // Type-qualified patterns
        let patterns = vec!["branch:main".to_string()];
        assert!(matches_refs(
            Some(&patterns),
            &GitRef::Branch("main".to_string())
        ));
        assert!(!matches_refs(
            Some(&patterns),
            &GitRef::Tag("main".to_string())
        ));

        let patterns = vec!["tag:v*".to_string()];
        assert!(matches_refs(
            Some(&patterns),
            &GitRef::Tag("v1.0".to_string())
        ));
        assert!(!matches_refs(
            Some(&patterns),
            &GitRef::Branch("v1.0".to_string())
        ));

        // Commit patterns
        let patterns = vec!["commit:*".to_string()];
        assert!(matches_refs(
            Some(&patterns),
            &GitRef::Commit("abc123".to_string())
        ));
        assert!(!matches_refs(
            Some(&patterns),
            &GitRef::Branch("main".to_string())
        ));
    }

    #[test]
    fn test_validate_reserved_placeholders() {
        // Branch placeholder with branch ref - ok
        assert!(
            validate_reserved_placeholders("report-{branch}.pdf", &GitRef::Branch("main".into()))
                .is_ok()
        );

        // Branch placeholder with tag ref - error
        assert!(
            validate_reserved_placeholders("report-{branch}.pdf", &GitRef::Tag("v1.0".into()))
                .is_err()
        );

        // Tag placeholder with tag ref - ok
        assert!(
            validate_reserved_placeholders("report-{tag}.pdf", &GitRef::Tag("v1.0".into())).is_ok()
        );

        // Tag placeholder with branch ref - error
        assert!(
            validate_reserved_placeholders("report-{tag}.pdf", &GitRef::Branch("main".into()))
                .is_err()
        );

        // No placeholders - always ok
        assert!(
            validate_reserved_placeholders("report.pdf", &GitRef::Commit("abc123".into())).is_ok()
        );
    }

    #[test]
    fn test_check_reserved_conflicts() {
        // No conflicts
        let mut args = HashMap::new();
        args.insert("region".to_string(), vec!["north".to_string()]);
        assert!(check_reserved_conflicts(&args).is_ok());

        // Conflict with branch
        let mut args = HashMap::new();
        args.insert("branch".to_string(), vec!["main".to_string()]);
        assert!(check_reserved_conflicts(&args).is_err());

        // Conflict with tag
        let mut args = HashMap::new();
        args.insert("tag".to_string(), vec!["v1".to_string()]);
        assert!(check_reserved_conflicts(&args).is_err());

        // Conflict with i
        let mut args = HashMap::new();
        args.insert("i".to_string(), vec!["1".to_string()]);
        assert!(check_reserved_conflicts(&args).is_err());
    }
}
