use std::{
    any::type_name,
    collections::{HashMap, VecDeque},
    fmt::{self, Debug},
};

use common::{
    derive_more::{Deref, DerefMut},
    eyre::{bail, Report, Result},
    itertools::Itertools,
    serde::{de::DeserializeOwned, Deserialize, Serialize},
    serde_json::{self, Value as JsonValue},
};
use format::Format;
use node_id::NodeId;
use node_type::NodeProperty;

use crate::{
    prelude::AuthorType, replicate, Author, AuthorRole, AuthorRoleAuthor, AuthorRoleName, Block,
    CordOp, Inline, Node, ProvenanceCount, Timestamp,
};

/// Assign authorship to a node
///
/// Intended to be used only to initialize authorship information
/// on an node that has none. Will overwrite any existing authorship.
pub fn authorship<T: PatchNode>(node: &mut T, authors: Vec<AuthorRole>) -> Result<()> {
    let mut context = PatchContext {
        authors: Some(authors),
        ..Default::default()
    };
    node.authorship(&mut context)
}

/// Merge `new` node into `old` node and record authorship of changes
///
/// This function simply combines calls to [`diff`] and [`patch`].
pub fn merge<T: PatchNode + Debug>(
    old: &mut T,
    new: &T,
    format: Option<Format>,
    authors: Option<Vec<AuthorRole>>,
) -> Result<()> {
    let ops = diff(old, new, format, authors)?;
    patch(old, ops)
}

/// Generate the operations needed to patch `old` node into `new` node
pub fn diff<T: PatchNode>(
    old: &T,
    new: &T,
    format: Option<Format>,
    mut authors: Option<Vec<AuthorRole>>,
) -> Result<Patch> {
    // Ensure that each author role has a last_modified timestamp
    if let Some(roles) = authors.as_mut() {
        for role in roles {
            if role.last_modified.is_none() {
                role.last_modified = Some(Timestamp::now());
            }
        }
    }

    let mut context = PatchContext {
        format,
        ..Default::default()
    };
    old.diff(new, &mut context)?;
    let ops = context.ops;
    let format = context.format;

    Ok(Patch {
        ops,
        format,
        authors,
        ..Default::default()
    })
}

/// Apply patch operations to a node and record authorship of changes
pub fn patch<T: PatchNode + Debug>(node: &mut T, mut patch: Patch) -> Result<()> {
    let mut context = PatchContext {
        format: patch.format.clone(),
        authors: patch.authors.clone(),
        ..Default::default()
    };

    let applied = node.patch(&mut patch, &mut context)?;
    if !applied {
        bail!("Patch was not applied:\n\n{patch:?}\n\n{node:?}")
    }

    Ok(())
}

/// A context passed down to child nodes when walking across a node tree
/// during a call to `similarity`, `diff`, or `patch`
#[derive(Default)]
pub struct PatchContext {
    /// The current path on the depth first walk across nodes during a call to `diff`
    path: PatchPath,

    /// The target paths and operations collected during a call to `diff`.
    ops: Vec<(PatchPath, PatchOp)>,

    /// The source format from which a patch is being generated
    pub format: Option<Format>,

    /// The authors to which authorship of changes will be assigned during a call to `patch`.
    authors: Option<Vec<AuthorRole>>,

    /// Whether authorship has been "taken" already in the current application of an operation
    /// during a call to `patch`.
    authors_taken: bool,

    /// The author (0-based index) which should be using when making changes to the
    /// authorship of `Cord` nodes during a call to `patch`.
    author_index: Option<u8>,
}

impl PatchContext {
    /// Calculate the mean similarity
    ///
    /// A convenience function used in derive macros.
    pub fn mean_similarity(values: Vec<f32>) -> Result<f32> {
        let n = values.len() as f32;
        let sum = values.into_iter().fold(0., |sum, v| sum + v);
        Ok(sum / n)
    }

