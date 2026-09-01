use std::{
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
    process::Stdio,
};

use clap::{Parser, ValueEnum};
use eyre::{Result, WrapErr, bail};
use tokio::{io::AsyncWriteExt, sync::oneshot};

use stencila_cli_utils::{Code, ToStdout, color_print::cstr, message};
use stencila_dirs::closest_workspace_dir;
use stencila_document::Document;
use stencila_format::Format;
use stencila_graph::{
    Graph, GraphConnectedMode, GraphContainmentMode, GraphDiagnostic, GraphProjectionDetail,
    GraphProjectionOptions, GraphProjectionPreset, RuntimeEvidenceMode, StaticAnalysisDiagnostic,
    WorkspaceOptions, dot::to_dot, filter_graph_view_connected_to,
    graph_from_node_with_runtime_evidence, graph_from_path_with_diagnostics, project_graph,
    set_graph_source_metadata_from_path,
};
use stencila_server::{DEFAULT_PORT, ServeOptions, ServerStarted, get_server_token};

const GRAPH_FILE: &str = "graph.json";
const GRAPH_VIEW_QUERY: &str = "~view=graph";

/// Build, view, and export Stencila graphs
#[derive(Debug, Parser)]
#[command(after_long_help = CLI_AFTER_LONG_HELP)]
pub struct Cli {
    /// The workspace directory or document file to graph
    #[arg(default_value = ".")]
    path: PathBuf,

    /// Output path for exporting the graph, or `-` for stdout
    output: Option<PathBuf>,

    /// Output format, overriding inference from the output extension
    #[arg(long, visible_alias = "format", value_enum)]
    to: Option<GraphOutputFormat>,

    /// How to respond to information lost when exporting MIRA JSON-LD
    #[arg(long, default_value_t = stencila_codecs::LossesResponse::Warn)]
    output_losses: stencila_codecs::LossesResponse,

    /// Projection preset for DOT, SVG, and PNG graph exports
    #[arg(long, value_enum, default_value_t = GraphProjectionPreset::Auto)]
    view: GraphProjectionPreset,

    /// Detail level for projected graph exports
    #[arg(long, value_enum, default_value_t = GraphProjectionDetail::Medium)]
    detail: GraphProjectionDetail,

    /// How to represent containment in projected graph exports
    #[arg(long, value_enum, conflicts_with_all = ["structure", "no_structure"])]
    containment: Option<GraphContainmentMode>,

    /// Include structural containment as visual clusters in projected graph exports
    #[arg(long, conflicts_with_all = ["containment", "no_structure"])]
    structure: bool,

    /// Exclude structural containment context in projected graph exports
    #[arg(long, conflicts_with = "containment")]
    no_structure: bool,

    /// Exclude low-confidence edges in projected graph exports
    #[arg(long)]
    no_low_confidence: bool,

    /// Keep citation marker nodes visible in projected graph exports
    #[arg(long)]
    no_collapse_citations: bool,

    /// Report I/O that static analysis could not resolve, instead of serving
    ///
    /// Each unresolved read or write is listed with its source location, the
    /// expression that could not be resolved, why, and what would make it
    /// resolvable. Combine with an output path to export the graph as well.
    #[arg(long)]
    explain: bool,

    /// Do not inspect C2PA content credentials while building workspace graphs
    #[arg(long)]
    no_c2pa: bool,

    /// Do not include Git commit authors on file-backed workspace graph nodes
    #[arg(long)]
    no_git_authors: bool,

    /// Do not merge cached evidence from prior traced executions
    #[arg(long)]
    no_runtime_evidence: bool,

    /// Filter projected graph exports to nodes connected to matching nodes
    #[arg(long, value_name = "PATTERN")]
    connected_to: Vec<String>,

    /// How to traverse graph edges for connected-to filtering
    #[arg(long, value_enum, default_value_t = GraphConnectedMode::Directed)]
    connected_mode: GraphConnectedMode,

    /// The address to serve on
    #[arg(long, short, default_value = "127.0.0.1")]
    address: IpAddr,

    /// The port to serve on
    #[arg(long, short, default_value_t = DEFAULT_PORT)]
    port: u16,

    /// Do not open the graph view in a browser
    #[arg(long)]
    no_open: bool,

    /// Do not authenticate or authorize graph view requests
    #[arg(long)]
    no_auth: bool,
}

