use crate::{Strip, Targets};

impl<T> Strip for Vec<T>
where
    T: Strip,
{
    fn strip(&mut self, targets: Targets) -> &mut Self {
        for node in self.iter_mut() {
            node.strip(targets);
        }

        self
    }
}
