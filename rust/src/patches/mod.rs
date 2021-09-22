use crate::{
    errors::{invalid_patch_operation, Error},
    methods::encode::encode,
    utils::schemas,
};
use defaults::Defaults;
use derive_more::{Constructor, Deref, DerefMut};
use eyre::{bail, Result};
use itertools::Itertools;
use schemars::{gen::SchemaGenerator, schema::Schema, JsonSchema};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use similar::TextDiff;
use std::{
    any::{type_name, Any},
    collections::VecDeque,
    fmt::Debug,
    hash::Hasher,
    iter::FromIterator,
};
use stencila_schema::{BlockContent, Boolean, InlineContent, Integer, Node, Number};
use strum::{AsRefStr, ToString};

/// Are two nodes are the same type and value?
pub fn same<Type1, Type2>(node1: &Type1, node2: &Type2) -> bool
where
    Type1: Patchable,
    Type2: Clone + Send + 'static,
{
    node1.is_same(node2).is_ok()
}

/// Do two nodes of the same type have equal value?
pub fn equal<Type>(node1: &Type, node2: &Type) -> bool
where
    Type: Patchable,
{
    node1.is_equal(node2).is_ok()
}

/// Generate a [`Patch`] describing the difference between two nodes of the same type.
pub fn diff<Type>(node1: &Type, node2: &Type) -> Patch
where
    Type: Patchable,
{
    let mut differ = Differ::default();
    node1.diff_same(&mut differ, node2);
    differ.patch
}

/// Display the difference between two nodes as a "unified diff" of the nodes
/// converted to a given format.
///
/// This can provide a more intuitive way of visualizing the differences between the
/// nodes than the raw [`Operation`]s. Note that this is slightly different from first
/// converting each node and then taking the diff in that this generates and applies a
/// patch. This means any change operations not generated or applied by the functions
/// in this module will not appear in the difference.
pub async fn diff_display(node1: &Node, node2: &Node, format: &str) -> Result<String> {
    let patch = diff(node1, node2);
    let patched = apply_new(node1, &patch)?;

    let old = encode(node1, "string://", format, None).await?;
    let new = encode(&patched, "string://", format, None).await?;

    let mut bytes = Vec::new();
    TextDiff::from_lines(&old, &new)
        .unified_diff()
        .to_writer(&mut bytes)
        .unwrap();

    let display = String::from_utf8(bytes)?;
    Ok(display)
}

/// Apply a [`Patch`] to a node.
pub fn apply<Type>(node: &mut Type, id: Option<String>, patch: &Patch) -> Result<()>
where
    Type: Patchable,
{
    match id {
        Some(id) => match node.apply_maybe(&id, patch)? {
            true => Ok(()),
            false => bail!("Unable to apply patch. Is the node id {} correct?", id),
        },
        None => node.apply_patch(patch),
    }
}

/// Apply a [`Patch`] to a clone of a node.
///
/// In contrast to `apply`, this does not alter the original node.
pub fn apply_new<Type>(node: &Type, patch: &Patch) -> Result<Type>
where
    Type: Patchable + Clone,
{
    let mut node = node.clone();
    node.apply_patch(patch)?;
    Ok(node)
}

/// Merge changes from two or more derived versions of a node into
/// their common ancestor version.
///
/// This is equivalent to `git merge` except that there can be
/// more than two derived versions and conflicts are always resolved.
/// Conflicts are resolved by preferring the changes in 'later' derived
/// version (i.e. those that are later in the `derived` list).
///
/// # Arguments
///
/// - `ancestor`: The ancestor node
/// - `derived`: A list of derived nodes in ascending order of priority
///              when resolving merge conflicts i.e. the last in the list
///              will win over all other nodes that it conflicts with
pub fn merge<Type>(ancestor: &mut Type, derived: &[&Type]) -> Result<()>
where
    Type: Patchable,
{
    let patches: Vec<Patch> = derived.iter().map(|node| diff(ancestor, *node)).collect();

    // TODO transform operations (shift address based on other operations) and resolve conflicts
    tracing::warn!("Merging is work in progress");

    for patch in patches {
        apply(ancestor, None, &patch)?;
    }
    Ok(())
}

