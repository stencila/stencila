pub use autosurgeon::{Hydrate, Reconcile};
pub use monostate::MustBe;

pub use common::{
    defaults::Defaults,
    serde::{self, Deserialize, Serialize},
    serde_json,
};

/// Implementation of `Hydrate` and `Reconcile` for `monostate::MustBeStr`.
///
/// Since `MustBeStr` is a zero-sided type this does not actually do anything.
pub mod autosurgeon_must_be {
    use autosurgeon::{HydrateError, Prop, ReadDoc, Reconciler};
    use monostate::MustBeStr;

    pub fn hydrate<D: ReadDoc, T>(
        _doc: &D,
        _obj: &automerge::ObjId,
        _prop: Prop<'_>,
    ) -> Result<MustBeStr<T>, HydrateError> {
        Ok(MustBeStr::<T>::MustBeStr)
    }

    pub fn reconcile<R: Reconciler, T>(
        _must_be: &MustBeStr<T>,
        mut _reconciler: R,
    ) -> Result<(), R::Error> {
        Ok(())
    }
}
