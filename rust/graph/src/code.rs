//! Static extraction of computation-oriented facts from source code.
//!
//! This module analyzes Python, R, Julia, MATLAB, JavaScript, TypeScript, Rust,
//! Snakemake, and Nextflow source using embedded ast-grep rules plus small language-specific
//! normalizers. It turns imports,
//! assignments, symbol uses, calls, static file paths, dataframe columns, and
//! workflow directives into graph nodes and edges.
//!
//! Code graph extraction is deliberately static and graph-only. It should make
//! dependency and provenance queries useful without executing user code,
//! mutating document nodes, or filling runtime execution metadata. That keeps
//! graph construction deterministic and safe for workspace inventory, while
//! still surfacing relationships such as a plot file deriving from a CSV column
//! or one document chunk depending on a symbol declared by an earlier chunk.
//!
//! The implementation has two phases. Rule extraction gathers language-neutral
//! facts from one source unit, and graph projection resolves those facts into
//! Schema graph nodes using either workspace file ids or scoped synthetic ids.
//! Keeping those phases separate lets tests exercise parser behavior directly
//! and lets workspace and document callers share the same normalization logic.

mod analyze;
mod document;
mod facts;
mod language;
mod normalize;
mod project;
mod scan;
mod util;
mod workspace;

pub use crate::package::PackageFact;
pub use analyze::analyze_source;
pub(crate) use document::DocumentCodeIndex;
pub use facts::{
    CodeFacts, ColumnFact, IoDirection, IoFact, IoMode, IoPath, VariableFlowFact, WorkflowRuleFacts,
};
pub use language::CodeLanguage;
pub(crate) use workspace::add_workspace_code;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_python_facts() {
        let facts = analyze_source(
            CodeLanguage::Python,
            r#"
import pandas as pd
import matplotlib.pyplot as plt
df = pd.read_csv("data.csv")
named = pd.read_csv(filepath_or_buffer="named.csv", sep=",")
# pd.read_csv(filepath_or_buffer="comment.csv")
text = 'pd.read_csv(filepath_or_buffer="string.csv")'
my_read_csv(filepath_or_buffer="helper.csv")
plot = df[["A", "D"]]
df.to_csv(path_or_buf="named-out.csv", index=False)
plt.savefig("plot.png")
"#,
        );

        assert!(facts.imports.contains(&PackageFact::new("pypi", "pandas")));
        assert!(
            facts
                .imports
                .contains(&PackageFact::new("pypi", "matplotlib"))
        );
        assert!(facts.assignments.contains("df"));
        assert_io(&facts, IoDirection::Read, "data.csv");
        assert_io(&facts, IoDirection::Read, "named.csv");
        assert_no_io(&facts, "comment.csv");
        assert_no_io(&facts, "string.csv");
        assert_no_io(&facts, "helper.csv");
        assert_io(&facts, IoDirection::Write, "named-out.csv");
        assert_io(&facts, IoDirection::Write, "plot.png");
        assert!(
            facts
                .variable_flows
                .iter()
                .any(|flow| flow.source == "df" && flow.target == "plot")
        );
        assert!(facts.columns.iter().any(|column| column.column == "A"));
        assert!(facts.columns.iter().any(|column| column.column == "D"));
    }

    #[test]
    fn extracts_scientific_python_io_facts() {
        let facts = analyze_source(
            CodeLanguage::Python,
            r#"
import imageio.v3 as iio
from pathlib import Path
import gzip
import numpy as np
import pandas as pd
import requests
import torch
import xarray as xr
from shutil import copyfile, copytree, move
from urllib.request import urlretrieve

table = pd.read_parquet("data/table.parquet")
fixed = pd.read_fwf("data/fixed-width.txt")
xml = pd.read_xml(path_or_buffer="data/table.xml")
spss = pd.read_spss(path="data/table.sav")
workbook = pd.read_excel(io="https://example.org/workbook.xlsx")
array = np.load("data/array.npy")
matrix = np.loadtxt(fname="data/matrix.tsv")
cube = xr.open_dataset(filename_or_obj="https://example.org/cube.nc")
image = iio.imread("images/source.tif")
response = requests.get("https://example.org/api.json")
text = Path("data/config.txt").read_text()
with gzip.open("data/archive.csv.gz", "rt") as handle:
    compressed = handle.read()
urlretrieve("https://example.org/raw.csv", "downloads/raw.csv")
copyfile("downloads/raw.csv", "copies/raw.csv")
copytree("templates/project", "build/project")
move("staging/report.csv", "reports/report.csv")

table.to_parquet("outputs/table.parquet")
table.to_excel(excel_writer="outputs/table.xlsx")
table.to_xml("outputs/table.xml")
table.to_latex(buf="outputs/table.tex")
table.to_markdown("outputs/table.md")
np.save("outputs/array.npy", array)
torch.save(image, "outputs/model.pt")
torch.save(image, "outputs/model=v1.pt")
cube.to_netcdf(path="outputs/cube.nc")
iio.imwrite("outputs/image.tif", image)
image.save("outputs/pillow.png")
Path("outputs/config.txt").write_text(text)
"#,
        );

        assert_io(&facts, IoDirection::Read, "data/table.parquet");
        assert_io(&facts, IoDirection::Read, "data/fixed-width.txt");
        assert_io(&facts, IoDirection::Read, "data/table.xml");
        assert_io(&facts, IoDirection::Read, "data/table.sav");
        assert_io(
            &facts,
            IoDirection::Read,
            "https://example.org/workbook.xlsx",
        );
        assert_io(&facts, IoDirection::Read, "data/array.npy");
        assert_io(&facts, IoDirection::Read, "data/matrix.tsv");
        assert_io(&facts, IoDirection::Read, "https://example.org/cube.nc");
        assert_io(&facts, IoDirection::Read, "images/source.tif");
        assert_io(&facts, IoDirection::Read, "https://example.org/api.json");
        assert_io(&facts, IoDirection::Read, "data/config.txt");
        assert_io(&facts, IoDirection::Read, "data/archive.csv.gz");
        assert_io(&facts, IoDirection::Read, "https://example.org/raw.csv");
        assert_io(&facts, IoDirection::Read, "downloads/raw.csv");
        assert_io(&facts, IoDirection::Read, "templates/project");
        assert_io(&facts, IoDirection::Read, "staging/report.csv");
        assert_io(&facts, IoDirection::Write, "downloads/raw.csv");
        assert_io(&facts, IoDirection::Write, "copies/raw.csv");
        assert_io(&facts, IoDirection::Write, "build/project");
        assert_io(&facts, IoDirection::Write, "reports/report.csv");
        assert_io(&facts, IoDirection::Write, "outputs/table.parquet");
        assert_io(&facts, IoDirection::Write, "outputs/table.xlsx");
        assert_io(&facts, IoDirection::Write, "outputs/table.xml");
        assert_io(&facts, IoDirection::Write, "outputs/table.tex");
        assert_io(&facts, IoDirection::Write, "outputs/table.md");
        assert_io(&facts, IoDirection::Write, "outputs/array.npy");
        assert_io(&facts, IoDirection::Write, "outputs/model.pt");
        assert_io(&facts, IoDirection::Write, "outputs/model=v1.pt");
        assert_io(&facts, IoDirection::Write, "outputs/cube.nc");
        assert_io(&facts, IoDirection::Write, "outputs/image.tif");
        assert_io(&facts, IoDirection::Write, "outputs/pillow.png");
        assert_io(&facts, IoDirection::Write, "outputs/config.txt");
        assert_no_io(&facts, "image");
    }

    #[test]
    fn extracts_r_facts() {
        let facts = analyze_source(
            CodeLanguage::R,
            r#"
library(readr)
df <- read.csv("input.csv")
named <- read.csv(file = "named.csv", sep = ",")
# read.csv(file = "comment-r.csv")
text <- 'read.csv(file = "string-r.csv")'
my_read.csv(file = "helper-r.csv")
df$A
write.csv(df, file = "named-output.csv", row.names = FALSE)
write.csv(df, "output.csv")
"#,
        );

        assert!(facts.imports.contains(&PackageFact::new("cran", "readr")));
        assert!(facts.assignments.contains("df"));
        assert_io(&facts, IoDirection::Read, "input.csv");
        assert_io(&facts, IoDirection::Read, "named.csv");
        assert_no_io(&facts, "comment-r.csv");
        assert_no_io(&facts, "string-r.csv");
        assert_no_io(&facts, "helper-r.csv");
        assert_io(&facts, IoDirection::Write, "named-output.csv");
        assert_io(&facts, IoDirection::Write, "output.csv");
        assert!(facts.columns.iter().any(|column| column.column == "A"));
    }

    #[test]
    fn preserves_r_dotted_variable_flow_names() {
        let facts = analyze_source(
            CodeLanguage::R,
            r#"
raw.data <- read.csv("input.csv")
clean.data <- raw.data
write.csv(clean.data, "output.csv")
"#,
        );

        assert!(
            facts
                .variable_flows
                .iter()
                .any(|flow| flow.source == "raw.data" && flow.target == "clean.data")
        );
        assert!(facts.variable_flows.iter().all(|flow| flow.source != "raw"));
    }

    #[test]
    fn extracts_julia_facts() {
        let facts = analyze_source(
            CodeLanguage::Julia,
            r#"
using CSV
using DataFrames

df = CSV.read("data/input.csv", DataFrame)
total = sum(df.count)

function summarize(values)
    return sum(values)
end

CSV.write("results/output.csv", df)
"#,
        );

        assert!(facts.imports.contains(&PackageFact::new("julia", "CSV")));
        assert!(
            facts
                .imports
                .contains(&PackageFact::new("julia", "DataFrames"))
        );
        assert!(!facts.imports.contains(&PackageFact::new("julia", "df")));
        assert!(facts.assignments.contains("df"));
        assert!(facts.assignments.contains("total"));
        assert!(facts.declarations.contains("summarize"));
        assert!(facts.calls.contains("sum"));
        assert_io(&facts, IoDirection::Read, "data/input.csv");
        assert_io(&facts, IoDirection::Write, "results/output.csv");
        assert!(facts.columns.iter().any(|column| column.column == "count"));
    }

    #[test]
    fn extracts_matlab_facts() {
        let facts = analyze_source(
            CodeLanguage::Matlab,
            r#"
import stats.*

tbl = readtable("data/input.csv");
total = sum(tbl.count);

function result = summarize(values)
    result = sum(values);
end

writetable(tbl, "results/output.csv");
"#,
        );

        assert!(facts.imports.contains(&PackageFact::new("matlab", "stats")));
        assert!(facts.assignments.contains("tbl"));
        assert!(facts.assignments.contains("total"));
        assert!(facts.declarations.contains("summarize"));
        assert!(facts.calls.contains("sum"));
        assert_io(&facts, IoDirection::Read, "data/input.csv");
        assert_io(&facts, IoDirection::Write, "results/output.csv");
        assert!(facts.columns.iter().any(|column| column.column == "count"));
    }

    #[test]
    fn extracts_snakemake_facts() {
        let facts = analyze_source(
            CodeLanguage::Snakemake,
            r#"
rule plot:
    input: "data.csv"
    output: "plot.png"
    script: "scripts/plot.py"
"#,
        );

        assert!(facts.workflow_rules.contains("plot"));
        assert_io(&facts, IoDirection::Read, "data.csv");
        assert_io(&facts, IoDirection::Write, "plot.png");
        assert!(facts.script_links.contains("scripts/plot.py"));
    }

    #[test]
    fn extracts_javascript_facts() {
        let facts = analyze_source(
            CodeLanguage::JavaScript,
            r#"
import fs from "fs"
const stats = require("simple-statistics")
const data = readFileSync("data/input.txt")
fs.writeFileSync("results/output.txt", data)
function summarize() {
  return data
}
"#,
        );

        assert!(facts.imports.contains(&PackageFact::new("node", "fs")));
        assert!(
            facts
                .imports
                .contains(&PackageFact::new("npm", "simple-statistics"))
        );
        assert!(facts.assignments.contains("data"));
        assert!(facts.declarations.contains("summarize"));
        assert_io(&facts, IoDirection::Read, "data/input.txt");
        assert_io(&facts, IoDirection::Write, "results/output.txt");
        assert!(
            facts
                .io
                .iter()
                .any(|fact| fact.value.as_deref() == Some("data"))
        );
    }

    #[test]
    fn excludes_imported_aliases_from_variable_lineage() -> eyre::Result<()> {
        let facts = analyze_source(
            CodeLanguage::JavaScript,
            r#"
const stats = require("simple-statistics")
const data = readFileSync("data/input.txt")
const summary = stats.mean(data)
writeFileSync("results/output.txt", summary)
"#,
        );
        let graph = project_test_graph("analysis.js", CodeLanguage::JavaScript, &facts)?;

        assert!(!has_graph_edge(
            &graph,
            "symbol:analysis.js:javascript:stats",
            "symbol:analysis.js:javascript:summary",
            stencila_schema::GraphEdgeKind::DerivedInto
        ));
        assert!(has_graph_edge(
            &graph,
            "symbol:analysis.js:javascript:data",
            "symbol:analysis.js:javascript:summary",
            stencila_schema::GraphEdgeKind::DerivedInto
        ));

        Ok(())
    }

    #[test]
    fn keeps_coarse_lineage_when_precise_chain_is_partial() -> eyre::Result<()> {
        let facts = CodeFacts {
            assignments: std::collections::BTreeSet::from(["df".to_string()]),
            io: std::collections::BTreeSet::from([
                IoFact {
                    direction: IoDirection::Read,
                    path: IoPath::Static("input.csv".to_string()),
                    operation_offset: None,
                    target: None,
                    target_offset: None,
                    value: None,
                    value_offset: None,
                    function: None,
                    mode: None,
                },
                IoFact {
                    direction: IoDirection::Write,
                    path: IoPath::Static("output.csv".to_string()),
                    operation_offset: None,
                    target: None,
                    target_offset: None,
                    value: Some("df".to_string()),
                    value_offset: None,
                    function: None,
                    mode: None,
                },
            ]),
            ..Default::default()
        };
        let graph = project_test_graph("partial.py", CodeLanguage::Python, &facts)?;

        assert!(has_graph_edge(
            &graph,
            "code-file:partial.py:input.csv",
            "code-file:partial.py:output.csv",
            stencila_schema::GraphEdgeKind::DerivedInto
        ));
        assert!(has_graph_edge(
            &graph,
            "symbol:partial.py:python:df",
            "code-file:partial.py:output.csv",
            stencila_schema::GraphEdgeKind::DerivedInto
        ));

        Ok(())
    }

    #[test]
    fn ignores_later_reads_for_earlier_writes() -> eyre::Result<()> {
        let facts = analyze_source(
            CodeLanguage::Python,
            r#"
clean.to_csv("output.csv")
clean = read_csv("input.csv")
"#,
        );
        let graph = project_test_graph("ordering.py", CodeLanguage::Python, &facts)?;

        assert!(!has_graph_edge(
            &graph,
            "code-file:ordering.py:input.csv",
            "code-file:ordering.py:output.csv",
            stencila_schema::GraphEdgeKind::DerivedInto
        ));
        assert!(!has_graph_edge(
            &graph,
            "symbol:ordering.py:python:clean",
            "code-file:ordering.py:output.csv",
            stencila_schema::GraphEdgeKind::DerivedInto
        ));
        assert!(has_graph_edge(
            &graph,
            "code-file:ordering.py:input.csv",
            "symbol:ordering.py:python:clean",
            stencila_schema::GraphEdgeKind::DerivedInto
        ));

        Ok(())
    }

    #[test]
    fn ignores_later_definitions_for_earlier_variable_flows() -> eyre::Result<()> {
        let facts = analyze_source(
            CodeLanguage::Python,
            r#"
summary = clean
clean = read_csv("input.csv")
summary.to_csv("output.csv")
"#,
        );
        let graph = project_test_graph("flow-order.py", CodeLanguage::Python, &facts)?;

        assert!(!has_graph_edge(
            &graph,
            "symbol:flow-order.py:python:clean",
            "symbol:flow-order.py:python:summary",
            stencila_schema::GraphEdgeKind::DerivedInto
        ));
        assert!(has_graph_edge(
            &graph,
            "symbol:flow-order.py:python:summary",
            "code-file:flow-order.py:output.csv",
            stencila_schema::GraphEdgeKind::DerivedInto
        ));

        Ok(())
    }

    #[test]
    fn extracts_typescript_facts() {
        let facts = analyze_source(
            CodeLanguage::TypeScript,
            r#"
import { readFileSync, writeFileSync } from "node:fs"
const data = readFileSync("data/input.txt")
writeFileSync("results/output.txt", data)
const summarize = () => data
"#,
        );

        assert!(facts.imports.contains(&PackageFact::new("node", "fs")));
        assert!(facts.assignments.contains("data"));
        assert!(facts.declarations.contains("summarize"));
        assert_io(&facts, IoDirection::Read, "data/input.txt");
        assert_io(&facts, IoDirection::Write, "results/output.txt");
        assert!(
            facts
                .io
                .iter()
                .any(|fact| fact.value.as_deref() == Some("data"))
        );
    }

    #[test]
    fn extracts_rust_facts() {
        let facts = analyze_source(
            CodeLanguage::Rust,
            r#"
use serde::Serialize;

fn main() {
    let data = std::fs::read_to_string("data/input.txt");
    std::fs::write("results/output.txt", data);
}
"#,
        );

        assert!(facts.imports.contains(&PackageFact::new("cargo", "serde")));
        assert!(facts.assignments.contains("data"));
        assert!(facts.declarations.contains("main"));
        assert_io(&facts, IoDirection::Read, "data/input.txt");
        assert_io(&facts, IoDirection::Write, "results/output.txt");
        assert!(
            facts
                .io
                .iter()
                .any(|fact| fact.value.as_deref() == Some("data"))
        );
    }

    #[test]
    fn extracts_nextflow_facts() {
        let facts = analyze_source(
            CodeLanguage::Nextflow,
            r#"
process align {
  input:
  path "data/input.fq"
  output:
  path "results/aligned.bam"
  path "results/${sample}.bai"
  path "results/$sample.idx"
  script:
  """
  bwa mem ref.fa $reads > results/aligned.bam
  """
}
"#,
        );

        assert!(facts.workflow_rules.contains("align"));
        assert!(facts.declarations.contains("align"));
        assert_io(&facts, IoDirection::Read, "data/input.fq");
        assert_io(&facts, IoDirection::Write, "results/aligned.bam");
        assert_template_io(&facts, IoDirection::Write, "results/${sample}.bai");
        assert_template_io(&facts, IoDirection::Write, "results/$sample.idx");
        assert!(facts.calls.contains("script"));
        let rule = facts
            .workflow_rule_facts
            .get("align")
            .expect("align rule facts should be grouped");
        assert_rule_io(rule, IoDirection::Read, "data/input.fq");
        assert_rule_io(rule, IoDirection::Write, "results/aligned.bam");
    }

    #[test]
    fn skips_dynamic_paths_and_syntax_errors() {
        let dynamic = analyze_source(
            CodeLanguage::Python,
            "path = f\"data/{name}.csv\"\npd.read_csv(path)\n",
        );
        assert!(dynamic.io.iter().any(|fact| {
            fact.direction == IoDirection::Read
                && matches!(fact.path, IoPath::Unknown(ref path) if path == "path")
        }));

        let template = analyze_source(CodeLanguage::Python, "pd.read_csv(f\"data/{name}.csv\")\n");
        assert_template_io(&template, IoDirection::Read, "data/{name}.csv");

        let syntax = analyze_source(CodeLanguage::Python, "if (");
        assert!(syntax.syntax_error);
        assert!(syntax.io.is_empty());
    }

    fn assert_no_io(facts: &CodeFacts, path: &str) {
        assert!(
            facts.io.iter().all(|fact| fact.path.value() != path),
            "unexpected I/O fact for {path}"
        );
    }

    fn assert_io(facts: &CodeFacts, direction: IoDirection, path: &str) {
        assert!(
            facts.io.iter().any(|fact| {
                fact.direction == direction
                    && matches!(fact.path, IoPath::Static(ref value) if value == path)
            }),
            "missing {direction:?} I/O fact for {path}"
        );
    }

    fn assert_template_io(facts: &CodeFacts, direction: IoDirection, path: &str) {
        assert!(
            facts.io.iter().any(|fact| {
                fact.direction == direction
                    && matches!(fact.path, IoPath::Template(ref value) if value == path)
            }),
            "missing template {direction:?} I/O fact for {path}"
        );
    }

    fn assert_rule_io(rule: &WorkflowRuleFacts, direction: IoDirection, path: &str) {
        assert!(
            rule.io.iter().any(|fact| {
                fact.direction == direction
                    && matches!(fact.path, IoPath::Static(ref value) if value == path)
            }),
            "missing rule {direction:?} I/O fact for {path}"
        );
    }

    fn project_test_graph(
        scope: &str,
        language: CodeLanguage,
        facts: &CodeFacts,
    ) -> eyre::Result<stencila_schema::Graph> {
        let mut builder = crate::GraphBuilder::new("fixture:code-test");
        let unit_id = format!("code:{scope}");
        builder.add_schema_node(
            unit_id.clone(),
            stencila_schema::Node::String(unit_id.clone()),
        );
        super::project::add_code_facts_to_graph(
            &mut builder,
            &unit_id,
            scope,
            language,
            facts,
            0,
            None,
        );
        builder.build()
    }

    fn has_graph_edge(
        graph: &stencila_schema::Graph,
        source: &str,
        target: &str,
        kind: stencila_schema::GraphEdgeKind,
    ) -> bool {
        graph
            .edges
            .iter()
            .any(|edge| edge.source == source && edge.target == target && edge.kind == kind)
    }
}
