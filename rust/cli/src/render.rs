use std::{path::PathBuf, process::exit};

use clap::Parser;
use eyre::{Result, bail, eyre};

use stencila_ask::{Answer, AskLevel, AskOptions, ask_with};
use stencila_cli_utils::{Code, ToStdout, color_print::cstr, message};
use stencila_document::Document;
use stencila_format::Format;
use stencila_node_execute::ExecuteOptions;
use stencila_spread::{
    SpreadConfig, SpreadMode, apply_template, auto_append_placeholders_for_spread,
    infer_spread_mode,
};

use crate::{
    open,
    options::{DecodeOptions, EncodeOptions, StripOptions},
};

/// Render a document
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path of the document to render
    ///
    /// If not supplied, or if "-", the input content is read from `stdin`
    /// and assumed to be Markdown (but can be specified with the `--from` option).
    /// Note that the Markdown parser should handle alternative flavors so
    /// it may not be necessary to use the `--from` option for MyST, Quarto or
    /// Stencila Markdown.
    input: Option<PathBuf>,

    /// The paths of desired output files
    ///
    /// If an input was supplied, but no outputs, and the `--to` format option
    /// is not used, the document will be rendered in a browser window.
    /// If no outputs are supplied and the `--to` option is used the document
    /// will be rendered to `stdout` in that format.
    outputs: Vec<PathBuf>,

    /// Enable multi-variant (spread) execution mode
    ///
    /// When enabled, parameters with comma-separated values are expanded
    /// and the document is rendered multiple times.
    #[arg(long, value_name = "MODE", num_args = 0..=1, default_missing_value = "grid")]
    spread: Option<SpreadMode>,

    /// Maximum number of runs allowed in spread mode (default: 100)
    #[arg(long, default_value = "100")]
    spread_max: usize,

    /// Explicit parameter sets for cases mode
    ///
    /// Each --case takes a quoted string with space-separated key=value pairs.
    /// Only used with --spread=cases.
    #[arg(long, value_name = "PARAMS", action = clap::ArgAction::Append)]
    case: Vec<String>,

    /// Do not store the document after executing it
    #[arg(long)]
    no_store: bool,

    #[clap(flatten)]
    execute_options: ExecuteOptions,

    #[command(flatten)]
    decode_options: DecodeOptions,

    #[command(flatten)]
    encode_options: EncodeOptions,

    #[command(flatten)]
    strip_options: StripOptions,

    /// Arguments to pass to the document
    ///
    /// The name of each argument is matched against the document's parameters.
    /// If a match is found, then the argument value is coerced to the expected
    /// type of the parameter. If no corresponding parameter is found, then the
    /// argument is parsed as JSON and set as a variable in the document's default
    /// kernel (usually the first programming language used in the document).
    #[arg(last = true, allow_hyphen_values = true)]
    arguments: Vec<String>,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Render a document and preview in browser</dim>
  <b>stencila render</b> <g>document.smd</g>

  <dim># Render to a specific output format</dim>
  <b>stencila render</b> <g>report.md</g> <g>report.docx</g>

  <dim># Render to multiple formats</dim>
  <b>stencila render</b> <g>analysis.md</g> <g>output.html</g> <g>output.pdf</g>

  <dim># Render from stdin to stdout</dim>
  <b>echo</b> <y>\"# Hello\"</y> | <b>stencila render</b> <c>--to</c> <g>html</g>

  <dim># Render with document parameters</dim>
  <b>stencila render</b> <g>template.md</g> <g>output.html</g> -- <c>--name</c>=<y>\"John\"</y> <c>--year</c>=<g>2024</g>

  <dim># Render ignoring execution errors</dim>
  <b>stencila render</b> <g>notebook.md</g> <g>report.pdf</g> <c>--ignore-errors</c>

  <dim># Render without updating the document store</dim>
  <b>stencila render</b> <g>temp.md</g> <g>output.html</g> <c>--no-store</c>

  <dim># Spread render with multiple parameter combinations (grid)</dim>
  <b>stencila render</b> <g>report.md</g> <g>'report-{region}-{species}.pdf'</g> -- <c>region</c>=<g>north,south</g> <c>species</c>=<g>ABC,DEF</g>

  <dim># Spread render with positional pairing (zip) and output to nested folders</dim>
  <b>stencila render</b> <g>report.md</g> <g>'{region}/{species}/report.pdf'</g> <c>--spread=zip</c> -- <c>region</c>=<g>north,south</g> <c>species</c>=<g>ABC,DEF</g>

  <dim># Spread render with explicit cases</dim>
  <b>stencila render</b> <g>report.md</g> <g>'report-{i}.pdf'</g> <c>--spread=cases</c> <c>--case</c>=<y>\"region=north species=ABC\"</y> <c>--case</c>=<y>\"region=south species=DEF\"</y>
"
);

