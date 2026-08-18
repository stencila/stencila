//! Exposes semantic comparison of documents provided by the Rust node-compare crates
//!
//! Comparison is CPU-bound and wholly synchronous, so these bindings are synchronous
//! too: they release the interpreter lock and, where an input still has to be decoded,
//! drive the asynchronous codecs on the shared runtime. That keeps the module usable
//! from an ordinary script or notebook without an event loop.
//!
//! Every entry point returns one JSON envelope carrying the comparison and any reports
//! that were asked for. Reports are rendered here, while the decoded nodes are still in
//! hand, because a report reads one-sided content back out of them and the Python layer
//! never sees those nodes.

use std::{path::PathBuf, str::FromStr};

use pyo3::prelude::*;
use serde::Serialize;

use stencila_codecs::{DecodeOptions, Format, LossesResponse};
use stencila_node_compare::{
    CompareOptions, Comparison, DifferenceFilter, Selector, compare_with_options,
};
use stencila_node_compare_report::{Snapshot, html_report, text_report};
use stencila_schema::Node;

use crate::utilities::{from_json, runtime, runtime_error, to_json, value_error};

pub fn module(stencila: &Bound<'_, PyModule>) -> PyResult<()> {
    let compare = PyModule::new(stencila.py(), "compare")?;

    compare.add_function(wrap_pyfunction!(nodes, &compare)?)?;
    compare.add_function(wrap_pyfunction!(strings, &compare)?)?;
    compare.add_function(wrap_pyfunction!(paths, &compare)?)?;
    compare.add_function(wrap_pyfunction!(is_equal, &compare)?)?;

    stencila.add("compare", compare)
}

/// Comparison options
#[derive(FromPyObject)]
struct Options {
    /// The maximum number of candidate cells that sequence alignment may use
    #[pyo3(item)]
    alignment_cell_budget: Option<usize>,

    /// Selectors for differences to report
    #[pyo3(item)]
    include: Option<Vec<String>>,

    /// Selectors for differences not to report
    #[pyo3(item)]
    exclude: Option<Vec<String>>,

    /// The format of the left input
    #[pyo3(item)]
    left_format: Option<String>,

    /// The format of the right input
    #[pyo3(item)]
    right_format: Option<String>,

    /// What to call the left side in a report
    #[pyo3(item)]
    left_label: Option<String>,

    /// What to call the right side in a report
    #[pyo3(item)]
    right_label: Option<String>,

    /// Which reports to render: any of "text" and "html"
    #[pyo3(item)]
    reports: Option<Vec<String>>,

    /// Whether reports should stop after the counts
    #[pyo3(item)]
    summary: Option<bool>,
}

impl Options {
    /// The comparison options these options ask for
    ///
    /// Selectors are parsed here, before any input is decoded, so that a misspelled node
    /// type or property is reported as a bad argument rather than as a comparison
    /// failure.
    fn compare_options(&self) -> PyResult<CompareOptions> {
        let mut options = CompareOptions::default();

        if let Some(budget) = self.alignment_cell_budget {
            if budget == 0 {
                return Err(value_error(eyre::eyre!(
                    "`alignment_cell_budget` must be greater than zero"
                )));
            }
            options.alignment_cell_budget = budget;
        }

        options.filter = DifferenceFilter {
            include: selectors(self.include.as_deref())?,
            exclude: selectors(self.exclude.as_deref())?,
        };

        Ok(options)
    }

    /// The decoding options for one side
    ///
    /// Deliberately does not record where a snapshot came from: decoding otherwise
    /// stamps the repository, path and commit of the source onto the document, which
    /// would make two copies of the same content differ merely by being two files.
    /// Losses are ignored rather than reported, because there is no terminal to report
    /// them to.
    fn decode_options(format: Option<&String>) -> DecodeOptions {
        DecodeOptions {
            format: format.map(|format| Format::from_name(format)),
            reproducible: Some(false),
            losses: LossesResponse::Ignore,
            ..Default::default()
        }
    }

    /// Whether a report of a kind was asked for
    fn wants(&self, report: &str) -> bool {
        self.reports
            .as_ref()
            .is_some_and(|reports| reports.iter().any(|wanted| wanted == report))
    }
}

