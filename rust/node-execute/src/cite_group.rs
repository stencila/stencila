use schema::CiteGroup;

use crate::prelude::*;

impl Executable for CiteGroup {
    async fn link(&mut self, executor: &mut Executor) -> WalkControl {
        let node_id = self.node_id();
        tracing::trace!("Linking CiteGroup {node_id}");

        // Manually link cite notes because these are visited as part of walk
        // (i.e. there is not visit_cite method)
        for cite in &mut self.items {
            cite.link(executor).await;
        }

        // Break walk because all child nodes walked above
        WalkControl::Break
    }
}
