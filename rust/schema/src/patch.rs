use std::{
    any::type_name,
    collections::VecDeque,
    fmt::{self, Debug},
};

use common::{
    derive_more::{Deref, DerefMut},
    eyre::{bail, Report, Result},
    itertools::Itertools,
    serde::{de::DeserializeOwned, Serialize},
    serde_json::{self, Value as JsonValue},
};
use node_type::NodeProperty;

use crate::{Block, Inline, Node};

/// Merge `new` node into `old` node
///
/// This function simply combines calls to [`diff`] and [`patch`].
pub fn merge<T: PatchNode>(old: &mut T, new: &T) -> Result<()> {
    let ops = diff(old, new)?;
    patch(old, ops)
}

/// Generate the operations needed to patch `old` node into `new` node
pub fn diff<T: PatchNode>(old: &T, new: &T) -> Result<Vec<(PatchPath, PatchOp)>> {
    let mut context = PatchContext::new();
    old.diff(new, &mut context)?;
    Ok(context.ops)
}

/// Apply operations to a node
pub fn patch<T: PatchNode>(old: &mut T, ops: Vec<(PatchPath, PatchOp)>) -> Result<()> {
    let mut context = PatchContext::new();
    for (mut path, op) in ops {
        old.patch(&mut path, op, &mut context)?
    }
    Ok(())
}

pub struct PatchContext {
    path: PatchPath,

    ops: Vec<(PatchPath, PatchOp)>,
}

impl PatchContext {
    pub fn new() -> Self {
        Self {
            path: PatchPath::new(),
            ops: Vec::new(),
        }
    }

    /// Calculate the mean similarity
    ///
    /// A convenience function used in derive macros.
    pub fn mean_similarity(values: Vec<f32>) -> Result<f32> {
        let n = values.len() as f32;
        let sum = values.into_iter().fold(0., |sum, v| sum + v);
        Ok(sum / n)
    }

    /// Enter a property during the walk
    pub fn enter_property(&mut self, node_property: NodeProperty) -> &mut Self {
        self.path.push_back(PatchSlot::Property(node_property));
        self
    }

    /// Exit a property during the walk
    pub fn exit_property(&mut self) -> &mut Self {
        let popped = self.path.pop_back();
        debug_assert!(matches!(popped, Some(PatchSlot::Property(..))));
        self
    }

    /// Enter an item in a vector during the walk
    pub fn enter_index(&mut self, index: usize) -> &mut Self {
        self.path.push_back(PatchSlot::Index(index));
        self
    }

    /// Exit an item in a vector during the walk
    pub fn exit_index(&mut self) -> &mut Self {
        let popped = self.path.pop_back();
        debug_assert!(matches!(popped, Some(PatchSlot::Index(..))));
        self
    }

    /// Create a [`PatchOp::Set`] operation at the current patch
    pub fn op_set(&mut self, value: PatchValue) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Set(value)));
        self
    }

    /// Create a [`PatchOp::Insert`] operation for the current path
    pub fn op_insert(&mut self, values: Vec<(usize, PatchValue)>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Insert(values)));
        self
    }

    /// Create a [`PatchOp::Push`] operation for the current path
    pub fn op_push(&mut self, value: PatchValue) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Push(value)));
        self
    }

    /// Create a [`PatchOp::Append`] operation for the current path
    pub fn op_append(&mut self, values: Vec<PatchValue>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Append(values)));
        self
    }

    /// Create a [`PatchOp::Replace`] operation for the current path
    pub fn op_replace(&mut self, values: Vec<(usize, PatchValue)>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Replace(values)));
        self
    }

    /// Create a [`PatchOp::Move`] operation for the current path
    pub fn op_move(&mut self, indices: Vec<(usize, usize)>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Move(indices)));
        self
    }

    /// Create a [`PatchOp::Remove`] operation for the current path
    pub fn op_remove(&mut self, indices: Vec<usize>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Remove(indices)));
        self
    }

    /// Create a [`PatchOp::Clear`] operation for the current path
    pub fn op_clear(&mut self) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Clear));
        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(crate = "common::serde")]