    /// Update the provenance of a node
    pub fn update_provenance(
        provenance: &mut Option<Vec<ProvenanceCount>>,
        children: Vec<Option<Vec<ProvenanceCount>>>,
    ) {
        // Aggregate counts from children
        let mut new: Vec<ProvenanceCount> = Vec::new();
        let mut sum: u64 = 0;
        for child in children.into_iter().flatten().flatten() {
            sum += child.character_count;
            if let Some(entry) = new
                .iter_mut()
                .find(|count| count.provenance_category == child.provenance_category)
            {
                entry.character_count += child.character_count;
            } else {
                new.push(child);
            }
        }

        // Calculate percentages
        if sum > 0 {
            for entry in new.iter_mut() {
                entry.character_percent = Some(((entry.character_count * 100) / sum).min(100));
            }
        }

        // Set the provenance if any. If no characters e.g. empty code chunk
        // then return None
        *provenance = if new.is_empty() || sum == 0 {
            None
        } else {
            Some(new)
        };
    }

    /// Flatten the results from calling provenance on several fields
    pub fn flatten_provenance(
        children: Vec<Option<Vec<ProvenanceCount>>>,
    ) -> Option<Vec<ProvenanceCount>> {
        let prov: Vec<ProvenanceCount> = children.into_iter().flatten().flatten().collect();
        (!prov.is_empty()).then_some(prov)
    }

    /// Enter a property during the walk
    pub fn enter_property(&mut self, node_property: NodeProperty) -> &mut Self {
        self.path.push_back(PatchSlot::Property(node_property));
        self
    }

    /// Exit a property during the diff walk
    pub fn exit_property(&mut self) -> &mut Self {
        let popped = self.path.pop_back();
        debug_assert!(matches!(popped, Some(PatchSlot::Property(..))));
        self
    }

    /// Enter an item in a vector during the diff walk
    pub fn enter_index(&mut self, index: usize) -> &mut Self {
        self.path.push_back(PatchSlot::Index(index));
        self
    }

    /// Exit an item in a vector during the diff walk
    pub fn exit_index(&mut self) -> &mut Self {
        let popped = self.path.pop_back();
        debug_assert!(matches!(popped, Some(PatchSlot::Index(..))));
        self
    }

