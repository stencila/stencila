//! The `compare` command
//!
//! A terminal-first adapter over the `stencila-node-compare` crate: it decodes two
//! local documents independently, compares them with neutral left/right roles, and
//! either prints a human-readable report, opens a side-by-side view of it in a
//! browser, or writes the unchanged `Comparison` artifact as JSON or YAML.
//!
//! Only presentation and input/output handling live here. No projection, matching or
//! difference logic is duplicated from the comparison crate.

use std::{
    fmt::Write as _,
    io::Write as _,
    path::{Path, PathBuf},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use clap::{Parser, ValueEnum};
use eyre::{Context, Result, bail};
use similar::{Algorithm, ChangeTag, TextDiff};
use url::Url;

use stencila_cli_utils::{color_print::cstr, message};
use stencila_codecs::{DecodeOptions, LossesResponse};
use stencila_format::Format;
use stencila_node_compare::{
    Alignment, CompareOptions, Comparison, Correspondence, Difference, DifferenceFilter, NodeRef,
    PropertyPresence, ScalarValue, Selector, Side, UnmatchedReason, ValueState,
    compare_with_options,
};
use stencila_codec_text_trait::to_text;
use stencila_node_path::NodePath;
use stencila_schema::{Node, NodeSet};

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
        let left_snapshot = Snapshot {
            path: &left,
            node: &left_node,
        };
        let right_snapshot = Snapshot {
            path: &right,
            node: &right_node,
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

/// One side's document: where it came from, and what was decoded from it
///
/// The reports need both: the path to label the side with, and the node to read one-sided
/// content back out of.
#[derive(Clone, Copy)]
struct Snapshot<'document> {
    path: &'document Path,
    node: &'document Node,
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

/// Render the human-readable report for a comparison
///
/// Deterministic human presentation, not a stable interchange format: use `--to json`
/// or `--to yaml` for that.
fn text_report(
    comparison: &Comparison,
    left: Snapshot,
    right: Snapshot,
    summary: bool,
) -> Result<String> {
    let differences = comparison.differences();
    let one_sided =
        OneSidedRoots::collect(comparison.alignment(), comparison.filter(), left, right);
    let counts = Counts::collect(comparison);

    let mut report = String::new();

    writeln!(
        report,
        "{}",
        if comparison.is_equal() {
            "equal"
        } else {
            "different"
        }
    )?;
    writeln!(report, "left:  {}", left.path.display())?;
    writeln!(report, "right: {}", right.path.display())?;
    writeln!(report)?;

    writeln!(
        report,
        "correspondences: {paired} paired, {left_only} left-only ({left_roots}), {right_only} right-only ({right_roots})",
        paired = counts.paired,
        left_only = counts.left_only,
        left_roots = plural(one_sided.left.len(), "root"),
        right_only = counts.right_only,
        right_roots = plural(one_sided.right.len(), "root"),
    )?;
    writeln!(report, "differences: {}", differences.len())?;
    writeln!(
        report,
        "  node type: {}  presence: {}  value: {}  parent: {}  reordered: {}",
        counts.node_type, counts.presence, counts.value, counts.parent, counts.reordered
    )?;
    if let Some((selectors, suppressed)) = filter_description(comparison) {
        writeln!(report, "filter:     {selectors}")?;
        writeln!(report, "suppressed: {suppressed}")?;
    }

    if summary
        || (one_sided.left.is_empty() && one_sided.right.is_empty() && differences.is_empty())
    {
        return Ok(report);
    }

    writeln!(report)?;

    for root in one_sided.left.iter().chain(one_sided.right.iter()) {
        write_one_sided(&mut report, root)?;
    }
    for difference in differences {
        write_difference(&mut report, difference)?;
    }

    Ok(report)
}

/// Render the side-by-side HTML view for a comparison
///
/// A self-contained page, with no external assets, so that it can be written to a
/// temporary file and opened directly by a browser. Like the text report, this is
/// human presentation rather than an interchange format.
fn html_report(
    comparison: &Comparison,
    left: Snapshot,
    right: Snapshot,
    summary: bool,
) -> Result<String> {
    let alignment = comparison.alignment();
    let differences = comparison.differences();
    let one_sided = OneSidedRoots::collect(alignment, comparison.filter(), left, right);
    let counts = Counts::collect(comparison);

    let (status, status_class) = if comparison.is_equal() {
        ("equal", "equal")
    } else {
        ("different", "different")
    };

    // Rows are read down the page, so they are put into the reading order of the
    // documents rather than grouped by kind
    let anchors = LeftAnchors::collect(alignment);
    let mut rows = Vec::new();
    for root in one_sided.left.iter().chain(one_sided.right.iter()) {
        rows.push(ViewRow::one_sided(root, &anchors));
    }
    for difference in differences {
        rows.push(ViewRow::difference(difference));
    }
    // A stable sort, so that several rows about the same occurrence keep the
    // canonical order that the comparison put them in
    rows.sort_by(|first, second| first.order.cmp(&second.order));

    let mut html = String::new();

    writeln!(
        html,
        r#"<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Comparison of {left_title} and {right_title}</title>
<style>{CSS}</style>
</head>
<body>
<header>
  <h1>Comparison <span class="status {status_class}">{status}</span></h1>
  <div class="paths">
    <div class="side"><span class="label">left</span> <span class="path">{left_path}</span></div>
    <div class="side"><span class="label">right</span> <span class="path">{right_path}</span></div>
  </div>
  <ul class="counts">
    <li><span>paired</span> {paired}</li>
    <li><span>left-only</span> {left_only} <em>({left_roots})</em></li>
    <li><span>right-only</span> {right_only} <em>({right_roots})</em></li>
    <li><span>differences</span> {differences_count}</li>
    <li><span>node type</span> {node_type}</li>
    <li><span>presence</span> {presence}</li>
    <li><span>value</span> {value}</li>
    <li><span>parent</span> {parent}</li>
    <li><span>reordered</span> {reordered}</li>
  </ul>
  {filter}
</header>"#,
        left_title = escape(&left.path.display().to_string()),
        right_title = escape(&right.path.display().to_string()),
        left_path = escape(&left.path.display().to_string()),
        right_path = escape(&right.path.display().to_string()),
        paired = counts.paired,
        left_only = counts.left_only,
        left_roots = escape(&plural(one_sided.left.len(), "root")),
        right_only = counts.right_only,
        right_roots = escape(&plural(one_sided.right.len(), "root")),
        differences_count = differences.len(),
        node_type = counts.node_type,
        presence = counts.presence,
        value = counts.value,
        parent = counts.parent,
        reordered = counts.reordered,
        filter = match filter_description(comparison) {
            Some((selectors, suppressed)) => format!(
                "<div class=\"filter\"><span class=\"label\">filter</span> \
                 <code>{selectors}</code> <em>suppressed {suppressed}</em></div>",
                selectors = escape(&selectors),
                suppressed = escape(&suppressed),
            ),
            None => String::new(),
        },
    )?;

    if summary || rows.is_empty() {
        let note = if rows.is_empty() {
            "The documents are equal."
        } else {
            "Only counts are reported because <code>--summary</code> was used."
        };
        writeln!(html, "<p class=\"note\">{note}</p>")?;
    } else {
        writeln!(
            html,
            r#"<table>
<thead><tr><th class="kind">&nbsp;</th><th>left</th><th>right</th></tr></thead>
<tbody>"#
        )?;
        for row in &rows {
            write_view_row(&mut html, row)?;
        }
        writeln!(html, "</tbody>\n</table>")?;
    }

    writeln!(html, "</body>\n</html>")?;

    Ok(html)
}

/// The styles for the HTML view
const CSS: &str = r#"
:root {
  --background: #ffffff;
  --foreground: #16181d;
  --muted: #5b6472;
  --border: #dfe3e9;
  --surface: #f6f7f9;
  --left: #b4442e;
  --right: #1f7a4d;
  --changed: #8a5a00;
  --removed-background: #ffd7d1;
  --removed-foreground: #7a2617;
  --added-background: #c8f0d8;
  --added-foreground: #10502f;
}
@media (prefers-color-scheme: dark) {
  :root {
    --background: #14161a;
    --foreground: #e7e9ee;
    --muted: #98a1b0;
    --border: #2b2f37;
    --surface: #1b1e24;
    --left: #f08f78;
    --right: #74d3a2;
    --changed: #e0b355;
    --removed-background: #5e2018;
    --removed-foreground: #ffd7d1;
    --added-background: #14472e;
    --added-foreground: #c8f0d8;
  }
}
* { box-sizing: border-box; }
body {
  margin: 0;
  padding: 1.5rem;
  background: var(--background);
  color: var(--foreground);
  font-family: system-ui, -apple-system, "Segoe UI", sans-serif;
  font-size: 15px;
  line-height: 1.5;
}
h1 { font-size: 1.25rem; margin: 0 0 0.75rem; }
.status {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  padding: 0.15rem 0.5rem;
  border-radius: 999px;
  border: 1px solid var(--border);
  vertical-align: middle;
}
.status.equal { color: var(--right); }
.status.different { color: var(--changed); }
.paths { display: flex; flex-wrap: wrap; gap: 1.5rem; margin-bottom: 0.75rem; }
.side .label {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
}
.path, code, .subject, .detail { font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
.counts { display: flex; flex-wrap: wrap; gap: 0.5rem; list-style: none; margin: 0; padding: 0; }
.counts li {
  background: var(--surface);
  border: 1px solid var(--border);
  border-radius: 0.375rem;
  padding: 0.15rem 0.5rem;
  font-size: 0.85rem;
}
.counts span { color: var(--muted); margin-right: 0.35rem; }
.counts em { color: var(--muted); font-style: normal; }
.note { color: var(--muted); margin-top: 1.25rem; }
.filter { margin-top: 0.5rem; font-size: 0.85rem; }
.filter em { color: var(--muted); font-style: normal; }
table { width: 100%; border-collapse: collapse; margin-top: 1.25rem; table-layout: fixed; }
th, td {
  text-align: left;
  vertical-align: top;
  padding: 0.5rem 0.65rem;
  border-bottom: 1px solid var(--border);
  overflow-wrap: anywhere;
}
thead th {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.06em;
  color: var(--muted);
  font-weight: 600;
}
th.kind, td.kind { width: 9.5rem; }
.marker { margin-right: 0.4rem; font-family: ui-monospace, SFMono-Regular, Menlo, monospace; }
tr.left-only .marker, tr.left-only .kind-label { color: var(--left); }
tr.right-only .marker, tr.right-only .kind-label { color: var(--right); }
tr.changed .marker, tr.changed .kind-label { color: var(--changed); }
.kind-label { font-size: 0.85rem; }
.note-inline { display: block; color: var(--muted); font-size: 0.8rem; }
.detail { display: block; color: var(--muted); font-size: 0.85rem; margin-top: 0.15rem; }
.detail .removed, .detail .added { border-radius: 0.15rem; padding: 0.05rem 0.1rem; }
.detail .removed { background: var(--removed-background); color: var(--removed-foreground); }
.detail .added { background: var(--added-background); color: var(--added-foreground); }
.absent { color: var(--muted); }
"#;

/// Where a row belongs in the reading order of the view
///
/// Correspondences and differences are both in canonical order, which is left path
/// order, so rows that have a left occurrence order themselves. Right-only rows have
/// no left path at all, which is why the comparison groups them ahead of everything
/// else, so they are instead anchored to the left path of the paired occurrence that
/// precedes them on the right (see `LeftAnchors`).
#[derive(PartialEq, Eq, PartialOrd, Ord)]
struct RowOrder {
    /// The left path the row sits at, or is anchored to
    anchor: Option<NodePath>,

    /// Whether the row belongs at that left path, or just after it
    ///
    /// A right-only row goes after the occurrence it is anchored to, but before that
    /// occurrence's descendants, which is where it was inserted on the right.
    after_anchor: bool,

    /// The right path, which orders right-only rows sharing an anchor
    right: Option<NodePath>,
}

/// The left path that each right path belongs after
///
/// Built from the paired occurrences, which are the only ones whose position is known
/// on both sides.
struct LeftAnchors<'comparison> {
    /// The paired occurrences as (right path, left path), ordered by right path
    pairs: Vec<(&'comparison NodePath, &'comparison NodePath)>,
}

impl<'comparison> LeftAnchors<'comparison> {
    fn collect(alignment: &'comparison Alignment) -> Self {
        let mut pairs: Vec<_> = alignment
            .pairs()
            .map(|(left, right, ..)| (&right.path, &left.path))
            .collect();
        pairs.sort();

        Self { pairs }
    }

    /// The left path that a right path is positioned after, if any
    ///
    /// The nearest paired occurrence preceding it on the right, which is `None` when
    /// nothing on the right precedes it, and the row belongs at the top.
    fn anchor(&self, right: &NodePath) -> Option<NodePath> {
        let following = self.pairs.partition_point(|(path, ..)| *path < right);
        following
            .checked_sub(1)
            .map(|preceding| self.pairs[preceding].1.clone())
    }
}

/// One side of a row of the side-by-side view
struct ViewCell {
    /// What the row is about on this side, usually a path
    subject: String,

    /// The state of that subject on this side, in runs to be shaded
    ///
    /// Empty when the row has no state to show, and a single unshaded run when it
    /// has one that is not worth comparing within.
    detail: Vec<Segment>,
}

impl ViewCell {
    /// A cell whose state is shown as it is, without shading
    fn new(subject: String, detail: Option<String>) -> Self {
        Self {
            subject,
            detail: detail.into_iter().map(Segment::unchanged).collect(),
        }
    }
}

/// A run of a rendered state, and whether the other side also has it
struct Segment {
    text: String,

    /// Whether this run is missing from the other side, and so is shaded
    changed: bool,
}

impl Segment {
    fn unchanged(text: String) -> Self {
        Self {
            text,
            changed: false,
        }
    }
}

/// The longest rendered states that are compared within
///
/// Beyond this the shading is more noise than signal, and the comparison is not worth
/// the time, so the states are shown plainly instead.
const MAX_SHADED_CHARACTERS: usize = 10_000;

/// How long to spend comparing within a pair of rendered states
const SHADING_TIMEOUT: Duration = Duration::from_millis(250);

/// Split two rendered states into runs, marking what the other side does not have
///
/// Compares by Unicode word, rather than by character, so that shading falls on whole
/// words instead of fragmenting inside them. The rendered states are compared, rather
/// than the raw strings, so that the type prefix and quoting stay in place and remain
/// unshaded when only the value within them changed.
fn shade(left: &str, right: &str) -> (Vec<Segment>, Vec<Segment>) {
    if left.len() > MAX_SHADED_CHARACTERS || right.len() > MAX_SHADED_CHARACTERS {
        return (
            vec![Segment::unchanged(left.to_string())],
            vec![Segment::unchanged(right.to_string())],
        );
    }

    let diff = TextDiff::configure()
        .algorithm(Algorithm::Patience)
        .timeout(SHADING_TIMEOUT)
        .diff_unicode_words(left, right);

    let mut left_segments: Vec<Segment> = Vec::new();
    let mut right_segments: Vec<Segment> = Vec::new();

    for change in diff.iter_all_changes() {
        let text = change.value();
        match change.tag() {
            ChangeTag::Equal => {
                push_segment(&mut left_segments, text, false);
                push_segment(&mut right_segments, text, false);
            }
            ChangeTag::Delete => push_segment(&mut left_segments, text, true),
            ChangeTag::Insert => push_segment(&mut right_segments, text, true),
        }
    }

    (
        merge_across_whitespace(left_segments),
        merge_across_whitespace(right_segments),
    )
}

/// Absorb the space between two changed words into the shading
///
/// Comparing by word reports the space between two changed words as equal to the
/// other side, which would otherwise break what reads as one change into two shaded
/// runs. Trailing space before unchanged text is left alone, because runs of the same
/// kind are already merged, so a whitespace-only run is only ever between two changes.
fn merge_across_whitespace(segments: Vec<Segment>) -> Vec<Segment> {
    let count = segments.len();
    let mut merged: Vec<Segment> = Vec::new();

    for (index, segment) in segments.into_iter().enumerate() {
        let bridges = !segment.changed
            && index + 1 < count
            && segment.text.chars().all(char::is_whitespace)
            && merged.last().is_some_and(|last| last.changed);

        push_segment(&mut merged, &segment.text, segment.changed || bridges);
    }

    merged
}

/// Add a run to a side, merging it into the previous run when they are alike
///
/// Merging keeps shading contiguous across adjacent changed words, rather than
/// breaking it at every word boundary.
fn push_segment(segments: &mut Vec<Segment>, text: &str, changed: bool) {
    match segments.last_mut() {
        Some(last) if last.changed == changed => last.text.push_str(text),
        _ => segments.push(Segment {
            text: text.to_string(),
            changed,
        }),
    }
}

/// One row of the side-by-side view
struct ViewRow {
    /// The kind of correspondence or difference
    kind: &'static str,

    /// The same marker character that the text report uses
    marker: &'static str,

    /// Which of the three row colors to use
    class: &'static str,

    /// The left side, or `None` when the row is right-only
    left: Option<ViewCell>,

    /// The right side, or `None` when the row is left-only
    right: Option<ViewCell>,

    /// Additional explanation of the row, such as why a subtree is one-sided
    note: Option<String>,

    /// Where the row belongs in the reading order of the view
    order: RowOrder,
}

impl ViewRow {
    /// Build a row for a one-sided subtree root
    fn one_sided(root: &OneSidedRoot, anchors: &LeftAnchors) -> Self {
        // The whole subtree is on one side, so all of its content is shaded, the same
        // way a value that only one side has is
        let cell = ViewCell {
            subject: occurrence(root.node),
            detail: root
                .content
                .iter()
                .map(|content| Segment {
                    text: content.clone(),
                    changed: true,
                })
                .collect(),
        };

        let reason = match root.reason {
            UnmatchedReason::NoCompatibleCandidate => "no compatible candidate",
            UnmatchedReason::GapCheaperThanPair => "gap cheaper than pair",
        };
        let note = Some(if root.occurrences > 1 {
            format!("{} occurrences; {reason}", root.occurrences)
        } else {
            reason.to_string()
        });

        match root.side {
            Side::Left => Self {
                kind: "left-only",
                marker: "-",
                class: "left-only",
                left: Some(cell),
                right: None,
                note,
                order: RowOrder {
                    anchor: Some(root.node.path.clone()),
                    after_anchor: false,
                    right: None,
                },
            },
            Side::Right => Self {
                kind: "right-only",
                marker: "+",
                class: "right-only",
                left: None,
                right: Some(cell),
                note,
                order: RowOrder {
                    anchor: anchors.anchor(&root.node.path),
                    after_anchor: true,
                    right: Some(root.node.path.clone()),
                },
            },
        }
    }

    /// Build a row for a difference
    fn difference(difference: &Difference) -> Self {
        let (kind, marker, left, right) = match difference {
            Difference::NodeTypeChanged { left, right } => (
                "node type",
                "≠",
                ViewCell::new(occurrence(left), Some(left.node_type.to_string())),
                ViewCell::new(occurrence(right), Some(right.node_type.to_string())),
            ),

            Difference::PropertyPresenceChanged {
                left,
                right,
                property,
                left_presence,
                right_presence,
            } => (
                "presence",
                "±",
                ViewCell::new(
                    value_subject(left, Some(property), None),
                    Some(presence(*left_presence).to_string()),
                ),
                ViewCell::new(
                    value_subject(right, Some(property), None),
                    Some(presence(*right_presence).to_string()),
                ),
            ),

            Difference::ValueChanged {
                location,
                left,
                right,
            } => {
                // The only difference where both sides carry content that is worth
                // comparing within, rather than a single term
                let (left_detail, right_detail) = shade(&value_state(left), &value_state(right));
                (
                    "value",
                    "~",
                    ViewCell {
                        subject: value_subject(
                            &location.left,
                            location.property.as_ref(),
                            location.left_index,
                        ),
                        detail: left_detail,
                    },
                    ViewCell {
                        subject: value_subject(
                            &location.right,
                            location.property.as_ref(),
                            location.right_index,
                        ),
                        detail: right_detail,
                    },
                )
            }

            Difference::ParentChanged {
                left,
                right,
                left_parent,
                right_parent,
                left_property,
                right_property,
            } => (
                "parent",
                "→",
                ViewCell::new(
                    occurrence(left),
                    Some(parent(left_parent.as_ref(), left_property.as_ref())),
                ),
                ViewCell::new(
                    occurrence(right),
                    Some(parent(right_parent.as_ref(), right_property.as_ref())),
                ),
            ),

            Difference::Reordered { left, right, .. } => (
                "reordered",
                "↕",
                ViewCell::new(occurrence(left), None),
                ViewCell::new(occurrence(right), None),
            ),
        };

        Self {
            kind,
            marker,
            class: "changed",
            left: Some(left),
            right: Some(right),
            note: None,
            order: RowOrder {
                anchor: Some(difference.left().path.clone()),
                after_anchor: false,
                right: Some(difference.right().path.clone()),
            },
        }
    }
}

/// Write a row of the side-by-side view
fn write_view_row(html: &mut String, row: &ViewRow) -> Result<()> {
    writeln!(
        html,
        r#"<tr class="{class}">
  <td class="kind"><span class="marker">{marker}</span><span class="kind-label">{kind}</span></td>
  {left}
  {right}
</tr>"#,
        class = row.class,
        marker = escape(row.marker),
        kind = escape(row.kind),
        left = view_cell(row.left.as_ref(), row.note.as_deref(), "removed"),
        right = view_cell(row.right.as_ref(), row.note.as_deref(), "added"),
    )?;

    Ok(())
}

/// Render one side of a row of the side-by-side view
///
/// `shading` is the class for the runs that the other side does not have, which is
/// what makes the same run read as removed on the left and added on the right.
fn view_cell(cell: Option<&ViewCell>, note: Option<&str>, shading: &str) -> String {
    let Some(cell) = cell else {
        return "<td class=\"absent\">—</td>".to_string();
    };

    let mut rendered = format!(
        "<td><span class=\"subject\">{}</span>",
        escape(&cell.subject)
    );
    if !cell.detail.is_empty() {
        rendered.push_str("<span class=\"detail\">");
        for segment in &cell.detail {
            let text = escape(&segment.text);
            if segment.changed {
                rendered.push_str(&format!("<span class=\"{shading}\">{text}</span>"));
            } else {
                rendered.push_str(&text);
            }
        }
        rendered.push_str("</span>");
    }
    if let Some(note) = note {
        rendered.push_str(&format!(
            "<span class=\"note-inline\">{}</span>",
            escape(note)
        ));
    }
    rendered.push_str("</td>");

    rendered
}

/// Escape text for inclusion in HTML
fn escape(text: &str) -> String {
    let mut escaped = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(character),
        }
    }
    escaped
}

