// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ArrayValidator } from "./ArrayValidator.js";
import { Primitive } from "./Primitive.js";
import { Thing } from "./Thing.js";

/**
 * A column of data within a `Datatable`.
 */
export class DatatableColumn extends Thing {
  type = "DatatableColumn";

  /**
   * The name of the item.
   */
  name: string;

  /**
   * The data values of the column.
   */
  values: Primitive[];

  /**
   * The validator to use to validate data in the column.
   */
  validator?: ArrayValidator;

  constructor(name: string, values: Primitive[], options?: Partial<DatatableColumn>) {
    super();
    if (options) Object.assign(this, options);
    this.name = name;
    this.values = values;
  }
}

/**
* Create a new `DatatableColumn`
*/
export function datatableColumn(name: string, values: Primitive[], options?: Partial<DatatableColumn>): DatatableColumn {
  return new DatatableColumn(name, values, options);
}
