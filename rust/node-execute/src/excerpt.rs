use schema::Excerpt;

use crate::prelude::*;

impl Executable for Excerpt {
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling Excerpt {node_id}");

        if let Some(id) = &self.id {
            executor
                .bibliography
                .insert(id.to_string(), self.source.clone());
        }

        WalkControl::Continue
    }
}
