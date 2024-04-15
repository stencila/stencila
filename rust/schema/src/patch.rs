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

/// A context passed down to child nodes when walking across a node tree
/// during a call to `similarity`, `diff`, or `patch`
///
/// Currently, this context is only used in calls to `diff` but may be used in
/// other methods in the future.
#[derive(Default)]
pub struct PatchContext {
    /// The current path on the depth first walk across nodes during a call to `diff`
    path: PatchPath,

    /// The target paths and operations collected during a call to `diff`.
    ops: Vec<(PatchPath, PatchOp)>,
}

impl PatchContext {
    pub fn new() -> Self {
        Self::default()
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

/// A patch operation
///
/// These are generated during a call to `diff` and applied in a
/// call to `patch`.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(crate = "common::serde")]
pub enum PatchOp {
    /// Set the value of a leaf node (e.g. a `String`) or `Option`
    Set(PatchValue),

    /// Insert items into a vector
    Insert(Vec<(usize, PatchValue)>),

    /// Push an item onto the end of a vector
    Push(PatchValue),

    /// Append items onto the end of a vector
    Append(Vec<PatchValue>),

    /// Replace items in a vector
    Replace(Vec<(usize, PatchValue)>),

    /// Move items within a vector (from, to)
    Move(Vec<(usize, usize)>),

    /// Remove items from a vector
    Remove(Vec<usize>),

    /// Clear a vector or `Option` (set to `None`)
    Clear,
}

/// A value in a patch operation
///
/// This enum allows use to store values in a patch operation so that
/// they can be used when applying that operation. It has variants for
/// the main union types in the Stencila Schema with a fallback
/// variant of a [`serde_json::Value`].
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
/// A [`VecDeque`], rather than a [`Vec`] so that when applying operations in
/// a call to `patch` the front of the path can be popped off.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut, Serialize)]
#[serde(crate = "common::serde")]
#[derive(Default)]
pub struct PatchPath(VecDeque<PatchSlot>);

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
pub trait PatchNode: Sized + Serialize + DeserializeOwned {
    /// TODO: There should be no default implementations here
    /// to make sure that meaningful implementations exist for each type

    /// Convert the node to a [`PatchValue`]
    ///
    /// This default implementation uses the fallback of marshalling to
    /// a [`serde_json::Value`]. This is avoided by overriding this
    /// method for types (such as [`Block`]) for which there is a corresponding
    /// variant in [`PatchValue`].
    fn to_value(&self) -> Result<PatchValue> {
        Ok(PatchValue::Json(serde_json::to_value(self)?))
    }

    /// Create a node of type `Self` from a [`PatchValue`]
    ///
    /// This default implementation assumes the [`PatchValue::Json`] variant.
    /// As for `to_value` this should be overridden for types that have a
    /// corresponding variant in [`PatchValue`].
    fn from_value(value: PatchValue) -> Result<Self> {
        match value {
            PatchValue::Json(json) => Ok(serde_json::from_value(json)?),
            _ => bail!("Invalid value for `{}`", type_name::<Self>()),
        }
    }

    /// Calculate the similarity between this node and another of the same type
    ///
    /// The similarity value should have a minimum of zero and a maximum of one.
    /// It should be non-zero for the same types and zero for different variants
    /// of an enum.
    #[allow(unused_variables)]
    fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
        Ok(0.0001)
    }

    /// Generate the [`PatchOp`]s needed to transform this node into the other
    #[allow(unused_variables)]
    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        Ok(())
    }

    /// Apply a [`PatchOp`] to a node at a path
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
impl<T> PatchNode for Box<T>
where
    T: PatchNode,
{
    fn to_value(&self) -> Result<PatchValue> {
        self.as_ref().to_value()
    }

    fn from_value(value: PatchValue) -> Result<Self> {
        Ok(Self::new(T::from_value(value)?))
    }

    fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
        self.as_ref().similarity(other, context)
    }

    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        self.as_ref().diff(other, context)
    }

    fn patch(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        context: &mut PatchContext,
    ) -> Result<()> {
        self.as_mut().patch(path, op, context)
    }
}

