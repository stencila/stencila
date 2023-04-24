use node_strip::{Strip, Targets};

use crate::{strip_execution, If};

impl Strip for If {
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        if targets.id {
            self.id = None;
        }

        if targets.execution {
            strip_execution!(self);
        }

        self.clauses.strip(targets);

        self
    }
}
