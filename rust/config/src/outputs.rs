use std::{collections::HashMap, fs, path::PathBuf};

use clap::ValueEnum;
use eyre::{Result, bail, eyre};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::Display;
use toml_edit::{DocumentMut, InlineTable, Item, Table, value};

use crate::{
    ConfigRelativePath, RESERVED_PLACEHOLDERS, SpreadMode, find_config_file, validate_placeholders,
};

/// Target for an output - either a simple source path or a full configuration
///
/// Outputs define files to be rendered/converted and uploaded to Stencila Cloud
/// workspace outputs. The key is the output path template, e.g.
///
/// ```toml
/// # Output mappings for rendered and static files
/// [outputs]
/// # Simple: source path (rendered if extension differs)
/// "report.pdf" = "report.md"
///
/// # Full config with options
/// "report.docx" = { source = "report.md", command = "render" }
///
/// # Static file (omit source = use key as source)
/// "data/results.csv" = {}
///
/// # Pattern for multiple files
/// "exports/*.csv" = { pattern = "exports/*.csv" }
///
/// # Spread with parameters
/// "{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }
/// ```
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum OutputTarget {
    /// Simple source path (rendered if extension differs from key)
    ///
    /// ```toml
    /// [outputs]
    /// "report.pdf" = "report.md"
    /// ```
    Source(ConfigRelativePath),

    /// Full configuration object
    ///
    /// ```toml
    /// [outputs]
    /// "report.pdf" = { source = "report.md", command = "render" }
    /// "data.csv" = {} # (static, source = output path)
    /// ```
    Config(OutputConfig),
}

impl OutputTarget {
    /// Validate the output target configuration
    ///
    /// Ensures that:
    /// - `arguments` and `spread` are only allowed with `command = render`
    /// - If `arguments` is present, it must be non-empty
    /// - `source` and `pattern` cannot both be set
    /// - If `refs` is present, it must be non-empty
    /// - Pattern keys must include `*.ext` suffix
    pub fn validate(&self, key: &str) -> Result<()> {
        match self {
            OutputTarget::Source(_) => Ok(()),
            OutputTarget::Config(config) => config.validate(key),
        }
    }

    /// Get the source path if this is a simple source target
    pub fn source(&self) -> Option<&ConfigRelativePath> {
        match self {
            OutputTarget::Source(path) => Some(path),
            OutputTarget::Config(_) => None,
        }
    }

    /// Get the configuration if this is a config target
    pub fn config(&self) -> Option<&OutputConfig> {
        match self {
            OutputTarget::Config(config) => Some(config),
            OutputTarget::Source(_) => None,
        }
    }

    /// Check if this is a spread output (has arguments)
    pub fn is_spread(&self) -> bool {
        match self {
            OutputTarget::Source(_) => false,
            OutputTarget::Config(config) => config.arguments.is_some(),
        }
    }

    /// Check if this is a pattern output
    pub fn is_pattern(&self) -> bool {
        match self {
            OutputTarget::Source(_) => false,
            OutputTarget::Config(config) => config.pattern.is_some(),
        }
    }
}

/// Full output configuration
///
/// Provides detailed control over how an output is processed and uploaded, e.g.
///
/// ```toml
/// # Configure render, spread, and pattern outputs
/// [outputs]
/// "report.pdf" = { source = "report.md", command = "render" }
/// "{region}/report.pdf" = { source = "report.md", arguments = { region = ["north", "south"] } }
/// "exports/*.csv" = { pattern = "exports/*.csv", exclude = ["temp-*.csv"] }
/// ```
#[skip_serializing_none]
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, JsonSchema)]
pub struct OutputConfig {
    /// Source file path (for single-file outputs)
    ///
    /// Path relative to the config file. If not specified and `pattern` is not set,
    /// the output key is used as the source path.
    pub source: Option<String>,

