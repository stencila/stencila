use stencila_schema::*;

use crate::Pointable;

impl Pointable for Null {}
impl Pointable for Boolean {}
impl Pointable for Integer {}
impl Pointable for Number {}
impl Pointable for String {}
impl Pointable for Array {}
impl Pointable for Object {}

// The pointable_struct macro can not be used for these
// because they have no `id`. In any case, that is probably not necessary.
impl Pointable for Date {}
impl Pointable for Time {}
impl Pointable for DateTime {}
impl Pointable for Timestamp {}
impl Pointable for Duration {}
