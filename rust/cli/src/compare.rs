//! The `compare` command
//!
//! A terminal-first adapter over the `stencila-node-compare` crate: it decodes two
//! local documents independently, compares them with neutral left/right roles, and
//! either prints a human-readable report, opens a side-by-side view of it in a
//! browser, or writes the unchanged `Comparison` artifact as JSON or YAML.
//!
//! Only argument handling and input/output live here. Comparison itself comes from
//! `stencila-node-compare`, and the text and HTML renderings from
//! `stencila-node-compare-report`, so neither is duplicated for the terminal.

use std::{
    io::Write as _,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use clap::{Parser, ValueEnum};
use eyre::{Context, Result, bail};
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_codecs::{DecodeOptions, LossesResponse};
use stencila_format::Format;
use stencila_node_compare::{CompareOptions, DifferenceFilter, Selector, Side, compare_with_options};
use stencila_node_compare_report::{Snapshot, html_report, text_report};
use stencila_schema::Node;

/// Compare two documents
///
/// Compares the two documents semantically, rather than as text, and reports how
/// their nodes correspond and differ. Neither document is presumed correct: they are
/// simply the left and right snapshots that you selected.
///
/// Exits with 0 when the documents are equal, 1 when they differ, and 2 on error.
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The path of the left document
    left: PathBuf,

    /// The path of the right document
    ///
    /// May be in a different format to the left document.
    right: PathBuf,

    /// The path of the output file
    ///
    /// If not supplied, or if "-", the report is written to `stdout`.
    output: Option<PathBuf>,

    /// The format of the left document
    ///
    /// If not supplied, is inferred from the file extension.
    /// See `stencila formats list` for available formats.
    #[arg(long)]
    left_from: Option<String>,

    /// The format of the right document
    ///
    /// If not supplied, is inferred from the file extension.
    /// See `stencila formats list` for available formats.
    #[arg(long)]
    right_from: Option<String>,

    /// The format of the output
    ///
    /// If not supplied, is inferred from the extension of the output file
    /// (`.txt`, `.html`, `.json`, `.yaml` or `.yml`), defaulting to `text` when
    /// writing to `stdout`.
    #[arg(long, short)]
    to: Option<OutputFormat>,

    /// Open a side-by-side view of the comparison in a browser
    ///
    /// Implies `--to html`. Unless an output path is supplied, the view is written
    /// to a temporary file.
    #[arg(long)]
    view: bool,

    /// Only report counts, not individual differences
    ///
    /// Only applies to `text` and `html` output.
    #[arg(long)]
    summary: bool,

    /// Action when there are losses decoding either input document
    #[arg(long, default_value = "warn")]
    input_losses: InputLosses,

    /// Only report differences matching this selector
    ///
    /// Repeatable. Only useful alongside a broader `--exclude`, because with nothing
    /// excluded every difference is already reported.
    #[arg(long, value_name = "SELECTOR", value_parser = parse_selector)]
    include: Vec<Selector>,

    /// Do not report differences matching this selector
    ///
    /// Repeatable. A selector is a property (`jatsRefType`), a property of one node type
    /// (`Link.id`), a node type (`Link`), or `*` (or `all`) for everything. The most
    /// specific
    /// matching selector wins, whatever order they are given in, so
    /// `--exclude id --include Figure.id` reports figure identifiers and no others.
    #[arg(long, value_name = "SELECTOR", value_parser = parse_selector)]
    exclude: Vec<Selector>,

    /// The maximum number of candidate cells that sequence alignment may use
    ///
    /// Increase this when comparing documents with very large collections of
    /// mutually unrecognizable siblings. Exceeding the budget is an error, rather
    /// than a silently approximated result.
    #[arg(long)]
    alignment_cell_budget: Option<usize>,
}

/// The format of the output of a comparison
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum OutputFormat {
    /// A human-readable report
    Text,
    /// A side-by-side report as a self-contained HTML page
    Html,
    /// The comparison artifact as JSON
    Json,
    /// The comparison artifact as YAML
    Yaml,
}

/// What to do when decoding an input document loses information
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
#[value(rename_all = "lower")]
pub enum InputLosses {
    /// Warn about losses and continue
    Warn,
    /// Say nothing about losses
    Ignore,
    /// Abort without comparing
    Abort,
}

