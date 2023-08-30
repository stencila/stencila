use node_strip::StripNode;

use crate::{Array, Null, Object, TextValue};

impl StripNode for Array {}
impl StripNode for Null {}
impl StripNode for Object {}
impl StripNode for TextValue {}