/// How many correspondences and differences of each kind there are
#[derive(Default)]
struct Counts {
    paired: usize,
    left_only: usize,
    right_only: usize,
    node_type: usize,
    presence: usize,
    value: usize,
    parent: usize,
    reordered: usize,
}

impl Counts {
    /// Count what the comparison reports
    ///
    /// One-sided counts come from the comparison's tally rather than from the raw
    /// correspondences, so that a filtered subtree is counted out whole rather than
    /// leaving its descendants behind.
    fn collect(comparison: &Comparison) -> Self {
        let mut counts = Self::default();

        for correspondence in comparison.alignment().correspondences() {
            if matches!(correspondence, Correspondence::Paired { .. }) {
                counts.paired += 1;
            }
        }

        let tally = comparison.one_sided_tally();
        counts.left_only = tally.left_only();
        counts.right_only = tally.right_only();

        for difference in comparison.differences() {
            match difference {
                Difference::NodeTypeChanged { .. } => counts.node_type += 1,
                Difference::PropertyPresenceChanged { .. } => counts.presence += 1,
                Difference::ValueChanged { .. } => counts.value += 1,
                Difference::ParentChanged { .. } => counts.parent += 1,
                Difference::Reordered { .. } => counts.reordered += 1,
            }
        }

        counts
    }
}