impl From<InputLosses> for LossesResponse {
    fn from(value: InputLosses) -> Self {
        match value {
            InputLosses::Warn => LossesResponse::Warn,
            InputLosses::Ignore => LossesResponse::Ignore,
            InputLosses::Abort => LossesResponse::Abort,
        }
    }
}

/// Whether the two compared documents were equal
///
/// Returned, rather than turned into an exit code here, so that mapping outcomes to
/// process exit statuses stays in one place.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOutcome {
    Equal,
    Different,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># Compare two documents in the terminal</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.smd</>

  <dim># Compare documents in different formats</dim>
  <b>stencila compare</> <g>original.smd</> <g>roundtripped.docx</>

  <dim># Only report how many differences there are</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.smd</> <c>--summary</>

  <dim># Open a side-by-side view of the comparison in a browser</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.smd</> <c>--view</>

  <dim># Write the side-by-side view to a HTML file, without opening it</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.smd</> <g>comparison.html</>

  <dim># Write the comparison artifact as JSON to stdout</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.smd</> <c>--to</> <g>json</>

  <dim># Write the comparison artifact to a file</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.smd</> <g>comparison.yaml</>

  <dim># Ignore differences that are only about identifiers</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.jats.xml</> <c>--exclude</> <g>id</>

  <dim># Ignore identifiers everywhere except on figures</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.jats.xml</> <c>--exclude</> <g>id</> <c>--include</> <g>Figure.id</>

  <dim># Ignore everything about links, and JATS reference types anywhere</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.jats.xml</> <c>--exclude</> <g>Link</> <c>--exclude</> <g>jatsRefType</>

  <dim># Report nothing but heading differences</dim>
  <b>stencila compare</> <g>before.smd</> <g>after.smd</> <c>--exclude</> <g>all</> <c>--include</> <g>Heading</>

  <dim># Override the format of an input document</dim>
  <b>stencila compare</> <g>before.txt</> <g>after.smd</> <c>--left-from</> <g>smd</>
"
);

impl Cli {
    pub async fn run(self) -> Result<CompareOutcome> {
        let Self {
            left,
            right,
            output,
            left_from,
            right_from,
            to,
            view,
            summary,
            input_losses,
            include,
            exclude,
            alignment_cell_budget,
        } = self;

        // Validate everything that can be validated without touching document
        // content, so that an unusable invocation fails before any decoding
        let destination = Destination::resolve(output.as_deref(), to, view)?;
        if summary && !matches!(destination.format, OutputFormat::Text | OutputFormat::Html) {
            bail!("The `--summary` option is only supported for `text` and `html` output");
        }
        check_input(&left, Side::Left)?;
        check_input(&right, Side::Right)?;
        destination.check_not_input(&left, &right)?;

        let left_node = decode(&left, left_from.as_deref(), Side::Left, input_losses).await?;
        let right_node = decode(&right, right_from.as_deref(), Side::Right, input_losses).await?;

        let mut options = CompareOptions::default();
        if let Some(budget) = alignment_cell_budget {
            if budget == 0 {
                bail!("The `--alignment-cell-budget` option must be greater than zero");
            }
            options.alignment_cell_budget = budget;
        }
        options.filter = DifferenceFilter { include, exclude };

        let comparison = compare_with_options(&left_node, &right_node, &options)?;

        // Render fully before writing anything, so that a rendering or serialization
        // failure never leaves a partial output behind
        let left_label = left.display().to_string();
        let right_label = right.display().to_string();
        let left_snapshot = Snapshot {
            node: &left_node,
            label: &left_label,
        };
        let right_snapshot = Snapshot {
            node: &right_node,
            label: &right_label,
        };

        let content = match destination.format {
            OutputFormat::Text => {
                text_report(&comparison, left_snapshot, right_snapshot, summary)?
            }
            OutputFormat::Html => {
                html_report(&comparison, left_snapshot, right_snapshot, summary)?
            }
            OutputFormat::Json => {
                let mut json = serde_json::to_string_pretty(&comparison)
                    .wrap_err("Unable to serialize the comparison as JSON")?;
                json.push('\n');
                json
            }
            OutputFormat::Yaml => {
                let mut yaml = serde_yaml::to_string(&comparison)
                    .wrap_err("Unable to serialize the comparison as YAML")?;
                if !yaml.ends_with('\n') {
                    yaml.push('\n');
                }
                yaml
            }
        };

        destination.write(&content)?;
        destination.open()?;

        Ok(if comparison.is_equal() {
            CompareOutcome::Equal
        } else {
            CompareOutcome::Different
        })
    }
}

