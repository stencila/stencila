use std::{any::Any, fmt::Debug};

use common::{
    serde::{Deserialize, Deserializer, Serialize, Serializer},
    serde_json,
    strum::Display,
    tracing,
};

/// Type for the `value` property of `Add` and `Replace` operations
///
/// Has variants for the most common types of values in patches with
/// a fallback `Any` variant.
#[derive(Debug, Display)]
pub enum Value {
    Any(Box<dyn Any + Send>),
}

impl Value {
    // Construct an any variant
    pub fn any<Type>(value: Type) -> Value
    where
        Type: Send + 'static,
    {
        Self::Any(Box::new(value))
    }

    // Check if the value is of a specific type
    pub fn is<Type>(&self) -> bool
    where
        Type: 'static,
    {
        use Value::*;
        match self {
            Any(any) => any.is::<Type>(),
            _ => false,
        }
    }

    // Downcast the value to a reference of a specific type
    pub fn downcast_ref<Type>(&self) -> Option<&Type>
    where
        Type: 'static,
    {
        use Value::*;
        match self {
            Any(any) => any.downcast_ref(),
            _ => None,
        }
    }

    // Downcast the value to a mutable reference of a specific type
    pub fn downcast_mut<Type>(&mut self) -> Option<&mut Type>
    where
        Type: 'static,
    {
        use Value::*;
        match self {
            Any(any) => any.downcast_mut(),
            _ => None,
        }
    }

    /// Generate HTML for the `value` field of an operation
    pub fn to_html(&self, root: &stencila_schema::Node) -> Option<String> {
        use codec_html::{EncodeContext, ToHtml};

        let mut context = EncodeContext {
            root,
            ..Default::default()
        };

        // Convert a node, boxed node, or vector of nodes to HTML
        macro_rules! to_html {
            ($type:ty) => {
                if let Some(node) = self.downcast_ref::<$type>() {
                    return Some(node.to_html(&mut context));
                }
                if let Some(boxed) = self.downcast_ref::<Box<$type>>() {
                    return Some(boxed.to_html(&mut context));
                }
                if let Some(nodes) = self.downcast_ref::<Vec<$type>>() {
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
                if let Some(node) = self.downcast_ref::<$type>() {
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
        if let Some(value) = self.downcast_ref::<serde_json::Value>() {
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
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value: serde_json::Value = Deserialize::deserialize(deserializer)?;
        Ok(Value::any(value))
    }
}

impl Serialize for Value {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        use stencila_schema::*;

        macro_rules! serialize {
            ($type:ty) => {
                if let Some(value) = self.downcast_ref::<$type>() {
                    return value.serialize(serializer);
                }
                if let Some(value) = self.downcast_ref::<Option<$type>>() {
                    return value.serialize(serializer);
                }
                if let Some(value) = self.downcast_ref::<Box<$type>>() {
                    return value.serialize(serializer);
                }
                if let Some(value) = self.downcast_ref::<Option<Box<$type>>>() {
                    return value.serialize(serializer);
                }
                if let Some(value) = self.downcast_ref::<Vec<$type>>() {
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
        if let Some(value) = self.downcast_ref::<serde_json::Value>() {
            return value.serialize(serializer);
        }

        tracing::error!("Unhandled value type when serializing patch operation");
        "<unserialized type>".serialize(serializer)
    }
}
