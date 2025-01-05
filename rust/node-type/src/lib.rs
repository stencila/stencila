#[rustfmt::skip]
mod node_property;
#[rustfmt::skip]
mod node_type;

pub use crate::node_type::NodeType;
pub use node_property::NodeProperty;

impl NodeType {
    /// Is the node type a block?
    pub fn is_block(&self) -> bool {
        use NodeType::*;
        matches!(
            self,
            Admonition
                | CallBlock
                | Chat
                | ChatMessage
                | ChatMessageGroup
                | Claim
                | CodeBlock
                | CodeChunk
                | DeleteBlock
                | Figure
                | ForBlock
                | Form
                | Heading
                | IfBlock
                | IncludeBlock
                | InsertBlock
                | InstructionBlock
                | List
                | MathBlock
                | ModifyBlock
                | Paragraph
                | PromptBlock
                | QuoteBlock
                | ReplaceBlock
                | Section
                | StyledBlock
                | SuggestionBlock
                | Table
                | ThematicBreak
                | Walkthrough
        )
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
}
