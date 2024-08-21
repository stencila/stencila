use crate::prelude::*;

/// A variable available within a kernel instance
#[derive(Default, Clone, Trace)]
#[rquickjs::class(rename_all = "camelCase")]
pub struct Variable {
    /// The name of the variable
    #[qjs(get, enumerable)]
    pub(super) name: String,

    /// The Stencila node type of the variable
    #[qjs(get, enumerable, rename = "type")]
    pub(super) r#type: Option<String>,

    /// A hint to the value of the variable
    #[qjs(get, enumerable)]
    pub(super) hint: Option<Hint>,

    /// The native name for the type of the variable
    #[qjs(get, enumerable)]
    pub(super) native_type: Option<String>,

    /// A natively generated hint to the value of the variable
    #[qjs(get, enumerable)]
    pub(super) native_hint: Option<String>,
}

impl From<schema::Variable> for Variable {
    fn from(var: schema::Variable) -> Self {
        Self {
            name: var.name.clone(),
            r#type: var.node_type.clone(),
            hint: var.hint.map(Hint::from),
            native_type: var.native_type.clone(),
            native_hint: var.native_hint.clone(),
        }
    }
}

/// A variable available within a kernel instance
#[derive(Clone, Trace)]
#[rquickjs::class(rename_all = "camelCase")]
pub enum Hint {
    Boolean(bool),
    Integer(i32),
    Number(f64),
    String {
        length: i32,
    },
    Array {
        length: i32,
        types: Option<Vec<String>>,
        minimum: Option<f64>,
        maximum: Option<f64>,
        nulls: Option<i32>,
    },
    Object {
        length: i32,
        keys: Vec<String>,
        values: Vec<Hint>,
    },
    Datatable {
        rows: i32,
        columns: Vec<Hint>,
    },
    DatatableColumn {
        name: String,
        r#type: String,
        minimum: Option<f64>,
        maximum: Option<f64>,
        nulls: Option<i32>,
    },
    Function,
    Unknown,
}

impl From<schema::Hint> for Hint {
    fn from(hint: schema::Hint) -> Self {
        use schema::Hint::*;
        match hint {
            Boolean(value) => Self::Boolean(value),
            Integer(value) => Self::Integer(value as i32),
            Number(value) => Self::Number(value),
            StringHint(hint) => Self::String {
                length: hint.chars as i32,
            },
            ArrayHint(hint) => Self::Array {
                length: hint.length as i32,
                types: hint.item_types,
                minimum: hint.minimum.and_then(primitive_to_f64),
                maximum: hint.maximum.and_then(primitive_to_f64),
                nulls: hint.nulls.map(|nulls| nulls as i32),
            },
            ObjectHint(hint) => Self::Object {
                length: hint.length as i32,
                keys: hint.keys,
                values: hint.values.into_iter().map(Hint::from).collect(),
            },
            DatatableHint(hint) => Self::Datatable {
                rows: hint.rows as i32,
                columns: hint.columns.into_iter().map(Hint::from).collect(),
            },
            Function(..) => Self::Function,
            Unknown(..) => Self::Unknown,
        }
    }
}

impl From<schema::DatatableColumnHint> for Hint {
    fn from(hint: schema::DatatableColumnHint) -> Self {
        Self::DatatableColumn {
            name: hint.name,
            r#type: hint.item_type,
            minimum: hint.minimum.and_then(primitive_to_f64),
            maximum: hint.maximum.and_then(primitive_to_f64),
            nulls: hint.nulls.map(|nulls| nulls as i32),
        }
    }
}

