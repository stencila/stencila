use std::{
    any::{type_name, Any},
    fmt::Debug,
};

use schemars::JsonSchema;
use similar::TextDiff;

use common::{
    defaults::Defaults,
    eyre::{bail, Result},
    serde::{de::DeserializeOwned, Deserialize, Deserializer, Serialize, Serializer},
    serde_json,
    serde_with::skip_serializing_none,
    strum::Display,
    tracing,
};
use node_address::{Address, Slot};
use node_pointer::{find_mut, resolve_mut, Pointable};
use stencila_schema::*;

/// Generate a [`Patch`] describing the difference between two nodes of the same type.
#[tracing::instrument(skip(node1, node2))]
pub fn diff<Type>(node1: &Type, node2: &Type) -> Patch
where
    Type: Patchable,
{
    let mut differ = Differ::default();
    node1.diff(node2, &mut differ);
    Patch::from_ops(differ.ops)
}

/// Generate a [`Patch`] describing the difference between two nodes of the same type
/// at a specific id.
#[tracing::instrument(skip(node1, node2))]
pub fn diff_id<Type>(id: &str, node1: &Type, node2: &Type) -> Patch
where
    Type: Patchable,
{
    let mut patch = diff(node1, node2);
    patch.target = Some(id.to_string());
    patch
}

/// Generate a [`Patch`] describing the difference between two nodes of the same type
/// at a specific address.
#[tracing::instrument(skip(node1, node2))]
pub fn diff_address<Type>(address: &Address, node1: &Type, node2: &Type) -> Patch
where
    Type: Patchable,
{
    let mut patch = diff(node1, node2);
    patch.address = Some(address.clone());
    patch
}

/// Generate a [`Patch`] using a recipe function
///
/// Inspired by [Immer](https://immerjs.github.io/immer/produce/)'s `produce` function.
pub fn produce<T: Clone + Patchable, F: Fn(&mut T)>(
    node: &T,
    node_id: Option<String>,
    node_address: Option<Address>,
    recipe: F,
) -> Patch {
    let mut draft = node.clone();
    recipe(&mut draft);

    let mut patch = diff(node, &draft);
    patch.target = node_id;
    patch.address = node_address;
    patch
}

pub fn produce_address<T: Clone + Patchable, F: Fn(&mut T)>(
    node: &T,
    address: &Address,
    recipe: F,
) -> Patch {
    produce(node, None, Some(address.clone()), recipe)
}

/// Generate a [`Patch`] using a mutating function
///
/// Like [`produce`] but mutates the node as well as generating a patch.
pub fn mutate<T: Clone + Patchable, F: Fn(&mut T)>(
    node: &mut T,
    node_id: Option<String>,
    node_address: Option<Address>,
    recipe: F,
) -> Patch {
    let before = node.clone();
    recipe(node);

    let mut patch = diff(&before, node);
    patch.target = node_id;
    patch.address = node_address;
    patch
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

    let old = codecs::to_string(node1, format, None).await?;
    let new = codecs::to_string(&patched, format, None).await?;

    let mut bytes = Vec::new();
    TextDiff::from_lines(&old, &new)
        .unified_diff()
        .to_writer(&mut bytes)
        .unwrap();

    let display = String::from_utf8(bytes)?;
    Ok(display)
}

