use crate::NodeType;

impl NodeType {
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
