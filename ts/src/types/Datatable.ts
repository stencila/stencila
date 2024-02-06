// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CreativeWork } from "./CreativeWork.js";
import { DatatableColumn } from "./DatatableColumn.js";

/**
 * A table of data.
 */
export class Datatable extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Datatable";

  /**
   * The columns of data.
   */
  columns: DatatableColumn[];

  constructor(columns: DatatableColumn[], options?: Partial<Datatable>) {
    super();
    this.type = "Datatable";
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