/// Apply a [`Patch`] to a node.
#[tracing::instrument(skip(node, patch))]
pub fn apply<Type>(node: &mut Type, patch: &Patch) -> Result<()>
where
    Type: Patchable + Pointable,
{
    if let Some(address) = &patch.address {
        let mut pointer = resolve_mut(node, address.clone())?;
        if let Some(inline) = pointer.as_inline_mut() {
            inline.apply_patch(patch)
        } else if let Some(block) = pointer.as_block_mut() {
            block.apply_patch(patch)
        } else if let Some(node) = pointer.as_node_mut() {
            node.apply_patch(patch)
        } else {
            bail!("Pointer points to unhandled node type")
        }
    } else if let Some(id) = &patch.target {
        let mut pointer = find_mut(node, id)?;
        if let Some(inline) = pointer.as_inline_mut() {
            inline.apply_patch(patch)
        } else if let Some(block) = pointer.as_block_mut() {
            block.apply_patch(patch)
        } else if let Some(node) = pointer.as_node_mut() {
            node.apply_patch(patch)
        } else {
            bail!("Pointer points to unhandled node type")
        }
    } else {
        node.apply_patch(patch)
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
#[tracing::instrument(skip(ancestor, derived))]
pub fn merge<Type>(ancestor: &mut Type, derived: &[&Type]) -> Result<()>
where
    Type: Patchable + Pointable,
{
    let patches: Vec<Patch> = derived.iter().map(|node| diff(ancestor, *node)).collect();

    // TODO transform operations (shift address based on other operations) and resolve conflicts
    tracing::warn!("Merging is work in progress");

    for patch in patches {
        apply(ancestor, &patch)?;
    }
    Ok(())
}

/// Type for the `value` property of `Add` and `Replace` operations
///
/// This open, dynamic type could be replaced with a enum (with a fixed number
/// of type variants) but that would require substantial refactoring
pub type Value = Box<dyn Any + Send>;

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
#[skip_serializing_none]
#[derive(Debug, Display, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
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
        #[schemars(skip)]
        value: Value,

        /// The number of items added
        length: usize,

        /// The HTML encoding of `value`
        html: Option<String>,
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
        #[schemars(skip)]
        value: Value,

        /// The number of items added
        length: usize,

        /// The HTML encoding of `value`
        html: Option<String>,
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
    /// Deserialize the `value` field of an operation
    ///
    /// This is needed so that the server can receive a `Patch` from the client and
    /// deserialize the JSON value into a `Value`.
    fn value_deserialize<'de, D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
        Ok(Box::new(value))
    }

    /// Serialize the `value` field of an operation
    ///
    /// This is needed so that the server can send a `Patch` to a client with
    /// the `value` field as JSON. It is also, more generally useful for serializing
    /// patches e.g. for test snapshots.
    fn value_serialize<S>(value: &Value, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        macro_rules! serialize {
            ($type:ty) => {
                if let Some(value) = value.downcast_ref::<$type>() {
                    return value.serialize(serializer);
                }
                if let Some(value) = value.downcast_ref::<Option<$type>>() {
                    return value.serialize(serializer);
                }
                if let Some(value) = value.downcast_ref::<Box<$type>>() {
                    return value.serialize(serializer);
                }
                if let Some(value) = value.downcast_ref::<Option<Box<$type>>>() {
                    return value.serialize(serializer);
                }
                if let Some(value) = value.downcast_ref::<Vec<$type>>() {
                    return value.serialize(serializer);
                }
            };
            ($($type:ty)*) => {
                $(serialize!($type);)*
            }
        }

        // For performance, types roughly ordered by expected incidence (more commonly used
        // types in patches first).
        serialize!(
            // Main content types
            InlineContent
            BlockContent

            // Types related to compilation and execution
            ExecutionStatus
            ExecutionRequired
            ExecutionAuto
            ExecutionDigest
            ExecutionDependency
            ExecutionDependencyRelation
            ExecutionDependencyNode
            ExecutionDependent
            ExecutionDependentRelation
            ExecutionDependentNode

            // Child types of the InlineContent and BlockContent
            CallArgument
            CodeChunkCaption
            CodeError
            Datatable
            DatatableColumn
            FigureCaption
            IfClause
            ListItem
            Node
            TableCaption
            TableCell
            TableCellCellType
            TableRow
            ValidatorTypes
            EnumValidator // Because "replaceable"

            // Properties of creative works
            Person
            Organization

            // Primitives
            Primitive
            String
            Number
            Integer
            Date
            Time
            DateTime
            Timestamp
            Duration
            Boolean
            Array
            Object
            Null

            // Types used on some properties e.g. `Heading.depth`, `TableCell.rowspan`
            u8
            u32
            u64
            i32
            f32

            // Used for vectors of vectors of blocks in `For` iterations
            Vec<BlockContent>
        );

        // The value may be a JSON value (if this patch was sent from a client).
        // In that case we can just serialize it.
        if let Some(value) = value.downcast_ref::<serde_json::Value>() {
            return value.serialize(serializer);
        }

        tracing::error!("Unhandled value type when serializing patch operation");
        "<unserialized type>".serialize(serializer)
    }

    /// Generate HTML for the `value` field of an operation
    fn value_html(value: &Value, root: &Node) -> Option<String> {
        use codec_html::{EncodeContext, ToHtml};

        let mut context = EncodeContext {
            root,
            ..Default::default()
        };

        // Convert a node, boxed node, or vector of nodes to HTML
        macro_rules! to_html {
            ($type:ty) => {
                if let Some(node) = value.downcast_ref::<$type>() {
                    return Some(node.to_html(&mut context));
                }
                if let Some(boxed) = value.downcast_ref::<Box<$type>>() {
                    return Some(boxed.to_html(&mut context));
                }
                if let Some(nodes) = value.downcast_ref::<Vec<$type>>() {
                    return Some(nodes.to_html(&mut context));
                }
            };
            ($($type:ty)*) => {
                $(to_html!($type);)*
            }
        }

        // For performance, types roughly ordered by expected incidence (more commonly used
        // types in patches first).
        to_html!(
            // Main content types
            InlineContent
            BlockContent

            // Types related to compilation of code
            ExecutionDependency
            ExecutionDependent

            // Child types of the above
            CallArgument
            CodeChunkCaption
            CodeError
            Datatable
            DatatableColumn
            FigureCaption
            IfClause
            ListItem
            Node
            TableCaption
            TableCell
            TableCellCellType
            TableRow
            ValidatorTypes
            EnumValidator // Because "replaceable"

            // Primitives
            Primitive
            String
            Number
            Integer
            Date
            Time
            DateTime
            Timestamp
            Duration
            Boolean
            Array
            Object
            Null
        );

        // Convert an atomic (used in some struct properties e.g. `Heading.depth`). These
        // don't usually need to be a HTML (they are handled differently) but for consistency
        // we generate it anyway
        macro_rules! to_html_atomic {
            ($type:ty) => {
                if let Some(node) = value.downcast_ref::<$type>() {
                    return Some(node.to_string())
                }
            };
            ($($type:ty)*) => {
                $(to_html_atomic!($type);)*
            }
        }
        to_html_atomic!(
            u8
            u32
            i32
            f32
        );

        // The value may be a JSON value (if this patch was sent from a client)
        // In that case we want to deserialize it to one of the above types and
        // then encode as HTML
        if let Some(value) = value.downcast_ref::<serde_json::Value>() {
            let html = if let Some(str) = value.as_str() {
                str.to_string()
            } else if let Ok(nodes) = serde_json::from_value::<InlineContent>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<Vec<InlineContent>>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<BlockContent>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<Vec<BlockContent>>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<ListItem>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<Vec<ListItem>>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<TableRow>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<Vec<TableRow>>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<TableCell>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<Vec<TableCell>>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<IfClause>(value.clone()) {
                nodes.to_html(&mut context)
            } else if let Ok(nodes) = serde_json::from_value::<ValidatorTypes>(value.clone()) {
                nodes.to_html(&mut context)
            } else {
                tracing::error!(
                    "Unhandled JSON value type when generating HTML for patch operation: {}",
                    value.to_string()
                );
                return None;
            };
            return Some(html);
        }

        // Return `None` to indicate no HTML representation for this value
        None
    }

    /// Set the `html` field from the `value` field
    fn html_set(&mut self, root: &Node) {
        match self {
            Operation::Add { value, html, .. } | Operation::Replace { value, html, .. } => {
                // As an optimization, if the patch value is string-like
                // (but not if it is a `InlineContent::String` or `Node::String`), then there
                // is no need to generate HTML since it is the same as the value and the `web`
                // module will fallback to `value` if necessary.
                if value.is::<String>() {
                    return;
                }
                if let Some(value) = value.downcast_mut::<serde_json::Value>() {
                    if value.is_string() {
                        return;
                    }
                }

                *html = Operation::value_html(value, root)
            }
            _ => {}
        }
    }
}

