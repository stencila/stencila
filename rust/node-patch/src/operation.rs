use std::fmt::Debug;

use schemars::JsonSchema;

use common::{
    eyre::Result,
    serde::{Deserialize, Deserializer, Serialize, Serializer},
    serde_json,
    serde_with::skip_serializing_none,
    strum::Display,
    tracing,
};
use node_address::Address;
use node_pointer::Pointable;

use crate::value::Value;

/// The operations within a patch
///
/// These are the same operations as described in [JSON Patch](http://jsonpatch.com/)
/// (with the exception of and `test`).
///
/// In addition, there is a `Transform` operation which can be used describe the transformation
/// of a node to another type that has a similar structure. Examples includes:
///
/// - a `String` to an `Emphasis`
/// - a `Paragraph` to a `QuoteBlock`
/// - a `CodeChunk` to a `CodeBlock`
///
/// Note that `Replace`, `Move` and `Copy` could be represented by combinations of `Remove` and `Add`.
/// They are included as a means of providing more semantically meaningful patches, and more
/// space efficient serializations (e.g. it is not necessary to represent the value being moved or copied).
///
/// The structure of these operations differs from JSON Patch operations:
///
/// - they have an `address` property (an array of sting or integer "slots"), rather than a
///   forward slash separated string `path`
///
/// - the `Remove`, `Replace`, `Move` and `Copy` operations have an `items` property which
///   allows several items in a string or an array to be operated on by a single operation
///
/// The `length` field on `Add` and `Replace` is not necessary for applying operations, but
/// is useful for generating them and for determining if there are conflicts between two patches
/// without having to downcast the `value`.
///
/// Note that for `String`s the integers in `address`, `items` and `length` all refer to Unicode
/// graphemes, not bytes.
#[skip_serializing_none]
#[derive(Debug, Display, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub enum Operation {
    Add(Add),
    Remove(Remove),
    Replace(Replace),
    Move(Move),
    Copy(Copy),
    Transform(Transform),
}

/// Add a value
#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Add {
    /// The address to which to add the value
    pub address: Address,

    /// The value to add
    #[serde(
        serialize_with = "Operation::value_serialize",
        deserialize_with = "Operation::value_deserialize"
    )]
    #[schemars(skip)]
    pub value: Value,

    /// The number of items added
    pub length: usize,

    /// The HTML encoding of `value`
    pub html: Option<String>,
}

/// Remove one or more values
#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Remove {
    /// The address from which to remove the value(s)
    pub address: Address,

    /// The number of items to remove
    pub items: usize,
}

/// Replace one or more values
#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Replace {
    /// The address which should be replaced
    pub address: Address,

    /// The number of items to replace
    pub items: usize,

    /// The replacement value
    #[serde(
        serialize_with = "Operation::value_serialize",
        deserialize_with = "Operation::value_deserialize"
    )]
    #[schemars(skip)]
    pub value: Value,

    /// The number of items added
    pub length: usize,

    /// The HTML encoding of `value`
    pub html: Option<String>,
}

/// Move a value from one address to another
#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Move {
    /// The address from which to remove the value
    pub from: Address,

    /// The number of items to move
    pub items: usize,

    /// The address to which to add the items
    pub to: Address,
}

/// Copy a value from one address to another
#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Copy {
    /// The address from which to copy the value
    pub from: Address,

    /// The number of items to copy
    pub items: usize,

    /// The address to which to copy the items
    pub to: Address,
}

/// Transform a value from one type to another
#[skip_serializing_none]
#[derive(Debug, JsonSchema, Serialize, Deserialize)]
#[serde(tag = "type", crate = "common::serde")]
pub struct Transform {
    /// The address of the `Node` to transform
    pub address: Address,

    /// The type of `Node` to transform from
    pub from: String,

    /// The type of `Node` to transform to
    pub to: String,
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
        Ok(Value::any(value))
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
        use stencila_schema::*;

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
    fn value_html(value: &Value, root: &stencila_schema::Node) -> Option<String> {
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

        use stencila_schema::*;

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
    pub fn html_set(&mut self, root: &stencila_schema::Node) {
        match self {
            Operation::Add(Add { value, html, .. })
            | Operation::Replace(Replace { value, html, .. }) => {
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
