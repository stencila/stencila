use node_strip::{Strip, Targets};

use crate::{CodeChunk, strip_execution, strip_code};

impl Strip for CodeChunk {
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

        if targets.outputs {
            self.outputs = None;
        }

        self
    }
}