impl Cli {
    /// Parse document arguments into name/value pairs
    fn arguments(&self) -> Result<Vec<(&str, &str)>> {
        let mut parsed = Vec::new();
        let mut index = 0;
        while index < self.arguments.len() {
            let current = &self.arguments[index];

            if current.starts_with("--") {
                // Handle --name=value format
                if let Some((name, value)) = current.split_once('=') {
                    let name = name
                        .strip_prefix("--")
                        .ok_or_else(|| eyre!("Invalid argument format: '{current}'"))?;

                    if name.is_empty() || value.is_empty() {
                        bail!(
                            "Invalid argument format: '{current}'. Name and value cannot be empty"
                        );
                    }

                    parsed.push((name, value));
                    index += 1;
                }
                // Handle --name value format
                else {
                    let name = current
                        .strip_prefix("--")
                        .ok_or_else(|| eyre!("Invalid argument format: '{current}'"))?;

                    if name.is_empty() {
                        bail!("Invalid argument format: '{current}'. Name cannot be empty");
                    }

                    // Check if there's a next argument for the value
                    if index + 1 >= self.arguments.len() {
                        bail!("Parameter '{current}' requires a value");
                    }

                    let value = &self.arguments[index + 1];

                    // Make sure the value doesn't look like another argument
                    if value.starts_with("--") {
                        bail!(
                            "Parameter '{current}' requires a value, but found another argument '{value}'"
                        );
                    }

                    parsed.push((name, value));
                    index += 2;
                }
            }
            // Handle name=value format (no dashes)
            else if current.contains('=') {
                if let Some((name, value)) = current.split_once('=') {
                    let name = name.trim();
                    let value = value.trim();

                    if name.is_empty() || value.is_empty() {
                        bail!(
                            "Invalid argument format: '{current}'. Name and value cannot be empty"
                        );
                    }

                    parsed.push((name, value));
                    index += 1;
                } else {
                    bail!("Invalid argument format: '{current}'");
                }
            }
            // Reject bare values
            else {
                bail!(
                    "Invalid argument '{current}'. Use 'name=value', '--name=value', or '--name value' format"
                );
            }
        }

        Ok(parsed)
    }

    /// Handle execution errors with optional user interaction
    ///
    /// Returns `true` if rendering should continue, `false` if it should stop.
    async fn handle_execution_errors(&self, errors: usize, context: &str) -> Result<bool> {
        if errors == 0 {
            return Ok(true);
        }

        if self.execute_options.ignore_errors {
            message!("▶️ Ignoring {errors} execution errors");
            return Ok(true);
        }

        // Interactive prompt
        if ask_with(
            &format!("Errors while executing `{context}`. Continue rendering?"),
            AskOptions {
                level: AskLevel::Warning,
                default: Some(Answer::Yes),
                ..Default::default()
            },
        )
        .await?
        .is_yes()
        {
            message("💡 Tip: use `--ignore-errors` to continue without prompts");
            Ok(true)
        } else {
            message("🛑 Stopping due to execution errors");
            Ok(false)
        }
    }

