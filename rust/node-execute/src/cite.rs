use schema::Cite;

use crate::prelude::*;

impl Executable for Cite {
    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking Cite {node_id}");

        if let Some(reference) = executor.targets.get(&self.target) {
            executor.references.push(reference.clone());

            if self.options.reference.is_none()
                || Some(reference) != self.options.reference.as_ref()
            {
                self.options.reference = Some(reference.clone());
                executor.patch(&node_id, [set(NodeProperty::Reference, reference.clone())]);
            }
        }

        WalkControl::Continue
    }
}