/// A slot, used as part of an [`Address`], to locate a value within a `Node` tree.
///
/// Slots can be used to identify a part of a larger object.
///
/// The `Name` variant can be used to identify:
///
/// - the property name of a `struct`
/// - the key of a `HashMap<String, ...>`
///
/// The `Integer` variant can be used to identify:
///
/// - the index of a `Vec`
/// - the index of a Unicode character in a `String`
///
/// The `None` variant is used in places where a `Slot` is required
/// but none applies to the particular type or use case.
///
/// In contrast to JSON Patch, which uses a [JSON Pointer](http://tools.ietf.org/html/rfc6901)
/// to describe the location of additions and removals, slots offer improved performance and
/// type safety.
#[derive(Debug, Clone, JsonSchema, Serialize, Deserialize, AsRefStr)]
#[serde(untagged)]
#[schemars(deny_unknown_fields)]
pub enum Slot {
    Index(usize),
    Name(String),
}

impl ToString for Slot {
    fn to_string(&self) -> String {
        match self {
            Slot::Name(name) => name.clone(),
            Slot::Index(index) => index.to_string(),
        }
    }
}

/// The address, defined by a list of [`Slot`]s, of a value within `Node` tree.
///
/// Implemented as a double-ended queue. Given that addresses usually have less than
/// six slots it may be more performant to use a stack allocated `tinyvec` here instead.
///
/// Note: This could instead have be called a "Path", but that name was avoided because
/// of potential confusion with file system paths.
#[derive(
    Debug, Constructor, Clone, Default, Deref, DerefMut, JsonSchema, Serialize, Deserialize,
)]
#[schemars(deny_unknown_fields)]
pub struct Address(VecDeque<Slot>);

impl ToString for Address {
    fn to_string(&self) -> String {
        self.iter()
            .map(|slot| slot.to_string())
            .collect_vec()
            .join(".")
    }
}

impl Address {
    /// Concatenate an address with another and return the result
    fn concat(&self, other: &Address) -> Self {
        let mut concat = self.clone();
        concat.append(&mut other.clone());
        concat
    }
}

impl From<usize> for Address {
    fn from(index: usize) -> Address {
        Address(VecDeque::from_iter([Slot::Index(index)]))
    }
}

impl From<&str> for Address {
    fn from(name: &str) -> Address {
        Address(VecDeque::from_iter([Slot::Name(name.to_string())]))
    }
}

/// The operations that can be used in a patch to mutate one node into another.
///
/// These are the same operations as described in [JSON Patch](http://jsonpatch.com/)
/// (with the exception of `copy` and `test`). Note that `Replace` and `Move` could be
/// represented by combinations of `Remove` and `Add`. They are included as a means of
/// providing more semantically meaningful patches, and more space efficient serializations
/// (e.g. it is not necessary to represent the value being moved or copied).
///
/// In addition, there is a `Transform` operation which can be used describe the transformation
/// of a node to another type, having a similar structure. Examples includes:
///
/// - a `String` to an `Emphasis`
/// - a `Paragraph` to a `QuoteBlock`
/// - a `CodeChunk` to a `CodeBlock`
///
/// The `length` field on `Add` and `Replace` is not necessary for applying operations, but
/// is useful for generating them and for determining if there are conflicts between two patches
/// without having to downcast the `value`.
///
/// Note that for `String`s the integers in `address`, `items` and `length` all refer to Unicode
/// characters not bytes.
#[derive(Debug, JsonSchema, Serialize, Deserialize, ToString)]
#[serde(tag = "type")]
#[schemars(deny_unknown_fields)]
pub enum Operation {
    /// Add a value
    #[schemars(title = "OperationAdd")]
    Add {
        /// The address to which to add the value
        address: Address,

        /// The value to add
        #[serde(
            serialize_with = "Operation::value_serialize",
            deserialize_with = "Operation::value_deserialize"
        )]
        #[schemars(schema_with = "Operation::value_schema")]
        value: Box<dyn Any + Send>,

        /// The number of items added
        length: usize,
    },
    /// Remove one or more values
    #[schemars(title = "OperationRemove")]
    Remove {
        /// The address from which to remove the value(s)
        address: Address,

        /// The number of items to remove
        items: usize,
    },
    /// Replace one or more values
    #[schemars(title = "OperationReplace")]
    Replace {
        /// The address which should be replaced
        address: Address,

        /// The number of items to replace
        items: usize,

        /// The replacement value
        #[serde(
            serialize_with = "Operation::value_serialize",
            deserialize_with = "Operation::value_deserialize"
        )]
        #[schemars(schema_with = "Operation::value_schema")]
        value: Box<dyn Any + Send>,

        /// The number of items added
        length: usize,
    },
    /// Move a value from one address to another
    #[schemars(title = "OperationMove")]
    Move {
        /// The address from which to remove the value
        from: Address,

        /// The number of items to move
        items: usize,

        /// The address to which to add the items
        to: Address,
    },
    /// Transform a value from one type to another
    #[schemars(title = "OperationTransform")]
    Transform {
        /// The address of the `Node` to transform
        address: Address,

        /// The type of `Node` to transform from
        from: String,

        /// The type of `Node` to transform to
        to: String,
    },
}

