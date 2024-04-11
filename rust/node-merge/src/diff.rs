use std::fmt::{self, Debug};

use common::similar::{
    algorithms::{diff_slices, Algorithm, Capture, Compact},
    DiffOp, DiffTag,
};

use schema::{CondenseContext, Node, NodePath, NodeSlot, PatchNode};

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

    let old_properties = diffable_properties(&old_context);
    let new_properties = diffable_properties(&new_context);

    // Calculate diff operations between te two sets of properties
    let mut diff_hook = Compact::new(Capture::new(), &old_properties, &new_properties);
    diff_slices(
        Algorithm::Patience,
        &mut diff_hook,
        &old_properties,
        &new_properties,
    )
    .unwrap();
    let diff_ops = diff_hook.into_inner().into_ops();

    let mut patch_ops: Vec<NodeOp> = Vec::new();
    for op in &diff_ops {
        let (op, old, new) = op.as_tag_tuple();
        match op {
            DiffTag::Insert => {
                // We need to look at the prior node to insertion.
                if old.start == 0 {
                    //
                    // patch_ops.push(NodeOp::Add((new_context.properties[pth, node)));
                    todo!("Handle insert at start")
                } else {
                    let mut prior_pth = old_context.properties[old.start - 1].path.clone();
                    for i in new {
                        let next_pth = new_context.properties[i].path.clone();
                        let pth = path_for_next(&prior_pth, &next_pth);
                        patch_ops.push(NodeOp::Set((pth, next_pth.clone())));
                        prior_pth = next_pth;
                    }
                }

                // for i in new {
                //     let pth = new_context.properties[i].path.clone();
                //     let node = new_context.properties[i].value.clone();
                //     patch_ops.push(NodeOp::Add((pth, node)));
                // }
                //
                // patch_ops.push(NodeOp::Add(
                //     NodePath(vec![NodeSlot::Index(*old_index)]),
                //     new[*new_index..*new_index + *new_len].join("\n"),
                // )
            }
            // DiffOp::Delete(..) => {}
            DiffTag::Delete => {
                for i in old {
                    patch_ops.push(NodeOp::Remove(old_context.properties[i].path.clone()));
                }
            }
            _ => {}
        }
    }

    DiffResult {
        #[cfg(debug_assertions)]
        old_context,
        #[cfg(debug_assertions)]
        new_context,
        diff_ops,
        node_ops: patch_ops,
    }
}

/// Get the properties as a diff-able tuple of (slot, value)
///
/// This excludes the ancestry and path of a property since they should
/// not be considered in the diffing (although both are used for creating
/// patches from the diff operations)
fn diffable_properties(context: &CondenseContext) -> Vec<(&NodeSlot, &String)> {
    context
        .properties
        .iter()
        .map(|node| (&node.slot, &node.value))
        .collect()
}

/// Work out where NodePath Needs to be added, following this one.
fn path_for_next(current: &NodePath, next: &NodePath) -> NodePath {
    let mut pth = NodePath::new();
    for slots in current.iter().zip(next.iter()) {
        if slots.0 == slots.1 {
            pth.push_back(slots.0.clone());
            continue;
        }

        // We're different. But what sort of difference.
        match (slots.0, slots.1) {
            (NodeSlot::Property(..), NodeSlot::Property(..)) => {
                // WHat here
                break;
            }
            (NodeSlot::Index(_i0), NodeSlot::Index(i1)) => {
                pth.push_back(NodeSlot::Index(*i1));
                continue;
            }
            (NodeSlot::Property(..), NodeSlot::Index(..)) => {
                break;
            }
            (NodeSlot::Index(..), NodeSlot::Property(..)) => {
                break;
            }
        }
    }
    pth
}

fn ancestor_index(current: &NodePath, other: &NodePath) -> Option<usize> {
    let mut i: usize = 0;
    for slots in current.iter().zip(other.iter()) {
        if slots.0 != slots.1 {
            break;
        }
        i += 1;
    }
    if i == 0 {
        None
    } else {
        Some(i - 1)
    }
}

fn find_ancestor_index(paths: &Vec<NodePath>) -> Option<usize> {
    let split = paths.split_first();
    let mut i: usize = 0;
    if split.is_some() {
        let (first, others) = split.unwrap();
        for o in others {
            let lca = ancestor_index(first, o);
            lca?;
            i = i.min(lca.unwrap());
        }
    }
    Some(i)
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
    /// Get the operations from the result
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

#[cfg(test)]
mod tests {
    use schema::{NodeProperty, NodeType};

    use super::*;

    #[test]
    fn test_path() {
        let p1 = NodePath::from([
            NodeSlot::Property((NodeType::Article, NodeProperty::Content)),
            NodeSlot::Index(0),
            NodeSlot::Property((NodeType::Paragraph, NodeProperty::Content)),
            NodeSlot::Index(1),
            NodeSlot::Property((NodeType::Text, NodeProperty::Value)),
        ]);

        let p2 = NodePath::from([
            NodeSlot::Property((NodeType::Article, NodeProperty::Content)),
            NodeSlot::Index(1),
            NodeSlot::Property((NodeType::Paragraph, NodeProperty::Content)),
            NodeSlot::Index(3),
            NodeSlot::Property((NodeType::Text, NodeProperty::Value)),
        ]);

        let p3 = path_for_next(&p1, &p2);
        println!("{:?}", p3);
    }
}
