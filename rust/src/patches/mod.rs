use crate::errors::{report, Error};
use defaults::Defaults;
use eyre::Result;
use serde::{Serialize, Serializer};
use std::{
    any::{type_name, Any},
    collections::VecDeque,
    fmt::Debug,
};
use stencila_schema::{BlockContent, Boolean, InlineContent, Integer, Number};

/// Whether or not two nodes are the same type and value.
pub fn same<Type1, Type2>(node1: &Type1, node2: &Type2) -> bool
where
    Type1: Diffable,
    Type2: Clone + 'static,
{
    node1.is_same(node2).is_ok()
}

/// Whether or not two nodes of the same type have equal value.
pub fn equal<Type>(node1: &Type, node2: &Type) -> bool
where
    Type: Diffable,
{
    node1.is_equal(node2).is_ok()
}

/// Generate a [`Patch`] describing the difference between two nodes of the same type.
pub fn diff<Type>(node1: &Type, node2: &Type) -> Patch
where
    Type: Diffable,
{
    let mut differ = Differ::default();
    node1.diff_same(&mut differ, node2);
    differ.patch
}

/// Apply a [`Patch`] to a node of any type.
pub fn apply<Type>(node: &mut Type, patch: &[Operation])
where
    Type: Diffable,
{
    for op in patch {
        node.apply(op)
    }
}

/// Apply a [`Patch`] to a clone of a node of any type.
pub fn apply_new<Type>(node: &Type, patch: &[Operation]) -> Type
where
    Type: Diffable + Clone,
{
    let mut node = node.clone();
    apply(&mut node, patch);
    node
}

/// A vector of [`Operation`]s describing the difference between two nodes.
type Patch = Vec<Operation>;

/// An enumeration of the types of operations that can be used in a [`Patch`] to
/// mutate one node into another.
///
/// These are the same operations as described in [JSON Patch](http://jsonpatch.com/)
/// (with the exception of `test`). Note that `Replace`, `Copy` and `Move` can all be
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
/// In contrast to JSON Patch, which uses a [JSON Pointer](http://tools.ietf.org/html/rfc6901)
/// to describe the location of additions and removals, these operations use a double ended queue
/// of [`Key`]s for improved performance and type safety.
#[derive(Debug, Serialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Operation {
    Add(Add),
    Remove(Remove),
    Replace(Replace),
    Copy(Copy),
    Move(Move),
    Transform(Transform),
}

/// Add a value into a node tree
#[derive(Debug, Serialize)]
pub struct Add {
    /// The location to which to add the value
    keys: Keys,

    /// The value to add
    #[serde(serialize_with = "serialize_value")]
    value: Box<dyn Any>,
}

/// Remove one or more values from a node tree
///
/// The `items` field is only relevant for sequences.
#[derive(Debug, Serialize)]
pub struct Remove {
    /// The location from which to remove the current value
    keys: Keys,

    /// The number of items to remove forward from the location
    items: usize,
}

/// Replace a value in the node tree with another value
#[derive(Debug, Serialize)]
pub struct Replace {
    /// The location which should be replaced
    keys: Keys,

    /// The number of items to remove forward from the location
    items: usize,

    /// The replacement value
    #[serde(serialize_with = "serialize_value")]
    value: Box<dyn Any>,
}

/// Copy a value from one location in a node tree, to another.
#[derive(Debug, Serialize)]
pub struct Copy {
    /// The location from which to remove the value
    from: Keys,

    /// The location to which to add the value
    to: Keys,
}

/// Move a value from one location in a node tree, to another.
#[derive(Debug, Serialize)]
pub struct Move {
    /// The location from which to remove the value
    from: Keys,

    /// The location to which to add the value
    to: Keys,
}

/// Transform a node from one type to another.
#[derive(Debug, Serialize)]
pub struct Transform {
    /// The location of the node to transform
    keys: Keys,

    /// The type of node to transform from
    from: String,

    /// The type of node to transform to
    to: String,
}