pub enum PatchOp {
    Set(PatchValue),

    Insert(Vec<(usize, PatchValue)>),
    Push(PatchValue),
    Append(Vec<PatchValue>),
    Replace(Vec<(usize, PatchValue)>),
    Move(Vec<(usize, usize)>),
    Remove(Vec<usize>),
    Clear,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged, crate = "common::serde")]
pub enum PatchValue {
    Inline(Inline),
    Block(Block),
    Node(Node),
    Json(JsonValue),
}

/// A slot in a node path: either a property identifier or the index of a vector.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PatchSlot {
    Property(NodeProperty),
    Index(usize),
}

impl Debug for PatchSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PatchSlot::Property(prop) => Debug::fmt(prop, f),
            PatchSlot::Index(index) => Debug::fmt(index, f),
        }
    }
}

impl Serialize for PatchSlot {
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: common::serde::Serializer,
    {
        match self {
            PatchSlot::Property(prop) => prop.to_string().serialize(serializer),
            PatchSlot::Index(index) => index.serialize(serializer),
        }
    }
}

/// A path to reach a node from the root: a vector of [`PatchSlot`]s
///
/// Used when applying a patch to a node to traverse directly to the
/// branch of the tree that a patch operation should be applied.
/// Similar to the `path` of JSON Patch (https://jsonpatch.com/).
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut, Serialize)]
#[serde(crate = "common::serde")]
pub struct PatchPath(VecDeque<PatchSlot>);

impl Default for PatchPath {
    fn default() -> Self {
        Self(VecDeque::new())
    }
}

impl PatchPath {
    pub fn new() -> Self {
        Self::default()
    }
}

impl<const N: usize> From<[PatchSlot; N]> for PatchPath {
    fn from(value: [PatchSlot; N]) -> Self {
        Self(value.into())
    }
}

impl Debug for PatchPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, slot) in self.iter().enumerate() {
            if index != 0 {
                f.write_str(".")?;
            }
            slot.fmt(f)?;
        }

        Ok(())
    }
}

/// A trait for diffing and patching nodes
pub trait PatchNode: Sized {
    fn to_value(&self) -> Result<PatchValue> {
        bail!("No implemented")
    }

    #[allow(unused_variables)]
    fn from_value(value: PatchValue) -> Result<Self> {
        bail!("No implemented")
    }

    #[allow(unused_variables)]
    fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
        Ok(0.0)
    }

    #[allow(unused_variables)]
    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        Ok(())
    }

    #[allow(unused_variables)]
    fn patch(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        context: &mut PatchContext,
    ) -> Result<()> {
        Ok(())
    }
}

// Implementation for simple "atomic" types not in schema
macro_rules! atom {
    ($type:ty) => {
        impl PatchNode for $type {
            fn to_value(&self) -> Result<PatchValue> {
                Ok(PatchValue::Json(serde_json::to_value(self)?))
            }

            fn from_value(value: PatchValue) -> Result<Self> {
                match value {
                    PatchValue::Json(json) => Ok(serde_json::from_value(json)?),
                    _ => bail!("Invalid value for `{}`", type_name::<Self>()),
                }
            }

            #[allow(unused_variables)]
            fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
                // Note non-zero similarity if unequal because types are
                // the same. At present it does not seem to be necessary to do
                // anything more sophisticated (e.g. proportional difference for numbers)
                Ok(if other == self { 1.0 } else { 0.00001 })
            }

            fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
                if other != self {
                    context.op_set(other.to_value()?);
                }
                Ok(())
            }

            #[allow(unused_variables)]
            fn patch(
                &mut self,
                path: &mut PatchPath,
                op: PatchOp,
                context: &mut PatchContext,
            ) -> Result<()> {
                let PatchOp::Set(value) = op else {
                    bail!("Invalid op for `{}`", type_name::<Self>());
                };

                if !path.is_empty() {
                    bail!("Invalid path `{path:?}` for atom");
                }

                *self = Self::from_value(value)?;

                Ok(())
            }
        }
    };
}
atom!(bool);
atom!(i32);
atom!(i64);
atom!(u64);
atom!(f64);
atom!(String);

