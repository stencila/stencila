use std::collections::BTreeSet;

use super::super::{
    facts::{CodeFacts, ColumnFact},
    language::CodeLanguage,
    util::{collect_string_literals, first_identifier},
};

/// Collect dataframe column facts for known dataframe-like variables.
///
/// Column extraction runs after IO normalization because it relies on
/// `variable_sources` seeded by static read calls. This lets the graph say that
/// `df["A"]` used column `A` from `data.csv` without analyzing the full
/// dataframe pipeline.
pub(in crate::code) fn collect_column_facts(
    language: CodeLanguage,
    source: &str,
    facts: &mut CodeFacts,
) {
    let dataframes = facts
        .variable_sources
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    for dataframe in dataframes {
        let source_path = facts.variable_sources.get(&dataframe).cloned();
        let columns = match language {
            CodeLanguage::Python => python_columns(source, &dataframe),
            CodeLanguage::R => r_columns(source, &dataframe),
            CodeLanguage::Julia => julia_columns(source, &dataframe),
            CodeLanguage::JavaScript
            | CodeLanguage::TypeScript
            | CodeLanguage::Rust
            | CodeLanguage::Snakemake
            | CodeLanguage::Nextflow => BTreeSet::new(),
        };
        for column in columns {
            facts.columns.insert(ColumnFact {
                dataframe: dataframe.clone(),
                column,
                source: source_path.clone(),
            });
        }
    }
}

/// Extract simple Python dataframe column literals.
///
/// This intentionally uses source scanning around recognized dataframe markers
/// rather than deeper semantic analysis. It catches common `df["A"]`,
/// `df[["A", "B"]]`, and `df.loc[..., "A"]` forms while ignoring computed
/// column expressions.
fn python_columns(source: &str, dataframe: &str) -> BTreeSet<String> {
    let mut columns = BTreeSet::new();
    for marker in [format!("{dataframe}["), format!("{dataframe}.loc[")] {
        for segment in source
            .match_indices(&marker)
            .map(|(index, _)| &source[index + marker.len()..])
        {
            let Some(end) = segment.find(']') else {
                continue;
            };
            collect_string_literals(&segment[..end], &mut columns);
        }
    }
    columns
}

/// Extract simple R dataframe column literals.
///
/// R column access is common through both `$` and `[[...]]`. Capturing only
/// identifier or string-literal columns keeps this pass deterministic and avoids
/// guessing at expressions such as computed names.
fn r_columns(source: &str, dataframe: &str) -> BTreeSet<String> {
    let mut columns = BTreeSet::new();
    let dollar_marker = format!("{dataframe}$");
    for segment in source
        .match_indices(&dollar_marker)
        .map(|(index, _)| &source[index + dollar_marker.len()..])
    {
        if let Some(column) = first_identifier(segment) {
            columns.insert(column.to_string());
        }
    }
    let bracket_marker = format!("{dataframe}[[");
    for segment in source
        .match_indices(&bracket_marker)
        .map(|(index, _)| &source[index + bracket_marker.len()..])
    {
        let Some(end) = segment.find("]]") else {
            continue;
        };
        collect_string_literals(&segment[..end], &mut columns);
    }
    columns
}

/// Extract simple Julia dataframe column literals.
///
/// DataFrames.jl commonly uses `df.column`, `df[!, "column"]`, and
/// `df[:, "column"]`. This mirrors the narrow static approach used for Python
/// and R: only literal or identifier columns tied to known dataframe variables
/// are represented.
fn julia_columns(source: &str, dataframe: &str) -> BTreeSet<String> {
    let mut columns = BTreeSet::new();
    let dot_marker = format!("{dataframe}.");
    for segment in source
        .match_indices(&dot_marker)
        .map(|(index, _)| &source[index + dot_marker.len()..])
    {
        if let Some(column) = first_identifier(segment) {
            columns.insert(column.to_string());
        }
    }
    let bracket_marker = format!("{dataframe}[");
    for segment in source
        .match_indices(&bracket_marker)
        .map(|(index, _)| &source[index + bracket_marker.len()..])
    {
        let Some(end) = segment.find(']') else {
            continue;
        };
        collect_string_literals(&segment[..end], &mut columns);
    }
    columns
}
