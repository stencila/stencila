use crate::{Strip, Targets};

impl<T> Strip for Box<T>
where
    T: Strip,
{
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        self.as_mut().strip(targets);

        self
    }
}
