use schema::{AppendixBreak, CompilationMessage};

use crate::prelude::*;

impl Executable for AppendixBreak {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling AppendixBreak {node_id}");

        if executor.appendix_count.is_none() {
            executor.appendix_count = Some(0);

            if self.options.compilation_messages.is_some() {
                executor.patch(&node_id, [none(NodeProperty::CompilationMessages)]);
            }
        } else if self.options.compilation_messages.is_none() {
            let messages = Some(vec![CompilationMessage {
                level: MessageLevel::Warning,
                message: "Extra appendix break; will be ignored".to_string(),
                ..Default::default()
            }]);
            executor.patch(&node_id, [set(NodeProperty::CompilationMessages, messages)]);
        }

        WalkControl::Continue
    }
}
