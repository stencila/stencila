use stencila_schema::*;

use super::prelude::*;

impl Patchable for Datatable {
    fn diff(&self, other: &Self, differ: &mut Differ) {
        // TODO: Implement diffing optimized (semantically and computationally) for datatables
        // e.g. `Add` and `Remove` for entire columns and entire rows,
        // `Replace` for individual cells
        differ.replace(other)
    }
}

patchable_struct!(DatatableColumn, name, validator, values);

patchable_struct!(Variable, name, namespace, kind, value);
