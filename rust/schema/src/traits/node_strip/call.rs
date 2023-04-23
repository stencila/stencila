use node_strip::{Strip, Targets};

use crate::{strip_execution, Call};

impl Strip for Call {
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        if targets.id {
            self.id = None;
        }

        if targets.code {
            self.source = String::new();
            self.media_type = None;
            self.select = None;
        }

        if targets.execution {
            strip_execution!(self);
        }

        if targets.outputs {
            self.content = None;
        }

        self.arguments.strip(targets);

        self
    }
}
