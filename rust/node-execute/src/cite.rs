use schema::Cite;

use crate::prelude::*;

impl Executable for Cite {
    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking Cite {node_id}");

        if let Some(reference) = executor.targets.get(&self.target) {
            if self.options.cites.is_none() || Some(reference) != self.options.cites.as_ref() {
                self.options.cites = Some(reference.clone());
                executor.patch(&node_id, [set(NodeProperty::Cite, reference.clone())]);
            }
        }

        if let Some(reference) = &self.options.cites {
            // Register any cited reference
            executor.references.push(reference.clone());
        }

        WalkControl::Continue
    }
}