/// The root of a maximal one-sided subtree, and how many occurrences it covers
struct OneSidedRoot<'comparison> {
    side: Side,
    node: &'comparison NodeRef,
    reason: UnmatchedReason,
    occurrences: usize,

    /// What the subtree says, read back out of the document it is only in
    content: Option<String>,
}

/// The maximal one-sided subtree roots of each side
#[derive(Default)]
struct OneSidedRoots<'comparison> {
    left: Vec<OneSidedRoot<'comparison>>,
    right: Vec<OneSidedRoot<'comparison>>,
}

impl<'comparison> OneSidedRoots<'comparison> {
    /// Collapse the one-sided correspondences onto their maximal roots
    ///
    /// Every structured descendant of a one-sided occurrence is itself one-sided, and
    /// correspondences are in canonical path order, so on each side the descendants
    /// of a root immediately follow it.
    fn collect(
        alignment: &'comparison Alignment,
        filter: &DifferenceFilter,
        left_snapshot: Snapshot,
        right_snapshot: Snapshot,
    ) -> Self {
        let mut roots = Self::default();

        for correspondence in alignment.correspondences() {
            let (side, node, reason, ancestor) = match correspondence {
                Correspondence::Paired { .. } => continue,
                Correspondence::LeftOnly {
                    left,
                    reason,
                    nearest_one_sided_ancestor,
                } => (Side::Left, left, reason, nearest_one_sided_ancestor),
                Correspondence::RightOnly {
                    right,
                    reason,
                    nearest_one_sided_ancestor,
                } => (Side::Right, right, reason, nearest_one_sided_ancestor),
            };

            let side_roots = match side {
                Side::Left => &mut roots.left,
                Side::Right => &mut roots.right,
            };

            match (ancestor, side_roots.last_mut()) {
                (Some(..), Some(root)) => root.occurrences += 1,
                _ => {
                    let snapshot = match side {
                        Side::Left => left_snapshot,
                        Side::Right => right_snapshot,
                    };
                    side_roots.push(OneSidedRoot {
                        side,
                        node,
                        reason: *reason,
                        occurrences: 1,
                        // Read only for a root: a descendant's content is already part
                        // of the root's, so rendering it again would repeat it
                        content: one_sided_content(snapshot.node, &node.path),
                    })
                }
            }
        }

        // Applied to the collapsed roots, not to each correspondence, so that excluding
        // a node type hides its whole subtree rather than just its root. Descendants
        // are already folded into the root's occurrence count by this point.
        roots.left.retain(|root| filter.allows_node(root.node));
        roots.right.retain(|root| filter.allows_node(root.node));

        roots
    }
}