    /// Create a [`PatchOp::Set`] operation at the current path during the diff walk
    pub fn op_set(&mut self, value: PatchValue) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Set(value)));
        self
    }

    /// Create a [`PatchOp::Apply`] operation at the current path during the diff walk
    pub fn op_apply(&mut self, cord_ops: Vec<CordOp>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Apply(cord_ops)));
        self
    }

    /// Create a [`PatchOp::Insert`] operation for the current path during the diff walk
    pub fn op_insert(&mut self, values: Vec<(usize, PatchValue)>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Insert(values)));
        self
    }

    /// Create a [`PatchOp::Push`] operation for the current path during the diff walk
    pub fn op_push(&mut self, value: PatchValue) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Push(value)));
        self
    }

    /// Create a [`PatchOp::Append`] operation for the current path during the diff walk
    pub fn op_append(&mut self, values: Vec<PatchValue>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Append(values)));
        self
    }

    /// Create a [`PatchOp::Replace`] operation for the current path during the diff walk
    pub fn op_replace(&mut self, values: Vec<(usize, PatchValue)>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Replace(values)));
        self
    }

    /// Create a [`PatchOp::Move`] operation for the current path during the diff walk
    pub fn op_move(&mut self, indices: Vec<(usize, usize)>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Move(indices)));
        self
    }

    /// Create a [`PatchOp::Copy`] operation for the current path during the diff walk
    pub fn op_copy(&mut self, indices: HashMap<usize, Vec<usize>>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Copy(indices)));
        self
    }

    /// Create a [`PatchOp::Remove`] operation for the current path during the diff walk
    pub fn op_remove(&mut self, indices: Vec<usize>) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Remove(indices)));
        self
    }

    /// Create a [`PatchOp::Clear`] operation for the current path during the diff walk
    pub fn op_clear(&mut self) -> &mut Self {
        self.ops.push((self.path.clone(), PatchOp::Clear));
        self
    }

    /// Update the authors of a node
    ///
    /// Called during calls to `authorship` and `patch` for nodes that have an `authors` property.
    ///
    /// # Parameters
    ///
    /// - `authors`: the `authors` property of the node
    ///
    /// - `take`: whether the authors of the context should be "taken" (i.e. no applied to child nodes
    ///    of the current node)
    ///
    /// - `overwrite`: whether to overwrite (i.e. reset) the current `authors` property of the node
    pub fn update_authors(
        &mut self,
        authors: &mut Option<Vec<Author>>,
        take: bool,
        overwrite: bool,
    ) -> bool {
        // Return early if context has no authors
        let Some(context_authors) = &self.authors else {
            return false;
        };

        // Return early if context authors already taken.
        if self.authors_taken {
            return false;
        }

        if overwrite || authors.is_none() {
            // Setting authorship or the node has no existing authors so set to the context's authors
            *authors = Some(
                context_authors
                    .clone()
                    .into_iter()
                    .map(Author::AuthorRole)
                    .collect(),
            );

            // Set the author id to the first author
            self.author_index = Some(0u8);
        } else if let Some(existing_authors) = authors {
            // The node has existing authors: if an author role is already present
            // update `last_modified`, otherwise add the author role.
            let mut author_id = None;
            for new_author_role in context_authors.iter() {
                let mut present = false;

                for (existing_index, mut existing_author) in existing_authors.iter_mut().enumerate()
                {
                    if let Author::AuthorRole(existing_author_role) = &mut existing_author {
                        if existing_author_role.author == new_author_role.author
                            && existing_author_role.role_name == new_author_role.role_name
                            && existing_author_role.format == new_author_role.format
                        {
                            existing_author_role.last_modified =
                                new_author_role.last_modified.clone();

                            present = true;

                            if author_id.is_none() {
                                author_id = Some(existing_index);
                            };

                            break;
                        }
                    }
                }

                if !present {
                    if author_id.is_none() {
                        author_id = Some(existing_authors.len());
                    };
                    existing_authors.push(Author::AuthorRole(new_author_role.clone()));
                }
            }

            // Set the author id to the index of the first in self.authors
            self.author_index = Some(author_id.unwrap_or(0) as u8);
        }

        if take {
            self.authors_taken = true;
        }

        self.authors_taken
    }

    /// Release the hold on authors taken by this node in a previous call to `update_authors`
    pub fn release_authors(&mut self) {
        self.authors_taken = false;
    }

    /// Get the current author index for the context
    pub fn author_index(&self) -> Option<u8> {
        self.author_index
    }

    /// Get the current author type for the context
    pub fn author_type(&self) -> Option<AuthorType> {
        if let (Some(authors), Some(index)) = (&self.authors, self.author_index) {
            authors
                .get(index as usize)
                .map(|author_role| match author_role.author {
                    AuthorRoleAuthor::SoftwareApplication(..) => AuthorType::Machine,
                    _ => AuthorType::Human,
                })
        } else {
            None
        }
    }

    /// Get the context authors but with a different role name
    pub fn authors_with_role(&self, role_name: AuthorRoleName) -> Option<Vec<AuthorRole>> {
        self.authors.as_ref().map(|authors| {
            authors
                .iter()
                .map(|author| AuthorRole {
                    role_name: role_name.clone(),
                    ..author.clone()
                })
                .collect()
        })
    }

    /// Create a patch to accept
    pub fn authors_as_accepters(&self) -> Patch {
        Patch {
            ops: vec![(PatchPath::new(), PatchOp::Nothing)],
            format: self.format.clone(),
            authors: self.authors_with_role(AuthorRoleName::Accepter),
            ..Default::default()
        }
    }

    /// Take the operations from the context
    pub fn ops(self) -> Vec<(PatchPath, PatchOp)> {
        self.ops
    }
}