impl Operation {
    /// Serialize the `value` field of an operation
    ///
    /// This is mainly for debugging and testing. Serialization of types is added as
    /// needed.
    fn value_serialize<S>(value: &Box<dyn Any + Send>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        macro_rules! serialize {
            ($( $type:ty )*) => {
                $(
                    if let Some(value) = value.downcast_ref::<$type>() {
                        return value.serialize(serializer);
                    }
                )*
            }
        }

        serialize!(
            u8
            i32
            Boolean
            Integer
            Number
            String
            InlineContent
            BlockContent
            Vec<u8>
            Vec<i32>
            Vec<Boolean>
            Vec<Integer>
            Vec<Number>
            Vec<String>
            Vec<InlineContent>
            Vec<BlockContent>
            serde_json::Value
        );

        serializer.serialize_str("<unserialized type>")
    }

    /// Deserialize the `value` field of an operation
    ///
    /// This is needed so that the server can receive a `Patch` from the client and
    /// deserialize the JSON value into a `Box<dyn Any + Send>`.
    fn value_deserialize<'de, D>(deserializer: D) -> Result<Box<dyn Any + Send>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
        Ok(Box::new(value))
    }

    /// Generate the JSON Schema for the `value` property
    fn value_schema(_generator: &mut SchemaGenerator) -> Schema {
        schemas::typescript("any", true)
    }
}

/// A set of [`Operation`]s
#[derive(Debug, Default, Deref, DerefMut, JsonSchema, Serialize, Deserialize)]
pub struct Patch(Vec<Operation>);

/// A DOM operation used to mutate the DOM.
///
/// A `DomOperation` is the DOM version of an [`Operation`].
/// The same names for operation variants and their properties
/// are used with the following exception:
///
/// - the `value` property of `Add` and `Replace` is renamed to `html` and is a string
///   representing the HTML of the DOM node (usually a HTML `Element` or `Text` node)
///   or part of it.
///
/// - the `length` property of `Add` and `Replace` is not included because it is not
///   needed (for merge conflict resolution as it is in `Operation`).
#[derive(Debug, JsonSchema, Serialize)]
#[serde(tag = "type")]
#[schemars(deny_unknown_fields)]
pub enum DomOperation {
    /// Add one or more DOM nodes
    #[schemars(title = "DomOperationAdd")]
    Add {
        /// The address to which to add the DOM node(s)
        address: Address,

        /// The HTML to add
        html: String,
    },
    /// Remove one or more DOM nodes
    #[schemars(title = "DomOperationRemove")]
    Remove {
        /// The address from which to remove the DOM node(s)
        address: Address,

        /// The number of items to remove
        items: usize,
    },
    /// Replace one or more DOM nodes
    #[schemars(title = "DomOperationReplace")]
    Replace {
        /// The address which should be replaced
        address: Address,

        /// The number of items to replace
        items: usize,

        /// The replacement HTML
        html: String,
    },
    /// Move a DOM node from one address to another
    #[schemars(title = "DomOperationMove")]
    Move {
        /// The address from which to remove the DOM node
        from: Address,

        /// The number of items to move
        items: usize,

        /// The address to which to add the items
        to: Address,
    },
    /// Transform a DOM node from one type to another
    #[schemars(title = "DomOperationTransform")]
    Transform {
        /// The address of the DOM node to transform
        address: Address,

        /// The type of `Node` to transform from
        from: String,

        /// The type of `Node` to transform to
        to: String,
    },
}

