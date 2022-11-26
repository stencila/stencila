use std::fmt::Debug;

use schemars::JsonSchema;

use common::{
    serde::{Deserialize, Serialize},
    serde_with::skip_serializing_none,
    tracing,
};
use node_address::{Address, Slot};
use stencila_schema::Node;

use crate::{
    operation::{
        Add, AddMany, Copy, Move, Operation, OperationFlag, OperationFlagSet, Remove, RemoveMany,
        Replace, ReplaceMany, Transform,
    },
    value::Values,
};

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
                                | Operation::AddMany(AddMany { address, .. })
                                | Operation::Remove(Remove { address, .. })
                                | Operation::RemoveMany(RemoveMany { address, .. })
                                | Operation::Replace(Replace { address, .. })
                                | Operation::ReplaceMany(ReplaceMany { address, .. })
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

    /// Compact the patch allowing for all operations
    pub fn compact_all(&mut self) -> Patch {
        self.compact(OperationFlag::all())
    }

    /// Compact the patch by replacing `Add` and `Remove` operations with `Replace`, `AddMany`
    /// `RemoveMany`, `ReplaceMany` where possible and depending upon `op_flags`.
    pub fn compact(&self, op_flags: OperationFlagSet) -> Patch {
        let mut ops = Vec::with_capacity(self.ops.len());

        for op in &self.ops {
            match op {
                Operation::Add(Add { address, value, .. }) => {
                    if let Some(last) = ops.last_mut() {
                        match last {
                            Operation::Add(Add {
                                address: last_address,
                                value: last_value,
                                ..
                            }) => {
                                if op_flags.contains(OperationFlag::AddMany)
                                    && address.follows(last_address, 1)
                                {
                                    *last = Operation::add_many(
                                        last_address.clone(),
                                        Values::from_pair(last_value.clone(), value.clone()),
                                    );
                                    continue;
                                }
                            }

                            Operation::AddMany(AddMany {
                                address: last_address,
                                values: last_values,
                                ..
                            }) => {
                                if address.follows(last_address, last_values.len()) {
                                    last_values.push(value.clone());
                                    continue;
                                }
                            }

                            Operation::Remove(Remove {
                                address: last_address,
                            }) => {
                                if op_flags.contains(OperationFlag::Replace)
                                    && address.follows(last_address, 0)
                                {
                                    *last = Operation::replace(last_address.clone(), value.clone());
                                    continue;
                                }
                            }

                            Operation::RemoveMany(RemoveMany {
                                address: last_address,
                                items,
                            }) => {
                                if op_flags.contains(OperationFlag::ReplaceMany)
                                    && address.follows(last_address, 0)
                                {
                                    *last = Operation::replace_many(
                                        last_address.clone(),
                                        *items,
                                        Values::from_single(value.clone()),
                                    );
                                    continue;
                                }
                            }

                            Operation::Replace(Replace {
                                address: last_address,
                                value: last_value,
                                ..
                            }) => {
                                if op_flags.contains(OperationFlag::ReplaceMany)
                                    && address.follows(last_address, 1)
                                {
                                    *last = Operation::replace_many(
                                        last_address.clone(),
                                        1,
                                        Values::from_pair(last_value.clone(), value.clone()),
                                    );
                                    continue;
                                }
                            }

                            Operation::ReplaceMany(ReplaceMany {
                                address: last_address,
                                values: last_values,
                                ..
                            }) => {
                                if address.follows(last_address, last_values.len()) {
                                    last_values.push(value.clone());
                                    continue;
                                }
                            }

                            _ => (),
                        }
                    }
                }
                Operation::Remove(Remove { address }) => {
                    if let Some(last) = ops.last_mut() {
                        match last {
                            Operation::Remove(Remove {
                                address: last_address,
                            }) => {
                                if op_flags.contains(OperationFlag::RemoveMany)
                                    && address == last_address
                                {
                                    *last = Operation::remove_many(last_address.clone(), 2);
                                    continue;
                                }
                            }

                            Operation::RemoveMany(RemoveMany {
                                address: last_address,
                                items,
                            }) => {
                                if address == last_address {
                                    *items += 1;
                                    continue;
                                }
                            }
                            _ => (),
                        }
                    }
                }
                Operation::Replace(Replace { address, value, .. }) => {
                    if let Some(last) = ops.last_mut() {
                        match last {
                            Operation::Replace(Replace {
                                address: last_address,
                                value: last_value,
                                ..
                            }) => {
                                if op_flags.contains(OperationFlag::ReplaceMany)
                                    && address.follows(last_address, 1)
                                {
                                    *last = Operation::replace_many(
                                        last_address.clone(),
                                        2,
                                        Values::from_pair(last_value.clone(), value.clone()),
                                    );
                                    continue;
                                }
                            }

                            Operation::ReplaceMany(ReplaceMany {
                                address: last_address,
                                values: last_values,
                                ..
                            }) => {
                                if address.follows(last_address, last_values.len()) {
                                    last_values.push(value.clone());
                                    continue;
                                }
                            }

                            Operation::Remove(Remove {
                                address: last_address,
                            }) => {
                                if op_flags.contains(OperationFlag::ReplaceMany)
                                    && address == last_address
                                {
                                    *last = Operation::replace_many(
                                        last_address.clone(),
                                        2,
                                        Values::from_single(value.clone()),
                                    );
                                    continue;
                                }
                            }

                            Operation::RemoveMany(RemoveMany {
                                address: last_address,
                                items,
                            }) => {
                                if op_flags.contains(OperationFlag::ReplaceMany)
                                    && address == last_address
                                {
                                    *last = Operation::replace_many(
                                        last_address.clone(),
                                        *items + 1,
                                        Values::from_single(value.clone()),
                                    );
                                    continue;
                                }
                            }
                            _ => (),
                        }
                    }
                }
                _ => {}
            }
            ops.push(op.clone())
        }

        Patch {
            ops,
            address: self.address.clone(),
            target: self.target.clone(),
            version: self.version,
            actor: self.actor.clone(),
        }
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
