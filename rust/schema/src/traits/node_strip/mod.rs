use node_strip::Strip;

use crate::{Array, Null, Object, TextValue};

impl Strip for Array {}
impl Strip for Null {}
impl Strip for Object {}
impl Strip for TextValue {}