/// Parse a filter selector, reporting schema mistakes as argument errors
///
/// Parsed here rather than after decoding, so that a misspelled node type or property
/// fails before either document is read.
fn parse_selector(text: &str) -> Result<Selector, String> {
    text.parse::<Selector>().map_err(|error| error.to_string())
}

/// Check that an input path is an existing file
fn check_input(path: &Path, side: Side) -> Result<()> {
    if !path.exists() {
        bail!(
            "The {side} document does not exist: {path}",
            path = path.display()
        );
    }
    if path.is_dir() {
        bail!(
            "The {side} document is a directory: {path}",
            path = path.display()
        );
    }

    Ok(())
}

/// Decode one side of the comparison
async fn decode(path: &Path, from: Option<&str>, side: Side, losses: InputLosses) -> Result<Node> {
    let format = match from {
        Some(name) => Format::from_name(name),
        None => Format::from_path(path),
    };

    let options = DecodeOptions {
        codec: from.map(String::from),
        format: Some(format),
        // Do not record where the snapshot came from. Decoding otherwise stamps the
        // repository, path and commit of the source onto the document, which would
        // make two copies of the same content differ merely by being two files.
        reproducible: Some(false),
        // Losses are responded to here, rather than by the codec, so that they can
        // be labelled with the side they came from
        losses: LossesResponse::Ignore,
        ..Default::default()
    };

    let (node, .., info) = stencila_codecs::from_path_with_info(path, Some(options))
        .await
        .wrap_err_with(|| {
            format!(
                "Unable to decode the {side} document: {path}",
                path = path.display()
            )
        })?;

    if !info.losses.is_empty() {
        info.losses.respond(
            format!(
                "While decoding the {side} document `{path}`",
                path = path.display()
            ),
            losses.into(),
        )?;
    }

    Ok(node)
}

/// Where, and in what format, to write the comparison
#[derive(Debug)]
struct Destination {
    /// The file to write to, or `None` for `stdout`
    path: Option<PathBuf>,

    /// The format to write
    format: OutputFormat,

    /// Whether to open the written file in a browser
    open: bool,
}

impl Destination {
    /// Resolve the output destination and format from the arguments
    fn resolve(output: Option<&Path>, to: Option<OutputFormat>, view: bool) -> Result<Self> {
        let mut path = match output {
            Some(path) if path != Path::new("-") => Some(path.to_path_buf()),
            _ => None,
        };

        let format = match (to, path.as_deref()) {
            (Some(format), ..) => format,
            (None, None) => {
                if view {
                    OutputFormat::Html
                } else {
                    OutputFormat::Text
                }
            }
            (None, Some(path)) => match path
                .extension()
                .map(|extension| extension.to_string_lossy().to_lowercase())
                .as_deref()
            {
                Some("txt") => OutputFormat::Text,
                Some("html" | "htm") => OutputFormat::Html,
                Some("json") => OutputFormat::Json,
                Some("yaml" | "yml") => OutputFormat::Yaml,
                _ => bail!(
                    "Unable to infer the output format from `{path}`; use `--to` to specify `text`, `html`, `json` or `yaml`",
                    path = path.display()
                ),
            },
        };

        if view {
            if format != OutputFormat::Html {
                bail!("The `--view` option is only supported for `html` output");
            }
            // A browser can only open a file, so a view that was not given an
            // output path gets a temporary one
            if path.is_none() {
                path = Some(temporary_view_path());
            }
        }

        if let Some(path) = path.as_deref()
            && path.is_dir()
        {
            bail!(
                "The output path is a directory: {path}",
                path = path.display()
            );
        }

        Ok(Self {
            path,
            format,
            open: view,
        })
    }

    /// Check that the output would not overwrite either input document
    fn check_not_input(&self, left: &Path, right: &Path) -> Result<()> {
        let Some(path) = self.path.as_deref() else {
            return Ok(());
        };

        for (input, side) in [(left, Side::Left), (right, Side::Right)] {
            if same_file(path, input) {
                bail!(
                    "The output path is the same as the {side} document: {path}",
                    path = path.display()
                );
            }
        }

        Ok(())
    }

