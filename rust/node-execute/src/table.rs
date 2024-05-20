use schema::{NodeProperty, Table};

use crate::prelude::*;

impl Executable for Table {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::debug!("Compiling Table {node_id}");

        executor.table_count += 1;

        if self.label_automatically.unwrap_or(true) {
            let label = executor.table_count.to_string();
            if Some(&label) != self.label.as_ref() {
                executor.patch(&node_id, [set(NodeProperty::Label, label)]);
            }
        }

        WalkControl::Continue
    }
}