    /// Glob pattern for matching multiple source files
    ///
    /// Mutually exclusive with `source`. The output key must include `*.ext`
    /// to specify the output format (e.g., `"reports/*.pdf"`).
    pub pattern: Option<String>,

    /// Processing command
    ///
    /// - `render`: Execute code, apply parameters, convert to output format (default if extensions differ)
    /// - `convert`: Format transformation only, no code execution
    /// - `none`: Copy file as-is (default if extensions are the same)
    pub command: Option<OutputCommand>,

    /// Spread mode for parameter variants
    ///
    /// Only valid with `command = render`.
    /// - `grid`: Cartesian product of all arguments (default)
    /// - `zip`: Positional pairing of values
    pub spread: Option<SpreadMode>,

    /// Parameter values for spread variants
    ///
    /// Only valid with `command = render`. Keys are parameter names,
    /// values are arrays of possible values.
    pub arguments: Option<HashMap<String, Vec<String>>>,

    /// Git ref patterns to filter when this output is processed and uploaded
    ///
    /// If set, the output is only processed when the current git ref matches
    /// one of these patterns. Supports glob matching.
    ///
    /// Examples: `["main"]`, `["release/*"]`, `["v*"]`
    pub refs: Option<Vec<String>>,

    /// Glob patterns to exclude from pattern matches
    ///
    /// Paths are relative to the repository root.
    /// Only applies when `pattern` is set.
    pub exclude: Option<Vec<String>>,
}

impl OutputConfig {
    /// Validate the output configuration
    pub fn validate(&self, key: &str) -> Result<()> {
        // Source and pattern are mutually exclusive
        if self.source.is_some() && self.pattern.is_some() {
            bail!("Output '{key}' cannot have both `source` and `pattern`");
        }

        // If pattern is set, key must include *.ext suffix
        if self.pattern.is_some() && !key.contains("*.") {
            bail!(
                "Output '{key}' with `pattern` must include `*.ext` suffix to specify output format (e.g., 'reports/*.pdf')"
            );
        }

        // Arguments and spread are only allowed with command = render
        let command = self.command.unwrap_or_default();
        if command != OutputCommand::Render {
            if self.arguments.is_some() {
                bail!("Output '{key}' has `arguments` but `command` is not `render`");
            }
            if self.spread.is_some() {
                bail!("Output '{key}' has `spread` but `command` is not `render`");
            }
        }

        // If arguments is present, it must be non-empty
        if let Some(args) = &self.arguments
            && args.is_empty()
        {
            bail!("Output '{key}' has empty `arguments`");
        }

        // If refs is present, it must be non-empty
        if let Some(refs) = &self.refs
            && refs.is_empty()
        {
            bail!("Output '{key}' has empty `refs`");
        }

        // Exclude only applies with pattern
        if self.exclude.is_some() && self.pattern.is_none() {
            bail!("Output '{key}' has `exclude` but no `pattern`");
        }

        // Validate that all placeholders have corresponding arguments
        // (except reserved placeholders like {tag} and {branch})
        validate_placeholders(key, self.arguments.as_ref(), "Output")?;

        Ok(())
    }
}

/// Processing command for outputs
///
/// Determines how source files are processed before upload:
/// - `render`: Execute code, apply parameters, then convert to output format
/// - `convert`: Pure format transformation (no code execution)
/// - `none`: Copy file as-is (static upload)
#[derive(
    Debug,
    Clone,
    Copy,
    Default,
    PartialEq,
    Eq,
    Deserialize,
    Serialize,
    JsonSchema,
    Display,
    ValueEnum,
)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum OutputCommand {
    /// Execute code and convert to output format (default for different extensions)
    #[default]
    Render,

    /// Format transformation only, no code execution
    Convert,

    /// Copy file as-is (default for same extensions)
    None,
}

