use crate::errors::{report, Error};
use defaults::Defaults;
use eyre::Result;
use serde::{Serialize, Serializer};
use std::{
    any::{type_name, Any},
    collections::VecDeque,
    fmt::Debug,
    hash::Hasher,
    iter::FromIterator,
};
use stencila_schema::{BlockContent, Boolean, InlineContent, Integer, Number};

/// Whether or not two nodes are the same type and value.
pub fn same<Type1, Type2>(node1: &Type1, node2: &Type2) -> bool
where
    Type1: Patchable,
    Type2: Clone + 'static,
{
    node1.is_same(node2).is_ok()
}

/// Whether or not two nodes of the same type have equal value.
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

/// Apply a [`Patch`] to a node of any type.
pub fn apply<Type>(node: &mut Type, patch: &[Operation])
where
    Type: Patchable,
{
    node.apply_patch(patch)
}

/// Apply a [`Patch`] to a clone of a node of any type.
pub fn apply_new<Type>(node: &Type, patch: &[Operation]) -> Type
where
    Type: Patchable + Clone,
{
    let mut node = node.clone();
    node.apply_patch(patch);
    node
}

/// A vector of [`Operation`]s describing the difference between two nodes.
type Patch = Vec<Operation>;

/// An enumeration of the types of operations that can be used in a [`Patch`] to
/// mutate one node into another.
///
/// These are the same operations as described in [JSON Patch](http://jsonpatch.com/)
/// (with the exception of `copy` and `test`). Note that `Replace` and `Move` can be
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
///
/// The `length` field on `Add` and `Replace` is not necessary for applying operations, but
/// is useful for generating them and for determining if there are conflicts between two patches
/// without having to downcast the `value`.
///
/// Note that for `String`s indices in `keys`, `items` and `length` all refer to Unicode characters,
/// not bytes.
#[derive(Debug, Serialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum Operation {
    /// Add a value
    Add {
        /// The location to which to add the value
        keys: Keys,

        /// The value to add
        #[serde(serialize_with = "serialize_value")]
        value: Box<dyn Any>,

        /// The number of items added
        length: usize,
    },
    /// Remove one or more values
    Remove {
        /// The location from which to remove the value/s
        keys: Keys,

        /// The number of items to remove
        items: usize,
    },
    /// Replace one or more values
    Replace {
        /// The location which should be replaced
        keys: Keys,

        /// The number of items to replace
        items: usize,

        /// The replacement value
        #[serde(serialize_with = "serialize_value")]
        value: Box<dyn Any>,

        /// The number of items added
        length: usize,
    },
    /// Move a value from one location in a node tree, to another
    Move {
        /// The location from which to remove the value
        from: Keys,

        /// The number of items to move
        items: usize,

        /// The location to which to add the items
        to: Keys,
    },
    /// Transform a value from one type to another
    Transform {
        /// The location of the node to transform
        keys: Keys,

        /// The type of node to transform from
        from: String,

        /// The type of node to transform to
        to: String,
    },
}

/// Serialize the `value` field of an operation
///
/// This is mainly for debugging and testing. Serialization of types is added as
/// needed.
fn serialize_value<S>(value: &Box<dyn Any>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    macro_rules! ser {
        ($type:ty) => {
            if let Some(value) = value.downcast_ref::<$type>() {
                return value.serialize(serializer);
            }
        };
    }

    ser!(Boolean);
    ser!(Integer);
    ser!(Number);
    ser!(String);
    ser!(InlineContent);
    ser!(BlockContent);

    ser!(Vec<Boolean>);
    ser!(Vec<Integer>);
    ser!(Vec<Number>);
    ser!(Vec<String>);
    ser!(Vec<InlineContent>);
    ser!(Vec<BlockContent>);

    serializer.serialize_str("<unserialized type>")
}

/// A key of a `struct`, `HashMap`, or `Vec` used to locate an operation.
#[derive(Debug, Clone, Serialize)]
#[serde(untagged)]
pub enum Key {
    Index(usize),
    Name(String),
}

pub type Keys = VecDeque<Key>;

fn keys_from_index(index: usize) -> Keys {
    VecDeque::from_iter(vec![Key::Index(index)])
}