/// Write a one-sided subtree root
fn write_one_sided(report: &mut String, root: &OneSidedRoot) -> Result<()> {
    let (marker, label) = match root.side {
        Side::Left => ("-", "left-only"),
        Side::Right => ("+", "right-only"),
    };

    let reason = match root.reason {
        UnmatchedReason::NoCompatibleCandidate => "no compatible candidate",
        UnmatchedReason::GapCheaperThanPair => "gap cheaper than pair",
    };

    let detail = if root.occurrences > 1 {
        format!("{} occurrences; {reason}", root.occurrences)
    } else {
        reason.to_string()
    };

    writeln!(
        report,
        "{marker} {label:<10} {subject} ({detail})",
        subject = occurrence(root.node)
    )?;
    if let Some(content) = &root.content {
        writeln!(
            report,
            "  {side}{content}",
            side = match root.side {
                Side::Left => "left:  ",
                Side::Right => "right: ",
            }
        )?;
    }

    Ok(())
}

/// Write a difference
fn write_difference(report: &mut String, difference: &Difference) -> Result<()> {
    match difference {
        Difference::NodeTypeChanged { left, right } => {
            writeln!(
                report,
                "≠ {:<10} {} ↔ {}",
                "node type",
                occurrence(left),
                occurrence(right)
            )?;
        }

        Difference::PropertyPresenceChanged {
            left,
            right,
            property,
            left_presence,
            right_presence,
        } => {
            let left_subject = value_subject(left, Some(property), None);
            let right_subject = value_subject(right, Some(property), None);
            writeln!(
                report,
                "± {:<10} {}",
                "presence",
                sides(&left_subject, &right_subject)
            )?;
            writeln!(report, "  left:  {}", presence(*left_presence))?;
            writeln!(report, "  right: {}", presence(*right_presence))?;
        }

        Difference::ValueChanged {
            location,
            left,
            right,
        } => {
            let left_subject = value_subject(
                &location.left,
                location.property.as_ref(),
                location.left_index,
            );
            let right_subject = value_subject(
                &location.right,
                location.property.as_ref(),
                location.right_index,
            );
            writeln!(
                report,
                "~ {:<10} {}",
                "value",
                sides(&left_subject, &right_subject)
            )?;
            writeln!(report, "  left:  {}", value_state(left))?;
            writeln!(report, "  right: {}", value_state(right))?;
        }

        Difference::ParentChanged {
            left,
            right,
            left_parent,
            right_parent,
            left_property,
            right_property,
        } => {
            writeln!(
                report,
                "→ {:<10} {} ↔ {}",
                "parent",
                occurrence(left),
                occurrence(right)
            )?;
            writeln!(
                report,
                "  left:  {}",
                parent(left_parent.as_ref(), left_property.as_ref())
            )?;
            writeln!(
                report,
                "  right: {}",
                parent(right_parent.as_ref(), right_property.as_ref())
            )?;
        }

        Difference::Reordered { left, right, .. } => {
            writeln!(
                report,
                "↕ {:<10} {} ↔ {}",
                "reordered",
                occurrence(left),
                occurrence(right)
            )?;
        }
    }

    Ok(())
}