impl DomOperation {
    /// Create a `DomOperation` from an `Operation`
    fn from_op(op: &Operation) -> DomOperation {
        match op {
            Operation::Add { address, value, .. } => DomOperation::Add {
                address: address.clone(),
                html: DomOperation::value_html(address, value),
            },

            Operation::Remove { address, items, .. } => DomOperation::Remove {
                address: address.clone(),
                items: *items,
            },

            Operation::Replace {
                address,
                items,
                value,
                ..
            } => DomOperation::Replace {
                address: address.clone(),
                items: *items,
                html: DomOperation::value_html(address, value),
            },

            Operation::Move { from, items, to } => DomOperation::Move {
                from: from.clone(),
                items: *items,
                to: to.clone(),
            },

            Operation::Transform { address, from, to } => DomOperation::Transform {
                address: address.clone(),
                from: from.clone(),
                to: to.clone(),
            },
        }
    }

    /// Generate HTML for the `value` field of an operation
    fn value_html(address: &Address, value: &Box<dyn Any + Send>) -> String {
        use crate::methods::encode::html::{Context, ToHtml};

        let slot = address.back().unwrap();
        let context = Context::new();
        if let Some(string) = value.downcast_ref::<String>() {
            string.clone()
        } else if let Some(inline) = value.downcast_ref::<InlineContent>() {
            inline.to_html(&slot.to_string(), &context)
        } else if let Some(block) = value.downcast_ref::<BlockContent>() {
            block.to_html(&slot.to_string(), &context)
        } else if let Some(inlines) = value.downcast_ref::<Vec<InlineContent>>() {
            match slot {
                // If the slot is a name then we're adding or replacing a property so we
                // want the `Vec` to have a wrapper element with the name as the slot attribute
                Slot::Name(name) => inlines.to_html(name, &context),
                // If the slot is an index then we're adding or replacing items in a
                // vector so we don't want a wrapper element
                Slot::Index(..) => inlines.to_html("", &context),
            }
        } else if let Some(blocks) = value.downcast_ref::<Vec<BlockContent>>() {
            match slot {
                // As above, but for blocks...
                Slot::Name(name) => blocks.to_html(name, &context),
                Slot::Index(..) => blocks.to_html("", &context),
            }
        } else {
            tracing::error!("Unhandled value type when generating `DomOperation`");
            "<span class=\"todo\">TODO</span>".to_string()
        }
    }
}

/// A set of [`DomOperation`]s
#[derive(Debug, Deref, JsonSchema, Serialize)]
pub struct DomPatch(Vec<DomOperation>);

impl From<&Patch> for DomPatch {
    fn from(patch: &Patch) -> DomPatch {
        DomPatch(patch.iter().map(|op| DomOperation::from_op(op)).collect())
    }
}

/// A differencing `struct` used as an optimization to track the address describing the
/// current location in a node tree while walking over it.
#[derive(Defaults)]
pub struct Differ {
    /// The list of address describing the current location in a node tree.
    address: Address,

    /// The operations generated by walking over a node tree.
    patch: Patch,
}

impl Differ {
    /// Difference a field of a `struct` or an item of a `HashMap`.
    ///
    /// Adds a `Name` key to `address` and then differences the two values.
    pub fn field<Type: Patchable>(&mut self, name: &str, value1: &Type, value2: &Type) {
        self.address.push_back(Slot::Name(name.to_string()));
        value1.diff_same(self, value2);
        self.address.pop_back();
    }

    /// Difference an item in a `Vec`.
    ///
    /// Adds an `Index` key to `address` and then differences the two values.
    pub fn item<Type: Patchable>(&mut self, index: usize, value1: &Type, value2: &Type) {
        self.address.push_back(Slot::Index(index));
        value1.diff_same(self, value2);
        self.address.pop_back();
    }

    /// Append a list of operations nested within the current address
    pub fn append(&mut self, ops: Vec<Operation>) {
        for op in ops {
            let op = match op {
                Operation::Add {
                    address,
                    value,
                    length,
                } => Operation::Add {
                    address: self.address.concat(&address),
                    value,
                    length,
                },
                Operation::Remove { address, items } => Operation::Remove {
                    address: self.address.concat(&address),
                    items,
                },
                Operation::Replace {
                    address,
                    items,
                    value,
                    length,
                } => Operation::Replace {
                    address: self.address.concat(&address),
                    items,
                    value,
                    length,
                },
                Operation::Move { from, items, to } => Operation::Move {
                    from: self.address.concat(&from),
                    items,
                    to: self.address.concat(&to),
                },
                Operation::Transform { address, from, to } => Operation::Transform {
                    address: self.address.concat(&address),
                    from,
                    to,
                },
            };
            self.patch.push(op)
        }
    }

