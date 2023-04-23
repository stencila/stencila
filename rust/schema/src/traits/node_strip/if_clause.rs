use node_strip::{Strip, Targets};

use crate::{strip_code, strip_execution, IfClause};

impl Strip for IfClause {
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        if targets.id {
            self.id = None;
        }

        if targets.code {
            strip_code!(self);
        }

        if targets.execution {
            strip_execution!(self);
        }

        self
    }
}
