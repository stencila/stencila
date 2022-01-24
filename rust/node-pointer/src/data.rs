use crate::{Pointable, Pointer, PointerMut, Visitor, VisitorMut};
use eyre::{bail, Result};
use node_address::Address;
use stencila_schema::*;

// Implementations for data related types

pointable_struct!(Datatable);
pointable_struct!(DatatableColumn);

pointable_struct!(Validator);
pointable_struct!(ArrayValidator);
pointable_struct!(BooleanValidator);
pointable_struct!(ConstantValidator);
pointable_struct!(EnumValidator);
pointable_struct!(IntegerValidator);
pointable_struct!(NumberValidator);
pointable_struct!(StringValidator);
pointable_struct!(TupleValidator);