    /// Add an `Add` operation to the patch.
    pub fn add<Value: Clone + Send + 'static>(&mut self, value: &Value) {
        self.patch.push(Operation::Add {
            address: self.address.clone(),
            value: Box::new(value.clone()),
            length: 1,
        })
    }

    /// Add a `Remove` operation to the patch.
    pub fn remove(&mut self) {
        self.patch.push(Operation::Remove {
            address: self.address.clone(),
            items: 1,
        })
    }

    /// Add a `Replace` operation to the patch.
    pub fn replace<Value: Clone + Send + 'static>(&mut self, value: &Value) {
        self.patch.push(Operation::Replace {
            address: self.address.clone(),
            items: 1,
            value: Box::new(value.clone()),
            length: 1,
        })
    }

    /// Add a `Transform` operation to the patch.
    pub fn transform(&mut self, from: &str, to: &str) {
        self.patch.push(Operation::Transform {
            address: self.address.clone(),
            from: from.into(),
            to: to.into(),
        })
    }
}

pub trait Patchable {
    /// Test whether a node is the same as (i.e. equal type and equal value)
    /// another node of any type.
    fn is_same<Other: Any + Clone + Send>(&self, other: &Other) -> Result<()>;

    /// Test whether a node is equal to (i.e. equal value) a node of the same type.
    fn is_equal(&self, other: &Self) -> Result<()>;

    /// Generate a hash of the patchable content of a node
    ///
    /// Used for identifying unique values, particularly when diffing sequences.
    fn make_hash<H: Hasher>(&self, state: &mut H);

    /// Generate the operations needed to mutate this node so that is the same as
    /// a node of any other type.
    ///
    /// `Other` needs to be `Clone` so that if necessary, we can keep a copy of it in a
    /// `Add` or `Replace operation.
    fn diff<Other: Any + Clone + Send>(&self, differ: &mut Differ, other: &Other);

    /// Generate the operations needed to mutate this node so that it is equal
    /// to a node of the same type.
    fn diff_same(&self, differ: &mut Differ, other: &Self);

    /// Generate the operations needed to mutate this node so that it is the
    /// same as a node of any other type.
    ///
    /// This allows node types to define a `Transform` patch operation, which
    /// is more semantically explicit, and will usually require less data changes
    /// than a full `Replace` operation. An example is transforming a `Emphasis`
    /// node to a `Strong` node.
    ///
    /// The default implementation simply replaces the current node. Override as
    /// suits.
    fn diff_other<Other: Any + Clone + Send>(&self, differ: &mut Differ, other: &Other) {
        differ.replace(other)
    }

    /// Apply a patch to this node if the id matches the node's id
    fn apply_maybe(&mut self, id: &str, patch: &Patch) -> Result<bool>;

    /// Apply a patch to this node.
    fn apply_patch(&mut self, patch: &Patch) -> Result<()> {
        tracing::debug!(
            "Applying patch to type '{}': {:?}",
            type_name::<Self>(),
            patch
        );
        for op in patch.iter() {
            self.apply_op(op)?
        }
        Ok(())
    }

    /// Apply an operation to this node.
    fn apply_op(&mut self, op: &Operation) -> Result<()> {
        match op {
            Operation::Add { address, value, .. } => self.apply_add(&mut address.clone(), value),
            Operation::Remove { address, items } => self.apply_remove(&mut address.clone(), *items),
            Operation::Replace {
                address,
                items,
                value,
                ..
            } => self.apply_replace(&mut address.clone(), *items, value),
            Operation::Move { from, items, to } => {
                self.apply_move(&mut from.clone(), *items, &mut to.clone())
            }
            Operation::Transform { address, from, to } => {
                self.apply_transform(&mut address.clone(), from, to)
            }
        }
    }

    /// Apply an `Add` patch operation
    fn apply_add(&mut self, _address: &mut Address, _value: &Box<dyn Any + Send>) -> Result<()> {
        bail!(invalid_patch_operation("add", self))
    }

    /// Apply a `Remove` patch operation
    fn apply_remove(&mut self, _address: &mut Address, _items: usize) -> Result<()> {
        bail!(invalid_patch_operation("remove", self))
    }

