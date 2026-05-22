use std::{
    net::{IpAddr, Ipv4Addr},
    path::{Path, PathBuf},
};

use clap::{Parser, ValueEnum};
use eyre::{Result, bail};
use tokio::sync::oneshot;

use stencila_cli_utils::{Code, ToStdout, color_print::cstr, message};
use stencila_document::Document;
use stencila_format::Format;
use stencila_graph::{Graph, WorkspaceOptions, graph_from_node, graph_from_path};
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
    #[arg(long, value_enum)]
    to: Option<GraphOutputFormat>,

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
}

impl From<GraphOutputFormat> for Format {
    fn from(format: GraphOutputFormat) -> Self {
        match format {
            GraphOutputFormat::Json => Format::Json,
            GraphOutputFormat::Yaml => Format::Yaml,
        }
    }
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
"
);

impl Cli {
    pub async fn run(self) -> Result<()> {
        let GraphSource { graph, path } = build_graph(&self.path).await?;

        match self.output {
            Some(output) => export_graph(&graph, &output, self.to).await,
            None => {
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
}

async fn export_graph(
    graph: &Graph,
    output: &Path,
    format: Option<GraphOutputFormat>,
) -> Result<()> {
    let format = output_format(output, format)?;

    if is_stdout_output(output) {
        Code::new_from(format.into(), graph)?.to_stdout();
    } else {
        let content = serialize_graph(graph, format)?;
        tokio::fs::write(output, content).await?;
    }

    Ok(())
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
    path: PathBuf,
}

async fn build_graph(path: &Path) -> Result<GraphSource> {
    let path = path.canonicalize()?;

    if path.is_dir() {
        let graph = graph_from_path(&path, Some(WorkspaceOptions::default())).await?;
        return Ok(GraphSource { graph, path });
    }

    if path.is_file() {
        let doc = Document::open(&path, None).await?;
        let node = doc.root().await;
        let subject = path
            .file_name()
            .and_then(|name| name.to_str())
            .map_or_else(|| "document".to_string(), |name| format!("document:{name}"));
        let graph = graph_from_node(subject, &node)?;
        return Ok(GraphSource { graph, path });
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

    match output.extension().and_then(|extension| extension.to_str()) {
        Some("json") => Ok(GraphOutputFormat::Json),
        Some("yaml" | "yml") => Ok(GraphOutputFormat::Yaml),
        _ => bail!(
            "Unable to infer graph export format from `{}`; use `--to json` or `--to yaml`",
            output.display()
        ),
    }
}

fn serialize_graph(graph: &Graph, format: GraphOutputFormat) -> Result<String> {
    let content = match format {
        GraphOutputFormat::Json => serde_json::to_string_pretty(graph)?,
        GraphOutputFormat::Yaml => serde_yaml::to_string(graph)?,
    };

    Ok(format!("{content}\n"))
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
    use stencila_schema::{File, Node};

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
    fn serializes_graph_as_json_and_yaml() -> Result<()> {
        let graph = graph()?;

        let json = serialize_graph(&graph, GraphOutputFormat::Json)?;
        assert!(json.contains(r#""type": "Graph""#));
        assert!(json.contains(r#""subject": "test:graph""#));

        let yaml = serialize_graph(&graph, GraphOutputFormat::Yaml)?;
        assert!(yaml.contains("type: Graph"));
        assert!(yaml.contains("subject: test:graph"));

        Ok(())
    }

    fn graph() -> Result<Graph> {
        let mut builder = GraphBuilder::new("test:graph");
        builder.add_schema_node(
            "file:data.csv",
            Node::File(File::new("data.csv".to_string(), "data.csv".to_string())),
        );
        builder.build()
    }
}