/// The longest rendering of a one-sided occurrence that is shown
///
/// A one-sided subtree root can be a whole section, and the point of showing it is to
/// recognise it, not to read it. Anything longer is elided.
const MAX_ONE_SIDED_CHARACTERS: usize = 160;

/// Render the content of a one-sided occurrence, from the document it came from
///
/// The comparison deliberately records no values for one-sided occurrences: the
/// alignment says only that a node of some type at some path has no counterpart. So the
/// content is read back out of the original snapshot, by the path the alignment gives.
///
/// `None` when the path does not resolve, or when the occurrence has no text of its own,
/// in which case the row shows just its path and type as before.
fn one_sided_content(node: &Node, path: &NodePath) -> Option<String> {
    let text = match stencila_schema::get(node, path.clone()).ok()? {
        NodeSet::One(node) => to_text(&node),
        NodeSet::Many(nodes) => nodes.iter().map(to_text).collect::<Vec<_>>().join(" "),
    };

    // Collapsed onto one line, because a row shows a single line and a block's own
    // newlines would otherwise decide where it wraps
    let text = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if text.is_empty() {
        return None;
    }

    Some(elide(&text, MAX_ONE_SIDED_CHARACTERS))
}

/// Shorten a rendering to a maximum number of characters
///
/// Counts characters rather than bytes, so that it never splits a multi-byte character.
fn elide(text: &str, maximum: usize) -> String {
    if text.chars().count() <= maximum {
        return text.to_string();
    }

    let mut elided: String = text.chars().take(maximum).collect();
    elided.push('…');
    elided
}