/// Graph export format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
enum GraphOutputFormat {
    /// Stencila Schema Graph as JSON.
    Json,

    /// Stencila Schema Graph as YAML.
    Yaml,

    /// Research discourse graph as MIRA JSON-LD.
    #[value(name = "mira-jsonld", alias = "mira", alias = "mira-json-ld")]
    MiraJsonLd,

    /// Projected graph as Graphviz DOT.
    Dot,

    /// Projected graph rendered to SVG by Graphviz.
    Svg,

    /// Projected graph rendered to PNG by Graphviz.
    Png,
}

pub static CLI_AFTER_LONG_HELP: &str = cstr!(
    "<bold><b>Examples</b></bold>
  <dim># View the current workspace graph in a browser</dim>
  <b>stencila graph</>

  <dim># View a workspace graph in a browser</dim>
  <b>stencila graph</> <g>.</>

  <dim># Start the graph server without opening a browser</dim>
  <b>stencila graph</> <g>.</> <c>--no-open</> <c>--port</> <g>9010</>

  <dim># Export graph JSON inferred from the output extension</dim>
  <b>stencila graph</> <g>.</> <g>graph.json</>

  <dim># Export graph YAML inferred from the output extension</dim>
  <b>stencila graph</> <g>report.smd</> <g>graph.yaml</>

  <dim># Export graph YAML to stdout</dim>
  <b>stencila graph</> <g>.</> <g>-</> <c>--to</> <g>yaml</>

  <dim># Export research discourse as MIRA JSON-LD</dim>
  <b>stencila graph</> <g>report.smd</> <g>report.mira.jsonld</>

  <dim># Export MIRA JSON-LD to stdout using an explicit format</dim>
  <b>stencila graph</> <g>report.smd</> <g>-</> <c>--to</> <g>mira-jsonld</>

  <dim># Export a projected data flow graph as Graphviz DOT</dim>
  <b>stencila graph</> <g>.</> <g>graph.dot</> <c>--view</> <g>flow</>

  <dim># Export a detailed data flow graph including local symbols</dim>
  <b>stencila graph</> <g>.</> <g>graph.dot</> <c>--view</> <g>flow</> <c>--detail</> <g>high</>

  <dim># Export only the data flow connected to a matching script</dim>
  <b>stencila graph</> <g>.</> <g>graph.png</> <c>--view</> <g>flow</> <c>--connected-to</> <g>analysis.R</>

  <dim># Export the full connected component through shared inputs</dim>
  <b>stencila graph</> <g>.</> <g>graph.png</> <c>--view</> <g>flow</> <c>--connected-to</> <g>analysis.R</> <c>--connected-mode</> <g>undirected</>

  <dim># Export the same graph without directory/document containment clusters</dim>
  <b>stencila graph</> <g>.</> <g>graph.dot</> <c>--view</> <g>flow</> <c>--containment</> <g>none</>

  <dim># Export a projected software dependency graph as SVG using Graphviz</dim>
  <b>stencila graph</> <g>.</> <g>graph.svg</> <c>--view</> <g>deps</>

  <dim># Report I/O that static analysis could not resolve</dim>
  <b>stencila graph</> <g>.</> <c>--explain</>
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let GraphSource {
            graph,
            diagnostics,
            graph_diagnostics,
            path,
        } = build_graph(
            &self.path,
            self.no_c2pa,
            self.no_git_authors,
            self.no_runtime_evidence,
        )
        .await?;

        if self.explain {
            if let Some(output) = &self.output {
                let projection_options = self.projection_options();
                export_graph(
                    &graph,
                    output,
                    self.to,
                    self.output_losses.clone(),
                    &projection_options,
                    &self.connected_to,
                    self.connected_mode,
                )
                .await?;
            }

            if diagnostics.is_empty() && graph_diagnostics.is_empty() {
                message!("No unresolved I/O or graph authoring issues found.");
            } else {
                let diagnostic_count = diagnostics.len();
                for diagnostic in diagnostics {
                    diagnostic.to_stderr()?;
                }
                report_graph_diagnostics(&graph_diagnostics);
                message!(
                    "{} unresolved I/O {}; {} graph authoring {}.",
                    diagnostic_count,
                    if diagnostic_count == 1 {
                        "operation"
                    } else {
                        "operations"
                    },
                    graph_diagnostics.len(),
                    if graph_diagnostics.len() == 1 {
                        "issue"
                    } else {
                        "issues"
                    },
                );
            }
            return Ok(());
        }

        report_graph_diagnostics(&graph_diagnostics);

        match &self.output {
            Some(output) => {
                let projection_options = self.projection_options();
                export_graph(
                    &graph,
                    output,
                    self.to,
                    self.output_losses.clone(),
                    &projection_options,
                    &self.connected_to,
                    self.connected_mode,
                )
                .await
            }
            None => {
                if !self.connected_to.is_empty() {
                    bail!("`--connected-to` is only supported for DOT, SVG, and PNG graph exports");
                }

                serve_graph(
                    graph,
                    path,
                    self.address,
                    self.port,
                    self.no_open,
                    self.no_auth,
                )
                .await
            }
        }
    }

    fn projection_options(&self) -> GraphProjectionOptions {
        GraphProjectionOptions {
            preset: self.view,
            detail: self.detail,
            containment: if self.structure {
                Some(GraphContainmentMode::Clusters)
            } else if self.no_structure {
                Some(GraphContainmentMode::None)
            } else {
                self.containment
            },
            include_structure_edges: None,
            include_low_confidence_edges: !self.no_low_confidence,
            collapse_citation_nodes: !self.no_collapse_citations,
        }
    }
}