    #[allow(clippy::print_stderr)]
    pub async fn run(self) -> Result<()> {
        let input = &self.input;
        let outputs = &self.outputs;
        let arguments = self.arguments()?;

        let input_path = input.clone().unwrap_or_else(|| PathBuf::from("-"));
        let input_is_stdin = input_path == PathBuf::from("-");
        let input_display = if input_is_stdin {
            "stdin".to_string()
        } else {
            input_path.to_string_lossy().to_string()
        };

        // Open document
        let doc = if input_is_stdin {
            let mut decode_options = self.decode_options.build(None, StripOptions::default());
            if decode_options.format.is_none() {
                decode_options.format = Some(Format::Markdown)
            }

            let root = stencila_codecs::from_stdin(Some(decode_options.clone())).await?;
            Document::from(root, None, Some(decode_options)).await?
        } else {
            let decode_options = self
                .decode_options
                .build(input.as_deref(), StripOptions::default());
            Document::open(&input_path, Some(decode_options)).await?
        };

        // Compile document (only needs to be done once)
        doc.compile().await?;

        // Infer spread mode if not explicitly set
        let mode = self.spread.or_else(|| {
            // If --case args provided, default to cases mode
            if !self.case.is_empty() {
                return Some(SpreadMode::Cases);
            }

            // Check output template for placeholders with multi-valued args
            if outputs.len() == 1
                && let Some(mode) = infer_spread_mode(&outputs[0].to_string_lossy(), &arguments)
            {
                message!("ℹ️ Auto-detected spread mode `{mode}` from output path template");
                return Some(mode);
            }
            None
        });

        if let Some(mode) = mode {
            // Validate: --case is only valid with --spread=cases
            if !self.case.is_empty() && mode != SpreadMode::Cases {
                bail!("`--case` is only valid with `--spread=cases`, not `--spread={mode}`");
            }

            if self.execute_options.dry_run {
                message("⚠️ Performing dry-run, no files will be actually rendered");
            }

            // Build spread config
            let config =
                SpreadConfig::from_arguments(mode, &arguments, &self.case, self.spread_max)?;
            let run_count = config.validate()?;
            let runs = config.generate_runs()?;

            // Spread mode requires exactly one output
            let output_template = if outputs.is_empty() {
                bail!("Spread mode requires an output path template");
            } else if outputs.len() > 1 {
                bail!(
                    "Spread mode only supports one output path template (got {})",
                    outputs.len()
                );
            } else {
                auto_append_placeholders_for_spread(
                    &outputs[0],
                    mode,
                    &config.params,
                    &config.cases,
                )
            };

            message!("📊 Spread rendering {input_display} ({mode} mode, {run_count} runs)");

            // Emit spread warnings
            for warning in config.check_warnings(run_count, &output_template, &self.arguments) {
                message!("⚠️ {}", warning.message());
            }

            // Execute each run
            for run in &runs {
                let output_path_str = apply_template(&output_template.to_string_lossy(), run)?;
                let output_path = PathBuf::from(&output_path_str);

                message!(
                    "📃 Rendering {}/{}: {} → `{}`",
                    run.index,
                    run_count,
                    run.to_terminal(),
                    output_path.display()
                );

                // Dry run: continue without rendering
                if self.execute_options.dry_run {
                    continue;
                }

                // Build arguments from run params
                let run_arguments: Vec<(&str, &str)> = run
                    .values
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_str()))
                    .collect();

                // Call document with arguments for this run
                doc.call(&run_arguments, self.execute_options.clone())
                    .await?;
                let (errors, ..) = doc.diagnostics_print().await?;

                // Error handling
                if !self
                    .handle_execution_errors(errors, &format!("run {}", run.index))
                    .await?
                {
                    exit(3);
                }

                // Export document to output path
                let completed = doc
                    .export(
                        &output_path,
                        Some(stencila_codecs::EncodeOptions {
                            render: Some(true),
                            ..self.encode_options.build(
                                input.as_deref(),
                                Some(&output_path),
                                Format::Markdown,
                                self.strip_options.clone(),
                            )
                        }),
                    )
                    .await?;

                if !completed {
                    message("⏭️ Skipped (no changes)");
                }
            }

            message!("✅ Spread render complete: {run_count} runs finished successfully");
        } else {
            // Dry-run: just print what would be rendered and exit
            if self.execute_options.dry_run {
                if outputs.is_empty() {
                    if let Some(format) = self
                        .encode_options
                        .to
                        .as_ref()
                        .map(|format| Format::from_name(format))
                        .or_else(|| input_is_stdin.then_some(Format::Markdown))
                    {
                        message!("📋 Would render: {input_display} → stdout ({format})");
                    } else {
                        message!("📋 Would render: {input_display} → browser");
                    }
                } else {
                    for output in outputs {
                        message!("📋 Would render: {input_display} → {}", output.display());
                    }
                }
                message("✅ Preview complete (no files rendered)");
                return Ok(());
            }

            // Call document with arguments
            doc.call(&arguments, self.execute_options.clone()).await?;
            let (errors, ..) = doc.diagnostics_print().await?;

            // Cache the document
            if !self.no_store && !input_is_stdin {
                doc.store().await?;
            }

            // Error handling
            if !self.handle_execution_errors(errors, &input_display).await? {
                exit(1);
            }

            // If no outputs, print to console or open in browser
            if outputs.is_empty() {
                if let Some(format) = self
                    .encode_options
                    .to
                    .as_ref()
                    .map(|format| Format::from_name(format))
                    .or_else(|| input_is_stdin.then_some(Format::Markdown))
                {
                    // Print to console
                    let content = doc
                        .dump(
                            format.clone(),
                            Some(stencila_codecs::EncodeOptions {
                                render: Some(true),
                                ..self.encode_options.build(
                                    input.as_deref(),
                                    None,
                                    Format::Markdown,
                                    self.strip_options,
                                )
                            }),
                        )
                        .await?;
                    Code::new(format, &content).to_stdout();
                } else if let Some(input) = input {
                    // Render in browser
                    // TODO: opening like this is temporary because (a) `open::Cli` might also open
                    // remotes, (b) needs to re-open doc from file, (b) is not aware of arguments passed.
                    open::Cli::new(input.to_path_buf()).run().await?;
                }

                return Ok(());
            }

            // Export document to output paths
            for output in outputs {
                let completed = doc
                    .export(
                        output,
                        Some(stencila_codecs::EncodeOptions {
                            render: Some(true),
                            ..self.encode_options.build(
                                input.as_deref(),
                                Some(output),
                                Format::Markdown,
                                self.strip_options.clone(),
                            )
                        }),
                    )
                    .await?;

                if completed {
                    eprintln!(
                        "📑 Successfully rendered `{input_display}` to `{}`",
                        output.display()
                    )
                } else {
                    eprintln!("⏭️  Skipped rendering `{input_display}`")
                }
            }
        }

        Ok(())
    }
}