/// Describe the filter a comparison was made under
///
/// Returns the selectors and what they suppressed, or `None` when nothing was filtered.
/// Always reported alongside the counts, so that an `equal` verdict is never read
/// without the filter that produced it.
fn filter_description(comparison: &Comparison) -> Option<(String, String)> {
    if !comparison.is_filtered() {
        return None;
    }

    let filter = comparison.filter();
    let selectors = filter
        .exclude
        .iter()
        .map(|selector| format!("-{selector}"))
        .chain(filter.include.iter().map(|selector| format!("+{selector}")))
        .collect::<Vec<_>>()
        .join(" ");

    let tally = comparison.one_sided_tally();
    let suppressed = format!(
        "{differences} {noun}, {left_only} left-only, {right_only} right-only",
        differences = comparison.suppressed_differences(),
        noun = if comparison.suppressed_differences() == 1 {
            "difference"
        } else {
            "differences"
        },
        left_only = tally.suppressed[0],
        right_only = tally.suppressed[1],
    );

    Some((selectors, suppressed))
}

/// Render a path, using `$` for the root
fn path(path: &NodePath) -> String {
    if path.is_empty() {
        "$".to_string()
    } else {
        format!("$/{path}")
    }
}

/// Render an occurrence as its path and node type
fn occurrence(node: &NodeRef) -> String {
    format!("{} {}", path(&node.path), node.node_type)
}

/// Render the two sides of a subject, collapsing them when they are the same
fn sides(left: &str, right: &str) -> String {
    if left == right {
        left.to_string()
    } else {
        format!("{left} ↔ {right}")
    }
}

/// Render the location of a value within its occurrence
///
/// Named as `$/path NodeType.property[index]`, so that the subject of a value or
/// presence difference reads the same way as every other row's, and a path is never
/// left to be understood on its own.
fn value_subject(
    node: &NodeRef,
    property: Option<&stencila_node_type::NodeProperty>,
    index: Option<usize>,
) -> String {
    let mut subject = occurrence(node);
    if let Some(property) = property {
        subject.push('.');
        subject.push_str(&property.to_string());
    }
    if let Some(index) = index {
        subject.push_str(&format!("[{index}]"));
    }
    subject
}

/// Render the parent side of a parent change
fn parent(node: Option<&NodeRef>, property: Option<&stencila_node_type::NodeProperty>) -> String {
    let mut rendered = match node {
        Some(node) => occurrence(node),
        None => "(none)".to_string(),
    };
    if let Some(property) = property {
        rendered.push_str(&format!(" .{property}"));
    }
    rendered
}

/// Render the presence of a property
fn presence(presence: PropertyPresence) -> &'static str {
    match presence {
        PropertyPresence::Undeclared => "undeclared",
        PropertyPresence::Absent => "absent",
        PropertyPresence::Present => "present",
    }
}

/// Render the complete state of one side of a value change
fn value_state(state: &ValueState) -> String {
    match state {
        ValueState::Absent => "absent".to_string(),
        ValueState::One { value } => scalar(value),
        ValueState::Many { values } => format!(
            "[{}]",
            values.iter().map(scalar).collect::<Vec<_>>().join(", ")
        ),
    }
}

