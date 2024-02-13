// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { DatatableColumnHint } from "./DatatableColumnHint.js";
import { Entity } from "./Entity.js";
import { Integer } from "./Integer.js";

/**
 * A hint to the structure of a table of data.
 */
export class DatatableHint extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "DatatableHint";

  /**
   * The number of rows of data.
   */
  rows: Integer;

  /**
   * A hint for each column of data.
   */
  columns: DatatableColumnHint[];

  constructor(rows: Integer, columns: DatatableColumnHint[], options?: Partial<DatatableHint>) {
    super();
    this.type = "DatatableHint";
    if (options) Object.assign(this, options);
    this.rows = rows;
    this.columns = columns;
  }
}

/**
* Create a new `DatatableHint`
*/
export function datatableHint(rows: Integer, columns: DatatableColumnHint[], options?: Partial<DatatableHint>): DatatableHint {
  return new DatatableHint(rows, columns, options);
}