    /// Write the rendered content
    ///
    /// File destinations are written to a temporary file in the same directory and
    /// renamed, so that a failed write never truncates an existing file.
    #[allow(clippy::print_stdout)]
    fn write(&self, content: &str) -> Result<()> {
        let Some(path) = self.path.as_deref() else {
            let mut stdout = std::io::stdout().lock();
            stdout.write_all(content.as_bytes())?;
            stdout.flush()?;
            return Ok(());
        };

        let dir = match path.parent() {
            Some(dir) if !dir.as_os_str().is_empty() => dir.to_path_buf(),
            _ => PathBuf::from("."),
        };

        let mut file = tempfile::NamedTempFile::new_in(&dir).wrap_err_with(|| {
            format!(
                "Unable to create a temporary file in `{dir}`",
                dir = dir.display()
            )
        })?;
        file.write_all(content.as_bytes())?;
        file.flush()?;
        file.persist(path).map_err(|error| error.error)?;

        Ok(())
    }

    /// Open the written file in a browser, if the view was asked for
    ///
    /// The path is always reported, so that the view is still reachable when no
    /// browser can be launched.
    fn open(&self) -> Result<()> {
        if !self.open {
            return Ok(());
        }

        let Some(path) = self.path.as_deref() else {
            return Ok(());
        };

        let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
        let url = match Url::from_file_path(&absolute) {
            Ok(url) => url.to_string(),
            Err(..) => absolute.to_string_lossy().to_string(),
        };

        message!("Comparison view at {}", url);
        if let Err(error) = webbrowser::open(&url) {
            tracing::warn!("Unable to open the comparison view in a browser: {error}");
        }

        Ok(())
    }
}

/// A temporary path to write a comparison view to
///
/// Not created here: it is written by the same rename-into-place path as any other
/// output file. Deliberately not deleted afterwards, because the browser may not
/// have loaded it by the time this process exits.
fn temporary_view_path() -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|elapsed| elapsed.as_nanos())
        .unwrap_or_default();

    std::env::temp_dir().join(format!(
        "stencila-comparison-{pid}-{unique}.html",
        pid = std::process::id()
    ))
}

/// Whether two paths refer to the same file
///
/// Falls back to comparing the paths themselves when either cannot be canonicalized,
/// which is the case for an output path that does not exist yet.
fn same_file(first: &Path, second: &Path) -> bool {
    match (first.canonicalize(), second.canonicalize()) {
        (Ok(first), Ok(second)) => first == second,
        _ => first == second,
    }
}


