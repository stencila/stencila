use super::prelude::*;

impl<Type: Diffable> Diffable for Option<Type>
where
    Type: Clone + 'static,
{
    diffable_is_same!(Option<Type>);

    fn is_equal(&self, other: &Self) -> Result<()> {
        match (self, other) {
            (None, None) => Ok(()),
            (None, Some(_)) | (Some(_), None) => bail!(Error::NotEqual),
            (Some(me), Some(other)) => me.is_equal(other),
        }
    }

    diffable_diff!(Option<Type>);

    fn diff_same(&self, differ: &mut Differ, other: &Self) {
        match (self, other) {
            (None, None) => (),
            (None, Some(value)) => differ.add(value),
            (Some(_), None) => differ.remove(),
            (Some(me), Some(other)) => me.diff_same(differ, other),
        }
    }
}