/// Serialize the `value` field of an operation
///
/// This is mainly for debugging and testing. Serialization of types is added as
/// needed.
fn serialize_value<S>(value: &Box<dyn Any>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    if let Some(value) = value.downcast_ref::<Boolean>() {
        value.serialize(serializer)
    } else if let Some(value) = value.downcast_ref::<Integer>() {
        value.serialize(serializer)
    } else if let Some(value) = value.downcast_ref::<Number>() {
        value.serialize(serializer)
    } else if let Some(value) = value.downcast_ref::<String>() {
        value.serialize(serializer)
    } else if let Some(value) = value.downcast_ref::<InlineContent>() {
        value.serialize(serializer)
    } else if let Some(value) = value.downcast_ref::<Vec<InlineContent>>() {
        value.serialize(serializer)
    } else if let Some(value) = value.downcast_ref::<BlockContent>() {
        value.serialize(serializer)
    } else if let Some(value) = value.downcast_ref::<Vec<BlockContent>>() {
        value.serialize(serializer)
    } else {
        serializer.serialize_str("<unserialized type>")
    }
}

/// A key of a `struct`, `HashMap`, or `Vec` used to locate an operation.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Key {
    Index(usize),
    Name(String),
}

pub type Keys = VecDeque<Key>;

/// A differencing `struct` used as an optimization to track the keys describing the
/// current location in a node tree while walking over it.
#[derive(Defaults)]
pub struct Differ {
    /// The list of keys describing the current location in a node tree.
    keys: Keys,

    /// The patch generated by walking over a node tree.
    patch: Patch,
}

impl Differ {
    /// Difference a field of a `struct` or an item of a `HashMap`.
    ///
    /// Adds a `Name` key to `keys` and then differences the two values.
    pub fn field<Type: Diffable>(&mut self, name: &str, value1: &Type, value2: &Type) {
        self.keys.push_back(Key::Name(name.to_string()));
        value1.diff_same(self, value2);
        self.keys.pop_back();
    }

    /// Difference an item in a `Vec`.
    ///
    /// Adds an `Index` key to `keys` and then differences the two values.
    pub fn item<Type: Diffable>(&mut self, index: usize, value1: &Type, value2: &Type) {
        self.keys.push_back(Key::Index(index));
        value1.diff_same(self, value2);
        self.keys.pop_back();
    }

    /// Append a list of operations nested within the current keys
    pub fn append(&mut self, ops: Vec<Operation>) {
        for op in ops {
            let mut keys = self.keys.clone();
            match op {
                Operation::Add(mut add) => {
                    keys.append(&mut add.keys);
                    add.keys = keys;
                    self.patch.push(Operation::Add(add))
                }
                Operation::Remove(mut remove) => {
                    keys.append(&mut remove.keys);
                    remove.keys = keys;
                    self.patch.push(Operation::Remove(remove))
                }
                Operation::Replace(mut replace) => {
                    keys.append(&mut replace.keys);
                    replace.keys = keys;
                    self.patch.push(Operation::Replace(replace))
                }
                Operation::Transform(mut transform) => {
                    keys.append(&mut transform.keys);
                    transform.keys = keys;
                    self.patch.push(Operation::Transform(transform))
                }
                _ => todo!(),
            }
        }
    }

    /// Add an `Add` operation to the patch.
    pub fn add<Value: Clone + 'static>(&mut self, value: &Value) {
        self.patch.push(Operation::Add(Add {
            keys: self.keys.clone(),
            value: Box::new(value.clone()),
        }))
    }

    /// Add a `Remove` operation to the patch.
    pub fn remove(&mut self) {
        self.patch.push(Operation::Remove(Remove {
            keys: self.keys.clone(),
            items: 1,
        }))
    }

    /// Add a `Replace` operation to the patch.
    pub fn replace<Value: Clone + 'static>(&mut self, value: &Value) {
        self.patch.push(Operation::Replace(Replace {
            keys: self.keys.clone(),
            items: 1,
            value: Box::new(value.clone()),
        }))
    }

    // TODO Add methods for copy, move

    /// Add a `Transform` operation to the patch.
    pub fn transform(&mut self, from: &str, to: &str) {
        self.patch.push(Operation::Transform(Transform {
            keys: self.keys.clone(),
            from: from.into(),
            to: to.into(),
        }))
    }
}

macro_rules! invalid_op {
    ($op:expr) => {
        report(Error::InvalidPatchOperation {
            op: $op.into(),
            type_name: type_name::<Self>().into(),
        })
    };
}

macro_rules! invalid_keys {
    ($keys:expr) => {
        report(Error::InvalidPatchKeys {
            keys: format!("{:?}", $keys),
            type_name: type_name::<Self>().into(),
        })
    };
}

macro_rules! invalid_name {
    ($name:expr) => {
        report(Error::InvalidPatchName {
            name: $name.into(),
            type_name: type_name::<Self>().into(),
        })
    };
}

