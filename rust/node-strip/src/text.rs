use schema::Text;

use crate::{Strip, Targets};

impl Strip for Text {
    fn strip(&mut self, targets: Targets) -> &mut Self {
        match targets {
            Targets::Id => {
                self.id = None;
            }
        }
        self
    }
}
