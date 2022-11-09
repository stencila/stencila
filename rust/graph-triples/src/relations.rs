use schemars::JsonSchema;

use common::{serde::Serialize, strum::Display};

/// The relation between two resources in a dependency graph (the edges of the graph)
///
/// Some relations carry additional information such whether the relation is active
/// (`Import` and `Convert`) or the range that they occur in code (`Assign`, `Use`, `Read`) etc
#[derive(Debug, Display, Clone, JsonSchema, Serialize)]
#[serde(tag = "type", crate = "common::serde")]
pub enum Relation {
    Alters(Alters),
    Assigns(Assigns),
    Calls,
    Converts(Converts),
    Declares(Declares),
    Derives,
    Embeds,
    Imports(Imports),
    Includes,
    Links,
    Reads(Reads),
    Requires(Requires),
    Uses(Uses),
    Writes(Writes),
}

/// The two dimensional range that a relation is defined within some
/// code (line start, column start, line end, column end).
pub type Range = (usize, usize, usize, usize);

/// A null range which can be used in places where we do not know where
/// in the `subject` the relation is defined.
pub const NULL_RANGE: Range = (0, 0, 0, 0);

/// Declares a symbol
///
/// For some languages, variable declaration is distinct to assignment
/// (e.g. `let` in JavaScript). This relation allows for those to be
/// distinguished.
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Declares {
    /// The range within code that the assignment is done
    pub range: Range,
}

/// Create a new `Declare` relation
pub fn declares(range: Range) -> Relation {
    Relation::Declares(Declares { range })
}

/// Assigns to a symbol
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Assigns {
    /// The range within code that the assignment is done
    pub range: Range,
}

/// Create a new `Assign` relation
pub fn assigns(range: Range) -> Relation {
    Relation::Assigns(Assigns { range })
}

/// Alters a symbol
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Alters {
    /// The range within code that the alter
    pub range: Range,
}

/// Create a new `Alter` relation
pub fn alters(range: Range) -> Relation {
    Relation::Alters(Alters { range })
}

/// Imports a `Module` or a `File`
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Imports {
    /// The range within code
    pub range: Range,
}

/// Create a new `Import` relation
pub fn imports(range: Range) -> Relation {
    Relation::Imports(Imports { range })
}

/// Converts a file into another
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Converts {
    /// Whether or not the conversion is automatically updated
    pub auto: bool,
}

/// Create a new `Convert` relation
pub fn converts(auto: bool) -> Relation {
    Relation::Converts(Converts { auto })
}

/// Reads from a file
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Reads {
    /// The range within code that the read is declared
    pub range: Range,
}

/// Create a new `Read` relation
pub fn reads(range: Range) -> Relation {
    Relation::Reads(Reads { range })
}

/// Uses a symbol or module
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Uses {
    /// The range within code that the use is declared
    pub range: Range,
}

/// Create a new `Use` relation
pub fn uses(range: Range) -> Relation {
    Relation::Uses(Uses { range })
}

/// Writes to a file
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Writes {
    /// The range within code that the write is declared
    pub range: Range,
}

/// Create a new `Write` relation
pub fn writes(range: Range) -> Relation {
    Relation::Writes(Writes { range })
}

/// Requires another code node to be executed first
///
/// Allows the dependency of one code resource on another to be
/// explicitly declared, using its id, rather than relying on semantic
/// analysis or `@uses` tags.
#[derive(Debug, Clone, JsonSchema, Serialize)]
#[serde(crate = "common::serde")]
#[schemars(deny_unknown_fields)]
pub struct Requires {
    /// The range within code that the require is declared
    /// (usually within a comment tag, `@requires`)
    pub range: Range,
}

/// Create a new `Require` relation
pub fn requires(range: Range) -> Relation {
    Relation::Requires(Requires { range })
}

/// Create a new `Derived` relation
pub fn derives() -> Relation {
    Relation::Derives
}