macro_rules! invalid_index {
    ($index:expr) => {
        report(Error::InvalidPatchIndex {
            index: $index.into(),
            type_name: type_name::<Self>().into(),
        })
    };
}

pub trait Diffable {
    /// Test whether a node is the same as (i.e. equal type and equal value)
    /// another node of any type.
    fn is_same<Other: Any + Clone>(&self, other: &Other) -> Result<()>;

    /// Test whether a node is equal to (i.e. equal value) a node of the same type.
    fn is_equal(&self, other: &Self) -> Result<()>;

    /// Generate the operations needed to mutate this node so that is the same as
    /// a node of any other type.
    ///
    /// `Other` needs to be `Clone` so that if necessary, we can keep a copy of it in a
    /// `Add` or `Replace operation.
    fn diff<Other: Any + Clone>(&self, differ: &mut Differ, other: &Other);

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
    fn diff_other<Other: Any + Clone>(&self, differ: &mut Differ, other: &Other) {
        differ.replace(other)
    }

    /// Apply a patch operation to this node.
    fn apply(&mut self, op: &Operation) {
        match op {
            Operation::Add(op) => self.apply_add(&mut op.keys.clone(), &op.value),
            Operation::Remove(op) => self.apply_remove(&mut op.keys.clone(), op.items),
            Operation::Replace(op) => self.apply_replace(&mut op.keys.clone(), op.items, &op.value),
            // TODO copy and move require us to get the from value first and then call self.add
            // with the value
            // Copy
            // Move
            Operation::Transform(op) => {
                self.apply_transform(&mut op.keys.clone(), &op.from, &op.to)
            }
            _ => todo!(),
        }
    }

    /// Apply an `Add` patch operation
    fn apply_add(&mut self, _keys: &mut Keys, _value: &Box<dyn Any>) {
        invalid_op!("add")
    }

    /// Apply a `Remove` patch operation
    fn apply_remove(&mut self, _keys: &mut Keys, _items: usize) {
        invalid_op!("remove")
    }

    /// Apply a `Replace` patch operation
    fn apply_replace(&mut self, _keys: &mut Keys, _items: usize, _value: &Box<dyn Any>) {
        invalid_op!("replace")
    }

    /// Apply a `Transform` patch operation
    fn apply_transform(&mut self, _keys: &mut Keys, _from: &str, _to: &str) {
        invalid_op!("transform")
    }
}

/// Macro to generate the `same` method for a type
macro_rules! diffable_is_same {
    ($type:ty) => {
        fn is_same<Other: Any + Clone>(&self, other: &Other) -> Result<()> {
            if let Some(other) = (other as &dyn Any).downcast_ref::<$type>() {
                self.is_equal(&other)
            } else {
                bail!(Error::NotSame)
            }
        }
    };
}

/// Macro to generate the `mutate` method for a type
macro_rules! diffable_diff {
    ($type:ty) => {
        fn diff<Other: Any + Clone>(&self, differ: &mut Differ, other: &Other) {
            if let Some(other) = (other as &dyn Any).downcast_ref::<$type>() {
                self.diff_same(differ, other)
            } else {
                self.diff_other(differ, other)
            }
        }
    };
}

mod prelude;

mod atomics;
mod string;

mod option;
mod vec;

mod blocks;
mod inlines;
mod works;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_json, assert_json_eq};
    use pretty_assertions::assert_eq;
    use stencila_schema::{Emphasis, InlineContent, Integer, Paragraph};

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
    fn test_diff_apply() {
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
        apply(&mut patched, &patch);
        assert_json_eq!(patched, empty);

        // Patching `empty` to `a` should:
        // - replace all content with the content of `a`

        let patch = diff(&empty, &a);
        assert_json!(
            patch,
            [{
                "op": "replace",
                "keys": ["content"],
                "items": 1,
                "value": ["word1", "word2"]
            }]
        );

        let mut patched = empty.clone();
        apply(&mut patched, &patch);
        assert_json_eq!(patched, a);

        // Patching `a` to `b` should:
        // - transform `content[0]` from a string to an `Emphasis`
        // - replace part of `content[1]`

        let patch = diff(&a, &b);
        assert_json!(
            patch,
            [{
                "op": "transform",
                "keys": ["content", 0],
                "from": "String",
                "to": "Emphasis"
            },{
                "op": "replace",
                "keys": ["content", 1, 2],
                "items": 3,
                "value": "two"
            }]
        );

        let mut patched = a.clone();
        apply(&mut patched, &patch);
        assert_json_eq!(patched, b);
    }
}