// Implementation for optional properties
impl<T> PatchNode for Option<T>
where
    T: PatchNode + Serialize + DeserializeOwned,
{
    fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
        match (self, other) {
            (Some(me), Some(other)) => me.similarity(other, context),
            (None, None) => Ok(1.0),
            _ => Ok(0.0),
        }
    }

    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        match (self, other) {
            (Some(me), Some(other)) => {
                me.diff(other, context)?;
            }
            (None, Some(other)) => {
                context.op_set(other.to_value()?);
            }
            (Some(..), None) => {
                context.op_clear();
            }
            _ => {}
        };

        Ok(())
    }

    fn patch(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        context: &mut PatchContext,
    ) -> Result<()> {
        if path.is_empty() {
            match op {
                PatchOp::Set(value) => {
                    *self = Some(T::from_value(value)?);
                    return Ok(());
                }
                PatchOp::Clear => {
                    *self = None;
                    return Ok(());
                }
                _ => {}
            }
        }

        if let Some(value) = self {
            value.patch(path, op, context)?;
        } else {
            bail!("Invalid op for option: {op:?}");
        }

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
        // TODO: this sub-optimal for things like paragraphs For example,
        // think about a paragraph that has had a `Strong` node inserted into it,
        // thereby going from length 1 to length 3. Maybe write a custom similarity
        // method for Vec<Inline> that deals with that.

        let num = self.len().max(other.len());
        let mut sum = 0.0;
        for index in 0..num {
            match (self.get(index), other.get(index)) {
                (Some(me), Some(other)) => sum += me.similarity(other, context)?,
                _ => {}
            }
        }

        Ok((sum / (num as f32)).max(0.0001))
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

        #[derive(Debug)]
        struct Pair {
            self_pos: usize,
            new_pos: usize,
            other_pos: usize,
            similarity: f32,
        }

        // Calculate pairwise similarities
        let mut candidate_pairs = Vec::new();
        let mut other_matches = Vec::new();
        for (self_pos, self_item) in self.iter().enumerate() {
            // Calculate the pairwise similarity with positions in `other`.
            // Note that this `break`s when a perfect match is found and skips
            // positions in other for which there is already a perfect match.

            // TODO: because of the aforementioned early break, this loop could be
            // optimized by not starting at the beginning of other but rather
            // by "radiating out" from self_pos, or maybe better, from the
            // last perfect match+1. That is, we should still search over all other_pos
            // stopping when we find a perfect match but we should do it in a smarter order.
            for (other_pos, other_item) in other.iter().enumerate() {
                // If the other pos is already perfectly matched then
                // skip calculating similarities
                if other_matches.contains(&other_pos) {
                    continue;
                }

                let similarity = self_item.similarity(other_item, context)?;
                candidate_pairs.push(Pair {
                    self_pos,
                    new_pos: self_pos,
                    other_pos,
                    similarity,
                });

                if similarity == 1.0 {
                    other_matches.push(other_pos);
                    break;
                }
            }
        }

        // Find the pairs with highest similarity
        let mut pairs: Vec<Pair> = Vec::with_capacity(self.len().min(other.len()));
        for candidate in candidate_pairs
            .into_iter()
            .sorted_by(|a, b| a.similarity.total_cmp(&b.similarity).reverse())
        {
            if pairs
                .iter()
                .find(|pair| {
                    pair.self_pos == candidate.self_pos || pair.other_pos == candidate.other_pos
                })
                .is_none()
            {
                pairs.push(candidate);
            }
        }
        debug_assert!(pairs.len() == self.len().min(other.len()));

        if other.len() > self.len() {
            // If other is longer then insert or append those items that do not have a pair
            let insert_num = other.len() - self.len();
            let mut insert = Vec::with_capacity(insert_num);
            for other_index in 0..other.len() {
                if insert.len() < insert_num
                    && !pairs.iter().any(|pair| pair.other_pos == other_index)
                {
                    insert.push(other_index);
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
                if keep.len() < other.len() && !keep.contains(&pair.self_pos) {
                    keep.push(pair.self_pos);
                }
            }

            // Remove indices not in `keep`
            let remove = (0..self.len())
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
            PatchOp::Move(indices) => {
                for (_from, _to) in indices {
                    todo!()
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
