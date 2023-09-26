// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CreativeWork } from "./CreativeWork.js";
import { DatatableColumn } from "./DatatableColumn.js";

/**
 * A table of data.
 */
export class Datatable extends CreativeWork {
  type = "Datatable";

  /**
   * The columns of data.
   */
  columns: DatatableColumn[];

  constructor(columns: DatatableColumn[], options?: Partial<Datatable>) {
    super();
    if (options) Object.assign(this, options);
    this.columns = columns;
  }
}

/**
* Create a new `Datatable`
*/
export function datatable(columns: DatatableColumn[], options?: Partial<Datatable>): Datatable {
  return new Datatable(columns, options);
}
