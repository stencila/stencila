mod content_type;
#[rustfmt::skip]
mod node_property;
#[rustfmt::skip]
mod node_type;
#[rustfmt::skip]
mod node_type_properties;

use std::str::FromStr;

use common::{eyre::Result, inflector::Inflector};
use node_type_properties::node_type_properties;

pub use crate::node_type::NodeType;
pub use content_type::ContentType;
pub use node_property::NodeProperty;

impl NodeType {
    /// Parse a node type from a string
    ///
    /// Allows for aliases not handled by `from_str`.
    pub fn from_name(name: &str) -> Result<Self> {
        Ok(match name {
            "code" | "chunk" | "cell" => NodeType::CodeChunk,
            "figure" | "fig" => NodeType::Figure,
            "heading" => NodeType::Heading,
            "list" => NodeType::List,
            "math" | "equation" | "eqn" => NodeType::MathBlock,
            "paragraph" | "para" => NodeType::Paragraph,
            "quote" => NodeType::QuoteBlock,
            "section" => NodeType::Section,
            "table" => NodeType::Table,
            _ => return Ok(NodeType::from_str(name)?),
        })
    }

    /// Can the node type be executed?
    ///
    /// Should only include node types which can be executed "manually"
    /// by the user.
    ///
    /// As such, it does not include node types such as `IfBlockClause`
    /// which, although they extend `Executable`, can not be executed
    /// independently of the other clauses in an `IfBlock`.
    pub fn can_execute(&self) -> bool {
        use NodeType::*;
        matches!(
            self,
            CallBlock
                | CodeChunk
                | CodeExpression
                | ForBlock
                | IfBlock
                | IncludeBlock
                | InstructionBlock
                | InstructionInline
        )
    }

    /// Get the properties of the node type
    pub fn properties(&self) -> Vec<NodeProperty> {
        node_type_properties(self)
    }
}

impl NodeProperty {
    /// The property as a camelCased string
    pub fn to_camel_case(&self) -> String {
        self.to_string().to_camel_case()
    }

    /// The property as a snake_cased string
    pub fn to_snake_case(&self) -> String {
        self.to_string().to_snake_case()
    }
}