/// Validate output configuration options before writing to config
///
/// Checks the same constraints as OutputConfig::validate to prevent
/// writing invalid configurations that would fail at load time.
#[allow(clippy::too_many_arguments)]
fn validate_output_options(
    key: &str,
    source: Option<&str>,
    command: Option<OutputCommand>,
    refs: Option<&[String]>,
    pattern: Option<&str>,
    exclude: Option<&[String]>,
    spread: Option<SpreadMode>,
    arguments: Option<&HashMap<String, Vec<String>>>,
) -> Result<()> {
    // Source and pattern are mutually exclusive
    if source.is_some() && pattern.is_some() {
        bail!("Cannot specify both --source and --pattern for output `{key}`",);
    }

    // If pattern is set, key must include *.ext suffix
    if pattern.is_some() && !key.contains("*.") {
        bail!("Output `{key}` with --pattern must include `*.ext` suffix (e.g., 'reports/*.pdf')",);
    }

    // Exclude only applies with pattern
    if exclude.is_some() && pattern.is_none() {
        bail!("Output `{key}` has --exclude but no --pattern");
    }

    // If refs is present, it must be non-empty
    if let Some(refs_list) = refs
        && refs_list.is_empty()
    {
        bail!("Output `{key}` has empty --refs");
    }

    // Extract placeholders from key (e.g., "{region}" -> "region")
    let placeholders: Vec<&str> = key
        .split('{')
        .skip(1)
        .filter_map(|s| s.split('}').next())
        .filter(|p| !RESERVED_PLACEHOLDERS.contains(p))
        .collect();

    // If arguments provided, key must have non-reserved placeholders
    if let Some(args) = arguments {
        if !args.is_empty() && placeholders.is_empty() {
            bail!(
                "Output `{key}` has --arguments but no placeholders. \
                 Use placeholders like {{region}} for spread outputs."
            );
        }

        // Check each placeholder has a corresponding argument
        for placeholder in &placeholders {
            if !args.contains_key(*placeholder) {
                bail!("Output `{key}` has placeholder {{{placeholder}}} but no matching argument");
            }
        }
    } else if !placeholders.is_empty() {
        // Has non-reserved placeholders but no arguments
        bail!(
            "Output `{key}` has placeholder(s) but no --arguments provided. \
             Either remove placeholders or add --arguments."
        );
    }

    // Spread mode only makes sense with arguments
    if spread.is_some() && arguments.is_none() {
        bail!("Output `{key}` has --spread but no --arguments");
    }

    // Arguments and spread are only allowed with command = render (or default)
    if let Some(cmd) = command
        && cmd != OutputCommand::Render
    {
        if arguments.is_some() {
            bail!("Output `{key}` has --arguments but --command is not `render`");
        }
        if spread.is_some() {
            bail!("Output `{key}` has --spread but --command is not `render`");
        }
    }

    Ok(())
}

