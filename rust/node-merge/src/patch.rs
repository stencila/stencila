use schema::{Node, NodePath, PatchNode};

/// Apply a `NodePatch` to a node
pub fn patch(old: &mut Node, new: &Node, ops: Vec<NodeOp>) {
    use NodeOp::*;
    for op in ops {
        match op {
            Set((mut old_path, mut new_path)) => {
                if let Some(node) = new.get_path(&mut new_path) {
                    old.set_path(&mut old_path, node)
                }
            }
            Insert((mut old_path, mut new_path)) => {
                if let Some(node) = new.get_path(&mut new_path) {
                    old.insert_path(&mut old_path, node)
                }
            }
            Remove(mut path) => old.remove_path(&mut path),
        }
    }
}

/// An operation within a patch
#[derive(Debug)]
pub enum NodeOp {
    Set((NodePath, NodePath)),
    Insert((NodePath, NodePath)),
    Remove(NodePath),
}
