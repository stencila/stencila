use layout::{
    backends::svg::SVGWriter,
    gv::{DotParser, GraphBuilder},
};

use kernel::{
    common::{
        async_trait::async_trait, eyre::Result, once_cell::sync::Lazy, regex::Regex, tracing,
    },
    format::Format,
    schema::{
        ExecutionMessage, ImageObject, MessageLevel, Node, SoftwareApplication,
        SoftwareApplicationOptions,
    },
    Kernel, KernelInstance,
};

const NAME: &str = "graphviz";

/// A kernel for rendering Graphviz DOT to SVGs
#[derive(Default)]
pub struct GraphvizKernel {}

impl Kernel for GraphvizKernel {
    fn name(&self) -> String {
        NAME.to_string()
    }

    fn supports_languages(&self) -> Vec<Format> {
        vec![Format::Dot]
    }

    fn create_instance(&self) -> Result<Box<dyn KernelInstance>> {
        Ok(Box::new(GraphvizKernelInstance {}))
    }
}

#[derive(Default)]
pub struct GraphvizKernelInstance {}

impl GraphvizKernelInstance {}

#[async_trait]
impl KernelInstance for GraphvizKernelInstance {
    fn name(&self) -> String {
        NAME.to_string()
    }

    async fn execute(&mut self, code: &str) -> Result<(Vec<Node>, Vec<ExecutionMessage>)> {
        tracing::info!("Compiling Graphviz DOT to SVG");

        let mut parser = DotParser::new(code);
        match parser.process() {
            Ok(graph) => {
                tracing::info!("OK");
                let svg = if graph.list.list.is_empty() {
                    // Avoid panic if graph is empty by returning empty SVG
                    r#"<svg xmlns="http://www.w3.org/2000/svg"></svg>"#.to_string()
                } else {
                    let mut graph_builder = GraphBuilder::new();
                    graph_builder.visit_graph(&graph);
                    let mut visual_graph = graph_builder.get();

                    let mut svg_writer = SVGWriter::new();
                    visual_graph.do_it(false, false, false, &mut svg_writer);
                    let svg = svg_writer.finalize();

                    svg.replace(
                        "<?xml version=\"1.0\" encoding=\"UTF-8\" standalone=\"no\"?>",
                        "",
                    )
                };

                // Based on the recommendation for creating SVG DataURIs at
                // https://gist.github.com/jennyknuth/222825e315d45a738ed9d6e04c7a88d0
                let svg = svg
                    .replace('"', "\'")
                    .replace('%', "%25")
                    .replace('#', "%23")
                    .replace('{', "%7B")
                    .replace('}', "%7D")
                    .replace('<', "%3C")
                    .replace('>', "%3E");
                static SPACES_RE: Lazy<Regex> =
                    Lazy::new(|| Regex::new(r"\s+").expect("invalid regex"));
                let svg = SPACES_RE.replace_all(&svg, " ");
                let data_uri = format!("data:image/svg+xml,{}", svg);

                let image = Node::ImageObject(ImageObject {
                    content_url: data_uri,
                    ..Default::default()
                });

                Ok((vec![image], Vec::new()))
            }
            Err(error) => Ok((
                Vec::new(),
                vec![ExecutionMessage::new(MessageLevel::Exception, error)],
            )),
        }
    }

    async fn info(&mut self) -> Result<SoftwareApplication> {
        tracing::trace!("Getting Graphviz runtime info");

        Ok(SoftwareApplication {
            name: "Graphviz Kernel".to_string(),
            options: Box::new(SoftwareApplicationOptions {
                operating_system: Some(std::env::consts::OS.to_string()),
                ..Default::default()
            }),
            ..Default::default()
        })
    }
}

#[cfg(test)]
mod tests {
    use common_dev::pretty_assertions::assert_eq;
    use kernel::common::tokio;

    use super::*;

    #[tokio::test]
    async fn empty() -> Result<()> {
        let mut kernel = GraphvizKernelInstance {};

        let (outputs, messages) = kernel.execute("").await?;
        assert_eq!(messages[0].message, "Expected (graph|digraph)");
        assert!(outputs.is_empty());

        let (outputs, messages) = kernel.execute("graph{}").await?;
        assert_eq!(messages, vec![]);
        assert!(matches!(outputs[0], Node::ImageObject(..)));

        let (outputs, messages) = kernel.execute("digraph{}").await?;
        assert_eq!(messages, vec![]);
        assert!(matches!(outputs[0], Node::ImageObject(..)));

        Ok(())
    }

    #[tokio::test]
    async fn syntax_errors() -> Result<()> {
        let mut kernel = GraphvizKernelInstance {};

        let (outputs, messages) = kernel.execute("digraph { A -> B").await?;
        assert_eq!(messages[0].message, "Unknown token");
        assert!(outputs.is_empty());

        let (outputs, messages) = kernel.execute("digraph { A -> }").await?;
        assert_eq!(messages[0].message, "port");
        assert!(outputs.is_empty());

        let (outputs, messages) = kernel.execute("digraph { A - }").await?;
        assert_eq!(messages[0].message, "Unsupported token");
        assert!(outputs.is_empty());

        let (outputs, messages) = kernel
            .execute(
                "digraph {
    A -> B -
}
",
            )
            .await?;
        assert_eq!(messages[0].message, "Unknown token");
        assert!(outputs.is_empty());

        Ok(())
    }
}