/// A patch for a node
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub struct Patch {
    /// The id of the node to which the `ops` should be applied
    pub node_id: Option<NodeId>,

    /// The operations which should be applied for the patch
    pub ops: Vec<(PatchPath, PatchOp)>,

    /// The source format from which the patch was generated
    ///
    /// If `None` then the update is assumed to be programmatically generated
    /// internally, rather than from a source format.
    pub format: Option<Format>,

    /// The authors of the patch
    pub authors: Option<Vec<AuthorRole>>,
}

impl Patch {
    /// Apply the operations in the patch to a node
    ///
    /// Note that this `drain`s the ops to avoid cloning ops.
    pub fn apply<T>(&mut self, node: &mut T, context: &mut PatchContext) -> Result<bool>
    where
        T: PatchNode,
    {
        for (mut path, op) in self.ops.drain(..) {
            node.apply(&mut path, op, context)?;
        }
        Ok(true)
    }
}

/// A patch operation
///
/// These are generated during a call to `diff` and applied in a
/// call to `patch`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
pub enum PatchOp {
    /// Set the value of a leaf node (e.g. a `String`) or `Option`
    Set(PatchValue),

    /// Apply `CordOp`s to a `Cord`
    Apply(Vec<CordOp>),

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

    /// Copy items within a vector (from, to)
    Copy(HashMap<usize, Vec<usize>>),

    /// Remove items from a vector
    Remove(Vec<usize>),

    /// Clear a vector
    Clear,

    /// Accept a suggestion for an instruction
    Accept(NodeId),

    /// Do no operation
    /// Used to be able to apply patches which only update
    /// the `authors` list of a node (e.g. when a node is accepted)
    Nothing,
}

/// A value in a patch operation
///
/// This enum allows use to store values in a patch operation so that
/// they can be used when applying that operation. It has variants for
/// the main union types in the Stencila Schema, and for those that are
/// often involved in patches for execution with a fallback
/// variant of a [`serde_json::Value`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
pub enum PatchValue {
    Inline(Inline),
    Block(Block),
    Node(Node),
    String(String),
    Json(JsonValue),
    None,
}

impl Default for PatchValue {
    fn default() -> Self {
        Self::Json(JsonValue::Null)
    }
}

/// A slot in a node path: either a property identifier or the index of a vector.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(untagged, crate = "common::serde")]
pub enum PatchSlot {
    Property(NodeProperty),
    Index(usize),
}

/// A path to reach a node from the root: a vector of [`PatchSlot`]s
///
/// A [`VecDeque`], rather than a [`Vec`] so that when applying operations in
/// a call to `patch` the front of the path can be popped off.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deref, DerefMut, Serialize, Deserialize)]
#[serde(crate = "common::serde")]
#[derive(Default)]
pub struct PatchPath(VecDeque<PatchSlot>);

impl PatchPath {
    pub fn new() -> Self {
        Self::default()
    }
}

impl From<NodeProperty> for PatchPath {
    fn from(value: NodeProperty) -> Self {
        Self(VecDeque::from([PatchSlot::Property(value)]))
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
    /// Assign authorship to the node
    ///
    /// This default implementation does nothing. Nodes with
    /// an `authors` or `authorship` property should override this.
    #[allow(unused_variables)]
    fn authorship(&mut self, context: &mut PatchContext) -> Result<()> {
        Ok(())
    }

    /// Get the provenance information for the node, if any
    ///
    /// This default returns nothing.
    fn provenance(&self) -> Option<Vec<ProvenanceCount>> {
        None
    }

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
        Ok(self.minimum_similarity())
    }

    /// The minimum similarity for nodes of the same type
    fn minimum_similarity(&self) -> f32 {
        0.00001
    }

    /// The maximum similarity
    fn maximum_similarity(&self) -> f32 {
        1.0
    }

