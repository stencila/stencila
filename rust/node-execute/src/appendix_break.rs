use schema::AppendixBreak;

use crate::prelude::*;

impl Executable for AppendixBreak {
    #[tracing::instrument(skip_all)]
    async fn compile(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Compiling AppendixBreak {node_id}");

        executor.appendix_count += 1;
        executor.figure_count = 0;
        executor.table_count = 0;
        executor.equation_count = 0;

        WalkControl::Continue
    }
}