/// A set of [`Operation`]s
#[skip_serializing_none]
#[derive(Debug, Default, JsonSchema, Serialize, Deserialize)]
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
                                Operation::Add { address, .. }
                                | Operation::Remove { address, .. }
                                | Operation::Replace { address, .. }
                                | Operation::Transform { address, .. } => {
                                    address.prepend(&patch_address)
                                }
                                Operation::Move { from, to, .. } => {
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
            if let Operation::Remove { address, .. } | Operation::Replace { address, .. } = op {
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
    /// The main purpose of this function is to generate HTML for each `Add` and `Replace`
    /// operation in the patch before it is sent to clients.
    #[tracing::instrument(skip(self, root))]
    pub fn prepublish(&mut self, version: u64, root: &Node) -> &mut Self {
        self.version = Some(version);
        for op in self.ops.iter_mut() {
            op.html_set(root);
        }
        self
    }
}

/// A differencing `struct` used as an optimization to track the address describing the
/// current location in a node tree while walking over it.
#[derive(Defaults)]
pub struct Differ {
    /// The list of address describing the current location in a node tree.
    address: Address,

    /// The operations generated by walking over a node tree.
    ops: Vec<Operation>,
}

impl Differ {
    /// Difference a field of a `struct` or an item of a `HashMap`.
    ///
    /// Adds a `Name` key to `address` and then differences the two values.
    pub fn field<Type: Patchable>(&mut self, name: &str, value1: &Type, value2: &Type) {
        self.address.push_back(Slot::Name(name.to_string()));
        value1.diff(value2, self);
        self.address.pop_back();
    }

    /// Difference an item in a `Vec`.
    ///
    /// Adds an `Index` key to `address` and then differences the two values.
    pub fn item<Type: Patchable>(&mut self, index: usize, value1: &Type, value2: &Type) {
        self.address.push_back(Slot::Index(index));
        value1.diff(value2, self);
        self.address.pop_back();
    }

    /// Push an operations nested within the current address
    pub fn push(&mut self, op: Operation) {
        let op = match op {
            Operation::Add {
                address,
                value,
                length,
                html,
            } => Operation::Add {
                address: self.address.concat(&address),
                value,
                length,
                html,
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
                html,
            } => Operation::Replace {
                address: self.address.concat(&address),
                items,
                value,
                length,
                html,
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
        self.ops.push(op)
    }

    /// Append a list of operations nested within the current address
    pub fn append(&mut self, ops: Vec<Operation>) {
        for op in ops {
            self.push(op)
        }
    }

    /// Add an `Add` operation to the patch.
    pub fn add<Value: Clone + Send + 'static>(&mut self, value: &Value) {
        self.ops.push(Operation::Add {
            address: self.address.clone(),
            value: Box::new(value.clone()),
            length: 1,
            html: None,
        })
    }

    /// Add a `Remove` operation to the patch.
    pub fn remove(&mut self) {
        self.ops.push(Operation::Remove {
            address: self.address.clone(),
            items: 1,
        })
    }

    /// Add a `Replace` operation to the patch.
    pub fn replace<Value: Clone + Send + 'static>(&mut self, value: &Value) {
        self.ops.push(Operation::Replace {
            address: self.address.clone(),
            items: 1,
            value: Box::new(value.clone()),
            length: 1,
            html: None,
        })
    }

    /// Add a `Transform` operation to the patch.
    pub fn transform(&mut self, from: &str, to: &str) {
        self.ops.push(Operation::Transform {
            address: self.address.clone(),
            from: from.into(),
            to: to.into(),
        })
    }
}

pub trait Patchable {
    /// Generate the operations needed to mutate this node so that it is equal
    /// to a node of the same type.
    fn diff(&self, other: &Self, differ: &mut Differ);

    /// Apply a patch to this node.
    fn apply_patch(&mut self, patch: &Patch) -> Result<()> {
        tracing::trace!("Applying patch to type '{}'", type_name::<Self>());
        for op in &patch.ops {
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
    fn apply_add(&mut self, _address: &mut Address, _value: &Value) -> Result<()> {
        bail!(invalid_patch_operation::<Self>("Add"))
    }

    /// Apply a `Remove` patch operation
    fn apply_remove(&mut self, _address: &mut Address, _items: usize) -> Result<()> {
        bail!(invalid_patch_operation::<Self>("Remove"))
    }

    /// Apply a `Replace` patch operation
    fn apply_replace(
        &mut self,
        _address: &mut Address,
        _items: usize,
        _value: &Value,
    ) -> Result<()> {
        bail!(invalid_patch_operation::<Self>("Replace"))
    }

    /// Apply a `Move` patch operation
    fn apply_move(&mut self, _from: &mut Address, _items: usize, _to: &mut Address) -> Result<()> {
        bail!(invalid_patch_operation::<Self>("Move"))
    }

    /// Apply a `Transform` patch operation
    fn apply_transform(&mut self, _address: &mut Address, _from: &str, _to: &str) -> Result<()> {
        bail!(invalid_patch_operation::<Self>("Transform"))
    }

    /// Cast a [`Value`] to an instance of the type
    fn from_value(value: &Value) -> Result<Self>
    where
        Self: Clone + DeserializeOwned + Sized + 'static,
    {
        let instance = if let Some(value) = value.downcast_ref::<Self>() {
            value.clone()
        } else if let Some(value) = value.downcast_ref::<serde_json::Value>() {
            Self::from_json_value(value)?
        } else {
            bail!(invalid_patch_value::<Self>())
        };
        Ok(instance)
    }

    /// Parse a JSON value to an instance of the type
    fn from_json_value(value: &serde_json::Value) -> Result<Self>
    where
        Self: Clone + DeserializeOwned + Sized + 'static,
    {
        if let Ok(value) = serde_json::from_value::<Self>(value.clone()) {
            // The JSON value was of the correct type e.g. `42` for a number
            Ok(value)
        } else if let Some(value) = value
            .as_str()
            .and_then(|json| serde_json::from_str::<Self>(json).ok())
        {
            // The JSON value was a string that could be parsed into the correct type e.g. `"42"` for a number
            Ok(value)
        } else {
            bail!(
                "Invalid JSON patch value for type `{}`: {:?}",
                std::any::type_name::<Self>(),
                value
            )
        }
    }
}

mod errors;
use errors::{invalid_patch_operation, invalid_patch_value};

mod prelude;

#[macro_use]
mod enums;
mod boxes;
mod options;
mod strings;
#[macro_use]
mod structs;
mod maps;
mod vecs;

mod blocks;
mod data;
mod inlines;
mod nodes;
mod others;
mod primitives;
mod works;

#[cfg(test)]
mod tests {
    use super::*;
    use stencila_schema::{Article, Emphasis, InlineContent, Paragraph};
    use test_utils::{assert_json_eq, assert_json_is, pretty_assertions::assert_eq};

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
        assert_json_is!(patch.ops, []);

        let mut patched = empty.clone();
        apply(&mut patched, &patch)?;
        assert_json_eq!(patched, empty);

        // Patching `empty` to `a` should:
        // - replace all content with the content of `a`

        let patch = diff(&empty, &a);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Add",
                "address": ["content", 0],
                "value": ["word1", "word2"],
                "length": 2
            }]
        );

        let mut patched = empty;
        apply(&mut patched, &patch)?;
        assert_json_eq!(patched, a);

        // Patching `a` to `b` should:
        // - transform `content[0]` from a string to an `Emphasis`
        // - replace part of `content[1]`

        let patch = diff(&a, &b);
        assert_json_is!(
            patch.ops,
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

        let mut patched = a;
        apply(&mut patched, &patch)?;
        assert_json_eq!(patched, b);

        Ok(())
    }

    #[test]
    fn test_serialization() {
        // Empty article
        let one = Node::Article(Article {
            content: Some(vec![]),
            ..Default::default()
        });

        // Add an empty paragraph
        let two = Node::Article(Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph::default())]),
            ..Default::default()
        });

        // Add words to the paragraph
        let three = Node::Article(Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph {
                content: vec![
                    InlineContent::String("first".to_string()),
                    InlineContent::String(" second".to_string()),
                ],
                ..Default::default()
            })]),
            ..Default::default()
        });

        // Modify a word
        let four = Node::Article(Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph {
                content: vec![
                    InlineContent::String("foot".to_string()),
                    InlineContent::String(" second".to_string()),
                ],
                ..Default::default()
            })]),
            ..Default::default()
        });

        // Move words
        let five = Node::Article(Article {
            content: Some(vec![BlockContent::Paragraph(Paragraph {
                content: vec![
                    InlineContent::String(" second".to_string()),
                    InlineContent::String("foot".to_string()),
                ],
                ..Default::default()
            })]),
            ..Default::default()
        });

        // one to one -> empty patch
        let patch = diff(&one, &one);
        assert!(patch.ops.is_empty());

        // one to two -> `Add` operation on the article's content
        let mut patch = diff(&one, &two);
        patch.prepublish(0, &two);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Add",
                "address": ["content", 0],
                "value": [{"type": "Paragraph", "content": []}],
                "length": 1,
                "html": "<p itemtype=\"https://schema.stenci.la/Paragraph\" itemscope></p>",
            }]
        );

        // two to three -> `Add` operation on the paragraph's content
        let mut patch = diff(&two, &three);
        patch.prepublish(0, &three);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Add",
                "address": ["content", 0, "content", 0],
                "value": ["first", " second"],
                "length": 2,
                "html": "<span>first</span><span> second</span>",
            }]
        );

        // three to four -> `Replace` operation on a word
        let mut patch = diff(&three, &four);
        patch.prepublish(0, &four);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Replace",
                "address": ["content", 0, "content", 0, 1],
                "items": 3,
                "value": "oo",
                "length": 2
                // No `html` because same as `value`
            }]
        );

        // four to five -> `Move` operation on the word
        let mut patch = diff(&four, &five);
        patch.prepublish(0, &five);
        assert_json_is!(
            patch.ops,
            [{
                "type": "Move",
                "from": ["content", 0, "content", 1],
                "items": 1,
                "to": ["content", 0, "content", 0],
            }]
        );
    }

    /// A regression test of serialization of an patch replacing execution status etc
    #[test]
    fn test_serialize_execute_enums() -> Result<()> {
        let patch = diff(
            &CodeChunk {
                execution_status: None,
                execution_required: Some(ExecutionRequired::NeverExecuted),
                ..Default::default()
            },
            &CodeChunk {
                execution_status: Some(ExecutionStatus::Scheduled),
                execution_required: Some(ExecutionRequired::SemanticsChanged),
                ..Default::default()
            },
        );

        match &patch.ops[0] {
            Operation::Replace { value, .. } => {
                if let Some(value) = value.downcast_ref::<ExecutionRequired>() {
                    assert_eq!(*value, ExecutionRequired::SemanticsChanged);
                } else {
                    bail!("Unexpected value type type");
                }
            }
            _ => bail!("Unexpected operation type"),
        }

        assert_json_is!(patch, {
            "ops": [
                {
                    "type": "Replace",
                    "address": ["executeRequired"],
                    "items": 1,
                    "value": "SemanticsChanged",
                    "length": 1
                },
                {
                    "type": "Add",
                    "address": ["executeStatus"],
                    "value": "Scheduled",
                    "length": 1
                },
            ]
        });

        Ok(())
    }
}