    /// Generate the [`PatchOp`]s needed to transform this node into the other
    #[allow(unused_variables)]
    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        Ok(())
    }

    /// Patch a node, or one of it's children
    ///
    /// If the `patch` has a `node_id` this method should apply the operations
    /// if the node has the same id and return `true`. Otherwise this should
    /// return false
    #[allow(unused_variables)]
    fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
        Ok(false)
    }

    /// Apply a [`PatchOp`] to a node at a path
    #[allow(unused_variables)]
    fn apply(
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

            fn similarity(&self, other: &Self, _context: &mut PatchContext) -> Result<f32> {
                // Note non-zero similarity if unequal because types are
                // the same. At present it does not seem to be necessary to do
                // anything more sophisticated (e.g. proportional difference for numbers)
                Ok(if other == self {
                    self.maximum_similarity()
                } else {
                    self.minimum_similarity()
                })
            }

            fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
                if other != self {
                    context.op_set(other.to_value()?);
                }
                Ok(())
            }

            fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
                if patch.node_id.is_some() {
                    return Ok(false);
                }

                patch.apply(self, context)
            }

            fn apply(
                &mut self,
                path: &mut PatchPath,
                op: PatchOp,
                _context: &mut PatchContext,
            ) -> Result<()> {
                let PatchOp::Set(value) = op else {
                    bail!("Invalid op for `{}`", type_name::<Self>());
                };

                if !path.is_empty() {
                    bail!("Invalid path `{path:?}` for an atom primitive");
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

// Implementation for `String` properties (note difference to `Cord`)
impl PatchNode for String {
    fn to_value(&self) -> Result<PatchValue> {
        Ok(PatchValue::String(self.clone()))
    }

    fn from_value(value: PatchValue) -> Result<Self> {
        match value {
            PatchValue::String(value) => Ok(value),
            PatchValue::Json(value) => match value {
                serde_json::Value::String(string) => Ok(string),
                _ => Ok(value.to_string()),
            },
            PatchValue::Node(Node::String(value)) => Ok(value),
            _ => bail!("Invalid value `{value:?}` for string"),
        }
    }

    fn similarity(&self, other: &Self, _context: &mut PatchContext) -> Result<f32> {
        Ok(if other == self {
            self.maximum_similarity()
        } else {
            self.minimum_similarity()
        })
    }

    fn diff(&self, other: &Self, context: &mut PatchContext) -> Result<()> {
        if other != self {
            context.op_set(other.to_value()?);
        }
        Ok(())
    }

    fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
        if patch.node_id.is_some() {
            return Ok(false);
        }

        patch.apply(self, context)
    }

    fn apply(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        _context: &mut PatchContext,
    ) -> Result<()> {
        let PatchOp::Set(value) = op else {
            bail!("Invalid op for `String`");
        };

        if !path.is_empty() {
            bail!("Invalid path `{path:?}` for String");
        }

        *self = Self::from_value(value)?;

        Ok(())
    }
}

// Implementation for boxed properties
impl<T> PatchNode for Box<T>
where
    T: PatchNode,
{
    fn authorship(&mut self, context: &mut PatchContext) -> Result<()> {
        self.as_mut().authorship(context)
    }

    fn provenance(&self) -> Option<Vec<ProvenanceCount>> {
        self.as_ref().provenance()
    }

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

    fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
        self.as_mut().patch(patch, context)
    }

    fn apply(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        context: &mut PatchContext,
    ) -> Result<()> {
        self.as_mut().apply(path, op, context)
    }
}

