use std::fmt::{self, Debug};

use common::similar::algorithms::Replace;
use common::similar::{
    algorithms::{diff_slices, Algorithm, Capture},
    DiffOp, DiffTag,
};

use schema::{CondenseContext, CondenseProperty, Node, NodePath, NodeSlot, PatchNode};

use crate::patch::NodeOp;

/// Diff two nodes
///
/// Calculates the diff operations necessary to make `old` the
/// same as the `new` node. Does so by condensing each node from a tree
/// of properties into a list of diff-able properties. Which properties
/// are treated as diff-able will be dependent on the format from which
/// the `new` node was decoded. Presently however, only one format,
/// Markdown is considered.
pub fn diff(old: &Node, new: &Node) -> DiffResult {
    // Condense each node into diffable properties
    let mut old_context = CondenseContext::new();
    old.condense(&mut old_context);

    let mut new_context = CondenseContext::new();
    new.condense(&mut new_context);

    old_context
        .properties
        .retain(|prop| matches!(prop.slot, NodeSlot::Property(..)));
    new_context
        .properties
        .retain(|prop| matches!(prop.slot, NodeSlot::Property(..)));

    let old_properties = diffable_properties(&old_context);
    let new_properties = diffable_properties(&new_context);

    // Calculate diff operations between te two sets of properties
    let mut diff_hook = Replace::new(Capture::new()); //, &old_properties, &new_properties);
    diff_slices(
        Algorithm::Patience,
        &mut diff_hook,
        &old_properties,
        &new_properties,
    )
    .unwrap();

    let diff_ops = diff_hook.into_inner().into_ops();
    let patch_ops = build_patches(&old_context, &new_context, &diff_ops);

    DiffResult {
        #[cfg(debug_assertions)]
        old_context,
        #[cfg(debug_assertions)]
        new_context,
        diff_ops,
        node_ops: patch_ops,
    }
}

fn build_patches(
    old_context: &CondenseContext,
    new_context: &CondenseContext,
    diff_ops: &Vec<DiffOp>,
) -> Vec<NodeOp> {
    let mut patch_ops: Vec<NodeOp> = Vec::new();

    for (i, op) in diff_ops.iter().enumerate() {
        let (op, old, new) = op.as_tag_tuple();
        if op == DiffTag::Insert {
            let prop_slice = &new_context.properties[new.start..new.end];

            // Find an insertion point.
            let insertion = old.start;
            if insertion > 0 {
                let prior = &old_context.properties[insertion - 1];
                let next = &prop_slice[0];
                // Walk until we find common ancestor
                let mut insert_path = NodePath::new();
                for (old, new) in prior.path.iter().zip(next.path.iter()) {
                    insert_path.push_back(old.clone());
                    if old != new {
                        break;
                    }
                }
                for prop in prop_slice {
                    if let NodeSlot::Property(..) = prop.slot {
                        patch_ops.push(NodeOp::Insert((insert_path.clone(), prop.path.clone())));
                    }
                }
            }

            //     for prop in prop_slice {
            //         if let NodeSlot::Property(..) = prop.slot {
            //             patch_ops.push(NodeOp::Insert((insertion.path.clone(), prop.path.clone())));
            //         }
            //     }
            // }
        }
    }
    patch_ops
}

// fn count_enter_exit(slice: &[CondenseProperty]) -> (usize, usize) {
//     slice
//         .iter()
//         .fold((0, 0), |(count_a, count_b), prop| match prop.slot {
//             NodeSlot::Enter(..) => (count_a + 1, count_b),
//             NodeSlot::Exit(..) => (count_a, count_b + 1),
//             _ => (count_a, count_b),
//         })
// }

/// Get the properties as a diff-able tuple of (slot, value)
///
/// This excludes the ancestry and path of a property since they should
/// not be considered in the diffing (although both are used for creating
/// patches from the diff operations)
fn diffable_properties(context: &CondenseContext) -> Vec<(NodePath, &Option<String>)> {
    context
        .properties
        .iter()
        .map(|node| (node.path.remove_indexes(), &node.value))
        .collect()
}

/// The result from a diff operation
///
/// During development the result includes the two condense contexts and diff ops
/// in addition to the generated patch ops. This is mostly done so that we can use them when creating
/// snapshots tests to understand the algorithm
pub struct DiffResult {
    #[cfg(debug_assertions)]
    old_context: CondenseContext,

    #[cfg(debug_assertions)]
    new_context: CondenseContext,

    #[cfg(debug_assertions)]
    diff_ops: Vec<DiffOp>,

    node_ops: Vec<NodeOp>,
}

impl DiffResult {
    /// Get the operations in the result
    pub fn ops(self) -> Vec<NodeOp> {
        self.node_ops
    }
}

/// Display the diff result as three sets of tables
///
/// Intended only for testing and debugging during development.
#[cfg(debug_assertions)]
impl Debug for DiffResult {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.old_context.fmt(f)?;
        writeln!(f)?;
        self.new_context.fmt(f)?;
        writeln!(f)?;

        writeln!(f, "DiffOp       Old range    New range")?;
        for op in &self.diff_ops {
            let (tag, old_range, new_range) = op.as_tag_tuple();
            writeln!(
                f,
                "{:<10}   {}..{}         {}..{}",
                format!("{tag:?}"),
                old_range.start,
                old_range.end,
                new_range.start,
                new_range.end
            )?;
        }

        writeln!(f)?;
        for op in &self.node_ops {
            writeln!(f, "{op:?}")?;
        }

        Ok(())
    }
}
