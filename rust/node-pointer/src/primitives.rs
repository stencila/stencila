use crate::Pointable;
use stencila_schema::*;

impl Pointable for Null {}
impl Pointable for Boolean {}
impl Pointable for Integer {}
impl Pointable for Number {}
impl Pointable for String {}
impl Pointable for Array {}
impl Pointable for Object {}