async fn export_graph(
    graph: &Graph,
    output: &Path,
    format: Option<GraphOutputFormat>,
    output_losses: stencila_codecs::LossesResponse,
    projection_options: &GraphProjectionOptions,
    connected_to: &[String],
    connected_mode: GraphConnectedMode,
) -> Result<()> {
    let format = output_format(output, format)?;

    match format {
        GraphOutputFormat::Json | GraphOutputFormat::Yaml => {
            if !connected_to.is_empty() {
                bail!("`--connected-to` is only supported for DOT, SVG, and PNG graph exports");
            }

            let content = serialize_graph(
                graph,
                format,
                projection_options,
                connected_to,
                connected_mode,
            )?;
            if is_stdout_output(output) {
                if let Some(format) = schema_format(format) {
                    Code::new_from(format, graph)?.to_stdout();
                } else {
                    tokio::io::stdout().write_all(content.as_bytes()).await?;
                }
            } else {
                tokio::fs::write(output, content).await?;
            }
        }
        GraphOutputFormat::MiraJsonLd => {
            validate_mira_export_options(graph, projection_options, connected_to)?;
            let content = serialize_mira_graph(graph, output_losses).await?;
            if is_stdout_output(output) {
                tokio::io::stdout().write_all(content.as_bytes()).await?;
            } else {
                tokio::fs::write(output, content).await?;
            }
        }
        GraphOutputFormat::Dot => {
            let content = serialize_graph(
                graph,
                format,
                projection_options,
                connected_to,
                connected_mode,
            )?;
            if is_stdout_output(output) {
                if let Some(format) = schema_format(format) {
                    Code::new_from(format, graph)?.to_stdout();
                } else {
                    tokio::io::stdout().write_all(content.as_bytes()).await?;
                }
            } else {
                tokio::fs::write(output, content).await?;
            }
        }
        GraphOutputFormat::Svg | GraphOutputFormat::Png => {
            let content = render_graph_image(
                graph,
                format,
                projection_options,
                connected_to,
                connected_mode,
            )
            .await?;
            if is_stdout_output(output) {
                tokio::io::stdout().write_all(&content).await?;
            } else {
                tokio::fs::write(output, content).await?;
            }
        }
    }

    Ok(())
}

async fn render_graph_image(
    graph: &Graph,
    format: GraphOutputFormat,
    projection_options: &GraphProjectionOptions,
    connected_to: &[String],
    connected_mode: GraphConnectedMode,
) -> Result<Vec<u8>> {
    let dot = serialize_graph(
        graph,
        GraphOutputFormat::Dot,
        projection_options,
        connected_to,
        connected_mode,
    )?;
    let image_format = match format {
        GraphOutputFormat::Svg => "svg",
        GraphOutputFormat::Png => "png",
        _ => bail!("Graphviz rendering only supports SVG and PNG outputs"),
    };

    let mut child = tokio::process::Command::new("dot")
        .arg(format!("-T{image_format}"))
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .wrap_err("unable to run Graphviz `dot`; install Graphviz and ensure `dot` is on PATH")?;

    let mut stdin = child
        .stdin
        .take()
        .ok_or_else(|| eyre::eyre!("unable to open Graphviz stdin"))?;
    stdin.write_all(dot.as_bytes()).await?;
    drop(stdin);

    let output = child.wait_with_output().await?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "Graphviz `dot` failed to render graph as {image_format}: {}",
            stderr.trim()
        );
    }

    Ok(output.stdout)
}

