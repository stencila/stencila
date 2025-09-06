// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";
import { DatatableColumn } from "./DatatableColumn.js";

/**
 * A table of data.
 */
export class Datatable extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Datatable";

  /**
   * A short label for the datatable.
   */
  label?: string;

  /**
   * Whether the datatable label should be automatically updated.
   */
  labelAutomatically?: boolean;

  /**
   * A caption for the datatable.
   */
  caption?: Block[];

  /**
   * The columns of data.
   */
  columns: DatatableColumn[];

  /**
   * Notes for the datatable.
   */
  notes?: Block[];

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
