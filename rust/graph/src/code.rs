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

pub use analyze::analyze_source;
pub(crate) use document::DocumentCodeIndex;
pub use facts::{CodeFacts, ColumnFact, WorkflowRuleFacts};
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
plot = df[["A", "D"]]
plt.savefig("plot.png")
"#,
        );

        assert!(facts.imports.contains("pandas"));
        assert!(facts.imports.contains("matplotlib"));
        assert!(facts.assignments.contains("df"));
        assert!(facts.reads.contains("data.csv"));
        assert!(facts.writes.contains("plot.png"));
        assert!(facts.columns.iter().any(|column| column.column == "A"));
        assert!(facts.columns.iter().any(|column| column.column == "D"));
    }

    #[test]
    fn extracts_r_facts() {
        let facts = analyze_source(
            CodeLanguage::R,
            r#"
library(readr)
df <- read.csv("input.csv")
df$A
write.csv(df, "output.csv")
"#,
        );

        assert!(facts.imports.contains("readr"));
        assert!(facts.assignments.contains("df"));
        assert!(facts.reads.contains("input.csv"));
        assert!(facts.writes.contains("output.csv"));
        assert!(facts.columns.iter().any(|column| column.column == "A"));
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

        assert!(facts.imports.contains("CSV"));
        assert!(facts.imports.contains("DataFrames"));
        assert!(!facts.imports.contains("df"));
        assert!(facts.assignments.contains("df"));
        assert!(facts.assignments.contains("total"));
        assert!(facts.declarations.contains("summarize"));
        assert!(facts.calls.contains("sum"));
        assert!(facts.reads.contains("data/input.csv"));
        assert!(facts.writes.contains("results/output.csv"));
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

        assert!(facts.imports.contains("stats"));
        assert!(facts.assignments.contains("tbl"));
        assert!(facts.assignments.contains("total"));
        assert!(facts.declarations.contains("summarize"));
        assert!(facts.calls.contains("sum"));
        assert!(facts.reads.contains("data/input.csv"));
        assert!(facts.writes.contains("results/output.csv"));
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
        assert!(facts.reads.contains("data.csv"));
        assert!(facts.writes.contains("plot.png"));
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

        assert!(facts.imports.contains("fs"));
        assert!(facts.imports.contains("simple-statistics"));
        assert!(facts.assignments.contains("data"));
        assert!(facts.declarations.contains("summarize"));
        assert!(facts.reads.contains("data/input.txt"));
        assert!(facts.writes.contains("results/output.txt"));
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

        assert!(facts.imports.contains("fs"));
        assert!(facts.assignments.contains("data"));
        assert!(facts.declarations.contains("summarize"));
        assert!(facts.reads.contains("data/input.txt"));
        assert!(facts.writes.contains("results/output.txt"));
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

        assert!(facts.imports.contains("serde"));
        assert!(facts.assignments.contains("data"));
        assert!(facts.declarations.contains("main"));
        assert!(facts.reads.contains("data/input.txt"));
        assert!(facts.writes.contains("results/output.txt"));
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
  script:
  """
  bwa mem ref.fa $reads > results/aligned.bam
  """
}
"#,
        );

        assert!(facts.workflow_rules.contains("align"));
        assert!(facts.declarations.contains("align"));
        assert!(facts.reads.contains("data/input.fq"));
        assert!(facts.writes.contains("results/aligned.bam"));
        assert!(!facts.writes.contains("results/${sample}.bai"));
        assert!(facts.calls.contains("script"));
        let rule = facts
            .workflow_rule_facts
            .get("align")
            .expect("align rule facts should be grouped");
        assert!(rule.reads.contains("data/input.fq"));
        assert!(rule.writes.contains("results/aligned.bam"));
    }

    #[test]
    fn skips_dynamic_paths_and_syntax_errors() {
        let dynamic = analyze_source(
            CodeLanguage::Python,
            "path = f\"data/{name}.csv\"\npd.read_csv(path)\n",
        );
        assert!(dynamic.reads.is_empty());

        let syntax = analyze_source(CodeLanguage::Python, "if (");
        assert!(syntax.syntax_error);
        assert!(syntax.reads.is_empty());
    }
}