    /// Apply a `Replace` patch operation
    fn apply_replace(
        &mut self,
        _address: &mut Address,
        _items: usize,
        _value: &Box<dyn Any + Send>,
    ) -> Result<()> {
        bail!(invalid_patch_operation("replace", self))
    }

    /// Apply a `Move` patch operation
    fn apply_move(&mut self, _from: &mut Address, _items: usize, _to: &mut Address) -> Result<()> {
        bail!(invalid_patch_operation("move", self))
    }

    /// Apply a `Transform` patch operation
    fn apply_transform(&mut self, _address: &mut Address, _from: &str, _to: &str) -> Result<()> {
        bail!(invalid_patch_operation("transform", self))
    }

    /// Cast a value, as stored in a `Add` or `Replace` operation, into a valid value for the type
    fn cast_value(value: &Box<dyn Any + Send>) -> Result<Self>
    where
        Self: Clone + Sized + 'static,
    {
        if let Some(value) = value.downcast_ref::<Self>() {
            Ok(value.clone())
        } else {
            bail!(Error::InvalidPatchValue {
                type_name: type_name::<Self>().to_string()
            })
        }
    }
}

/// Generate the `is_same` method for a type
macro_rules! patchable_is_same {
    () => {
        fn is_same<Other: Any + Clone>(&self, other: &Other) -> Result<()> {
            if let Some(other) = (other as &dyn Any).downcast_ref::<Self>() {
                self.is_equal(&other)
            } else {
                bail!(Error::NotSame)
            }
        }
    };
}

/// Generate the `diff` method for a type
macro_rules! patchable_diff {
    () => {
        fn diff<Other: Any + Clone + Send>(&self, differ: &mut Differ, other: &Other) {
            if let Some(other) = (other as &dyn Any).downcast_ref::<Self>() {
                self.diff_same(differ, other)
            } else {
                self.diff_other(differ, other)
            }
        }
    };
}

mod prelude;

mod atomics;
#[macro_use]
mod enums;
mod boxes;
mod options;
mod strings;
#[macro_use]
mod structs;
mod vecs;

mod blocks;
mod inlines;
mod nodes;
mod works;

#[allow(dead_code)]
#[derive(JsonSchema)]
enum PatchesSchema {
    Slot(Slot),
    Address(Address),
    Patch(Patch),
    Operation(Operation),
    DomPatch(DomPatch),
    DomOperation(DomOperation),
}

