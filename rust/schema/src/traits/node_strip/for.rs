use node_strip::{Strip, Targets};

use crate::{For, strip_execution, strip_code};

impl Strip for For {
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        if targets.id {
            self.id = None;
        }

        if targets.code {
            strip_code!(self);
            self.symbol = String::new();
            self.content = Vec::new();
            self.otherwise = None;
        }

        if targets.execution {
            strip_execution!(self);
        }

        if targets.outputs {
            self.iterations = None;
        }

        self
    }
}
