use common::tracing;

use crate::{patch, prelude::*, InstructionInline, SuggestionStatus};

impl InstructionInline {
    /// Custom implementation of [`PatchNode::apply`]
    pub fn apply_with(
        &mut self,
        path: &mut NodePath,
        op: &PatchOp,
        context: &mut PatchContext,
    ) -> Result<bool> {
        if path.is_empty() {
            if let PatchOp::Accept(suggestion_id) = op {
                if let Some(suggestions) = &mut self.suggestions {
                    for suggestion in suggestions.iter_mut() {
                        if &suggestion.node_id() == suggestion_id {
                            // Mark the accepted suggestion as such
                            suggestion.suggestion_status = Some(SuggestionStatus::Accepted);

                            // Record the patcher as the acceptor
                            let accepter_patch = context.authors_as_acceptors();
                            let mut content = suggestion.content.clone();
                            for node in &mut content {
                                if let Err(error) = patch(node, accepter_patch.clone()) {
                                    tracing::error!("While accepting block suggestion: {error}");
                                }
                            }
                        } else {
                            // Implicitly reject other suggestions
                            suggestion.suggestion_status = Some(SuggestionStatus::Rejected);
                        }
                    }
                }

                return Ok(true);
            }
        }

        Ok(false)
    }
}