/// Render a typed scalar value
fn scalar(value: &ScalarValue) -> String {
    match value {
        ScalarValue::Null => "null".to_string(),
        ScalarValue::Boolean { value } => format!("boolean {value}"),
        ScalarValue::Integer { value } => format!("integer {value}"),
        ScalarValue::UnsignedInteger { value } => format!("unsigned {value}"),
        ScalarValue::Number { value } => format!("number {value}"),
        ScalarValue::String { value } => format!("string {}", quote(value)),
        ScalarValue::Enum {
            schema_type,
            variant,
        } => format!("enum {schema_type}.{variant}"),
        ScalarValue::Array { items } => format!(
            "array [{}]",
            items.iter().map(scalar).collect::<Vec<_>>().join(", ")
        ),
        ScalarValue::Object { entries } => format!(
            "object {{{}}}",
            entries
                .iter()
                .map(|(key, value)| format!("{}: {}", quote(key), scalar(value)))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    }
}

/// Quote a string using JSON-style escaping
fn quote(value: &str) -> String {
    serde_json::to_string(value).unwrap_or_else(|_| format!("{value:?}"))
}

/// Render a count with a singular or plural noun
fn plural(count: usize, noun: &str) -> String {
    if count == 1 {
        format!("{count} {noun}")
    } else {
        format!("{count} {noun}s")
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use clap::Parser;
    use stencila_node_compare::compare;
    use stencila_node_type::{NodeProperty, NodeType};
    use stencila_schema::{Article, Block, Heading, Node, Paragraph, Section, shortcuts::t};

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

    /// The two sides of a comparison, as the reports take them
    fn snapshots<'node>(
        left: &'node Node,
        right: &'node Node,
    ) -> (Snapshot<'node>, Snapshot<'node>) {
        (
            Snapshot {
                path: Path::new("left.smd"),
                node: left,
            },
            Snapshot {
                path: Path::new("right.smd"),
                node: right,
            },
        )
    }

    fn report(left: &Node, right: &Node, summary: bool) -> String {
        let comparison = compare(left, right).unwrap();
        let (left, right) = snapshots(left, right);
        text_report(&comparison, left, right, summary).unwrap()
    }

    fn html(left: &Node, right: &Node, summary: bool) -> String {
        let comparison = compare(left, right).unwrap();
        let (left, right) = snapshots(left, right);
        html_report(&comparison, left, right, summary).unwrap()
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
    fn reports_equality() {
        let node = article(vec![para("One")]);
        let report = report(&node, &node, false);

        assert_eq!(
            report,
            r#"equal
left:  left.smd
right: right.smd

correspondences: 3 paired, 0 left-only (0 roots), 0 right-only (0 roots)
differences: 0
  node type: 0  presence: 0  value: 0  parent: 0  reordered: 0
"#
        );
    }

    /// A one-sided subtree shows what it says, not just where it is
    #[test]
    fn one_sided_rows_show_their_content() {
        let left = article(vec![para("Kept"), para("Only on the left")]);
        let right = article(vec![para("Kept")]);

        let report = report(&left, &right, false);
        assert!(
            report.contains("- left-only  $/content/1 Paragraph"),
            "{report}"
        );
        assert!(
            report.contains("  left:  Only on the left"),
            "the content is read back out of the left document: {report}"
        );

        // In the view it is shaded, the same way a value only the left has is
        let page = html(&left, &right, false);
        assert!(
            page.contains("<span class=\"removed\">Only on the left</span>"),
            "{page}"
        );

        // A right-only subtree is read out of the right document, and shaded as added
        let page = html(&right, &left, false);
        assert!(
            page.contains("<span class=\"added\">Only on the left</span>"),
            "{page}"
        );
    }

    /// A rendering longer than the limit is elided rather than filling the row
    #[test]
    fn long_one_sided_content_is_elided() {
        let long = "word ".repeat(200);
        let left = article(vec![para("Kept"), para(&long)]);
        let right = article(vec![para("Kept")]);

        let report = report(&left, &right, false);
        let line = report
            .lines()
            .find(|line| line.starts_with("  left:  word"))
            .expect("Expected the one-sided content");
        assert!(line.ends_with('…'), "{line}");
        assert!(
            line.chars().count() <= MAX_ONE_SIDED_CHARACTERS + 10,
            "{line}"
        );
    }

    /// Every subject names the node type, so that a path is never read on its own
    #[test]
    fn subjects_name_the_node_type() {
        // `authors` is a structured property, so a change of presence is reported as a
        // presence difference rather than as a value change
        let left = Node::Article(Article {
            authors: Some(vec![]),
            ..Article::new(vec![para("Methods")])
        });
        let right = Node::Article(Article::new(vec![para("Method")]));

        let report = report(&left, &right, false);
        assert!(
            report.contains("± presence   $ Article.authors
"),
            "presence subjects name the type: {report}"
        );
        assert!(
            report.contains("~ value      $/content/0/content/0 Text.value
"),
            "value subjects name the type: {report}"
        );

        // And the same subjects reach the side-by-side view
        let page = html(&left, &right, false);
        assert!(page.contains("$ Article.authors"), "{page}");
        assert!(
            page.contains("$/content/0/content/0 Text.value"),
            "{page}"
        );
    }

    #[test]
    fn reports_value_changes_with_typed_states() {
        let left = article(vec![para("Methods")]);
        let right = article(vec![para("Method")]);
        let report = report(&left, &right, false);

        assert!(report.starts_with("different\n"), "{report}");
        assert!(
            report.contains("~ value      $/content/0/content/0 Text.value\n"),
            "{report}"
        );
        assert!(report.contains("  left:  string \"Methods\"\n"), "{report}");
        assert!(report.contains("  right: string \"Method\"\n"), "{report}");
    }

    #[test]
    fn summary_stops_after_the_counts() {
        let left = article(vec![para("Methods")]);
        let right = article(vec![para("Method")]);

        let report = report(&left, &right, true);
        assert_eq!(
            report,
            r#"different
left:  left.smd
right: right.smd

correspondences: 3 paired, 0 left-only (0 roots), 0 right-only (0 roots)
differences: 1
  node type: 0  presence: 0  value: 1  parent: 0  reordered: 0
"#
        );
    }

    #[test]
    fn reports_one_sided_subtrees_collapsed_onto_their_roots() {
        let left = article(vec![para("One")]);
        let right = article(vec![
            para("One"),
            Block::Section(Section {
                content: vec![para("Two"), para("Three")],
                ..Default::default()
            }),
        ]);
        let report = report(&left, &right, false);

        // The section and its four descendants collapse onto one root
        assert!(
            report.contains(
                "correspondences: 3 paired, 0 left-only (0 roots), 5 right-only (1 root)\n"
            ),
            "{report}"
        );

        let one_sided: Vec<&str> = report
            .lines()
            .filter(|line| line.starts_with('+') || line.starts_with('-'))
            .collect();
        assert_eq!(one_sided.len(), 1, "{report}");
        assert!(
            one_sided[0].starts_with("+ right-only $/content/1 Section ("),
            "{report}"
        );
        assert!(one_sided[0].contains("5 occurrences;"), "{report}");
    }

    #[test]
    fn reports_node_type_changes() {
        let left = article(vec![para("Title")]);
        let right = article(vec![Block::Heading(Heading::new(1, vec![t("Title")]))]);
        let report = report(&left, &right, false);

        assert!(
            report.contains("≠ node type  $/content/0 Paragraph ↔ $/content/0 Heading\n"),
            "{report}"
        );
    }

    #[test]
    fn reports_reordering() {
        let left = article(vec![para("One"), para("Two"), para("Three")]);
        let right = article(vec![para("Two"), para("Three"), para("One")]);
        let report = report(&left, &right, false);

        let reordered: Vec<&str> = report
            .lines()
            .filter(|line| line.starts_with('↕'))
            .collect();
        assert!(!reordered.is_empty(), "{report}");
        assert!(reordered[0].contains(" ↔ "), "{report}");
        assert!(reordered[0].contains("Paragraph"), "{report}");
    }

    #[test]
    fn html_view_is_self_contained_and_side_by_side() {
        let left = article(vec![para("Methods")]);
        let right = article(vec![para("Method")]);
        let page = html(&left, &right, false);

        // Nothing to fetch, so that the page works from a temporary file
        assert!(page.starts_with("<!DOCTYPE html>"), "{page}");
        assert!(!page.contains("http://"), "{page}");
        assert!(!page.contains("https://"), "{page}");
        assert!(!page.contains("<script"), "{page}");

        assert!(
            page.contains(r#"<span class="status different">different</span>"#),
            "{page}"
        );
        assert!(
            page.contains(r#"<span class="path">left.smd</span>"#),
            "{page}"
        );
        assert!(
            page.contains(r#"<span class="path">right.smd</span>"#),
            "{page}"
        );

        // Both sides of the value change, in their own cells
        assert!(
            page.contains(r#"<span class="removed">Methods</span>"#),
            "{page}"
        );
        assert!(
            page.contains(r#"<span class="added">Method</span>"#),
            "{page}"
        );
    }

    #[test]
    fn html_view_leaves_one_sided_cells_empty() {
        let left = article(vec![para("One")]);
        let right = article(vec![
            para("One"),
            Block::Section(Section {
                content: vec![para("Two")],
                ..Default::default()
            }),
        ]);
        let page = html(&left, &right, false);

        assert!(page.contains(r#"<tr class="right-only">"#), "{page}");
        assert!(page.contains(r#"<td class="absent">—</td>"#), "{page}");
        assert!(page.contains("$/content/1 Section"), "{page}");
        assert!(!page.contains(r#"<tr class="left-only">"#), "{page}");
    }

    #[test]
    fn html_view_rows_are_in_reading_order() {
        let left = article(vec![
            para("Alpha first paragraph"),
            para("Beta second paragraph"),
            para("Gamma third paragraph"),
        ]);
        let right = article(vec![
            para("Alpha first paragraph, revised"),
            Block::Section(Section {
                content: vec![para("An inserted section")],
                ..Default::default()
            }),
            para("Beta second paragraph, revised"),
            para("Gamma third paragraph"),
        ]);
        let page = html(&left, &right, false);

        let row = |needle: &str| {
            page.find(needle)
                .unwrap_or_else(|| panic!("Expected {needle} in: {page}"))
        };

        // The right-only section sits between the occurrences it was inserted
        // between, rather than ahead of every difference
        assert!(row("$/content/0/content/0 Text.value") < row("$/content/1 Section"));
        assert!(row("$/content/1 Section") < row("$/content/1/content/0 Text.value"));
    }

    #[test]
    fn html_view_puts_a_leading_insertion_first() {
        let left = article(vec![para("Alpha first paragraph")]);
        let right = article(vec![
            Block::Section(Section {
                content: vec![para("An inserted section")],
                ..Default::default()
            }),
            para("Alpha first paragraph, revised"),
        ]);
        let page = html(&left, &right, false);

        // Nothing on the right precedes the section, so it anchors to nothing
        assert!(
            page.find("$/content/0 Section")
                .expect("Expected the section")
                < page
                    .find("$/content/0/content/0 Text.value")
                    .expect("Expected the revision"),
            "{page}"
        );
    }

    #[test]
    fn html_view_shades_what_the_other_side_does_not_have() {
        let left = article(vec![para("The quick brown fox jumps over the lazy dog")]);
        let right = article(vec![para("The quick red fox jumps over the dog")]);
        let page = html(&left, &right, false);

        // Removed on the left, added on the right, never the other way round
        assert!(
            page.contains(r#"<span class="removed">brown</span>"#),
            "{page}"
        );
        assert!(page.contains(r#"<span class="added">red</span>"#), "{page}");
        assert!(
            !page.contains(r#"<span class="added">brown</span>"#),
            "{page}"
        );
        assert!(
            page.contains(r#"<span class="removed">lazy </span>"#),
            "{page}"
        );

        // What both sides have is left unshaded, including the rendering of the type
        // and the quoting around the value
        assert!(
            page.contains(r#"<span class="detail">string &quot;The quick <span"#),
            "{page}"
        );
    }

    #[test]
    fn shading_merges_adjacent_changes() {
        let (left, right) = shade("one two three four", "one five six four");

        let rendered = |segments: &[Segment]| {
            segments
                .iter()
                .map(|segment| {
                    if segment.changed {
                        format!("[{}]", segment.text)
                    } else {
                        segment.text.clone()
                    }
                })
                .collect::<String>()
        };

        // Two changed words become one shaded run, not two
        assert_eq!(rendered(&left), "one [two three] four");
        assert_eq!(rendered(&right), "one [five six] four");
    }

    #[test]
    fn shading_of_equal_states_changes_nothing() {
        let (left, right) = shade("string \"same\"", "string \"same\"");

        assert!(left.iter().all(|segment| !segment.changed));
        assert!(right.iter().all(|segment| !segment.changed));
    }

    #[test]
    fn very_long_states_are_not_shaded() {
        let long = "word ".repeat(MAX_SHADED_CHARACTERS);
        let (left, right) = shade(&long, "word");

        // Shown plainly, rather than spending the time to compare within them
        assert_eq!(left.len(), 1);
        assert!(!left[0].changed);
        assert_eq!(right.len(), 1);
        assert!(!right[0].changed);
    }

    #[test]
    fn html_view_escapes_document_content() {
        let left = article(vec![para("A <script>alert('x')</script> in the text")]);
        let right = article(vec![para("A <span>span</span> in the text")]);
        let page = html(&left, &right, false);

        assert!(!page.contains("<script"), "{page}");
        assert!(page.contains("&lt;"), "{page}");
        assert!(page.contains("script"), "{page}");
    }

    #[test]
    fn html_view_summary_stops_after_the_counts() {
        let left = article(vec![para("Methods")]);
        let right = article(vec![para("Method")]);

        let page = html(&left, &right, true);
        assert!(page.contains("<li><span>value</span> 1</li>"), "{page}");
        assert!(!page.contains("<table>"), "{page}");
        assert!(page.contains("--summary"), "{page}");

        // Equal documents have nothing to tabulate either
        let page = html(&left, &left, false);
        assert!(
            page.contains(r#"<span class="status equal">equal</span>"#),
            "{page}"
        );
        assert!(!page.contains("<table>"), "{page}");
        assert!(page.contains("The documents are equal."), "{page}");
    }

    #[test]
    fn renders_root_paths_as_dollar() {
        assert_eq!(path(&NodePath::new()), "$");
        assert_eq!(
            path(&NodePath::from_str("content/0").unwrap()),
            "$/content/0"
        );
    }

    #[test]
    fn renders_typed_scalars() {
        assert_eq!(scalar(&ScalarValue::Null), "null");
        assert_eq!(
            scalar(&ScalarValue::Boolean { value: true }),
            "boolean true"
        );
        assert_eq!(scalar(&ScalarValue::Integer { value: -3 }), "integer -3");
        assert_eq!(
            scalar(&ScalarValue::UnsignedInteger { value: 3 }),
            "unsigned 3"
        );
        assert_eq!(scalar(&ScalarValue::number(1.5)), "number 1.5");
        assert_eq!(
            scalar(&ScalarValue::string("a \"quoted\"\nvalue")),
            r#"string "a \"quoted\"\nvalue""#
        );
        assert_eq!(
            scalar(&ScalarValue::Enum {
                schema_type: "CitationMode".to_string(),
                variant: "Parenthetical".to_string()
            }),
            "enum CitationMode.Parenthetical"
        );
        assert_eq!(
            scalar(&ScalarValue::Array {
                items: vec![ScalarValue::Integer { value: 1 }, ScalarValue::Null]
            }),
            "array [integer 1, null]"
        );
        assert_eq!(
            scalar(
                &ScalarValue::object([
                    ("b".to_string(), ScalarValue::Null),
                    ("a".to_string(), ScalarValue::string("x"))
                ])
                .unwrap()
            ),
            r#"object {"a": string "x", "b": null}"#
        );
    }

    #[test]
    fn renders_value_states() {
        assert_eq!(value_state(&ValueState::Absent), "absent");
        assert_eq!(
            value_state(&ValueState::Many {
                values: vec![ScalarValue::string("a"), ScalarValue::string("b")]
            }),
            r#"[string "a", string "b"]"#
        );
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