/// Add an output to the [outputs] section of stencila.toml
///
/// This function:
/// 1. Validates the output configuration options
/// 2. Finds the nearest stencila.toml (or creates one if none exists)
/// 3. Adds or updates the output entry for the given key
/// 4. Supports both simple source paths and full config objects
///
/// Returns the path to the modified config file.
#[allow(clippy::too_many_arguments)]
pub fn config_add_output(
    key: &str,
    source: Option<&str>,
    command: Option<OutputCommand>,
    refs: Option<&[String]>,
    pattern: Option<&str>,
    exclude: Option<&[String]>,
    spread: Option<SpreadMode>,
    arguments: Option<&HashMap<String, Vec<String>>>,
) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    // Validate options before writing config
    validate_output_options(
        key, source, command, refs, pattern, exclude, spread, arguments,
    )?;

    let cwd = std::env::current_dir()?;
    let config_path =
        find_config_file(&cwd, CONFIG_FILENAME).unwrap_or_else(|| cwd.join(CONFIG_FILENAME));

    // Canonicalize config_path if it exists
    let config_path = if config_path.exists() {
        config_path.canonicalize().unwrap_or(config_path)
    } else {
        config_path
            .parent()
            .and_then(|p| p.canonicalize().ok())
            .map(|p| p.join(CONFIG_FILENAME))
            .unwrap_or(config_path)
    };

    // Load existing config or create empty
    let contents = if config_path.exists() {
        fs::read_to_string(&config_path)?
    } else {
        String::new()
    };

    let mut doc = contents
        .parse::<DocumentMut>()
        .unwrap_or_else(|_| DocumentMut::new());

    // Ensure [outputs] table exists
    if doc.get("outputs").is_none() {
        doc["outputs"] = Item::Table(Table::new());
    }

    let outputs_table = doc
        .get_mut("outputs")
        .and_then(|v| v.as_table_mut())
        .ok_or_else(|| eyre!("outputs field is not a table"))?;

    // Determine if we need a simple string or full config
    let needs_config = command.is_some()
        || refs.is_some()
        || pattern.is_some()
        || exclude.is_some()
        || spread.is_some()
        || arguments.is_some()
        || source.is_none();

    if needs_config {
        // Build inline table for full config
        let mut config_table = InlineTable::new();

        if let Some(src) = source {
            config_table.insert("source", src.into());
        }

        if let Some(pat) = pattern {
            config_table.insert("pattern", pat.into());
        }

        if let Some(cmd) = command {
            config_table.insert("command", cmd.to_string().into());
        }

        if let Some(spread_mode) = spread {
            let spread_str = match spread_mode {
                SpreadMode::Grid => "grid",
                SpreadMode::Zip => "zip",
            };
            config_table.insert("spread", spread_str.into());
        }

        if let Some(args) = arguments {
            let mut args_table = InlineTable::new();
            for (key, values) in args {
                let mut arr = toml_edit::Array::new();
                for v in values {
                    arr.push(v.as_str());
                }
                args_table.insert(key.as_str(), toml_edit::Value::Array(arr));
            }
            config_table.insert("arguments", toml_edit::Value::InlineTable(args_table));
        }

        if let Some(refs_list) = refs {
            let mut arr = toml_edit::Array::new();
            for r in refs_list {
                arr.push(r.as_str());
            }
            config_table.insert("refs", toml_edit::Value::Array(arr));
        }

        if let Some(exclude_list) = exclude {
            let mut arr = toml_edit::Array::new();
            for e in exclude_list {
                arr.push(e.as_str());
            }
            config_table.insert("exclude", toml_edit::Value::Array(arr));
        }

        outputs_table[key] = value(config_table);
    } else if let Some(src) = source {
        // Simple string value
        outputs_table[key] = value(src);
    }

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}

/// Remove an output from the [outputs] section of stencila.toml
///
/// This function:
/// 1. Finds the nearest stencila.toml
/// 2. Removes the output entry for the given key
///
/// Returns the path to the modified config file.
pub fn config_remove_output(key: &str) -> Result<PathBuf> {
    use crate::CONFIG_FILENAME;

    let cwd = std::env::current_dir()?;
    let config_path = find_config_file(&cwd, CONFIG_FILENAME)
        .ok_or_else(|| eyre!("No `{CONFIG_FILENAME}` found"))?;

    // Load existing config
    let contents = fs::read_to_string(&config_path)?;
    let mut doc = contents.parse::<DocumentMut>()?;

    // Get the outputs table
    let outputs_table = doc
        .get_mut("outputs")
        .ok_or_else(|| eyre!("No [outputs] section in `{CONFIG_FILENAME}`"))?
        .as_table_mut()
        .ok_or_else(|| eyre!("outputs field is not a table"))?;

    // Remove the key
    outputs_table
        .remove(key)
        .ok_or_else(|| eyre!("Output `{key}` not found"))?;

    // Write back to file
    fs::write(&config_path, doc.to_string())?;

    Ok(config_path)
}