async fn serve_graph(
    graph: Graph,
    path: PathBuf,
    address: IpAddr,
    port: u16,
    no_open: bool,
    no_auth: bool,
) -> Result<()> {
    let temp_dir = tempfile::tempdir()?;
    let graph_file = temp_dir.path().join(GRAPH_FILE);
    let json = serde_json::to_string_pretty(&graph)?;
    tokio::fs::write(&graph_file, format!("{json}\n")).await?;

    let server_token = (!no_auth).then(get_server_token);

    message!("Starting graph view for `{}`", path.display());
    let (started_tx, started_rx) = oneshot::channel::<ServerStarted>();
    let dir = temp_dir.path().to_path_buf();
    let options = ServeOptions {
        dir,
        address,
        port,
        no_auth,
        server_token: server_token.clone(),
        started_sender: Some(started_tx),
        ..Default::default()
    };
    let serve = tokio::spawn(async move { stencila_server::serve(options).await });

    let started = match started_rx.await {
        Ok(started) => started,
        Err(_) => {
            serve.await??;
            bail!("Server stopped before graph view started");
        }
    };

    let host = browser_host(address);
    let graph_path = format!("{GRAPH_FILE}?{GRAPH_VIEW_QUERY}");
    let graph_next = graph_path.replace('?', "%3F").replace('=', "%3D");
    let url = match server_token {
        Some(token) => format!(
            "http://{host}:{}/~login?sst={token}&next={graph_next}",
            started.port,
        ),
        None => format!("http://{host}:{}/{graph_path}", started.port),
    };

    message!("Graph view at {}", url);
    if !no_open {
        webbrowser::open(&url)?;
    }

    serve.await??;
    drop(temp_dir);

    Ok(())
}

struct GraphSource {
    graph: Graph,
    diagnostics: Vec<StaticAnalysisDiagnostic>,
    graph_diagnostics: Vec<GraphDiagnostic>,
    path: PathBuf,
}

fn report_graph_diagnostics(diagnostics: &[GraphDiagnostic]) {
    for diagnostic in diagnostics {
        message!("⚠️  {}", diagnostic);
    }
}

async fn build_graph(
    path: &Path,
    no_c2pa: bool,
    no_git_authors: bool,
    no_runtime_evidence: bool,
) -> Result<GraphSource> {
    let path = path.canonicalize()?;

    if path.is_dir() {
        let analysis = graph_from_path_with_diagnostics(
            &path,
            Some(WorkspaceOptions {
                include_c2pa: !no_c2pa,
                git_file_authors: !no_git_authors,
                runtime_evidence: if no_runtime_evidence {
                    RuntimeEvidenceMode::None
                } else {
                    RuntimeEvidenceMode::Cached
                },
                ..Default::default()
            }),
        )
        .await?;
        return Ok(GraphSource {
            graph: analysis.graph,
            diagnostics: analysis.diagnostics,
            graph_diagnostics: analysis.graph_diagnostics,
            path,
        });
    }

    if path.is_file() {
        let doc = Document::open(&path, None).await?;
        let node = doc.root().await;
        let subject = url::Url::from_file_path(&path)
            .map(String::from)
            .map_err(|()| eyre::eyre!("unable to create a file URL for {}", path.display()))?;
        let root = closest_workspace_dir(&path, false).await?;
        let scope = path
            .strip_prefix(&root)
            .ok()
            .map(Path::to_path_buf)
            .or_else(|| path.file_name().map(PathBuf::from))
            .unwrap_or_else(|| PathBuf::from("document"))
            .to_string_lossy()
            .replace(std::path::MAIN_SEPARATOR, "/");
        let mut analysis = graph_from_node_with_runtime_evidence(
            subject,
            &scope,
            &node,
            &root,
            if no_runtime_evidence {
                RuntimeEvidenceMode::None
            } else {
                RuntimeEvidenceMode::Cached
            },
        )?;
        set_graph_source_metadata_from_path(&mut analysis.graph, &path)?;
        return Ok(GraphSource {
            graph: analysis.graph,
            diagnostics: analysis.diagnostics,
            graph_diagnostics: analysis.graph_diagnostics,
            path,
        });
    }

    bail!("Graph path is not a file or directory: {}", path.display())
}