// Implementation for optional properties
impl<T> PatchNode for Option<T>
where
    T: PatchNode + Serialize + DeserializeOwned + Default,
{
    fn to_value(&self) -> Result<PatchValue> {
        match self {
            Some(value) => value.to_value(),
            None => Ok(PatchValue::None),
        }
    }

    fn from_value(value: PatchValue) -> Result<Self> {
        match value {
            PatchValue::None => Ok(None),
            _ => T::from_value(value).map(Some),
        }
    }

    fn authorship(&mut self, context: &mut PatchContext) -> Result<()> {
        match self {
            Some(value) => value.authorship(context),
            None => Ok(()),
        }
    }

    fn provenance(&self) -> Option<Vec<ProvenanceCount>> {
        self.as_ref().and_then(|value| value.provenance())
    }

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
                context.op_set(PatchValue::None);
            }
            _ => {}
        };

        Ok(())
    }

    fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
        match self {
            Some(value) => value.patch(patch, context),
            None => Ok(false), // Patch was not applied
        }
    }

    fn apply(
        &mut self,
        path: &mut PatchPath,
        op: PatchOp,
        context: &mut PatchContext,
    ) -> Result<()> {
        if path.is_empty() {
            if let PatchOp::Set(value) = op {
                *self = Self::from_value(value)?;
                return Ok(());
            }

            if self.is_none() && matches!(op, PatchOp::Append(..) | PatchOp::Push(..)) {
                // Vector operations can be applied to `None`
                let mut value = T::default();
                value.apply(path, op, context)?;
                *self = Some(value);
                return Ok(());
            }
        }

        if let Some(value) = self {
            value.apply(path, op, context)?;
        } else {
            bail!("Invalid op for option: {op:?}");
        }

        Ok(())
    }
}