#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use clap::Parser;
    use stencila_node_compare::{Comparison, compare};
    use stencila_node_type::{NodeProperty, NodeType};
    use stencila_schema::{Article, Block, Paragraph, shortcuts::t};

    use crate::Cli as TopLevelCli;

    use super::*;

    /// Parse a `stencila compare` command line into its arguments
    fn parse(args: &[&str]) -> Result<Cli, clap::Error> {
        Cli::try_parse_from(std::iter::once("compare").chain(args.iter().copied()))
    }

    fn article(blocks: Vec<Block>) -> Node {
        Node::Article(Article::new(blocks))
    }

    fn para(text: &str) -> Block {
        Block::Paragraph(Paragraph::new(vec![t(text)]))
    }

    #[test]
    fn parses_arguments_and_options() {
        let cli = parse(&[
            "left.smd",
            "right.docx",
            "out.json",
            "--left-from",
            "smd",
            "--right-from",
            "docx",
            "--to",
            "yaml",
            "--summary",
            "--input-losses",
            "abort",
            "--alignment-cell-budget",
            "42",
            "--exclude",
            "id",
            "--exclude",
            "Link.jatsRefType",
            "--include",
            "Figure.id",
        ])
        .unwrap();

        assert_eq!(cli.left, PathBuf::from("left.smd"));
        assert_eq!(cli.right, PathBuf::from("right.docx"));
        assert_eq!(cli.output, Some(PathBuf::from("out.json")));
        assert_eq!(cli.left_from.as_deref(), Some("smd"));
        assert_eq!(cli.right_from.as_deref(), Some("docx"));
        assert_eq!(cli.to, Some(OutputFormat::Yaml));
        assert!(cli.summary);
        assert!(!cli.view);
        assert_eq!(cli.input_losses, InputLosses::Abort);
        assert_eq!(cli.alignment_cell_budget, Some(42));
        assert_eq!(
            cli.exclude,
            vec![
                Selector::Property(NodeProperty::Id),
                Selector::TypeProperty(NodeType::Link, NodeProperty::JatsRefType),
            ]
        );
        assert_eq!(
            cli.include,
            vec![Selector::TypeProperty(NodeType::Figure, NodeProperty::Id)]
        );
    }

    #[test]
    fn selectors_are_validated_against_the_schema() {
        // Accepted: each form of the grammar
        for selector in ["id", "Link.id", "Link", "*"] {
            assert!(
                parse(&["left.smd", "right.smd", "--exclude", selector]).is_ok(),
                "`{selector}` should be accepted"
            );
        }

        // Rejected before either document is read, rather than matching nothing
        for selector in ["", "Lnk", "jatsReftype", "Link.href", "Link.rowSpan"] {
            let error = parse(&["left.smd", "right.smd", "--exclude", selector])
                .expect_err("Expected an error")
                .to_string();
            assert!(error.contains("--exclude"), "for `{selector}`: {error}");
        }
    }

    #[test]
    fn defaults_are_terminal_first() {
        let cli = parse(&["left.smd", "right.smd"]).unwrap();

        assert_eq!(cli.output, None);
        assert_eq!(cli.to, None);
        assert!(!cli.view);
        assert!(!cli.summary);
        assert_eq!(cli.input_losses, InputLosses::Warn);
        assert_eq!(cli.alignment_cell_budget, None);
        assert!(cli.include.is_empty());
        assert!(cli.exclude.is_empty());
    }

    #[test]
    fn requires_two_inputs() {
        assert!(parse(&[]).is_err());
        assert!(parse(&["left.smd"]).is_err());
    }

    #[test]
    fn is_wired_into_the_top_level_command() {
        let cli = TopLevelCli::try_parse_from(["stencila", "compare", "a.smd", "b.smd"]).unwrap();
        assert!(matches!(cli.command, Some(crate::Command::Compare(..))));
    }

    #[test]
    fn infers_the_output_format_from_the_extension() {
        for (path, expected) in [
            ("report.txt", OutputFormat::Text),
            ("comparison.html", OutputFormat::Html),
            ("comparison.htm", OutputFormat::Html),
            ("comparison.json", OutputFormat::Json),
            ("comparison.yaml", OutputFormat::Yaml),
            ("comparison.yml", OutputFormat::Yaml),
            ("comparison.JSON", OutputFormat::Json),
        ] {
            let destination = Destination::resolve(Some(Path::new(path)), None, false).unwrap();
            assert_eq!(destination.format, expected, "for {path}");
            assert_eq!(destination.path.as_deref(), Some(Path::new(path)));
        }
    }

    #[test]
    fn defaults_to_text_on_stdout() {
        for output in [None, Some(Path::new("-"))] {
            let destination = Destination::resolve(output, None, false).unwrap();
            assert_eq!(destination.format, OutputFormat::Text);
            assert_eq!(destination.path, None);
        }
    }

    #[test]
    fn to_option_overrides_inference() {
        let destination = Destination::resolve(
            Some(Path::new("comparison.json")),
            Some(OutputFormat::Yaml),
            false,
        )
        .unwrap();
        assert_eq!(destination.format, OutputFormat::Yaml);

        let destination = Destination::resolve(None, Some(OutputFormat::Json), false).unwrap();
        assert_eq!(destination.format, OutputFormat::Json);
        assert_eq!(destination.path, None);
    }

    #[test]
    fn view_implies_html_in_a_temporary_file() {
        let destination = Destination::resolve(None, None, true).unwrap();
        assert_eq!(destination.format, OutputFormat::Html);
        assert!(destination.open);

        // Somewhere writable, not the working directory, and not yet created
        let path = destination.path.expect("Expected a path to view");
        assert_eq!(path.parent(), Some(std::env::temp_dir().as_path()));
        assert_eq!(
            path.extension()
                .map(|extension| extension.to_string_lossy()),
            Some("html".into())
        );
        assert!(!path.exists());

        // An explicit output path is viewed instead of a temporary one
        let destination = Destination::resolve(Some(Path::new("comparison.html")), None, true)
            .expect("Expected a destination");
        assert_eq!(
            destination.path.as_deref(),
            Some(Path::new("comparison.html"))
        );
        assert!(destination.open);
    }

    #[test]
    fn view_requires_html_output() {
        for (output, to) in [
            (None, Some(OutputFormat::Json)),
            (None, Some(OutputFormat::Text)),
            (Some(Path::new("comparison.yaml")), None),
        ] {
            let error = Destination::resolve(output, to, true)
                .expect_err("Expected an error")
                .to_string();
            assert!(error.contains("`--view`"), "{error}");
        }
    }

    #[test]
    fn html_is_only_opened_when_viewing() {
        let destination =
            Destination::resolve(Some(Path::new("comparison.html")), None, false).unwrap();
        assert_eq!(destination.format, OutputFormat::Html);
        assert!(!destination.open);
    }

    #[test]
    fn unknown_extensions_are_an_error() {
        assert!(Destination::resolve(Some(Path::new("comparison.toml")), None, false).is_err());
        assert!(Destination::resolve(Some(Path::new("comparison")), None, false).is_err());
    }

    #[test]
    fn output_may_not_be_an_input() {
        let dir = tempfile::tempdir().unwrap();
        let left = dir.path().join("left.smd");
        let right = dir.path().join("right.smd");
        std::fs::write(&left, "One\n").unwrap();
        std::fs::write(&right, "Two\n").unwrap();

        let destination =
            Destination::resolve(Some(&left), Some(OutputFormat::Json), false).unwrap();
        assert!(destination.check_not_input(&left, &right).is_err());

        let destination =
            Destination::resolve(Some(&right), Some(OutputFormat::Json), false).unwrap();
        assert!(destination.check_not_input(&left, &right).is_err());

        let output = dir.path().join("comparison.json");
        let destination = Destination::resolve(Some(&output), None, false).unwrap();
        assert!(destination.check_not_input(&left, &right).is_ok());

        // Stdout is never an input
        let destination = Destination::resolve(None, None, false).unwrap();
        assert!(destination.check_not_input(&left, &right).is_ok());
    }

    #[tokio::test]
    async fn summary_conflicts_with_machine_formats() {
        for format in ["json", "yaml"] {
            let cli = parse(&["left.smd", "right.smd", "--summary", "--to", format]).unwrap();
            let error = cli.run().await.unwrap_err().to_string();
            assert!(error.contains("`--summary`"), "{error}");
        }
    }

    #[tokio::test]
    async fn missing_inputs_are_an_error() {
        let dir = tempfile::tempdir().unwrap();
        let left = dir.path().join("left.smd");
        std::fs::write(&left, "One\n").unwrap();
        let missing = dir.path().join("missing.smd");

        let cli = parse(&[
            &left.to_string_lossy(),
            &missing.to_string_lossy(),
            "--to",
            "json",
        ])
        .unwrap();
        let error = cli.run().await.unwrap_err().to_string();
        assert!(error.contains("right document does not exist"), "{error}");

        let cli = parse(&[
            &missing.to_string_lossy(),
            &left.to_string_lossy(),
            "--to",
            "json",
        ])
        .unwrap();
        let error = cli.run().await.unwrap_err().to_string();
        assert!(error.contains("left document does not exist"), "{error}");

        // A directory is not a document
        let cli = parse(&[
            &dir.path().to_string_lossy(),
            &left.to_string_lossy(),
            "--to",
            "json",
        ])
        .unwrap();
        let error = cli.run().await.unwrap_err().to_string();
        assert!(error.contains("left document is a directory"), "{error}");
    }

    #[test]
    fn machine_output_round_trips_without_cli_metadata() {
        let left = article(vec![para("One")]);
        let right = article(vec![para("Two")]);
        let comparison = compare(&left, &right).unwrap();

        let json = serde_json::to_string_pretty(&comparison).unwrap();
        let from_json: Comparison = serde_json::from_str(&json).unwrap();
        assert_eq!(from_json, comparison);

        let yaml = serde_yaml::to_string(&comparison).unwrap();
        let from_yaml: Comparison = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(from_yaml, comparison);

        // No CLI envelope
        let value: serde_json::Value = serde_json::from_str(&json).unwrap();
        let keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        assert_eq!(keys, ["formatVersion", "alignment", "differences"]);
    }
}
