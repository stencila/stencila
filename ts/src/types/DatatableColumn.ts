// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ArrayValidator } from "./ArrayValidator.js";
import { Primitive } from "./Primitive.js";
import { Thing } from "./Thing.js";

/**
 * A column of data within a `Datatable`.
 */
export class DatatableColumn extends Thing {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "DatatableColumn";

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
    this.type = "DatatableColumn";
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