// Implementation for vector properties
impl<T> PatchNode for Vec<T>
where
    T: PatchNode + Clone,
{
    fn authorship(&mut self, context: &mut PatchContext) -> Result<()> {
        for item in self.iter_mut() {
            item.authorship(context)?;
        }

        Ok(())
    }

    fn provenance(&self) -> Option<Vec<ProvenanceCount>> {
        let provenance: Vec<ProvenanceCount> = self
            .iter()
            .flat_map(|item| item.provenance().into_iter().flatten())
            .collect();
        (!provenance.is_empty()).then_some(provenance)
    }

    #[allow(unused_variables)]
    fn similarity(&self, other: &Self, context: &mut PatchContext) -> Result<f32> {
        // TODO: this sub-optimal for things like paragraphs For example,
        // think about a paragraph that has had a `Strong` node inserted into it,
        // thereby going from length 1 to length 3. Maybe write a custom similarity
        // method for Vec<Inline> that deals with that.

        let num = self.len().max(other.len());
        let mut sum = 0.0;
        for index in 0..num {
            if let (Some(me), Some(other)) = (self.get(index), other.get(index)) {
                sum += me.similarity(other, context)?;
            }
        }

        Ok((sum / (num as f32)).max(self.minimum_similarity()))
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

        #[derive(Clone)]
        struct Pair {
            self_pos: usize,
            new_pos: usize,
            other_pos: usize,
            similarity: f32,
        }

        // Calculate pairwise similarities
        // This code attempts to reduce the number of pairwise similarities that are calculated.
        // It does that by (a) breaking the inner loop if a perfect match is found, and
        // (b) by starting the next inner loop just after the previous perfect match and
        // alternating steps up and down the other array (rather that starting at the
        // beginning each time).
        let mut candidate_pairs = Vec::new();
        let mut perfect_matches = Vec::new();
        let mut last_perfect_match = 0;
        for (self_pos, self_item) in self.iter().enumerate() {
            let mut other_pos = if self_pos == 0 {
                0
            } else {
                (last_perfect_match + 1).min(other.len() - 1)
            };
            let mut direction = 1;
            let mut up_pos = other_pos;
            let mut down_pos = other_pos;
            let mut hit_top = false;
            let mut hit_bottom = false;
            loop {
                // If the other pos is already perfectly matched then
                // skip calculating similarities
                if !perfect_matches.contains(&other_pos) {
                    let other_item = &other[other_pos];

                    let similarity = self_item.similarity(other_item, context)?;
                    candidate_pairs.push(Pair {
                        self_pos,
                        new_pos: self_pos,
                        other_pos,
                        similarity,
                    });

                    // Record and break on perfect matches
                    if similarity == 1.0 {
                        perfect_matches.push(other_pos);
                        last_perfect_match = other_pos;
                        break;
                    }
                }

                // Check if reached ends of the other vector
                if other_pos == 0 {
                    hit_top = true;
                }
                if other_pos >= other.len() - 1 {
                    hit_bottom = true;
                }
                if hit_top && hit_bottom {
                    break;
                }

                // Swap direction if not yet hit either end
                if direction == 1 && !hit_top {
                    direction = -1;
                } else if direction == -1 && !hit_bottom {
                    direction = 1;
                }

                // Move in the new direction
                if direction == 1 {
                    down_pos = down_pos.saturating_add(1);
                    other_pos = down_pos;
                } else {
                    up_pos = up_pos.saturating_sub(1);
                    other_pos = up_pos;
                }
            }
        }

        // Find the pairs with highest similarity
        let mut best_pairs: Vec<Pair> = Vec::with_capacity(self.len().min(other.len()));
        for candidate in candidate_pairs
            .iter()
            .sorted_by(|a, b| a.similarity.total_cmp(&b.similarity).reverse())
        {
            if !best_pairs.iter().any(|pair| {
                pair.self_pos == candidate.self_pos || pair.other_pos == candidate.other_pos
            }) {
                best_pairs.push(candidate.clone());
            }
        }
        debug_assert!(best_pairs.len() == self.len().min(other.len()));

        #[allow(clippy::comparison_chain)]
        if other.len() > self.len() {
            // If other is longer then insert or append those items that do not have a pair
            let length_difference = other.len() - self.len();
            let mut insert = Vec::new();
            let mut copy: HashMap<usize, Vec<(usize, f32)>> = HashMap::new();
            for (other_pos, other_item) in other.iter().enumerate() {
                if insert.len() + copy.len() == length_difference {
                    break;
                }

                if !best_pairs.iter().any(|pair| pair.other_pos == other_pos) {
                    let mut is_copied = false;
                    const COPY_SIMILARITY: f32 = 0.95;

                    // Attempt to find a close match in self
                    for (self_pos, self_item) in self.iter().enumerate() {
                        let similarity = self_item.similarity(other_item, context)?;
                        if similarity >= COPY_SIMILARITY {
                            // Generate a copy operation
                            let entry = (other_pos, similarity);
                            copy.entry(self_pos)
                                .and_modify(|to| to.push(entry))
                                .or_insert_with(|| vec![entry]);
                            is_copied = true;

                            break;
                        }
                    }

                    // If not copied, then insert
                    if !is_copied {
                        insert.push(other_pos);
                    }
                }
            }
            insert.sort();

            let first = insert.first().cloned().unwrap_or_default();
            if copy.is_empty() && first == self.len() {
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
                // Adjust new_index of best pairs for the inserts and copies
                for pair in best_pairs.iter_mut() {
                    for &pos in &insert {
                        if pos <= pair.new_pos {
                            pair.new_pos += 1;
                        }
                    }
                    for &(pos, ..) in copy.values().flatten() {
                        if pos <= pair.new_pos {
                            pair.new_pos += 1;
                        }
                    }
                }

                if !copy.is_empty() {
                    // Generate copy ops
                    let indices = copy
                        .clone()
                        .into_iter()
                        .map(|(from, tos)| (from, tos.into_iter().map(|(to, ..)| to).collect_vec()))
                        .collect();
                    context.op_copy(indices);

                    // Generate other ops for copy destinations that are not exactly the same
                    for (from, tos) in copy {
                        for (to, similarity) in tos {
                            if similarity < 1.0 {
                                context.enter_index(to);
                                self[from].diff(&other[to], context)?;
                                context.exit_index();
                            }
                        }
                    }
                }

                // Generate insert op
                if !insert.is_empty() {
                    context.op_insert(
                        insert
                            .into_iter()
                            .map(|index| Ok::<_, Report>((index, other[index].to_value()?)))
                            .try_collect()?,
                    );
                }
            }
        } else if other.len() < self.len() {
            // If other is shorter then keep those with the highest similarity and remove the rest.
            let mut keep = Vec::new();
            for pair in best_pairs.iter() {
                if keep.len() < other.len() && !keep.contains(&pair.self_pos) {
                    keep.push(pair.self_pos);
                }
            }

            // Remove indices not in `keep`
            let remove = (0..self.len())
                .filter(|index| !keep.contains(index))
                .collect_vec();

            // Adjust new_index of best pairs for the removals
            for pair in best_pairs.iter_mut() {
                for &pos in &remove {
                    if pos <= pair.self_pos {
                        pair.new_pos -= 1;
                    }
                }
            }

            // Remove pairs not in `keep` to avoid unnecessary diffing
            best_pairs.retain(|pair| keep.contains(&pair.self_pos));

            // Generate remove op
            context.op_remove(remove);
        }

        // Create a move operation that moves items from current new_pos to other_pos.
        let mut moves: Vec<(usize, usize)> = Vec::new();
        for index in 0..best_pairs.len() {
            let Pair {
                new_pos, other_pos, ..
            } = best_pairs[index];

            // Skip is new pos id the same as other pos
            if new_pos == other_pos {
                continue;
            }

            // Add a move for this
            moves.push((new_pos, other_pos));

            // Update new pos for this pair and for every pair where the new_pos will be
            // affected by this move operation
            best_pairs[index].new_pos = other_pos;
            if index < best_pairs.len() - 1 {
                for pair in &mut best_pairs[(index + 1)..] {
                    if new_pos < pair.new_pos && other_pos >= pair.new_pos {
                        pair.new_pos -= 1;
                    } else if new_pos > pair.new_pos && other_pos <= pair.new_pos {
                        pair.new_pos += 1;
                    }
                }
            }
        }
        if !moves.is_empty() {
            context.op_move(moves);
        }

        // Iterate over pairs and diff
        let mut replace = Vec::new();
        for Pair {
            self_pos,
            new_pos,
            other_pos,
            similarity,
            ..
        } in best_pairs
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
                context.enter_index(other_pos);
                self[self_pos].diff(&other[other_pos], context)?;
                context.exit_index();
            }
        }
        if !replace.is_empty() {
            context.op_replace(replace);
        }

        Ok(())
    }

    fn patch(&mut self, patch: &mut Patch, context: &mut PatchContext) -> Result<bool> {
        if patch.node_id.is_none() {
            // Apply patch here
            return patch.apply(self, context);
        }

        // Try to apply patches to children
        for child in self {
            if child.patch(patch, context)? {
                return Ok(true); // Patch was applied
            }
        }

        Ok(false) // Patch was not applied
    }

    fn apply(
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

            return child.apply(path, op, context);
        }

        let mut from_value = |value: PatchValue| -> Result<T> {
            let mut node = T::from_value(value)?;
            node.authorship(context)?;
            Ok(node)
        };

        match op {
            PatchOp::Insert(values) => {
                for (index, value) in values {
                    self.insert(index, from_value(value)?);
                }
            }
            PatchOp::Push(value) => {
                self.push(from_value(value)?);
            }
            PatchOp::Append(values) => {
                self.append(&mut values.into_iter().map(from_value).try_collect()?);
            }
            PatchOp::Replace(values) => {
                for (index, value) in values {
                    self[index] = from_value(value)?;
                }
            }
            PatchOp::Move(indices) => {
                for (from, to) in indices {
                    let item = self.remove(from);
                    self.insert(to, item);
                }
            }
            PatchOp::Copy(indices) => {
                for (from, tos) in indices {
                    let item = self[from].clone();
                    for to in tos {
                        let replica = replicate(&item)?;
                        if to < self.len() {
                            self.insert(to, replica);
                        } else {
                            self.push(replica);
                        }
                    }
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