/// Parse filter selectors
fn selectors(texts: Option<&[String]>) -> PyResult<Vec<Selector>> {
    texts
        .unwrap_or_default()
        .iter()
        .map(|text| {
            Selector::from_str(text).map_err(|error| value_error(eyre::eyre!("{error}")))
        })
        .collect()
}

/// The result of a comparison, and any reports that were asked for
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Envelope {
    /// The comparison artifact, unchanged
    comparison: Comparison,

    /// The human-readable report, when it was asked for
    #[serde(skip_serializing_if = "Option::is_none")]
    text: Option<String>,

    /// The side-by-side report, when it was asked for
    #[serde(skip_serializing_if = "Option::is_none")]
    html: Option<String>,
}

/// Compare two nodes and render any reports that were asked for
///
/// The one place a comparison is made, so that every entry point reports the same way
/// whatever its inputs were.
fn run(left: &Node, right: &Node, options: &Options) -> PyResult<String> {
    let comparison = compare_with_options(left, right, &options.compare_options()?)
        .map_err(|error| runtime_error(eyre::eyre!("{error}")))?;

    let (text, html) = if options.wants("text") || options.wants("html") {
        let left = Snapshot {
            node: left,
            label: options.left_label.as_deref().unwrap_or("left"),
        };
        let right = Snapshot {
            node: right,
            label: options.right_label.as_deref().unwrap_or("right"),
        };
        let summary = options.summary.unwrap_or_default();

        (
            options
                .wants("text")
                .then(|| text_report(&comparison, left, right, summary))
                .transpose()
                .map_err(runtime_error)?,
            options
                .wants("html")
                .then(|| html_report(&comparison, left, right, summary))
                .transpose()
                .map_err(runtime_error)?,
        )
    } else {
        (None, None)
    };

    to_json(&Envelope {
        comparison,
        text,
        html,
    })
}

/// Compare two Stencila Schema nodes
#[pyfunction]
fn nodes(py: Python, left: String, right: String, options: Options) -> PyResult<String> {
    py.detach(move || {
        let left: Node = from_json(&left)?;
        let right: Node = from_json(&right)?;

        run(&left, &right, &options)
    })
}

/// Compare two documents in strings
#[pyfunction]
fn strings(py: Python, left: String, right: String, options: Options) -> PyResult<String> {
    py.detach(move || {
        let (left, right) = runtime().block_on(async {
            let left = stencila_codecs::from_str(
                &left,
                Some(Options::decode_options(options.left_format.as_ref())),
            )
            .await?;
            let right = stencila_codecs::from_str(
                &right,
                Some(Options::decode_options(options.right_format.as_ref())),
            )
            .await?;

            Ok::<_, eyre::Report>((left, right))
        })
        .map_err(runtime_error)?;

        run(&left, &right, &options)
    })
}

/// Compare two documents at file system paths
///
/// Unless the caller says otherwise, each side is labelled with its path, which is what
/// a report of a comparison of two files should say.
#[pyfunction]
fn paths(py: Python, left: String, right: String, mut options: Options) -> PyResult<String> {
    options.left_label.get_or_insert_with(|| left.clone());
    options.right_label.get_or_insert_with(|| right.clone());

    py.detach(move || {
        let (left, right) = runtime().block_on(async {
            let left = stencila_codecs::from_path(
                &PathBuf::from(&left),
                Some(Options::decode_options(options.left_format.as_ref())),
            )
            .await?;
            let right = stencila_codecs::from_path(
                &PathBuf::from(&right),
                Some(Options::decode_options(options.right_format.as_ref())),
            )
            .await?;

            Ok::<_, eyre::Report>((left, right))
        })
        .map_err(runtime_error)?;

        run(&left, &right, &options)
    })
}

/// Whether a comparison found the two documents equal
///
/// Answered by the comparison crate rather than by reading the artifact, because
/// equality is not just an absence of differences: a document with a whole subtree that
/// the other lacks has no differences to report about it, only a one-sided
/// correspondence. Takes a serialized comparison so that an artifact read back from a
/// file can be asked the same question.
#[pyfunction]
fn is_equal(py: Python, comparison: String) -> PyResult<bool> {
    py.detach(move || {
        let comparison: Comparison = from_json(&comparison)?;

        Ok(comparison.is_equal())
    })
}