fn output_format(output: &Path, requested: Option<GraphOutputFormat>) -> Result<GraphOutputFormat> {
    if let Some(format) = requested {
        return Ok(format);
    }

    if is_stdout_output(output) {
        return Ok(GraphOutputFormat::Json);
    }

    if Format::from_path(output) == Format::MiraJsonLd {
        return Ok(GraphOutputFormat::MiraJsonLd);
    }

    match output.extension().and_then(|extension| extension.to_str()) {
        Some("json") => Ok(GraphOutputFormat::Json),
        Some("yaml" | "yml") => Ok(GraphOutputFormat::Yaml),
        Some("dot" | "gv") => Ok(GraphOutputFormat::Dot),
        Some("svg") => Ok(GraphOutputFormat::Svg),
        Some("png") => Ok(GraphOutputFormat::Png),
        _ => bail!(
            "Unable to infer graph export format from `{}`; use `--to json`, `--to yaml`, `--to mira-jsonld`, `--to dot`, `--to svg`, or `--to png`",
            output.display()
        ),
    }
}

fn serialize_graph(
    graph: &Graph,
    format: GraphOutputFormat,
    projection_options: &GraphProjectionOptions,
    connected_to: &[String],
    connected_mode: GraphConnectedMode,
) -> Result<String> {
    let content = match format {
        GraphOutputFormat::Json => serde_json::to_string_pretty(graph)?,
        GraphOutputFormat::Yaml => serde_yaml::to_string(graph)?,
        GraphOutputFormat::MiraJsonLd => {
            bail!("MIRA JSON-LD graphs are serialized asynchronously")
        }
        GraphOutputFormat::Dot => {
            let view = project_graph(graph, projection_options);
            let view = filter_graph_view_connected_to(&view, connected_to, connected_mode)?;
            to_dot(&view)
        }
        GraphOutputFormat::Svg | GraphOutputFormat::Png => {
            bail!("image graph exports must be rendered with Graphviz")
        }
    };

    if content.ends_with('\n') {
        Ok(content)
    } else {
        Ok(format!("{content}\n"))
    }
}

fn validate_mira_export_options(
    graph: &Graph,
    projection_options: &GraphProjectionOptions,
    connected_to: &[String],
) -> Result<()> {
    if !connected_to.is_empty() {
        bail!("`--connected-to` is not supported for MIRA JSON-LD exports");
    }
    if projection_options != &GraphProjectionOptions::default() {
        bail!(
            "graph projection options are not supported for MIRA JSON-LD exports; remove `--view`, `--detail`, `--containment`, `--structure`, `--no-structure`, `--no-low-confidence`, and `--no-collapse-citations`"
        );
    }
    if !graph
        .nodes
        .iter()
        .any(|node| node.node.as_research_object().is_some())
    {
        bail!(
            "unable to export MIRA JSON-LD because the graph contains no supported research objects; add a Claim, Evidence, Question, Protocol, or Request block"
        );
    }

    Ok(())
}

async fn serialize_mira_graph(
    graph: &Graph,
    output_losses: stencila_codecs::LossesResponse,
) -> Result<String> {
    stencila_codecs::to_string(
        &stencila_schema::Node::Graph(graph.clone()),
        Some(stencila_codecs::EncodeOptions {
            format: Some(Format::MiraJsonLd),
            losses: output_losses,
            ..Default::default()
        }),
    )
    .await
}

fn schema_format(format: GraphOutputFormat) -> Option<Format> {
    match format {
        GraphOutputFormat::Json => Some(Format::Json),
        GraphOutputFormat::Yaml => Some(Format::Yaml),
        _ => None,
    }
}

fn is_stdout_output(output: &Path) -> bool {
    output == Path::new("-")
}

