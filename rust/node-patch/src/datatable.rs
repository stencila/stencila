use super::prelude::*;
use stencila_schema::{Datatable, DatatableColumn};

impl Patchable for Datatable {
    patchable_struct_is_equal!(columns);
    patchable_struct_hash!(columns);

    fn diff(&self, other: &Self, differ: &mut Differ) {
        // TODO: Implement diffing optimized (semantically and computationally) for datatables
        // e.g. `Add` and `Remove` for entire columns and entire rows,
        // `Replace` for individual cells
        differ.replace(other)
    }
}

patchable_struct!(DatatableColumn, name, values);
