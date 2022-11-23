use std::fmt::Debug;

use schemars::JsonSchema;

use common::{
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    tracing,
};
use node_address::{Address, Slot};

use stencila_schema::Node;

use crate::operation::{Add, Copy, Move, Operation, Remove, Replace, Transform};

/// A set of [`Operation`]s
#[skip_serializing_none]
#[derive(Clone, Debug, Default, JsonSchema, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Patch {
    /// The [`Operation`]s to apply
    pub ops: Vec<Operation>,

    /// The address of the node to which apply this patch
    pub address: Option<Address>,

    /// The id of the node to which to apply this patch
    ///
    /// If `target` is supplied, the `address` will be resolved starting
    /// at the node with the id.
    /// If `target` is `None`, the `address` will be resolved starting at
    /// the root node of the document.
    pub target: Option<String>,

    /// The version number of the patch
    ///
    /// Should be present on published patches.
    /// Used by clients to check that they have received all patches
    /// published for a document in the correct order (and to panic if they haven't).
    pub version: Option<u64>,

    /// The id of the actor that generated this patch
    /// e.g. a web browser client, or file watcher
    ///
    /// Should be present on published patches.
    /// Used so that actors can ignore patches that they created and
    /// that hae already been applied.
    pub actor: Option<String>,
}

impl Patch {
    /// Create a new patch from a set of operations
    pub fn from_ops(ops: Vec<Operation>) -> Self {
        Self {
            ops,
            ..Default::default()
        }
    }

    /// Create a new patch by combining a set of patches
    ///
    /// For each patch, if the patch has an address, then that address will be prepended
    /// to each of its operations before they are combined.
    pub fn from_patches(patches: Vec<Patch>) -> Self {
        let ops = patches
            .into_iter()
            .flat_map(|patch| {
                if let Some(patch_address) = patch.address {
                    patch
                        .ops
                        .into_iter()
                        .map(|mut op| {
                            match &mut op {
                                Operation::Add(Add { address, .. })
                                | Operation::Remove(Remove { address, .. })
                                | Operation::Replace(Replace { address, .. })
                                | Operation::Transform(Transform { address, .. }) => {
                                    address.prepend(&patch_address)
                                }
                                Operation::Move(Move { from, to, .. })
                                | Operation::Copy(Copy { from, to, .. }) => {
                                    from.prepend(&patch_address);
                                    to.prepend(&patch_address);
                                }
                            };
                            op
                        })
                        .collect()
                } else {
                    patch.ops
                }
            })
            .collect();
        Patch::from_ops(ops)
    }

    /// Does the patch have any operations?
    pub fn is_empty(&self) -> bool {
        self.ops.is_empty()
    }

    /// Ignore patch operations that would overwrite derived fields
    ///
    /// Often we want to load new content for a `Node` from a new file but do not want to
    /// loose fields that have been derived during compile and usually only in-memory. This
    /// removes `Replace` and `Remove` operations of `compile_digest`, `execute_digest` etc.
    pub fn remove_overwrite_derived(&mut self) {
        self.ops.retain(|op| {
            if let Operation::Remove(Remove { address, .. })
            | Operation::Replace(Replace { address, .. }) = op
            {
                for slot in address.iter() {
                    if let Slot::Name(name) = slot {
                        if matches!(
                            name.as_str(),
                            "compile_digest"
                                | "execute_digest"
                                | "execute_duration"
                                | "execute_ended"
                                | "execution_required"
                                | "execution_status"
                        ) {
                            return false;
                        }
                    }
                }
            }
            true
        })
    }

    /// Prepare the patch for publishing
    ///
    /// The main purpose of this function is to attach a version number and
    /// generate HTML for each `Add` and `Replace` operation in the patch before it is sent to clients.
    #[tracing::instrument(skip(self, root))]
    pub fn prepublish(&mut self, version: u64, root: &Node) -> &mut Self {
        self.version = Some(version);

        for op in self.ops.iter_mut() {
            match op {
                Operation::Add(Add { value, html, .. })
                | Operation::Replace(Replace { value, html, .. }) => *html = value.to_html(root),
                _ => {}
            }
        }

        self
    }
}