// Implementation for boxed properties
impl<T> PatchNode for Box<T> where T: PatchNode {}

// Implementation for optional properties
impl<T> PatchNode for Option<T>
where
    T: PatchNode + Serialize + DeserializeOwned,
{
    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        match (self, other) {
            (Some(me), Some(other)) => me.diff(other, context),
            _ => Ok(()),
        }
    }

    fn patch(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        context: &mut PatchContext,
    ) -> Result<()> {
        let Some(value) = self else {
            bail!("Invalid op for None");
        };
        value.patch(path, op, context)?;

        Ok(())
    }
}

// Implementation for vector properties
impl<T> PatchNode for Vec<T>
where
    T: PatchNode,
{
    #[allow(unused_variables)]
    fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
        PatchContext::mean_similarity(
            self.iter()
                .zip(other.iter())
                .map(|(me, other)| me.similarity(other, context))
                .try_collect()?,
        )
    }

    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        // Shortcuts if this vector is empty
        if self.is_empty() {
            if other.is_empty() {
                // No difference
            } else if other.len() == 1 {
                // Push new value
                context.op_push(other[0].to_value()?);
            } else {
                // Append new values
                context.op_append(other.iter().map(|item| item.to_value()).try_collect()?);
            }
            return Ok(());
        }

        // Shortcut if this other vector is empty
        if other.is_empty() {
            context.op_clear();
            return Ok(());
        }

        // Calculate the pairwise similarity
        let mut similarities = Vec::new();
        let mut other_matches = Vec::new();
        for (self_pos, self_item) in self.iter().enumerate() {
            for (other_pos, other_item) in other.iter().enumerate() {
                // If the other pos is already perfectly matched then
                // skip calculating similarities
                if other_matches.contains(&other_pos) {
                    continue;
                }

                let similarity = self_item.similarity(other_item, context)?;
                similarities.push((self_pos, other_pos, similarity));

                if similarity == 1.0 {
                    other_matches.push(other_pos);
                    break;
                }
            }
        }

        // Sort the pairs by descending order of similarity
        similarities.sort_by(|a, b| a.2.total_cmp(&b.2).reverse());

        // Find the pairs with highest similarity
        #[derive(Debug)]
        struct Pair {
            self_pos: usize,
            new_pos: usize,
            other_pos: usize,
            similarity: f32,
        }
        let mut pairs: Vec<Pair> = Vec::with_capacity(self.len().min(other.len()));
        for (self_pos, other_pos, similarity) in similarities {
            if pairs
                .iter()
                .find(|pair| pair.self_pos == self_pos || pair.other_pos == other_pos)
                .is_none()
            {
                pairs.push(Pair {
                    self_pos,
                    new_pos: self_pos,
                    other_pos,
                    similarity,
                });
            }
        }
        debug_assert!(pairs.len() == self.len().min(other.len()));

        if other.len() > self.len() {
            // If other is longer then insert or append those items that do not have a pair
            let insert_num = other.len() - self.len();
            let mut insert = Vec::with_capacity(insert_num);
            for other_index in 0..other.len() {
                if insert.len() < insert_num {
                    if pairs
                        .iter()
                        .find(|pair| pair.other_pos == other_index)
                        .is_none()
                    {
                        insert.push(other_index);
                    }
                }

                if insert.len() == insert_num {
                    break;
                }
            }
            insert.sort();

            let first = insert.first().cloned().unwrap_or_default();
            debug_assert!(first <= self.len());
            if first == self.len() {
                // If the first position to insert corresponds to the end of self then
                // generate either an append or a push op
                if insert.len() == 1 {
                    context.op_push(other[first].to_value()?);
                } else {
                    context.op_append(
                        insert
                            .into_iter()
                            .map(|index| other[index].to_value())
                            .try_collect()?,
                    );
                }
            } else {
                // Adjust new_index of pairs for the inserts
                for pair in pairs.iter_mut() {
                    for &index in &insert {
                        if index <= pair.new_pos {
                            pair.new_pos += 1;
                        }
                    }
                }

                // Generate insert op
                context.op_insert(
                    insert
                        .into_iter()
                        .map(|index| Ok::<_, Report>((index, other[index].to_value()?)))
                        .try_collect()?,
                );
            }
        } else if other.len() < self.len() {
            // If other is shorter then keep those with the highest similarity and remove the rest.
            let mut keep = Vec::new();
            for pair in pairs.iter() {
                if keep.len() < other.len() {
                    if !keep.contains(&pair.self_pos) {
                        keep.push(pair.self_pos.clone());
                    }
                }
            }

            // Remove indices not in `keep`
            let remove = (0..self.len())
                .into_iter()
                .filter(|index| !keep.contains(index))
                .collect_vec();

            // Adjust new_index of pairs for the removals
            for pair in pairs.iter_mut() {
                for &index in &remove {
                    if index <= pair.self_pos {
                        pair.new_pos -= 1;
                    }
                }
            }

            // Remove pairs not in `keep` to avoid unnecessary diffing
            pairs.retain(|pair| keep.contains(&pair.self_pos));

            // Generate remove op
            context.op_remove(remove);
        }

        println!("PAIRS {pairs:#?}");

        // TODO: perform moves

        // Iterate over pairs and diff
        let mut replace = Vec::new();
        for Pair {
            self_pos,
            new_pos,
            other_pos,
            similarity,
            ..
        } in pairs
        {
            // If positions and items are equal then nothing to do
            if new_pos == other_pos && similarity == 1.0 {
                continue;
            }

            if similarity == 0.0 {
                // If the similarity is zero (i.e. different types)
                // then do a replacement.
                replace.push((new_pos, other[other_pos].to_value()?));
            } else {
                // Otherwise diff the two items
                // Note uses of new_pos (not other_pos) here are important!
                context.enter_index(new_pos);
                self[self_pos].diff(&other[new_pos], context)?;
                context.exit_index();
            }
        }
        if !replace.is_empty() {
            context.op_replace(replace);
        }

        Ok(())
    }

    fn patch(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        context: &mut PatchContext,
    ) -> Result<()> {
        if let Some(slot) = path.pop_front() {
            let PatchSlot::Index(index) = slot else {
                bail!("Invalid slot for Vec: {slot:?}")
            };

            let Some(child) = self.get_mut(index) else {
                bail!("Invalid index for Vec: {index}")
            };

            return child.patch(path, op, context);
        }

        match op {
            PatchOp::Insert(values) => {
                for (index, value) in values {
                    self.insert(index, T::from_value(value)?);
                }
            }
            PatchOp::Push(value) => {
                self.push(T::from_value(value)?);
            }
            PatchOp::Append(values) => {
                self.append(
                    &mut values
                        .into_iter()
                        .map(|value| T::from_value(value))
                        .try_collect()?,
                );
            }
            PatchOp::Replace(values) => {
                for (index, value) in values {
                    self[index] = T::from_value(value)?;
                }
            }
            PatchOp::Remove(indices) => {
                let mut index = 0usize;
                self.retain(|_| {
                    let retain = !indices.contains(&index);
                    index += 1;
                    retain
                });
            }
            PatchOp::Clear => {
                self.clear();
            }
            _ => bail!("Invalid op for Vec: {op:?}"),
        }

        Ok(())
    }
}
