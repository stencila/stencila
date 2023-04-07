use common::indexmap::IndexMap;

use crate::{Strip, Targets};

impl<T> Strip for IndexMap<String, T>
where
    T: Strip,
{
    fn strip(&mut self, targets: Targets) -> &mut Self {
        for node in self.values_mut() {
            node.strip(targets);
        }

        self
    }
}