#[rquickjs::methods]
impl Hint {
    /// Get the variable as a JavaScript value
    ///
    /// Only applies to `Boolean`, `Integer`, and `Number` variables. For all other
    /// variable types returns `None`.
    #[qjs(get, enumerable)]
    fn value<'js>(&self, ctx: Ctx<'js>) -> Value<'js> {
        match self {
            Self::Boolean(value) => Value::new_bool(ctx, *value),
            Self::Integer(value) => Value::new_int(ctx, *value),
            Self::Number(value) => Value::new_number(ctx, *value),
            _ => Value::new_undefined(ctx),
        }
    }

    /// Get the name of a datatable column
    ///
    /// Only applies to datatable columns. For all other
    /// variable types and hints returns `None`.
    #[qjs(get, enumerable)]
    fn name(&self) -> Option<String> {
        match self {
            Self::DatatableColumn { name, .. } => Some(name.clone()),
            _ => None,
        }
    }

    /// Get the keys of an object or column names of a datatable
    ///
    /// Only applies to `Object` and `Datatable` variables. For all other
    /// variable types and hints returns `None`.
    #[qjs(get, enumerable)]
    fn names(&self) -> Option<Vec<String>> {
        match self {
            Self::Object { keys, .. } => Some(keys.clone()),
            Self::Datatable { columns, .. } => Some(
                columns
                    .iter()
                    .filter_map(|column| column.name().clone())
                    .collect(),
            ),
            _ => None,
        }
    }

    /// Get the type of values in an array or data table column
    ///
    /// Only applies to arrays (first type if multiple) and datatable columns.
    /// For all other variables returns `None`.
    #[qjs(get, enumerable, rename = "type")]
    fn r#type(&self) -> Option<String> {
        match self {
            Self::Array { types, .. } => types.iter().flatten().next().cloned(),
            Self::DatatableColumn { r#type, .. } => Some(r#type.clone()),
            _ => None,
        }
    }

    /// Get the types of values in an array or data table column
    ///
    /// Only applies to arrays and datatable columns (which only ever have one type).
    /// For all other variables returns `None`.
    #[qjs(get, enumerable)]
    fn types(&self) -> Option<Vec<String>> {
        match self {
            Self::Array { types, .. } => types.clone(),
            Self::DatatableColumn { r#type, .. } => Some(vec![r#type.clone()]),
            _ => None,
        }
    }

    /// Get the length of the variable
    ///
    /// For a `Datatable`, this is an alias for `rows`.
    #[qjs(get, enumerable)]
    fn length(&self) -> Option<i32> {
        match self {
            Self::Boolean(..) | Self::Integer(..) | Self::Number(..) => Some(1),
            Self::String { length } | Self::Array { length, .. } | Self::Object { length, .. } => {
                Some(*length)
            }
            Self::Datatable { rows, .. } => Some(*rows),
            _ => None,
        }
    }

    /// Get the number of rows in a `Datatable`
    #[qjs(get, enumerable)]
    fn rows(&self) -> Option<i32> {
        match self {
            Self::Datatable { rows, .. } => Some(*rows),
            _ => None,
        }
    }

    /// Get the minimum of values in an array or data table column
    ///
    /// Only applies to arrays and datatable columns. For all other variables returns `None`.
    #[qjs(get, enumerable)]
    fn minimum(&self) -> Option<f64> {
        match self {
            Self::Array { minimum, .. } | Self::DatatableColumn { minimum, .. } => minimum.clone(),
            _ => None,
        }
    }

    /// Get the maximum of values in an array or data table column
    ///
    /// Only applies to arrays and datatable columns. For all other variables returns `None`.
    #[qjs(get, enumerable)]
    fn maximum(&self) -> Option<f64> {
        match self {
            Self::Array { maximum, .. } | Self::DatatableColumn { maximum, .. } => maximum.clone(),
            _ => None,
        }
    }

    /// Get the count of null value in an array or data table column
    ///
    /// Only applies to arrays and datatable columns. For all other variables returns `None`.
    #[qjs(get, enumerable)]
    fn nulls(&self) -> Option<i32> {
        match self {
            Self::Array { nulls, .. } | Self::DatatableColumn { nulls, .. } => nulls.clone(),
            _ => None,
        }
    }

    /// Get hints for the values of an object
    ///
    /// Only applies to `Object` variables. For all other variables returns `None`.
    #[qjs(get, enumerable)]
    fn values(&self) -> Option<Vec<Hint>> {
        match self {
            Self::Object { values, .. } => Some(values.clone()),
            _ => None,
        }
    }

    /// Get hints for the columns of a datatable
    ///
    /// Only applies to `Datatable` variables. For all other variables returns `None`.
    #[qjs(get, enumerable)]
    fn columns(&self) -> Option<Vec<Hint>> {
        match self {
            Self::Datatable { columns, .. } => Some(columns.clone()),
            _ => None,
        }
    }
}

fn primitive_to_f64(primitive: schema::Primitive) -> Option<f64> {
    match primitive {
        schema::Primitive::Boolean(value) => Some(value as i32 as f64),
        schema::Primitive::Integer(value) => Some(value as f64),
        schema::Primitive::UnsignedInteger(value) => Some(value as f64),
        schema::Primitive::Number(value) => Some(value),
        _ => None,
    }
}
