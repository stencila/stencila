use crate::{Strip, Targets};

impl<T> Strip for Option<T>
where
    T: Strip,
{
    fn strip(&mut self, targets: &Targets) -> &mut Self {
        if let Some(value) = self {
            value.strip(targets);
        }

        self
    }
}