fn browser_host(address: IpAddr) -> String {
    if address.is_unspecified() {
        IpAddr::V4(Ipv4Addr::LOCALHOST).to_string()
    } else {
        address.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use eyre::Result;
    use stencila_graph::GraphBuilder;
    use stencila_schema::{Claim, ClaimType, File, Node, SoftwareSourceCode};

    #[test]
    fn infers_output_format() -> Result<()> {
        assert_eq!(
            output_format(Path::new("graph.json"), None)?,
            GraphOutputFormat::Json
        );
        assert_eq!(
            output_format(Path::new("graph.yaml"), None)?,
            GraphOutputFormat::Yaml
        );
        assert_eq!(
            output_format(Path::new("graph.yml"), None)?,
            GraphOutputFormat::Yaml
        );
        assert_eq!(
            output_format(Path::new("graph.mira.jsonld"), None)?,
            GraphOutputFormat::MiraJsonLd
        );
        assert_eq!(
            output_format(Path::new("graph.mira.json"), None)?,
            GraphOutputFormat::MiraJsonLd
        );
        assert!(output_format(Path::new("graph.jsonld"), None).is_err());
        assert_eq!(
            output_format(Path::new("graph.dot"), None)?,
            GraphOutputFormat::Dot
        );
        assert_eq!(
            output_format(Path::new("graph.gv"), None)?,
            GraphOutputFormat::Dot
        );
        assert_eq!(
            output_format(Path::new("graph.svg"), None)?,
            GraphOutputFormat::Svg
        );
        assert_eq!(
            output_format(Path::new("graph.png"), None)?,
            GraphOutputFormat::Png
        );
        assert_eq!(
            output_format(Path::new("-"), None)?,
            GraphOutputFormat::Json
        );
        assert_eq!(
            output_format(Path::new("graph.txt"), Some(GraphOutputFormat::Yaml))?,
            GraphOutputFormat::Yaml
        );
        assert!(output_format(Path::new("graph.txt"), None).is_err());

        Ok(())
    }

    #[test]
    fn parses_explicit_mira_jsonld_format() -> Result<()> {
        for flag in ["--to", "--format"] {
            let cli = Cli::try_parse_from(["graph", "report.smd", "-", flag, "mira-jsonld"])?;
            assert_eq!(cli.to, Some(GraphOutputFormat::MiraJsonLd));
        }

        let cli = Cli::try_parse_from(["graph", "report.smd", "-", "--to", "mira"])?;
        assert_eq!(cli.to, Some(GraphOutputFormat::MiraJsonLd));
        assert_eq!(cli.output_losses, stencila_codecs::LossesResponse::Warn);

        let cli = Cli::try_parse_from([
            "graph",
            "report.smd",
            "-",
            "--to",
            "mira",
            "--output-losses",
            "abort",
        ])?;
        assert_eq!(cli.output_losses, stencila_codecs::LossesResponse::Abort);
        Ok(())
    }

    #[test]
    fn serializes_graph_as_json_and_yaml() -> Result<()> {
        let graph = graph()?;
        let options = GraphProjectionOptions::default();

        let json = serialize_graph(
            &graph,
            GraphOutputFormat::Json,
            &options,
            &[],
            GraphConnectedMode::Directed,
        )?;
        assert!(json.contains(r#""type": "Graph""#));
        assert!(json.contains(r#""subject": "test:graph""#));

        let yaml = serialize_graph(
            &graph,
            GraphOutputFormat::Yaml,
            &options,
            &[],
            GraphConnectedMode::Directed,
        )?;
        assert!(yaml.contains("type: Graph"));
        assert!(yaml.contains("subject: test:graph"));

        Ok(())
    }

    #[test]
    fn serializes_graph_as_dot() -> Result<()> {
        let graph = graph()?;
        let dot = serialize_graph(
            &graph,
            GraphOutputFormat::Dot,
            &GraphProjectionOptions::default(),
            &[],
            GraphConnectedMode::Directed,
        )?;

        assert!(dot.contains("digraph stencila_graph"));
        assert!(dot.contains("\"file:data.csv\""));
        Ok(())
    }

    #[test]
    fn serializes_projected_dot_connected_to_pattern() -> Result<()> {
        let graph = graph()?;
        let dot = serialize_graph(
            &graph,
            GraphOutputFormat::Dot,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
            &["analysis.py".to_string()],
            GraphConnectedMode::Directed,
        )?;

        assert!(dot.contains("\"code:analysis.py\""));
        assert!(dot.contains("\"file:data.csv\""));
        assert!(dot.contains("\"file:plot.png\""));
        assert!(!dot.contains("\"code:other.py\""));
        Ok(())
    }

    #[test]
    fn serializes_projected_dot_connected_to_undirected() -> Result<()> {
        let graph = graph()?;
        let dot = serialize_graph(
            &graph,
            GraphOutputFormat::Dot,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
            &["analysis.py".to_string()],
            GraphConnectedMode::Undirected,
        )?;

        assert!(dot.contains("\"code:analysis.py\""));
        assert!(dot.contains("\"file:data.csv\""));
        assert!(dot.contains("\"code:other.py\""));
        assert!(dot.contains("\"file:other-plot.png\""));
        Ok(())
    }

    #[tokio::test]
    async fn rejects_connected_to_for_raw_exports() -> Result<()> {
        let graph = graph()?;
        let error = export_graph(
            &graph,
            Path::new("-"),
            Some(GraphOutputFormat::Json),
            stencila_codecs::LossesResponse::Warn,
            &GraphProjectionOptions::default(),
            &["analysis.py".to_string()],
            GraphConnectedMode::Directed,
        )
        .await
        .expect_err("connected-to should not apply to raw graph exports");

        assert!(
            error
                .to_string()
                .contains("`--connected-to` is only supported for DOT, SVG, and PNG")
        );
        Ok(())
    }

    #[test]
    fn rejects_projection_options_and_empty_graph_for_mira() -> Result<()> {
        let graph = graph()?;
        let error = validate_mira_export_options(
            &graph,
            &GraphProjectionOptions {
                preset: GraphProjectionPreset::Flow,
                ..Default::default()
            },
            &[],
        )
        .expect_err("MIRA export should reject projection options");
        assert!(error.to_string().contains("projection options"));

        let error = validate_mira_export_options(&graph, &GraphProjectionOptions::default(), &[])
            .expect_err("MIRA export should require a research object");
        assert!(error.to_string().contains("no supported research objects"));
        Ok(())
    }

    #[tokio::test]
    async fn strict_mira_loss_policy_aborts_export() -> Result<()> {
        let mut claim = Claim::new(Vec::new());
        claim.id = Some("claim-1".to_string());
        claim.claim_type = Some(ClaimType::Statement);
        let graph = Graph::new(
            "https://example.org/report.smd".to_string(),
            vec![stencila_schema::GraphNode::new(
                "claim".to_string(),
                Box::new(Node::Claim(claim)),
            )],
            Vec::new(),
        );

        let error = serialize_mira_graph(&graph, stencila_codecs::LossesResponse::Abort)
            .await
            .expect_err("strict MIRA export should abort on a claim type loss");
        assert!(format!("{error:?}").contains("Claim.claimType"));
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn exports_markdown_research_relations_as_mira_jsonld() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        std::fs::create_dir_all(temp_dir.path().join(".stencila/cache/runtime"))?;
        let path = temp_dir.path().join("report.smd");
        std::fs::write(
            &path,
            r#"::: claim #claim-1
:supported-by: #evidence-1

The intervention improves recovery.

:::

::: evidence #evidence-1

Participants recovered sooner.

:::
"#,
        )?;

        let source = build_graph(&path, true, true, true).await?;
        validate_mira_export_options(&source.graph, &GraphProjectionOptions::default(), &[])?;
        let inferred_output = temp_dir.path().join("report.mira.jsonld");
        export_graph(
            &source.graph,
            &inferred_output,
            None,
            stencila_codecs::LossesResponse::Warn,
            &GraphProjectionOptions::default(),
            &[],
            GraphConnectedMode::Directed,
        )
        .await?;
        let actual = std::fs::read_to_string(&inferred_output)?;
        let expected = stencila_codecs::to_string(
            &Node::Graph(source.graph.clone()),
            Some(stencila_codecs::EncodeOptions {
                format: Some(Format::MiraJsonLd),
                ..Default::default()
            }),
        )
        .await?;
        assert_eq!(actual, expected);

        let explicit_output = temp_dir.path().join("report.jsonld");
        export_graph(
            &source.graph,
            &explicit_output,
            Some(GraphOutputFormat::MiraJsonLd),
            stencila_codecs::LossesResponse::Warn,
            &GraphProjectionOptions::default(),
            &[],
            GraphConnectedMode::Directed,
        )
        .await?;
        assert_eq!(std::fs::read_to_string(explicit_output)?, expected);

        let value: serde_json::Value = serde_json::from_str(&actual)?;
        let document_id = url::Url::from_file_path(&path.canonicalize()?)
            .map(String::from)
            .map_err(|()| eyre::eyre!("unable to create document URL"))?;
        assert_eq!(value["@id"], document_id);
        assert_eq!(value["@context"][1]["@base"], document_id);
        let items = value["@graph"]
            .as_array()
            .ok_or_else(|| eyre::eyre!("MIRA graph should be an array"))?;
        assert!(
            items
                .iter()
                .any(|item| item["@type"] == "mira:Claim" && item["@id"] == "#claim-1")
        );
        assert!(
            items
                .iter()
                .any(|item| item["@type"] == "mira:Evidence" && item["@id"] == "#evidence-1")
        );
        assert!(items.iter().any(|item| {
            item["@type"] == "mira:supportedBy"
                && item["source"] == "#claim-1"
                && item["destination"] == "#evidence-1"
        }));
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn file_graph_uses_closest_runtime_workspace() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let nested = temp_dir.path().join("nested");
        std::fs::create_dir_all(temp_dir.path().join(".stencila/cache/runtime"))?;
        std::fs::create_dir_all(&nested)?;
        let path = nested.join("report.md");
        std::fs::write(&path, "# Report\n")?;

        let source = build_graph(&path, true, true, true).await?;

        assert!(
            source
                .graph
                .nodes
                .iter()
                .any(|node| node.id.starts_with("node:nested/report.md"))
        );
        Ok(())
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn file_graph_preserves_repository_source_metadata() -> Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let nested = temp_dir.path().join("nested");
        std::fs::create_dir_all(temp_dir.path().join(".stencila/cache/runtime"))?;
        std::fs::create_dir_all(&nested)?;
        run_git(temp_dir.path(), &["init", "--quiet"])?;
        run_git(
            temp_dir.path(),
            &[
                "remote",
                "add",
                "origin",
                "https://github.com/example/research.git",
            ],
        )?;
        let path = nested.join("report.smd");
        std::fs::write(&path, "::: claim #claim-1\n\nClaim.\n\n:::\n")?;

        let source = build_graph(&path, true, true, true).await?;

        assert_eq!(
            source.graph.options.repository.as_deref(),
            Some("https://github.com/example/research")
        );
        assert_eq!(
            source.graph.options.path.as_deref(),
            Some("nested/report.smd")
        );
        Ok(())
    }

    fn run_git(dir: &Path, args: &[&str]) -> Result<()> {
        let output = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(args)
            .output()?;
        if !output.status.success() {
            bail!(
                "git command failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        Ok(())
    }

    fn graph() -> Result<Graph> {
        let mut builder = GraphBuilder::new("test:graph");
        builder.add_schema_node(
            "file:data.csv",
            Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
        );
        builder.add_schema_node(
            "code:analysis.py",
            Node::SoftwareSourceCode(SoftwareSourceCode {
                name: "analysis.py".to_string(),
                path: Some("analysis.py".to_string()),
                programming_language: "python".to_string(),
                ..Default::default()
            }),
        );
        builder.add_schema_node(
            "file:plot.png",
            Node::File(File::new("plot.png".to_string(), "plot.png".to_string())),
        );
        builder.add_schema_node(
            "code:other.py",
            Node::SoftwareSourceCode(SoftwareSourceCode {
                name: "other.py".to_string(),
                path: Some("other.py".to_string()),
                programming_language: "python".to_string(),
                ..Default::default()
            }),
        );
        builder.add_schema_node(
            "file:other-plot.png",
            Node::File(File::new(
                "other-plot.png".to_string(),
                "other-plot.png".to_string(),
            )),
        );
        builder.add_read("file:data.csv", "code:analysis.py", []);
        builder.add_generation("code:analysis.py", "file:plot.png", []);
        builder.add_read("file:data.csv", "code:other.py", []);
        builder.add_generation("code:other.py", "file:other-plot.png", []);
        builder.build()
    }
}
