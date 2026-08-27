//! Weave the differences between two documents into one merged document
//!
//! Given two snapshots of a document, this crate produces a single [`Node`] that is
//! the *left* snapshot with every difference expressed as ordinary Stencila Schema
//! nodes: [`stencila_schema::SuggestionBlock`] and
//! [`stencila_schema::SuggestionInline`] for the differences that are substitutions of
//! content, and [`stencila_schema::Comment`] for those that are not.
//!
//! The point of producing a document rather than a rendering is that a document goes
//! everywhere. It encodes to any format — Stencila Markdown round-trips suggestions
//! and comments natively — it can be read in a browser as a tracked-changes view of
//! the left document, and every suggestion in it can be accepted or rejected with
//! `stencila-node-suggestions`.
//!
//! ```no_run
//! # use stencila_schema::Node;
//! # fn example(before: &Node, after: &Node) -> Result<(), Box<dyn std::error::Error>> {
//! let merged = stencila_node_merge::merge(before, after)?;
//! assert!(merged.report().is_complete());
//! # Ok(())
//! # }
//! ```
//!
//! # What it does not decide
//!
//! Which occurrences correspond is decided entirely by `stencila-node-compare`, and
//! nothing here can change it. This crate is a *planner*: it turns an already-derived
//! comparison into edits, and applies them with the schema's own patching machinery.
//! No matching, projection or difference logic is duplicated from the comparison.
//!
//! # What it cannot express
//!
//! Not every difference is a substitution of content. An occurrence that moved or was
//! reordered is paired on both sides, so there is no one-sided content to insert or
//! delete; a right-only list item has no slot that accepts a suggestion; a change to a
//! property under `authors` has no block or inline ancestor to replace. Those are
//! described in comments and recorded in [`MergeReport::unrepresentable`], which is
//! empty exactly when accepting every suggestion reproduces the right document.

mod apply;
mod comments;
mod container;
mod diff;
mod error;
mod index;
mod options;
mod plan;
mod report;

pub use error::{MergeError, MergeResult};
pub use options::{CommentMode, EditCoalescing, MergeOptions, MetadataChanges};
pub use report::{MergeReport, Unrepresentable, UnrepresentableReason};

use stencila_node_compare::{Comparison, compare_with_options};
use stencila_schema::Node;

use plan::MergePlan;

/// A merged document, and an account of what could not be expressed in it
pub struct Merged {
    node: Node,
    report: MergeReport,
}

impl Merged {
    /// The merged document
    pub fn node(&self) -> &Node {
        &self.node
    }

    /// Take the merged document
    pub fn into_node(self) -> Node {
        self.node
    }

    /// What the merge produced, and what it could not express
    pub fn report(&self) -> &MergeReport {
        &self.report
    }
}

/// Compare two nodes and merge them
///
/// The left node is the base: the merged document reads as the left one, with the
/// right one's changes proposed on top of it.
pub fn merge(left: &Node, right: &Node) -> MergeResult<Merged> {
    merge_with_options(left, right, &MergeOptions::default())
}

/// Compare two nodes and merge them, with options
pub fn merge_with_options(
    left: &Node,
    right: &Node,
    options: &MergeOptions,
) -> MergeResult<Merged> {
    let comparison = compare_with_options(left, right, &options.compare)?;
    merge_comparison_with_options(left, right, &comparison, options)
}

/// Merge two nodes using a comparison already derived from them
///
/// For a caller that has compared the two nodes already, so that they are not compared
/// twice. The comparison is validated against both snapshots first, because one
/// derived from different documents would address the wrong occurrences and silently
/// produce a wrong merge rather than an error.
pub fn merge_comparison(left: &Node, right: &Node, comparison: &Comparison) -> MergeResult<Merged> {
    merge_comparison_with_options(left, right, comparison, &MergeOptions::default())
}

/// Merge two nodes using a comparison already derived from them, with options
pub fn merge_comparison_with_options(
    left: &Node,
    right: &Node,
    comparison: &Comparison,
    options: &MergeOptions,
) -> MergeResult<Merged> {
    comparison.validate(left, right)?;

    let mut plan = MergePlan::build(comparison, options);

    let mut node = left.clone();

    // Before any rewrite, while the comparison's paths still address the document
    // they were derived from
    let locations = comments::identify_targets(&mut node, &plan.comments, options);

    // The report is taken out so that applying can add to it while reading the plan
    let mut report = std::mem::take(&mut plan.report);
    apply::apply(&mut node, right, &plan, options, &mut report)?;

    report.comments = comments::attach(&mut node, &plan.comments, &locations, options)?;

    Ok(Merged { node, report })
}
