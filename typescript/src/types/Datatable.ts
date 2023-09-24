// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from './CreativeWork';
import { DatatableColumn } from './DatatableColumn';

// A table of data.
export class Datatable extends CreativeWork {
  type = "Datatable";

  // The columns of data.
  columns: DatatableColumn[];

  constructor(columns: DatatableColumn[], options?: Datatable) {
    super()
    if (options) Object.assign(this, options)
    this.columns = columns;
  }

  static from(other: Datatable): Datatable {
    return new Datatable(other.columns!, other)
  }
}
