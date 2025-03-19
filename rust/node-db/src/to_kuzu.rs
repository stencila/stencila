use kuzu::{LogicalType, Value};

use schema::*;

/// A trait for converting Stencila nodes to Kuzu values
pub trait ToKuzu {
    /// Get the corresponding Kuzu logical type
    fn to_kuzu_type() -> LogicalType;

    /// Convert to a Kuzu value
    fn to_kuzu_value(&self) -> Value;
}

impl ToKuzu for bool {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::Bool
    }

    fn to_kuzu_value(&self) -> Value {
        Value::Bool(*self)
    }
}

impl ToKuzu for i64 {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::Int64
    }

    fn to_kuzu_value(&self) -> Value {
        Value::Int64(*self)
    }
}

impl ToKuzu for u64 {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::UInt64
    }

    fn to_kuzu_value(&self) -> Value {
        Value::UInt64(*self)
    }
}

impl ToKuzu for usize {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::UInt64
    }

    fn to_kuzu_value(&self) -> Value {
        Value::UInt64(*self as u64)
    }
}

impl ToKuzu for f64 {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::Double
    }

    fn to_kuzu_value(&self) -> Value {
        Value::Double(*self)
    }
}

impl ToKuzu for String {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::String
    }

    fn to_kuzu_value(&self) -> Value {
        Value::String(self.clone())
    }
}

impl<T> ToKuzu for Option<T>
where
    T: ToKuzu,
{
    fn to_kuzu_type() -> LogicalType {
        T::to_kuzu_type()
    }

    fn to_kuzu_value(&self) -> Value {
        match self {
            None => Value::Null(T::to_kuzu_type()),
            Some(value) => value.to_kuzu_value(),
        }
    }
}

impl<T> ToKuzu for Vec<T>
where
    T: ToKuzu,
{
    fn to_kuzu_type() -> LogicalType {
        LogicalType::List {
            child_type: Box::new(T::to_kuzu_type()),
        }
    }

    fn to_kuzu_value(&self) -> Value {
        Value::List(
            T::to_kuzu_type(),
            self.iter().map(|item| item.to_kuzu_value()).collect(),
        )
    }
}

impl ToKuzu for NodeId {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::String
    }

    fn to_kuzu_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl ToKuzu for NodePath {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::String
    }

    fn to_kuzu_value(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl ToKuzu for Date {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::Date
    }

    fn to_kuzu_value(&self) -> Value {
        match self.try_into() {
            Ok(value) => Value::Date(value),
            Err(..) => Value::Null(Self::to_kuzu_type()),
        }
    }
}

impl ToKuzu for DateTime {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::Timestamp
    }

    fn to_kuzu_value(&self) -> Value {
        match self.try_into() {
            Ok(value) => Value::Timestamp(value),
            Err(..) => Value::Null(Self::to_kuzu_type()),
        }
    }
}

impl ToKuzu for Timestamp {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::Timestamp
    }

    fn to_kuzu_value(&self) -> Value {
        match self.try_into() {
            Ok(value) => Value::Timestamp(value),
            Err(..) => Value::Null(Self::to_kuzu_type()),
        }
    }
}

impl ToKuzu for Duration {
    fn to_kuzu_type() -> LogicalType {
        LogicalType::Interval
    }

    fn to_kuzu_value(&self) -> Value {
        Value::Interval(self.into())
    }
}

macro_rules! enumeration {
    ($( $type:ident ),*) => {
        $(impl ToKuzu for $type {
            fn to_kuzu_type() -> LogicalType {
                LogicalType::String
            }

            fn to_kuzu_value(&self) -> Value {
                Value::String(self.to_string())
            }
        })*
    };
}

enumeration!(
    AdmonitionType,
    ExecutionMode,
    ExecutionBounds,
    ExecutionStatus,
    ExecutionRequired,
    AuthorRoleName,
    CitationMode,
    CitationIntent,
    ClaimType,
    LabelType,
    ListOrder,
    NoteType,
    SectionType,
    TableCellType,
    TableRowType,
    VerticalAlignment,
    HorizontalAlignment
);