/// Get JSON Schemas for this module
pub fn schemas() -> Result<serde_json::Value> {
    schemas::generate::<PatchesSchema>()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_json, assert_json_eq};
    use serde_json::json;
    use stencila_schema::{Article, Emphasis, InlineContent, Integer, Paragraph};

    #[test]
    fn test_same_equal() {
        let int_a: Integer = 1;
        let int_b: Integer = 2;
        let opt_a: Option<Integer> = None;
        let opt_b: Option<Integer> = Some(1);
        let vec_a: Vec<Integer> = vec![1, 2, 3];
        let vec_b: Vec<Integer> = vec![3, 2, 1];

        assert!(same(&int_a, &int_a));
        assert!(!same(&int_a, &int_b));
        assert!(!same(&int_a, &vec_a));
        assert!(!same(&int_a, &opt_a));
        assert!(!same(&vec_a, &vec_b));

        assert!(equal(&int_a, &int_a));
        assert!(!equal(&int_a, &int_b));
        assert!(equal(&opt_a, &opt_a));
        assert!(!equal(&opt_a, &opt_b));
        assert!(equal(&vec_a, &vec_a));
        assert!(!equal(&vec_a, &vec_b));
    }

    #[test]
    fn test_diff_apply() -> Result<()> {
        let empty = Paragraph::default();
        let a = Paragraph {
            content: vec![
                InlineContent::String("word1".to_string()),
                InlineContent::String("word2".to_string()),
            ],
            ..Default::default()
        };
        let b = Paragraph {
            content: vec![
                InlineContent::Emphasis(Emphasis {
                    content: vec![InlineContent::String("word1".to_string())],
                    ..Default::default()
                }),
                InlineContent::String("wotwo".to_string()),
            ],
            ..Default::default()
        };

        // Patching `empty` to `a` should return no difference

        let patch = diff(&empty, &empty);
        assert_json!(patch, []);

        let mut patched = empty.clone();
        apply(&mut patched, None, &patch)?;
        assert_json_eq!(patched, empty);

        // Patching `empty` to `a` should:
        // - replace all content with the content of `a`

        let patch = diff(&empty, &a);
        assert_json!(
            patch,
            [{
                "type": "Add",
                "address": ["content", 0],
                "value": ["word1", "word2"],
                "length": 2
            }]
        );

        let mut patched = empty.clone();
        apply(&mut patched, None, &patch)?;
        assert_json_eq!(patched, a);

        // Patching `a` to `b` should:
        // - transform `content[0]` from a string to an `Emphasis`
        // - replace part of `content[1]`

        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{
                "type": "Transform",
                "address": ["content", 0],
                "from": "String",
                "to": "Emphasis"
            },{
                "type": "Replace",
                "address": ["content", 1, 2],
                "items": 3,
                "value": "two",
                "length": 3
            }]
        );

        let mut patched = a.clone();
        apply(&mut patched, None, &patch)?;
        assert_json_eq!(patched, b);

        Ok(())
    }

    #[test]
    fn test_dom_patch() {
        // Empty article
        let one = Article::default();

        // Add an empty paragraph
        let two = Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph::default())]),
            ..Default::default()
        };

        // Add words to the paragraph
        let three = Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph {
                content: vec![
                    InlineContent::String("first".to_string()),
                    InlineContent::String(" second".to_string()),
                ],
                ..Default::default()
            })]),
            ..Default::default()
        };

        // Modify a word
        let four = Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph {
                content: vec![
                    InlineContent::String("foot".to_string()),
                    InlineContent::String(" second".to_string()),
                ],
                ..Default::default()
            })]),
            ..Default::default()
        };

        // Move words
        let five = Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph {
                content: vec![
                    InlineContent::String(" second".to_string()),
                    InlineContent::String("foot".to_string()),
                ],
                ..Default::default()
            })]),
            ..Default::default()
        };

        // one to one -> empty patch
        let patch = diff(&one, &one);
        assert_json_eq!(patch, json!([]));
        let dom_patch = DomPatch::from(&patch);
        assert_json_eq!(dom_patch, json!([]));

        // one to two -> `Add` operation on the article's optional content
        let patch = diff(&one, &two);
        assert_json_eq!(
            patch,
            json!([{
                "type": "Add",
                "address": ["content"],
                "value": [{"type": "Paragraph", "content": []}],
                "length": 1
            }])
        );
        let dom_patch = DomPatch::from(&patch);
        assert_json_eq!(
            dom_patch,
            json!([{
                "type": "Add",
                "address": ["content"],
                "html": "<div slot=\"content\"><p itemtype=\"https://stenci.la/Paragraph\" itemscope></p></div>"
            }])
        );

        // two to three -> `Add` operation on the paragraph's content
        let patch = diff(&two, &three);
        assert_json_eq!(
            patch,
            json!([{
                "type": "Add",
                "address": ["content", 0, "content", 0],
                "value": ["first", " second"],
                "length": 2
            }])
        );
        let dom_patch = DomPatch::from(&patch);
        assert_json_eq!(
            dom_patch,
            json!([{
                "type": "Add",
                "address": ["content", 0, "content", 0],
                "html": "first second"
            }])
        );

        // three to four -> `Replace` operation on a word
        let patch = diff(&three, &four);
        assert_json_eq!(
            patch,
            json!([{
                "type": "Replace",
                "address": ["content", 0, "content", 0, 1],
                "items": 3,
                "value": "oo",
                "length": 2
            }])
        );
        let dom_patch = DomPatch::from(&patch);
        assert_json_eq!(
            dom_patch,
            json!([{
                "type": "Replace",
                "address": ["content", 0, "content", 0, 1],
                "items": 3,
                "html": "oo"
            }])
        );

        // four to five -> `Move` operation on the word
        let patch = diff(&four, &five);
        assert_json_eq!(
            patch,
            json!([{
                "type": "Move",
                "from": ["content", 0, "content", 1],
                "items": 1,
                "to": ["content", 0, "content", 0],
            }])
        );
        let dom_patch = DomPatch::from(&patch);
        assert_json_eq!(
            dom_patch,
            json!([{
                "type": "Move",
                "from": ["content", 0, "content", 1],
                "items": 1,
                "to": ["content", 0, "content", 0],
            }])
        );
    }
}