fn keys_concat(begin: &Keys, end: &Keys) -> Keys {
    let mut keys = begin.clone();
    keys.append(&mut end.clone());
    keys
}

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
    pub fn field<Type: Patchable>(&mut self, name: &str, value1: &Type, value2: &Type) {
        self.keys.push_back(Key::Name(name.to_string()));
        value1.diff_same(self, value2);
        self.keys.pop_back();
    }

    /// Difference an item in a `Vec`.
    ///
    /// Adds an `Index` key to `keys` and then differences the two values.
    pub fn item<Type: Patchable>(&mut self, index: usize, value1: &Type, value2: &Type) {
        self.keys.push_back(Key::Index(index));
        value1.diff_same(self, value2);
        self.keys.pop_back();
    }

    /// Append a list of operations nested within the current keys
    pub fn append(&mut self, ops: Vec<Operation>) {
        for op in ops {
            let op = match op {
                Operation::Add {
                    keys,
                    value,
                    length,
                } => Operation::Add {
                    keys: keys_concat(&self.keys, &keys),
                    value,
                    length,
                },
                Operation::Remove { keys, items } => Operation::Remove {
                    keys: keys_concat(&self.keys, &keys),
                    items,
                },
                Operation::Replace {
                    keys,
                    items,
                    value,
                    length,
                } => Operation::Replace {
                    keys: keys_concat(&self.keys, &keys),
                    items,
                    value,
                    length,
                },
                Operation::Move { from, items, to } => Operation::Move {
                    from: keys_concat(&self.keys, &from),
                    items,
                    to: keys_concat(&self.keys, &to),
                },
                Operation::Transform { keys, from, to } => Operation::Transform {
                    keys: keys_concat(&self.keys, &keys),
                    from,
                    to,
                },
            };
            self.patch.push(op)
        }
    }

    /// Add an `Add` operation to the patch.
    pub fn add<Value: Clone + 'static>(&mut self, value: &Value) {
        self.patch.push(Operation::Add {
            keys: self.keys.clone(),
            value: Box::new(value.clone()),
            length: 1,
        })
    }

    /// Add a `Remove` operation to the patch.
    pub fn remove(&mut self) {
        self.patch.push(Operation::Remove {
            keys: self.keys.clone(),
            items: 1,
        })
    }

    /// Add a `Replace` operation to the patch.
    pub fn replace<Value: Clone + 'static>(&mut self, value: &Value) {
        self.patch.push(Operation::Replace {
            keys: self.keys.clone(),
            items: 1,
            value: Box::new(value.clone()),
            length: 1,
        })
    }

    /// Add a `Transform` operation to the patch.
    pub fn transform(&mut self, from: &str, to: &str) {
        self.patch.push(Operation::Transform {
            keys: self.keys.clone(),
            from: from.into(),
            to: to.into(),
        })
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

macro_rules! invalid_value {
    () => {
        report(Error::InvalidPatchValue {
            type_name: type_name::<Self>().into(),
        })
    };
}

pub trait Patchable {
    /// Test whether a node is the same as (i.e. equal type and equal value)
    /// another node of any type.
    fn is_same<Other: Any + Clone>(&self, other: &Other) -> Result<()>;

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

    /// Apply a patch to this node.
    fn apply_patch(&mut self, patch: &[Operation]) {
        for op in patch {
            self.apply_op(op)
        }
    }

    /// Apply an operation to this node.
    fn apply_op(&mut self, op: &Operation) {
        match op {
            Operation::Add { keys, value, .. } => self.apply_add(&mut keys.clone(), value),
            Operation::Remove { keys, items } => self.apply_remove(&mut keys.clone(), *items),
            Operation::Replace {
                keys, items, value, ..
            } => self.apply_replace(&mut keys.clone(), *items, value),
            Operation::Move { from, items, to } => {
                self.apply_move(&mut from.clone(), *items, &mut to.clone())
            }
            Operation::Transform { keys, from, to } => {
                self.apply_transform(&mut keys.clone(), from, to)
            }
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

    /// Apply a `Move` patch operation
    fn apply_move(&mut self, _from: &mut Keys, _items: usize, _to: &mut Keys) {
        invalid_op!("move")
    }

    /// Apply a `Transform` patch operation
    fn apply_transform(&mut self, _keys: &mut Keys, _from: &str, _to: &str) {
        invalid_op!("transform")
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

/// Generate the `is_equal` method for a `struct`
macro_rules! patchable_is_equal {
    ($( $field:ident )*) => {
        fn is_equal(&self, other: &Self) -> Result<()> {
            $(
                self.$field.is_equal(&other.$field)?;
            )*
            Ok(())
        }
    };
}

/// Generate the `hash` method for a `struct`
macro_rules! patchable_hash {
    ($( $field:ident )*) => {
        fn make_hash<H: std::hash::Hasher>(&self, state: &mut H) {
            // Include the type name in the hash (to avoid clash when structs
            // of different types have the same values for different fields)
            use std::hash::Hash;
            type_name::<Self>().hash(state);
            // Include the hash of supplied fields. Because we include the type
            // name in the hash, we do no need to include the field names.
            $(
                self.$field.make_hash(state);
            )*
        }
    };
}

/// Generate the `diff` method for a type
macro_rules! patchable_diff {
    () => {
        fn diff<Other: Any + Clone>(&self, differ: &mut Differ, other: &Other) {
            if let Some(other) = (other as &dyn Any).downcast_ref::<Self>() {
                self.diff_same(differ, other)
            } else {
                self.diff_other(differ, other)
            }
        }
    };
}

/// Generate the `diff_same` method for a `struct`
macro_rules! patchable_diff_same {
    ($( $field:ident )*) => {
        fn diff_same(&self, differ: &mut Differ, other: &Self) {
            $(
                differ.field(stringify!($field), &self.$field, &other.$field);
            )*
        }
    };
}

/// Generate the `apply_add` method for a `struct`
macro_rules! patchable_apply_add {
    ($( $field:ident )*) => {
        fn apply_add(&mut self, keys: &mut Keys, value: &Box<dyn Any>) {
            if let Some(Key::Name(name)) = keys.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_add(keys, value),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_keys!(keys)
            }
        }
    };
}

/// Generate the `apply_remove` method for a `struct`
macro_rules! patchable_apply_remove {
    ($( $field:ident )*) => {
        fn apply_remove(&mut self, keys: &mut Keys, items: usize) {
            if let Some(Key::Name(name)) = keys.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_remove(keys, items),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_keys!(keys)
            }
        }
    };
}

/// Generate the `apply_replace` method for a `struct`
macro_rules! patchable_apply_replace {
    ($( $field:ident )*) => {
        fn apply_replace(&mut self, keys: &mut Keys, items: usize, value: &Box<dyn Any>) {
            if let Some(Key::Name(name)) = keys.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_replace(keys, items, value),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_keys!(keys)
            }
        }
    };
}

/// Generate the `apply_move` method for a `struct`
macro_rules! patchable_apply_move {
    ($( $field:ident )*) => {
        fn apply_move(&mut self, from: &mut Keys, items: usize, to: &mut Keys) {
            if let (Some(Key::Name(name)), Some(Key::Name(_name_again))) = (from.pop_front(), to.pop_front()) {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_move(from, items, to),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_keys!(from)
            }
        }
    };
}

/// Generate the `apply_transform` method for a `struct`
macro_rules! patchable_apply_transform {
    ($( $field:ident )*) => {
        fn apply_transform(&mut self, keys: &mut Keys, from: &str, to: &str) {
            if let Some(Key::Name(name)) = keys.pop_front() {
                match name.as_str() {
                    $(
                        stringify!($field) => self.$field.apply_transform(keys, from, to),
                    )*
                    _ => invalid_name!(name),
                }
            } else {
                invalid_keys!(from)
            }
        }
    };
}

/// Generate all the patchable methods for a `struct` by passing
/// a list of fields for comparison, diffing, and applying ops.
macro_rules! patchable_struct {
    ($type:ty $(, $field:ident )*) => {
        impl Patchable for $type {
            patchable_is_same!();
            patchable_is_equal!($( $field )*);
            patchable_hash!($( $field )*);

            patchable_diff!();
            patchable_diff_same!($( $field )*);

            patchable_apply_add!($( $field )*);
            patchable_apply_remove!($( $field )*);
            patchable_apply_replace!($( $field )*);
            patchable_apply_move!($( $field )*);
            patchable_apply_transform!($( $field )*);
        }
    };
}

/// Generate the `impl Patchable` for a type
macro_rules! patchable_todo {
    ($type:ty) => {
        impl Patchable for $type {
            patchable_is_same!();

            fn is_equal(&self, _other: &Self) -> Result<()> {
                todo!()
            }

            fn make_hash<H: Hasher>(&self, _state: &mut H) {
                todo!()
            }

            patchable_diff!();

            fn diff_same(&self, _differ: &mut Differ, _other: &Self) {
                todo!()
            }
        }
    };
}

mod prelude;

mod atomics;
mod strings;

mod boxes;
mod options;
mod vecs;

mod blocks;
mod inlines;
mod works;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{assert_json, assert_json_eq};
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
                "op": "add",
                "keys": ["content", 0],
                "value": ["word1", "word2"],
                "length": 2
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
                "value": "two",
                "length": 3
            }]
        );

        let mut patched = a.clone();
        apply(&mut patched, &patch);
        assert_json_eq!(patched, b);
    }
}
